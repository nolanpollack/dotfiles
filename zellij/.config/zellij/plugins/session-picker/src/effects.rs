//! Requests for work at the Zellij boundary.
//!
//! State and flow code produce these values; the plugin adapter will execute them. Keeping the
//! payloads as application data lets transitions be tested without loading Zellij.

use std::path::PathBuf;

use crate::agent_refresh::RequestId;
use crate::picker_refresh::RefreshId;
use crate::sessions::SessionLifecycle;

pub enum Effect {
    RefreshSessions,
    RefreshPickerSessions {
        refresh_id: RefreshId,
    },
    LookupGit(GitLookup),
    FetchDirectoryCandidates,
    ValidateWorktree {
        request: crate::create::worktree::Request,
    },
    CreateWorktree {
        request: crate::create::worktree::Request,
    },
    SwitchSession {
        name: String,
    },
    SwitchToAgent {
        session_name: String,
        pane_id: u32,
    },
    FetchAgents {
        bridge: String,
        request_id: RequestId,
    },
    MarkAgentSeen {
        bridge: String,
        id: String,
    },
    RenameAgentSession {
        bridge: String,
        old: String,
        new: String,
    },
    CreateSession {
        name: String,
        cwd: PathBuf,
    },
    RenameCurrentSession {
        name: String,
    },
    DeleteSession {
        name: String,
        lifecycle: SessionLifecycle,
    },
    RenamePluginPane {
        title: &'static str,
    },
    HidePlugin,
    ScheduleAnimationFrame,
}

pub enum GitLookup {
    BySessionName { session_name: String },
    AtDirectory { session_name: String, cwd: PathBuf },
}
