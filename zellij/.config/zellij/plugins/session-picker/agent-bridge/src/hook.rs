use std::env;
use std::io::{self, Read};

use agent_core::{Activity, AgentRecord, AgentState, AgentTarget};
use serde_json::{json, Value};

use crate::{aliases, store};

const PREVIEW_LIMIT: usize = 512;

/// Canonical parsed form of a raw hook event-name string. Both Codex and Claude emit this same
/// vocabulary today, so decision logic below matches on this instead of re-parsing raw strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HookEvent {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    Notification,
    PreCompact,
    PostCompact,
    SubagentStart,
    SubagentStop,
    Stop,
    SessionEnd,
    Unknown,
}

impl HookEvent {
    pub(crate) fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "sessionstart" => Self::SessionStart,
            "userpromptsubmit" => Self::UserPromptSubmit,
            "pretooluse" => Self::PreToolUse,
            "posttooluse" => Self::PostToolUse,
            "posttoolusefailure" => Self::PostToolUseFailure,
            "permissionrequest" => Self::PermissionRequest,
            "notification" => Self::Notification,
            "precompact" => Self::PreCompact,
            "postcompact" => Self::PostCompact,
            "subagentstart" => Self::SubagentStart,
            "subagentstop" => Self::SubagentStop,
            "stop" => Self::Stop,
            "sessionend" => Self::SessionEnd,
            _ => Self::Unknown,
        }
    }
}

/// Everything needed to decide how one hook invocation updates the record store, gathered from
/// env/stdin. Plain data — building it involves I/O, but the struct itself does not.
pub(crate) struct HookContext {
    pub(crate) agent: &'static str,
    pub(crate) session_name: String,
    pub(crate) pane_id: u32,
    pub(crate) event: HookEvent,
    pub(crate) raw_event: String,
    pub(crate) tool: String,
    pub(crate) payload: Value,
    pub(crate) owner_pid: u32,
    pub(crate) agent_session_id: String,
    pub(crate) id: String,
    pub(crate) observed_at_ms: u64,
}

impl HookContext {
    /// Gathers a `HookContext` from env/stdin for one hook invocation. Returns `Ok(None)` when
    /// the invocation is missing the Zellij pane info it needs to be attributed to a session.
    pub(crate) fn gather(agent: &'static str) -> Result<Option<Self>, String> {
        let Some(session_name) = env::var("ZELLIJ_SESSION_NAME")
            .ok()
            .filter(|v| !v.is_empty())
        else {
            return Ok(None);
        };
        let Some(pane_id) = env::var("ZELLIJ_PANE_ID")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
        else {
            return Ok(None);
        };

        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        let payload: Value = if input.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&input).map_err(|e| format!("invalid hook JSON: {e}"))?
        };

        let session_name = aliases::resolve_session_alias(&session_name);
        let raw_event = string_at(&payload, &["hook_event_name", "event_name", "event"])
            .unwrap_or("unknown")
            .to_string();
        let event = HookEvent::parse(&raw_event);
        let tool = string_at(&payload, &["tool_name", "toolName"])
            .unwrap_or("")
            .to_string();
        let observed_at_ms = store::now_ms();
        let owner_pid = env::var("SESSION_PICKER_AGENT_PID")
            .ok()
            .and_then(|v| v.parse().ok())
            // Hook processes and their process groups are ephemeral. Without a PID supplied by the
            // agent, the Zellij pane is the durable owner of this record.
            .unwrap_or_default();
        let agent_session_id = string_at(&payload, &["session_id", "sessionId"])
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("pid-{owner_pid}"));
        let id = format!("{session_name}:{pane_id}:{agent}:{agent_session_id}");

        Ok(Some(Self {
            agent,
            session_name,
            pane_id,
            event,
            raw_event,
            tool,
            payload,
            owner_pid,
            agent_session_id,
            id,
            observed_at_ms,
        }))
    }
}

/// Outcome of applying one hook event on top of the previous record for this id, if any.
pub(crate) enum RecordUpdate {
    /// SessionEnd (or equivalent) — the record should be removed.
    Remove,
    /// A stale/out-of-order event — no-op.
    Ignore,
    /// Write this record.
    Upsert(AgentRecord),
}

