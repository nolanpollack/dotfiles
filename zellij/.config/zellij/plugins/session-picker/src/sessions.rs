use std::collections::{BTreeMap, HashSet};

use zellij_tile::prelude::*;
use zellij_tile::prelude::SessionInfo as ZellijSessionInfo;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionInfo {
    pub name: String,
    pub is_current: bool,
    /// false for resurrectable sessions that aren't currently running.
    pub is_active: bool,
    /// Git branch checked out in the session's cwd, if resolved. `None` until a lookup
    /// completes (or if the session has no cwd/branch to report).
    pub branch: Option<String>,
    /// Absolute path to the session's repo's main checkout, if resolved. Shared by every
    /// worktree of the same repo.
    pub repo_root: Option<String>,
    /// True if this session's cwd is `repo_root` itself (the main checkout, not a worktree).
    pub is_main_checkout: bool,
    /// Set by `group_by_repo`: true if this session was nested under its repo's main-checkout
    /// session for display, and should be rendered indented.
    pub nested_worktree: bool,
}

pub fn fetch_sessions() -> Option<Vec<SessionInfo>> {
    let snapshot = get_session_list().ok()?;
    Some(sessions_from_snapshot(snapshot.live_sessions, snapshot.resurrectable_sessions))
}

/// Builds the picker's session list from the same (live, resurrectable) pair the host sends both
/// in `get_session_list`'s snapshot and in `Event::SessionUpdate` pushes.
pub fn sessions_from_snapshot(
    live_sessions: Vec<ZellijSessionInfo>,
    resurrectable_sessions: Vec<(String, std::time::Duration)>,
) -> Vec<SessionInfo> {
    let mut sessions: Vec<SessionInfo> = live_sessions
        .into_iter()
        .map(|s| SessionInfo {
            name: s.name,
            is_current: s.is_current_session,
            is_active: true,
            ..Default::default()
        })
        .collect();

    let mut resurrectable: Vec<SessionInfo> = resurrectable_sessions
        .into_iter()
        .map(|(name, _)| SessionInfo { name, is_active: false, ..Default::default() })
        .collect();
    resurrectable.sort_by(|a, b| a.name.cmp(&b.name));
    sessions.extend(resurrectable);

    sessions
}

/// Reorders `sessions` so each repo's worktrees sit directly after its main-checkout session
/// (sorted by name), marking them `nested_worktree` for indented display. Repos with no
/// main-checkout session currently open are left untouched — there's nothing to nest an orphaned
/// worktree under.
pub fn group_by_repo(sessions: Vec<SessionInfo>) -> Vec<SessionInfo> {
    let repos_with_main_checkout: HashSet<String> = sessions
        .iter()
        .filter(|s| s.is_main_checkout)
        .filter_map(|s| s.repo_root.clone())
        .collect();

    let mut worktrees_by_repo: BTreeMap<String, Vec<SessionInfo>> = BTreeMap::new();
    let mut remaining: Vec<SessionInfo> = Vec::new();
    for mut session in sessions {
        match &session.repo_root {
            Some(root) if !session.is_main_checkout && repos_with_main_checkout.contains(root) => {
                session.nested_worktree = true;
                worktrees_by_repo.entry(root.clone()).or_default().push(session);
            }
            _ => remaining.push(session),
        }
    }
    for siblings in worktrees_by_repo.values_mut() {
        siblings.sort_by(|a, b| a.name.cmp(&b.name));
    }

    let mut result = Vec::new();
    for session in remaining {
        let repo_with_worktrees = session
            .is_main_checkout
            .then(|| session.repo_root.clone())
            .flatten()
            .filter(|root| worktrees_by_repo.contains_key(root));
        result.push(session);
        if let Some(root) = repo_with_worktrees {
            if let Some(mut siblings) = worktrees_by_repo.remove(&root) {
                result.append(&mut siblings);
            }
        }
    }
    result
}
