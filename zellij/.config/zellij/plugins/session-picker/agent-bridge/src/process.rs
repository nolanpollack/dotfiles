use std::io;

use agent_core::Agent;

/// `0` denotes a pane-owned record. Hook lifecycle events and SessionEnd manage it, while the
/// picker filters it to active Zellij sessions. Do not mistake a hook subprocess for its owner.
pub(crate) fn process_is_alive(record: &Agent) -> bool {
    if record.owner_pid == 0 {
        return true;
    }
    let result = unsafe { libc::kill(record.owner_pid as i32, 0) };
    result == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Activity, AgentState, AgentTarget};

    #[test]
    fn liveness_falls_back_when_process_metadata_is_unavailable() {
        let record = Agent {
            id: "id".into(),
            agent_label: "codex".into(),
            state: AgentState::Working,
            seen: true,
            target: AgentTarget {
                session_name: "s".into(),
                pane_id: 1,
            },
            activity: Activity {
                kind: "tool".into(),
                label: "tool".into(),
                preview: "tool".into(),
            },
            owner_pid: std::process::id(),
            process_fingerprint: String::new(),
            agent_session_id: "a".into(),
            observed_at_ms: 1,
            pending_permissions: Vec::new(),
        };
        assert!(process_is_alive(&record));
    }

    #[test]
    fn pane_owned_record_is_not_pruned_as_a_dead_hook_process() {
        let record = Agent {
            id: "id".into(),
            agent_label: "claude".into(),
            state: AgentState::Working,
            seen: true,
            target: AgentTarget {
                session_name: "s".into(),
                pane_id: 1,
            },
            activity: Activity {
                kind: "prompt".into(),
                label: "responding".into(),
                preview: "test".into(),
            },
            owner_pid: 0,
            process_fingerprint: String::new(),
            agent_session_id: "a".into(),
            observed_at_ms: 1,
            pending_permissions: Vec::new(),
        };
        assert!(process_is_alive(&record));
    }
}
