use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionLifecycle {
    Active {
        current: bool,
    },
    #[default]
    Resurrectable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub name: String,
    pub lifecycle: SessionLifecycle,
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

impl SessionInfo {
    pub fn is_active(&self) -> bool {
        matches!(self.lifecycle, SessionLifecycle::Active { .. })
    }

    pub fn is_current(&self) -> bool {
        matches!(self.lifecycle, SessionLifecycle::Active { current: true })
    }
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
                worktrees_by_repo
                    .entry(root.clone())
                    .or_default()
                    .push(session);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn session(name: &str, root: &str, main: bool) -> SessionInfo {
        SessionInfo {
            name: name.into(),
            lifecycle: SessionLifecycle::Active { current: false },
            repo_root: Some(root.into()),
            is_main_checkout: main,
            ..Default::default()
        }
    }

    #[test]
    fn lifecycle_cannot_represent_current_resurrectable_session() {
        let dead = SessionInfo::default();
        assert!(!dead.is_active());
        assert!(!dead.is_current());
    }

    #[test]
    fn worktrees_follow_their_main_checkout() {
        let grouped = group_by_repo(vec![
            session("feature", "/repo", false),
            session("main", "/repo", true),
        ]);
        assert_eq!(
            grouped
                .iter()
                .map(|session| session.name.as_str())
                .collect::<Vec<_>>(),
            vec!["main", "feature"]
        );
        assert!(grouped[1].nested_worktree);
    }
}
