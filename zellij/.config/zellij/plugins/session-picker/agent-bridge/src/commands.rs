use std::process::{Command, Stdio};

use agent_core::AgentList;

use crate::hook;
use crate::{agents, aliases, process, store};

// The picker plugin can be open in a session other than the one whose hook just fired, and one
// running zellij session can't see another's plugin instances, so this notifies every session's
// server and lets each one's plugin (if not running there) ignore it. Best-effort: a session may
// have exited between `list-sessions` and the pipe call, or have no zellij server at all (e.g.
// under `cargo test`), so failures here are silently ignored.
fn notify_picker() {
    // Plain (non `--short`) output tags exited-but-resurrectable sessions with "EXITED"; skip
    // those since there is no running server to pipe to.
    let Ok(output) = Command::new("zellij")
        .args(["list-sessions", "--no-formatting"])
        .stdin(Stdio::null())
        .output()
    else {
        return;
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.contains("EXITED") {
            continue;
        }
        let Some(session) = line.split_whitespace().next() else {
            continue;
        };
        // Fire-and-forget: don't block hook processing on the plugin's response.
        let _ = Command::new("zellij")
            .args([
                "--session",
                session,
                "action",
                "pipe",
                "--plugin",
                "session-picker",
                "--name",
                "agent-refresh",
                "--",
                "",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

pub(crate) fn parse_agent(value: Option<&str>) -> Result<&'static str, String> {
    value
        .and_then(agents::find)
        .map(|a| a.label())
        .ok_or_else(|| "agent must be codex or claude".into())
}

pub(crate) fn ingest_hook(agent: &'static str) -> Result<(), String> {
    let Some(ctx) = hook::HookContext::gather(agent)? else {
        return Ok(());
    };
    let session_name = ctx.session_name.clone();
    let pane_id = ctx.pane_id;

    let changed = store::with_lock(|| {
        let path = store::record_path(&ctx.id)?;
        let previous = store::read_record(&path).ok();
        match hook::decide(&ctx, previous.as_ref()) {
            hook::RecordUpdate::Ignore => Ok(false),
            hook::RecordUpdate::Remove => store::remove_matching(|r| r.id == ctx.id).map(|_| true),
            hook::RecordUpdate::Upsert(record) => {
                // A pane can host only one top-level instance of a particular agent at a time.
                store::remove_matching(|r| {
                    r.agent_label == agent
                        && r.target.session_name == session_name
                        && r.target.pane_id == pane_id
                        && r.id != ctx.id
                })?;
                store::write_record(&path, &record)?;
                Ok(true)
            }
        }
    })?;
    if changed {
        notify_picker();
    }
    Ok(())
}

pub(crate) fn list_agents() -> Result<(), String> {
    let agents = store::with_lock(|| {
        let mut agents = Vec::new();
        for path in store::record_paths()? {
            let Ok(record) = store::read_record(&path) else {
                let _ = std::fs::remove_file(path);
                continue;
            };
            if process::process_is_alive(&record) {
                agents.push(record);
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
        agents.sort_by(|a, b| {
            (&a.target.session_name, a.target.pane_id, &a.agent_label).cmp(&(
                &b.target.session_name,
                b.target.pane_id,
                &b.agent_label,
            ))
        });
        Ok(agents)
    })?;
    println!(
        "{}",
        serde_json::to_string(&AgentList::new(agents)).map_err(|e| e.to_string())?
    );
    Ok(())
}

pub(crate) fn mark_seen(id: String) -> Result<(), String> {
    store::with_lock(|| {
        let path = store::record_path(&id)?;
        let mut record = store::read_record(&path)?;
        record.seen = true;
        store::write_record(&path, &record)
    })?;
    notify_picker();
    Ok(())
}

pub(crate) fn rename_session(old: &str, new: &str) -> Result<(), String> {
    store::with_lock(|| {
        let mut map = aliases::read_aliases();
        for target in map.values_mut() {
            if target == old {
                *target = new.to_string();
            }
        }
        map.insert(old.to_string(), new.to_string());
        aliases::write_aliases(&map)?;
        for path in store::record_paths()? {
            let Ok(mut record) = store::read_record(&path) else {
                continue;
            };
            if record.target.session_name == old {
                record.target.session_name = new.to_string();
                store::write_record(&path, &record)?;
            }
        }
        Ok(())
    })?;
    notify_picker();
    Ok(())
}
