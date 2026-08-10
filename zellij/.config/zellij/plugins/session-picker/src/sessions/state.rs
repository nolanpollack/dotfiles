//! Persistent session data, Git enrichment, and asynchronous lookup state.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::effects::GitLookup;
use crate::git_info::GitInfo;
use crate::sessions::{self, Session};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitStatus {
    NotRequested,
    Loading { cached: Option<CachedGit> },
    Loaded(CachedGit),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedGit {
    pub info: Option<GitInfo>,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsSnapshot {
    #[serde(default)]
    pub sessions: Vec<Session>,
    #[serde(default)]
    pub git: BTreeMap<String, CachedGit>,
}

pub struct SessionsUpdate {
    pub changed: bool,
    pub lookups: Vec<GitLookup>,
}

#[derive(Default)]
pub struct Sessions {
    items: Vec<Session>,
    git: BTreeMap<String, GitStatus>,
}

impl Sessions {
    const GIT_FRESHNESS_MS: u64 = 5 * 60 * 1_000;

    pub fn from_snapshot(snapshot: SessionsSnapshot) -> Self {
        Self {
            items: snapshot.sessions,
            git: snapshot
                .git
                .into_iter()
                .map(|(name, cached)| (name, GitStatus::Loaded(cached)))
                .collect(),
        }
    }

    pub fn snapshot(&self) -> SessionsSnapshot {
        SessionsSnapshot {
            sessions: self.items.clone(),
            git: self
                .git
                .iter()
                .filter_map(|(name, status)| {
                    status
                        .cached()
                        .cloned()
                        .map(|cached| (name.clone(), cached))
                })
                .collect(),
        }
    }

    pub fn replace(&mut self, mut incoming: Vec<Session>) -> SessionsUpdate {
        self.git
            .retain(|name, _| incoming.iter().any(|session| &session.name == name));

        let mut lookups = Vec::new();
        for session in &mut incoming {
            session.branch = None;
            session.repo_root = None;
            session.is_main_worktree = false;

            let status = self
                .git
                .entry(session.name.clone())
                .or_insert(GitStatus::NotRequested);
            if session.is_active() {
                let should_lookup = match status {
                    GitStatus::NotRequested => {
                        *status = GitStatus::Loading { cached: None };
                        true
                    }
                    GitStatus::Loaded(cached)
                        if now_ms().saturating_sub(cached.observed_at_ms)
                            >= Self::GIT_FRESHNESS_MS =>
                    {
                        *status = GitStatus::Loading {
                            cached: Some(cached.clone()),
                        };
                        true
                    }
                    _ => false,
                };
                if should_lookup {
                    lookups.push(GitLookup::BySessionName {
                        session_name: session.name.clone(),
                    });
                }
            }
            if let Some(info) = status.cached().and_then(|cached| cached.info.as_ref()) {
                session.branch = info.branch.clone();
                session.repo_root = info.repo_root.clone();
                session.is_main_worktree = info.is_main_worktree;
            }
        }

        let incoming = sessions::group_by_repo(incoming);
        let changed = incoming != self.items;
        if changed {
            self.items = incoming;
        }
        SessionsUpdate { changed, lookups }
    }

    pub fn apply_git(&mut self, session_name: String, info: GitInfo) -> SessionsUpdate {
        let info = (info != GitInfo::default()).then_some(info);
        self.git.insert(
            session_name,
            GitStatus::Loaded(CachedGit {
                info,
                observed_at_ms: now_ms(),
            }),
        );
        let current = self.items.clone();
        self.replace(current)
    }

    pub fn lookup_current(&self, cwd: PathBuf) -> Option<GitLookup> {
        self.items
            .iter()
            .find(|session| session.is_current())
            .map(|session| GitLookup::AtDirectory {
                session_name: session.name.clone(),
                cwd,
            })
    }

    pub fn items(&self) -> &[Session] {
        &self.items
    }

    #[cfg(test)]
    pub(crate) fn git_status(&self, name: &str) -> Option<&GitStatus> {
        self.git.get(name)
    }
}

impl GitStatus {
    fn cached(&self) -> Option<&CachedGit> {
        match self {
            Self::Loading { cached } => cached.as_ref(),
            Self::Loaded(cached) => Some(cached),
            Self::NotRequested => None,
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::SessionLifecycle;

    fn active(name: &str) -> Session {
        Session {
            name: name.into(),
            lifecycle: SessionLifecycle::Active { current: false },
            ..Default::default()
        }
    }

    #[test]
    fn lookup_state_is_explicit_and_not_restarted() {
        let mut sessions = Sessions::default();
        let first = sessions.replace(vec![active("one")]);
        assert_eq!(first.lookups.len(), 1);
        assert!(matches!(
            sessions.git_status("one"),
            Some(GitStatus::Loading { cached: None })
        ));
        let second = sessions.replace(vec![active("one")]);
        assert!(second.lookups.is_empty());
    }

    #[test]
    fn removed_sessions_are_pruned() {
        let mut sessions = Sessions::default();
        sessions.replace(vec![active("gone")]);
        sessions.replace(vec![active("kept")]);
        assert_eq!(sessions.git_status("gone"), None);
    }

    #[test]
    fn empty_git_result_is_a_loaded_result() {
        let mut sessions = Sessions::default();
        sessions.replace(vec![active("one")]);
        sessions.apply_git("one".into(), GitInfo::default());
        assert!(matches!(
            sessions.git_status("one"),
            Some(GitStatus::Loaded(CachedGit { info: None, .. }))
        ));
        assert!(sessions.replace(vec![active("one")]).lookups.is_empty());
    }

    #[test]
    fn snapshot_restores_fresh_git_without_restarting_lookup() {
        let mut sessions = Sessions::default();
        sessions.replace(vec![active("one")]);
        sessions.apply_git(
            "one".into(),
            GitInfo {
                branch: Some("cached".into()),
                ..Default::default()
            },
        );

        let mut restored = Sessions::from_snapshot(sessions.snapshot());
        let update = restored.replace(vec![active("one")]);
        assert!(update.lookups.is_empty());
        assert_eq!(restored.items()[0].branch.as_deref(), Some("cached"));
    }
}
