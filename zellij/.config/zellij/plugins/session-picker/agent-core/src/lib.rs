use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

/// Raw lifecycle status derived directly from the agent's own hook events.
/// `Blocked` means the agent is waiting on the user (permission request,
/// clarifying question); it takes priority over the other states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Blocked,
    Working,
    Idle,
    Unknown,
}

/// The Zellij session/pane a given agent is running in, i.e. where to jump
/// to if the user selects it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTarget {
    pub session_name: String,
    pub pane_id: u32,
}

/// Human-readable description of what the agent is currently doing,
/// derived from the most recent hook payload (e.g. kind "tool_use",
/// label "using Bash", preview "cargo test").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub kind: String,
    pub label: String,
    pub preview: String,
}

/// A snapshot of one running (or just-finished) agent, as written by
/// `agent-bridge` in response to a hook event and read back by the plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    /// Name of the coding agent that produced this record, e.g. "codex" or "claude". Just a
    /// label set by whichever agent-bridge integration wrote the record.
    pub agent_label: String,
    pub state: AgentState,
    /// Whether the user has already viewed this record's current state.
    /// Distinguishes a freshly-finished agent (`Idle` + unseen -> `Done`)
    /// from one whose completion has already been acknowledged.
    pub seen: bool,
    pub target: AgentTarget,
    pub activity: Activity,
    /// PID that owns this record's lifetime, or 0 if the record is
    /// pane-owned instead (kept alive as long as the pane exists, rather
    /// than tied to a specific hook subprocess).
    pub owner_pid: u32,
    pub process_fingerprint: String,
    /// Opaque ID from the agent's own session, used to correlate hook
    /// events belonging to the same run.
    pub agent_session_id: String,
    pub observed_at_ms: u64,
    /// Tool permission requests currently awaiting a user decision; any
    /// entries here force the display state to `Blocked`.
    #[serde(default)]
    pub pending_permissions: Vec<String>,
}

/// Wire envelope printed by `agent-bridge list` and parsed by the plugin;
/// `schema_version` lets the plugin reject records from an incompatible
/// bridge build instead of silently misparsing them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentList {
    pub schema_version: u32,
    pub agents: Vec<Agent>,
}

impl AgentList {
    pub fn new(agents: Vec<Agent>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            agents,
        }
    }
}

/// Derives the UI-facing status from raw state plus the `seen` flag,
/// splitting `Idle` into `Done` (just finished, not yet acknowledged) vs
/// `Idle` (finished and already seen).
pub fn display_state(record: &Agent) -> DisplayState {
    match (record.state, record.seen) {
        (AgentState::Blocked, _) => DisplayState::Blocked,
        (AgentState::Working, _) => DisplayState::Working,
        (AgentState::Idle, false) => DisplayState::Done,
        (AgentState::Idle, true) => DisplayState::Idle,
        (AgentState::Unknown, _) => DisplayState::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayState {
    Blocked,
    Working,
    Done,
    Idle,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(state: AgentState, seen: bool) -> Agent {
        Agent {
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
            process_fingerprint: "p".into(),
            agent_session_id: "a".into(),
            observed_at_ms: 1,
            pending_permissions: Vec::new(),
        }
    }

    #[test]
    fn unseen_idle_is_done() {
        assert_eq!(
            display_state(&record(AgentState::Idle, false)),
            DisplayState::Done
        );
        assert_eq!(
            display_state(&record(AgentState::Idle, true)),
            DisplayState::Idle
        );
    }
}
