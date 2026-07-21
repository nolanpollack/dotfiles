mod backend;
mod create;
mod git_info;
mod input;
mod picker;
mod sessions;
mod ui;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use create::CreateFlow;
use git_info::GitInfo;
use input::{key_to_action, Action};
use picker::Picker;
use sessions::{fetch_sessions, SessionInfo};
use zellij_tile::prelude::*;

fn default_zellij_style() -> Style {
    Style { colors: DEFAULT_STYLES, rounded_corners: true, hide_session_name: false }
}

const LIST_PANE_TITLE: &str = "Session Picker";
const CREATE_PANE_TITLE: &str = "New Session";

struct State {
    picker: Picker<SessionInfo>,
    theme: ui::Theme,
    theme_overrides: ui::ThemeOverrides,
    /// This plugin's own pane id, used to rename the pane's title while create-mode is active
    /// (see `Action::CreateNew`). Fetched once in `load`.
    plugin_id: u32,
    /// Draft text for an in-progress rename of the current session; `None` when not renaming.
    renaming: Option<String>,
    /// The screen create-mode is showing (type chooser, or a specific flow's form); `None` when
    /// not in create-mode. Mutually exclusive with `renaming` by convention, not by type.
    creating: Option<CreateFlow>,
    /// Resolved git info per session name; absent means unresolved (never looked up, still
    /// pending, or the lookup found nothing to report).
    git_info: BTreeMap<String, GitInfo>,
    /// Session names a git-info lookup has already been fired for, so we don't re-spawn one on
    /// every `SessionUpdate` push.
    git_lookups_started: BTreeSet<String>,
}

impl Default for State {
    fn default() -> Self {
        let theme_overrides = ui::ThemeOverrides::default();
        let theme = ui::Theme::from_zellij(&default_zellij_style(), &theme_overrides);
        Self {
            picker: Picker::new(|s: &SessionInfo| s.name.as_str()),
            theme,
            theme_overrides,
            plugin_id: 0,
            renaming: None,
            creating: None,
            git_info: BTreeMap::new(),
            git_lookups_started: BTreeSet::new(),
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: std::collections::BTreeMap<String, String>) {
        self.theme_overrides = ui::ThemeOverrides::from_config(&configuration);
        self.theme = ui::Theme::from_zellij(&default_zellij_style(), &self.theme_overrides);
        self.plugin_id = get_plugin_ids().plugin_id;
        rename_plugin_pane(self.plugin_id, LIST_PANE_TITLE);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::PermissionRequestResult,
            EventType::ModeUpdate,
            EventType::SessionUpdate,
            EventType::RunCommandResult,
            EventType::HostFolderChanged,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => self.refresh(),
            Event::ModeUpdate(info) => {
                self.theme = ui::Theme::from_zellij(&info.style, &self.theme_overrides);
                true
            }
            Event::Key(key) => {
                if let Some(flow) = &mut self.creating {
                    match create::apply_key(flow, &key, self.picker.items()) {
                        create::CreateOutcome::Continue => true,
                        create::CreateOutcome::Cancelled | create::CreateOutcome::Created => {
                            self.creating = None;
                            rename_plugin_pane(self.plugin_id, LIST_PANE_TITLE);
                            true
                        }
                    }
                } else if self.renaming.is_some() {
                    self.apply_rename_key(&key)
                } else {
                    key_to_action(&key).map(|a| self.apply_action(a)).unwrap_or(false)
                }
            }
            Event::SessionUpdate(live_sessions, resurrectable_sessions) => {
                self.set_sessions(sessions::sessions_from_snapshot(live_sessions, resurrectable_sessions))
            }
            Event::RunCommandResult(_exit_code, stdout, _stderr, context) => {
                if self.apply_git_info_result(&context, &stdout) {
                    true
                } else if let Some(flow) = &mut self.creating {
                    create::apply_discovery_result(flow, &context, &stdout)
                } else {
                    false
                }
            }
            Event::HostFolderChanged(new_cwd) => self.refresh_current_git_info(new_cwd),
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if let Some(flow) = &self.creating {
            ui::render_create(flow, &self.theme, rows, cols);
        } else if let Some(draft) = &self.renaming {
            let hints = [("enter", "confirm rename"), ("esc", "cancel")];
            ui::render(self.picker.view(), &self.theme, &hints, Some(draft.as_str()), rows, cols);
        } else {
            let hints: Vec<_> = input::hints().collect();
            ui::render(self.picker.view(), &self.theme, &hints, None, rows, cols);
        }
    }
}

impl State {
    fn refresh(&mut self) -> bool {
        let Some(new_sessions) = fetch_sessions() else {
            return false;
        };
        self.set_sessions(new_sessions)
    }