/// Pure decision function: no filesystem, env, or locking. Folds in SessionEnd removal, the
/// staleness check, state derivation, pending-permission tracking, the seen-flag ladder, and
/// activity derivation.
pub(crate) fn decide(ctx: &HookContext, previous: Option<&AgentRecord>) -> RecordUpdate {
    if ctx.event == HookEvent::SessionEnd {
        return RecordUpdate::Remove;
    }
    if previous.is_some_and(|r| r.observed_at_ms > ctx.observed_at_ms) {
        return RecordUpdate::Ignore;
    }

    let mut pending = previous
        .map(|r| r.pending_permissions.clone())
        .unwrap_or_default();
    update_pending_permissions(ctx.event, &ctx.tool, &mut pending);
    let mut state = event_state(ctx.event, &ctx.tool, &ctx.payload);
    if !pending.is_empty() && state != AgentState::Idle {
        state = AgentState::Blocked;
    }
    let seen = compute_seen(state, previous);
    let activity = activity_for(ctx.event, &ctx.raw_event, &ctx.tool, &ctx.payload, previous);

    RecordUpdate::Upsert(AgentRecord {
        id: ctx.id.clone(),
        agent_label: ctx.agent.to_string(),
        state,
        seen,
        target: AgentTarget {
            session_name: ctx.session_name.clone(),
            pane_id: ctx.pane_id,
        },
        activity,
        owner_pid: ctx.owner_pid,
        process_fingerprint: String::new(),
        agent_session_id: ctx.agent_session_id.clone(),
        observed_at_ms: ctx.observed_at_ms,
        pending_permissions: pending,
    })
}

/// A freshly-idle agent is unseen (surfaces as "done") only if it was previously mid-task;
/// otherwise seen status carries over from the previous record (or defaults to seen).
fn compute_seen(state: AgentState, previous: Option<&AgentRecord>) -> bool {
    if state == AgentState::Idle {
        !previous.is_some_and(|r| matches!(r.state, AgentState::Working | AgentState::Blocked))
    } else {
        previous.map(|r| r.seen).unwrap_or(true)
    }
}

fn update_pending_permissions(event: HookEvent, tool: &str, pending: &mut Vec<String>) {
    let key = if tool.is_empty() { "request" } else { tool };
    match event {
        HookEvent::PermissionRequest => {
            if !pending.iter().any(|p| p == key) {
                pending.push(key.to_string());
            }
        }
        HookEvent::PostToolUse | HookEvent::PostToolUseFailure => pending.retain(|p| p != key),
        HookEvent::SessionStart | HookEvent::UserPromptSubmit | HookEvent::Stop | HookEvent::SessionEnd => {
            pending.clear()
        }
        _ => {}
    }
}

fn event_state(event: HookEvent, tool: &str, payload: &Value) -> AgentState {
    match event {
        HookEvent::SessionStart | HookEvent::Stop => AgentState::Idle,
        HookEvent::PermissionRequest | HookEvent::Notification => AgentState::Blocked,
        HookEvent::UserPromptSubmit
        | HookEvent::PreCompact
        | HookEvent::PostCompact
        | HookEvent::SubagentStart
        | HookEvent::SubagentStop
        | HookEvent::PostToolUse
        | HookEvent::PostToolUseFailure => AgentState::Working,
        HookEvent::PreToolUse if tool.eq_ignore_ascii_case("AskUserQuestion") => AgentState::Blocked,
        HookEvent::PreToolUse => AgentState::Working,
        _ if payload.get("error").is_some() => AgentState::Blocked,
        _ => AgentState::Unknown,
    }
}

fn activity_for(
    event: HookEvent,
    raw_event: &str,
    tool: &str,
    payload: &Value,
    previous: Option<&AgentRecord>,
) -> Activity {
    let (kind, label, candidate) = match event {
        HookEvent::SessionStart => (
            "session",
            "waiting for input".to_string(),
            string_at(payload, &["cwd"]),
        ),
        HookEvent::UserPromptSubmit => (
            "prompt",
            "responding".to_string(),
            string_at(payload, &["prompt", "user_prompt", "userPrompt"]),
        ),
        HookEvent::PermissionRequest => (
            "permission",
            format!("waiting for approval{}", tool_suffix(tool)),
            tool_preview(payload),
        ),
        HookEvent::Notification => (
            "notification",
            "waiting for input".to_string(),
            string_at(payload, &["message", "notification", "title"]),
        ),
        HookEvent::PreToolUse if tool.eq_ignore_ascii_case("AskUserQuestion") => (
            "question",
            "waiting for an answer".to_string(),
            tool_preview(payload),
        ),
        HookEvent::PreToolUse | HookEvent::PostToolUse | HookEvent::PostToolUseFailure => (
            "tool",
            if tool.is_empty() {
                "using a tool".to_string()
            } else {
                format!("using {tool}")
            },
            tool_preview(payload),
        ),
        HookEvent::PreCompact | HookEvent::PostCompact => {
            ("compact", "compacting context".to_string(), None)
        }
        HookEvent::SubagentStart | HookEvent::SubagentStop => (
            "subagent",
            "running subagents".to_string(),
            string_at(payload, &["agent_type", "subagent_type"]),
        ),
        HookEvent::Stop => (
            "stop",
            "finished".to_string(),
            string_at(payload, &["last_assistant_message", "reason"]),
        ),
        _ => ("event", raw_event.to_string(), None),
    };
    let preview = candidate
        .map(truncate_preview)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            previous
                .map(|r| r.activity.preview.clone())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| label.clone());
    Activity {
        kind: kind.into(),
        label,
        preview,
    }
}

