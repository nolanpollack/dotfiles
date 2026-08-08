//! Session ownership, filtering, ordering, and asynchronous git enrichment.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::effects::GitLookup;
use crate::git_info::GitInfo;
use crate::picker::{Picker, PickerState, View};
use crate::sessions::{self, SessionInfo};

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
pub struct SessionCatalogSnapshot {
    #[serde(default)]
    pub sessions: Vec<SessionInfo>,
    #[serde(default)]
    pub git: BTreeMap<String, CachedGit>,
}

pub struct CatalogUpdate {
    pub changed: bool,
    pub lookups: Vec<GitLookup>,
}

pub struct SessionCatalog {
    picker: Picker<SessionInfo>,
    git: BTreeMap<String, GitStatus>,
}

impl Default for SessionCatalog {
    fn default() -> Self {
        Self {
            picker: Picker::new(|session: &SessionInfo| session.name.as_str()),
            git: BTreeMap::new(),
        }
    }
}

impl SessionCatalog {
    const GIT_FRESHNESS_MS: u64 = 5 * 60 * 1_000;

    pub fn from_snapshot(snapshot: SessionCatalogSnapshot) -> Self {
        let mut picker = Picker::new(|session: &SessionInfo| session.name.as_str());
        picker.set_items(snapshot.sessions);
        Self {
            picker,
            git: snapshot
                .git
                .into_iter()
                .map(|(name, cached)| (name, GitStatus::Loaded(cached)))
                .collect(),
        }
    }

    pub fn snapshot(&self) -> SessionCatalogSnapshot {
        SessionCatalogSnapshot {
            sessions: self.picker.items().to_vec(),
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

    pub fn replace(
        &mut self,
        picker_state: &mut PickerState,
        mut incoming: Vec<SessionInfo>,
    ) -> CatalogUpdate {
        self.git
            .retain(|name, _| incoming.iter().any(|session| &session.name == name));

        let mut lookups = Vec::new();
        for session in &mut incoming {
            session.branch = None;
            session.repo_root = None;
            session.is_main_checkout = false;
            session.nested_worktree = false;

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
                session.is_main_checkout = info.is_main_checkout;
            }
        }

        let incoming = sessions::group_by_repo(incoming);
        let changed = incoming != self.picker.items();
        if changed {
            self.picker.set_items(incoming);
            self.picker.clamp(picker_state);
        }
        CatalogUpdate { changed, lookups }
    }

    pub fn apply_git(
        &mut self,
        picker_state: &mut PickerState,
        session_name: String,
        info: GitInfo,
    ) -> CatalogUpdate {
        let info = (info != GitInfo::default()).then_some(info);
        self.git.insert(
            session_name,
            GitStatus::Loaded(CachedGit {
                info,
                observed_at_ms: now_ms(),
            }),
        );
        self.replace(picker_state, self.picker.items().to_vec())
    }

    pub fn lookup_current(&self, cwd: PathBuf) -> Option<GitLookup> {
        self.items()
            .iter()
            .find(|session| session.is_current())
            .map(|session| GitLookup::AtDirectory {
                session_name: session.name.clone(),
                cwd,
            })
    }

    pub fn items(&self) -> &[SessionInfo] {
        self.picker.items()
    }

    pub fn selected(&self, picker_state: &PickerState) -> Option<&SessionInfo> {
        self.picker.selected_item(picker_state)
    }

    pub fn view<'a>(&'a self, picker_state: &'a PickerState) -> View<'a, SessionInfo> {
        self.picker.view(picker_state)
    }

    pub fn move_up(&self, picker_state: &mut PickerState) {
        self.picker.move_up(picker_state);
    }

    pub fn move_down(&self, picker_state: &mut PickerState) {
        self.picker.move_down(picker_state);
    }

    pub fn push_char(&self, picker_state: &mut PickerState, character: char) {
        self.picker.push_char(picker_state, character);
    }

    pub fn pop_char(&self, picker_state: &mut PickerState) {
        self.picker.pop_char(picker_state);
    }

    pub fn clear_query(&self, picker_state: &mut PickerState) {
        self.picker.clear_query(picker_state);
    }

    #[cfg(test)]
    fn git_status(&self, name: &str) -> Option<&GitStatus> {
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

    fn active(name: &str) -> SessionInfo {
        SessionInfo {
            name: name.into(),
            lifecycle: SessionLifecycle::Active { current: false },
            ..Default::default()
        }
    }

    #[test]
    fn lookup_state_is_explicit_and_not_restarted() {
        let mut catalog = SessionCatalog::default();
        let mut picker_state = PickerState::default();
        let first = catalog.replace(&mut picker_state, vec![active("one")]);
        assert_eq!(first.lookups.len(), 1);
        assert!(matches!(
            catalog.git_status("one"),
            Some(GitStatus::Loading { cached: None })
        ));
        let second = catalog.replace(&mut picker_state, vec![active("one")]);
        assert!(second.lookups.is_empty());
    }

    #[test]
    fn removed_sessions_are_pruned() {
        let mut catalog = SessionCatalog::default();
        let mut picker_state = PickerState::default();
        catalog.replace(&mut picker_state, vec![active("gone")]);
        catalog.replace(&mut picker_state, vec![active("kept")]);
        assert_eq!(catalog.git_status("gone"), None);
    }

    #[test]
    fn empty_git_result_is_a_loaded_result() {
        let mut catalog = SessionCatalog::default();
        let mut picker_state = PickerState::default();
        catalog.replace(&mut picker_state, vec![active("one")]);
        catalog.apply_git(&mut picker_state, "one".into(), GitInfo::default());
        assert!(matches!(
            catalog.git_status("one"),
            Some(GitStatus::Loaded(CachedGit { info: None, .. }))
        ));
        assert!(catalog
            .replace(&mut picker_state, vec![active("one")])
            .lookups
            .is_empty());
    }

    #[test]
    fn snapshot_restores_fresh_git_without_restarting_lookup() {
        let mut catalog = SessionCatalog::default();
        let mut picker_state = PickerState::default();
        catalog.replace(&mut picker_state, vec![active("one")]);
        catalog.apply_git(
            &mut picker_state,
            "one".into(),
            GitInfo {
                branch: Some("cached".into()),
                ..Default::default()
            },
        );

        let mut restored = SessionCatalog::from_snapshot(catalog.snapshot());
        let update = restored.replace(&mut PickerState::default(), vec![active("one")]);
        assert!(update.lookups.is_empty());
        assert_eq!(restored.items()[0].branch.as_deref(), Some("cached"));
    }
}