    fn set_sessions(&mut self, mut new_sessions: Vec<SessionInfo>) -> bool {
        for session in &mut new_sessions {
            if session.is_active {
                self.ensure_git_info_lookup(&session.name);
            }
            if let Some(info) = self.git_info.get(&session.name) {
                session.branch = info.branch.clone();
                session.repo_root = info.repo_root.clone();
                session.is_main_checkout = info.is_main_checkout;
            }
        }
        let new_sessions = sessions::group_by_repo(new_sessions);
        if new_sessions == self.picker.items() {
            return false;
        }
        self.picker.set_items(new_sessions);
        true
    }

    /// Fires a git-info lookup for `name`'s cwd, at most once per session name.
    fn ensure_git_info_lookup(&mut self, name: &str) {
        if self.git_lookups_started.insert(name.to_string()) {
            git_info::spawn_lookup_by_name(name);
        }
    }

    /// Applies a git-info lookup's result (if any) to the cache and current picker items.
    fn apply_git_info_result(&mut self, context: &BTreeMap<String, String>, stdout: &[u8]) -> bool {
        let Some((name, info)) = git_info::parse_result(context, stdout) else {
            return false;
        };
        self.git_info.insert(name, info);
        let items = self.picker.items().to_vec();
        self.set_sessions(items)
    }

    /// Re-resolves the current session's git info directly against its new cwd, bypassing the
    /// on-disk layout cache (which may not have caught up yet).
    fn refresh_current_git_info(&mut self, new_cwd: PathBuf) -> bool {
        let Some(name) = self.picker.items().iter().find(|s| s.is_current).map(|s| s.name.clone())
        else {
            return false;
        };
        git_info::spawn_lookup_at_dir(&name, new_cwd);
        false
    }

    fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::MoveDown => {
                self.picker.move_down();
                true
            }
            Action::MoveUp => {
                self.picker.move_up();
                true
            }
            Action::PushChar(c) => {
                self.picker.push_char(c);
                true
            }
            Action::PopChar => {
                self.picker.pop_char();
                true
            }
            Action::Delete => {
                if let Some(session) = self.picker.selected_item() {
                    if session.is_current {
                        return false;
                    }
                    if session.is_active {
                        kill_sessions(&[session.name.clone()]).ok();
                    } else {
                        delete_dead_session(&session.name).ok();
                    }
                    return self.refresh();
                }
                false
            }
            Action::Confirm => {
                if let Some(session) = self.picker.selected_item() {
                    switch_session_with_focus(&session.name, None, None);
                }
                close_self();
                false
            }
            Action::Rename => {
                let current_name = self.picker.items().iter().find(|s| s.is_current).map(|s| s.name.clone());
                if let Some(name) = current_name {
                    self.picker.clear_query();
                    self.renaming = Some(name);
                    true
                } else {
                    false
                }
            }
            Action::CreateNew => {
                self.creating = Some(CreateFlow::new());
                rename_plugin_pane(self.plugin_id, CREATE_PANE_TITLE);
                true
            }
            // Tab has no meaning on the main list; it's create-mode's field-navigation key.
            Action::NextField => false,
            Action::Cancel => {
                close_self();
                false
            }
        }
    }

    fn apply_rename_key(&mut self, key: &KeyWithModifier) -> bool {
        let Some(draft) = self.renaming.as_mut() else {
            return false;
        };
        match key.bare_key {
            BareKey::Esc => {
                self.renaming = None;
                true
            }
            BareKey::Enter => {
                let new_name = draft.clone();
                self.renaming = None;
                let old_name = self.picker.items().iter().find(|s| s.is_current).map(|s| s.name.clone());
                if !new_name.is_empty() && old_name.as_deref() != Some(new_name.as_str()) {
                    rename_session(&new_name);
                }
                true
            }
            BareKey::Backspace => {
                draft.pop();
                true
            }
            BareKey::Char(c) if !key.key_modifiers.contains(&KeyModifier::Ctrl) => {
                draft.push(c);
                true
            }
            _ => false,
        }
    }
}