fn tool_suffix(tool: &str) -> String {
    if tool.is_empty() {
        String::new()
    } else {
        format!(" ({tool})")
    }
}

fn tool_preview(payload: &Value) -> Option<&str> {
    let input = payload
        .get("tool_input")
        .or_else(|| payload.get("toolInput"))?;
    for key in [
        "command",
        "file_path",
        "path",
        "query",
        "url",
        "prompt",
        "question",
        "description",
    ] {
        if let Some(value) = input.get(key).and_then(Value::as_str) {
            return Some(value);
        }
    }
    None
}

fn truncate_preview(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let prefix: String = chars.by_ref().take(PREVIEW_LIMIT).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

pub(crate) fn string_at<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(state: AgentState, seen: bool, pending: Vec<String>) -> AgentRecord {
        AgentRecord {
            id: "id".into(),
            agent_label: "codex".into(),
            state,
            seen,
            target: AgentTarget {
                session_name: "s".into(),
                pane_id: 1,
            },
            activity: Activity {
                kind: "stop".into(),
                label: "done".into(),
                preview: "done".into(),
            },
            owner_pid: 1,
            process_fingerprint: String::new(),
            agent_session_id: "a".into(),
            observed_at_ms: 1,
            pending_permissions: pending,
        }
    }

    fn ctx(event: HookEvent, tool: &str, payload: Value, observed_at_ms: u64) -> HookContext {
        HookContext {
            agent: "codex",
            session_name: "s".into(),
            pane_id: 1,
            event,
            raw_event: format!("{event:?}"),
            tool: tool.into(),
            payload,
            owner_pid: 1,
            agent_session_id: "a".into(),
            id: "id".into(),
            observed_at_ms,
        }
    }

    #[test]
    fn event_mapping_covers_attention_and_completion() {
        assert_eq!(
            event_state(HookEvent::PermissionRequest, "Bash", &json!({})),
            AgentState::Blocked
        );
        assert_eq!(
            event_state(HookEvent::PreToolUse, "AskUserQuestion", &json!({})),
            AgentState::Blocked
        );
        assert_eq!(
            event_state(HookEvent::UserPromptSubmit, "", &json!({})),
            AgentState::Working
        );
        assert_eq!(
            event_state(HookEvent::Stop, "", &json!({})),
            AgentState::Idle
        );
    }

    #[test]
    fn previews_are_rich_and_bounded() {
        let payload = json!({"tool_input": {"command": "cargo   test\n--all"}});
        let activity = activity_for(HookEvent::PreToolUse, "PreToolUse", "Bash", &payload, None);
        assert_eq!(activity.label, "using Bash");
        assert_eq!(activity.preview, "cargo test --all");
        assert!(truncate_preview(&"x".repeat(600)).chars().count() <= PREVIEW_LIMIT + 1);
    }

    #[test]
    fn permission_tracking_does_not_clear_unrelated_request() {
        let mut pending = Vec::new();
        update_pending_permissions(HookEvent::PermissionRequest, "Bash", &mut pending);
        update_pending_permissions(HookEvent::PermissionRequest, "apply_patch", &mut pending);
        update_pending_permissions(HookEvent::PostToolUse, "Bash", &mut pending);
        assert_eq!(pending, vec!["apply_patch"]);
    }

    #[test]
    fn seen_flag_tracks_whether_completion_was_already_observed() {
        let working = record(AgentState::Working, true, Vec::new());
        assert!(!compute_seen(AgentState::Idle, Some(&working)));

        let idle = record(AgentState::Idle, true, Vec::new());
        assert!(compute_seen(AgentState::Idle, Some(&idle)));
        assert!(compute_seen(AgentState::Idle, None));

        let unseen_done = record(AgentState::Idle, false, Vec::new());
        assert!(!compute_seen(AgentState::Working, Some(&unseen_done)));
    }

    #[test]
    fn session_end_removes_regardless_of_payload() {
        let ctx = ctx(HookEvent::SessionEnd, "", json!({"error": "boom"}), 5);
        assert!(matches!(decide(&ctx, None), RecordUpdate::Remove));
    }

    #[test]
    fn stale_event_is_ignored() {
        let previous = record(AgentState::Working, true, Vec::new());
        let ctx = ctx(HookEvent::Stop, "", json!({}), 0);
        assert!(matches!(decide(&ctx, Some(&previous)), RecordUpdate::Ignore));
    }

    #[test]
    fn leftover_pending_permission_escalates_to_blocked() {
        let previous = record(AgentState::Blocked, true, vec!["apply_patch".to_string()]);
        let ctx = ctx(HookEvent::PostToolUse, "Bash", json!({}), 10);
        match decide(&ctx, Some(&previous)) {
            RecordUpdate::Upsert(record) => {
                assert_eq!(record.state, AgentState::Blocked);
                assert_eq!(record.pending_permissions, vec!["apply_patch".to_string()]);
            }
            _ => panic!("expected an upsert"),
        }
    }
}
