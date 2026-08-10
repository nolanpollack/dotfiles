//! Host-independent application state and transitions.

use std::path::PathBuf;

use agent_core::Agent;
use serde::{Deserialize, Serialize};

use crate::agent_refresh::{AgentRefresh, RequestId};
use crate::agents::Agents;
use crate::create::{self, CreateFlow};
use crate::effects::Effect;
use crate::git_info::GitInfo;
use crate::input::{self, Key, ListAction, RenameAction};
use crate::picker_refresh::{PickerRefresh, RefreshId};
use crate::sessions::{Session, Sessions, SessionsSnapshot, SessionsUpdate};
use crate::ui::screens::list::{Destination, Transition};
use crate::ui::{self, Screen, Ui};

const LIST_PANE_TITLE: &str = "Session Picker";
const CREATE_PANE_TITLE: &str = "New Session";

/// Runtime configuration is intentionally excluded from `AppSnapshot`, so current host
/// configuration wins when restoring persisted state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub agent_bridge: String,
    pub worktree: create::worktree::Config,
}

pub enum Message {
    PermissionGranted,
    ThemeChanged(ui::Theme),
    Key(Key),
    ListAction(ListAction),
    SessionsLoaded(Vec<Session>),
    PickerSessionsFinished {
        refresh_id: RefreshId,
        result: Result<Vec<Session>, ()>,
    },
    AgentsFetchFinished {
        request_id: RequestId,
        result: Result<Vec<Agent>, ()>,
    },
    GitLoaded {
        session_name: String,
        info: GitInfo,
    },
    DirectoryCandidatesLoaded(Vec<PathBuf>),
    WorktreeValidationFinished {
        result: Result<(), String>,
    },
    WorktreeCreationFinished {
        result: Result<(), String>,
    },
    HostFolderChanged(PathBuf),
    VisibilityChanged(bool),
    ExternalAgentUpdate,
    AnimationFrame,
}

#[derive(Default)]
pub struct Update {
    pub redraw: bool,
    pub effects: Vec<Effect>,
}

impl Update {
    fn redraw() -> Self {
        Self {
            redraw: true,
            effects: Vec::new(),
        }
    }

    fn effects(effects: impl IntoIterator<Item = Effect>) -> Self {
        Self {
            redraw: false,
            effects: effects.into_iter().collect(),
        }
    }

    fn redraw_with(effects: impl IntoIterator<Item = Effect>) -> Self {
        Self {
            redraw: true,
            effects: effects.into_iter().collect(),
        }
    }
}

pub struct App {
    pub(crate) sessions: Sessions,
    pub(crate) agents: Agents,
    pub(crate) ui: Ui,
    config: AppConfig,
    pub(crate) visible: bool,
    pub(crate) agent_refresh: AgentRefresh,
    agent_refresh_id: Option<RefreshId>,
    pub(crate) picker_refresh: PickerRefresh,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSnapshot {
    #[serde(default)]
    pub sessions: SessionsSnapshot,
    #[serde(default)]
    pub agents: Option<Vec<Agent>>,
}

impl App {
    pub fn new(theme: ui::Theme, config: AppConfig) -> Self {
        Self {
            sessions: Sessions::default(),
            agents: Agents::default(),
            ui: Ui::new(theme),
            config,
            visible: false,
            agent_refresh: AgentRefresh::default(),
            agent_refresh_id: None,
            picker_refresh: PickerRefresh::default(),
        }
    }

    pub fn restore(theme: ui::Theme, config: AppConfig, snapshot: AppSnapshot) -> Self {
        let mut app = Self::new(theme, config);
        app.sessions = Sessions::from_snapshot(snapshot.sessions);
        app.ui.update_sessions(
            app.sessions.items(),
            SessionsUpdate {
                changed: true,
                lookups: Vec::new(),
            },
        );
        if let Some(agents) = snapshot.agents {
            app.agents
                .replace(agents, &active_session_order(app.sessions.items()));
            app.ui.sync_agent_state(app.agents.items().len());
            app.agent_refresh.restore_cached();
        }
        app
    }

    pub fn persistent_state(&self) -> AppSnapshot {
        AppSnapshot {
            sessions: self.sessions.snapshot(),
            agents: self
                .agent_refresh
                .has_cached_data()
                .then(|| self.agents.items().to_vec()),
        }
    }

    pub fn initial_update(&self) -> Update {
        Update::effects([Effect::RenamePluginPane {
            title: LIST_PANE_TITLE,
        }])
    }

    pub fn update(&mut self, message: Message) -> Update {
        match message {
            Message::PermissionGranted => {
                self.agent_refresh.grant_permissions();
                // Zellij does not reliably emit `Visible(true)` when a plugin is started or
                // reloaded through an action. Treat the initial permission grant as visible so
                // agent status is fetched on first render; later `Visible(false)` events still
                // stop refresh work when the picker is hidden.
                self.visibility_changed(true)
            }
            Message::ThemeChanged(theme) => {
                self.ui.set_theme(theme);
                Update::redraw()
            }
            Message::Key(key) => self.apply_key(key),
            Message::ListAction(action) if matches!(self.ui.state.screen, Screen::List) => {
                self.apply_list_action(action)
            }
            Message::ListAction(_) => Update::default(),
            Message::SessionsLoaded(sessions) => {
                let sessions_update = self.sessions.replace(sessions);
                self.retain_agents();
                let transition = self
                    .ui
                    .update_sessions(self.sessions.items(), sessions_update);
                self.finish_list_transition(transition)
            }
            Message::PickerSessionsFinished { refresh_id, result } => {
                self.finish_picker_sessions(refresh_id, result)
            }
            Message::AgentsFetchFinished { request_id, result } => {
                self.finish_agent_fetch(request_id, result)
            }
            Message::GitLoaded { session_name, info } => {
                let sessions_update = self.sessions.apply_git(session_name, info);
                self.retain_agents();
                let transition = self
                    .ui
                    .update_sessions(self.sessions.items(), sessions_update);
                self.finish_list_transition(transition)
            }
            Message::DirectoryCandidatesLoaded(paths) => {
                if let Screen::Create(flow) = &mut self.ui.state.screen {
                    create::set_directory_candidates(flow, paths);
                    Update::redraw()
                } else {
                    Update::default()
                }
            }
            Message::WorktreeValidationFinished { result } => {
                self.finish_worktree_validation(result)
            }
            Message::WorktreeCreationFinished { result } => self.finish_worktree_creation(result),
            Message::HostFolderChanged(cwd) => self
                .sessions
                .lookup_current(cwd)
                .map(|lookup| Update::effects([Effect::LookupGit(lookup)]))
                .unwrap_or_default(),
            Message::VisibilityChanged(visible) => self.visibility_changed(visible),
            Message::ExternalAgentUpdate => self.external_agent_update(),
            Message::AnimationFrame => self.animation_frame(),
        }
    }

    pub fn theme(&self) -> &ui::Theme {
        self.ui.theme()
    }

    fn apply_key(&mut self, key: Key) -> Update {
        match &mut self.ui.state.screen {
            Screen::List => input::list_action(key)
                .map(|action| self.apply_list_action(action))
                .unwrap_or_default(),
            Screen::Rename { .. } => input::rename_action(key)
                .map(|action| self.apply_rename_action(action))
                .unwrap_or_default(),
            Screen::Create(flow) => match create::apply_key(flow, key, self.sessions.items()) {
                create::CreateOutcome::Continue => Update::redraw(),
                create::CreateOutcome::Cancelled => {
                    self.reset_ui();
                    Update::redraw_with([
                        Effect::RenamePluginPane {
                            title: LIST_PANE_TITLE,
                        },
                        Effect::HidePlugin,
                    ])
                }
                create::CreateOutcome::CreateSession { name, cwd } => {
                    self.reset_ui();
                    Update::redraw_with([
                        Effect::RenamePluginPane {
                            title: LIST_PANE_TITLE,
                        },
                        Effect::HidePlugin,
                        Effect::CreateSession { name, cwd },
                    ])
                }
                create::CreateOutcome::StartWorktree(request) => {
                    Update::redraw_with([Effect::ValidateWorktree { request }])
                }
            },
        }
    }

    fn apply_list_action(&mut self, action: ListAction) -> Update {
        let transition = self.ui.apply_list_action(
            self.sessions.items(),
            self.agents.items(),
            action,
            &self.config.agent_bridge,
        );
        self.finish_list_transition(transition)
    }

    fn finish_list_transition(&mut self, transition: Transition) -> Update {
        if transition.dismiss {
            self.reset_ui();
        }
        if let Some(destination) = transition.destination {
            match destination {
                Destination::Rename(name) => {
                    self.ui.state.screen = Screen::Rename {
                        original: name.clone(),
                        draft: name,
                    };
                }
                Destination::Create => {
                    let flow = CreateFlow::new();
                    self.ui.state.screen = Screen::Create(Box::new(flow));
                    return Update {
                        redraw: true,
                        effects: transition
                            .effects
                            .into_iter()
                            .chain([
                                Effect::FetchDirectoryCandidates,
                                Effect::RenamePluginPane {
                                    title: CREATE_PANE_TITLE,
                                },
                            ])
                            .collect(),
                    };
                }
                Destination::CreateWorktree(selected) => {
                    let flow = CreateFlow::Worktree(create::worktree::Form::new(
                        self.config.worktree.clone(),
                        selected.as_ref(),
                    ));
                    self.ui.state.screen = Screen::Create(Box::new(flow));
                    return Update::redraw_with([Effect::RenamePluginPane {
                        title: "New Worktree",
                    }]);
                }
            }
        }
        Update {
            // Ask Zellij to render the default UI even though the same update hides the pane.
            // This replaces the saved pane buffer before it can be shown again.
            redraw: transition.redraw || transition.dismiss,
            effects: transition.effects,
        }
    }

    fn finish_worktree_validation(&mut self, result: Result<(), String>) -> Update {
        let Screen::Create(flow) = &mut self.ui.state.screen else {
            return Update::default();
        };
        let CreateFlow::Worktree(form) = &mut **flow else {
            return Update::default();
        };
        match result {
            Ok(()) => {
                form.begin_create();
                let Some(request) = form.pending_request().cloned() else {
                    return Update::default();
                };
                Update::redraw_with([Effect::CreateWorktree { request }])
            }
            Err(error) => {
                form.fail(error);
                Update::redraw()
            }
        }
    }

    fn finish_worktree_creation(&mut self, result: Result<(), String>) -> Update {
        let Screen::Create(flow) = &mut self.ui.state.screen else {
            return Update::default();
        };
        let CreateFlow::Worktree(form) = &mut **flow else {
            return Update::default();
        };
        match result {
            Ok(()) => {
                let Some(request) = form.pending_request().cloned() else {
                    return Update::default();
                };
                self.reset_ui();
                Update::redraw_with([
                    Effect::RenamePluginPane {
                        title: LIST_PANE_TITLE,
                    },
                    Effect::HidePlugin,
                    Effect::CreateSession {
                        name: request.session_name,
                        cwd: request.worktree,
                    },
                ])
            }
            Err(error) => {
                form.fail(error);
                Update::redraw()
            }
        }
    }

    fn apply_rename_action(&mut self, action: RenameAction) -> Update {
        match action {
            RenameAction::Cancel => {
                self.ui.state.screen = Screen::List;
                Update::redraw()
            }
            RenameAction::PopChar => {
                if let Screen::Rename { draft, .. } = &mut self.ui.state.screen {
                    draft.pop();
                }
                Update::redraw()
            }
            RenameAction::PushChar(character) => {
                if let Screen::Rename { draft, .. } = &mut self.ui.state.screen {
                    draft.push(character);
                }
                Update::redraw()
            }
            RenameAction::Confirm => {
                let Screen::Rename { original, draft } = &self.ui.state.screen else {
                    return Update::default();
                };
                let old = original.clone();
                let new = draft.trim().to_string();
                self.ui.state.screen = Screen::List;
                if new.is_empty() || new == old {
                    return Update::redraw();
                }
                Update::redraw_with([
                    Effect::RenameAgentSession {
                        bridge: self.config.agent_bridge.clone(),
                        old,
                        new: new.clone(),
                    },
                    Effect::RenameCurrentSession { name: new },
                ])
            }
        }
    }

    fn reset_ui(&mut self) {
        self.ui.reset();
    }

    fn visibility_changed(&mut self, visible: bool) -> Update {
        let changed = self.visible != visible;
        self.visible = visible;
        let was_non_list = !matches!(self.ui.state.screen, Screen::List);
        if changed && !visible {
            self.reset_ui();
        }
        let mut update = Update {
            redraw: visible && changed,
            effects: Vec::new(),
        };
        if changed && !visible && was_non_list {
            update.effects.push(Effect::RenamePluginPane {
                title: LIST_PANE_TITLE,
            });
        }
        if visible && matches!(self.ui.state.screen, Screen::List) {
            update.redraw = true;
            update.effects.extend(self.request_picker_refresh());
        }
        update
    }

    // Triggered by an external `agent-refresh` pipe message so the cache stays fresh even while
    // the plugin instance is hidden (Zellij never emits a lifecycle event for that case). Only
    // agent status is refetched here, not the full session list: `agent_refresh.request` starts
    // an async subprocess (non-blocking), whereas `request_picker_refresh`'s session-list half
    // calls Zellij's `get_session_list()`, which blocks the plugin on the host until it replies.
    // These pipes fire on every hook event across every session, so routing them through that
    // synchronous call froze key handling and rendering for the whole time a fetch was pending.
    // Agent hooks don't change the session list anyway, so skipping it here loses nothing.
    fn external_agent_update(&mut self) -> Update {
        if !self.agent_refresh.permissions_granted() {
            return Update::default();
        }
        let Some(effect) = self.agent_refresh.request(&self.config.agent_bridge) else {
            // Already mid-fetch (or not yet ready) — the in-flight fetch will pick up the
            // latest state once it completes, so there's nothing to do here.
            return Update::default();
        };
        Update {
            redraw: false,
            effects: vec![effect],
        }
    }

    fn request_picker_refresh(&mut self) -> Vec<Effect> {
        // `is_busy` must be checked here, before `picker_refresh.request`, rather than after:
        // an external `agent-refresh` pipe (see `external_agent_update`) can leave `agent_refresh`
        // mid-fetch outside of any picker_refresh cycle. If picker_refresh recorded
        // `agents_pending: true` for a fetch that then turned out not to start, nothing would
        // ever call `finish_agents` for it and this refresh would be stuck "refreshing" forever.
        let include_agents =
            self.agent_refresh.permissions_granted() && !self.agent_refresh.is_busy();
        let Some(refresh_id) = self.picker_refresh.request(include_agents) else {
            return Vec::new();
        };
        let mut effects = vec![Effect::RefreshPickerSessions { refresh_id }];
        if include_agents {
            if let Some(effect) = self.agent_refresh.request(&self.config.agent_bridge) {
                self.agent_refresh_id = Some(refresh_id);
                effects.push(effect);
            }
        }
        effects
    }

    fn finish_picker_sessions(
        &mut self,
        refresh_id: RefreshId,
        result: Result<Vec<Session>, ()>,
    ) -> Update {
        let mut update = Update {
            redraw: self.visible,
            effects: Vec::new(),
        };
        let success = result.is_ok();
        if let Ok(sessions) = result {
            let sessions_update = self.sessions.replace(sessions);
            self.retain_agents();
            let transition = self
                .ui
                .update_sessions(self.sessions.items(), sessions_update);
            update.redraw |= self.visible && transition.redraw;
            update.effects.extend(transition.effects);
        }
        if let Some(next) = self.picker_refresh.finish_sessions(refresh_id, success) {
            update
                .effects
                .extend(self.start_queued_picker_refresh(next));
        }
        update
    }

    fn finish_agent_fetch(
        &mut self,
        request_id: RequestId,
        result: Result<Vec<Agent>, ()>,
    ) -> Update {
        let Some(result) = self.agent_refresh.finish(request_id, result) else {
            return Update::default();
        };
        let mut update = Update {
            // Even an unchanged snapshot must clear the visible refreshing indicator.
            redraw: self.visible,
            effects: Vec::new(),
        };
        let success = result.is_ok();
        if let Ok(agents) = result {
            let changed = self
                .agents
                .replace(agents, &active_session_order(self.sessions.items()));
            if changed {
                self.ui.sync_agent_state(self.agents.items().len());
            }
            update.redraw |= self.visible && changed;
        }
        // Agent request IDs and picker refresh IDs share the same monotonic ordering because
        // each picker refresh starts at most one agent fetch.
        if let Some(refresh_id) = self.agent_refresh_id.take() {
            if let Some(next) = self.picker_refresh.finish_agents(refresh_id, success) {
                update
                    .effects
                    .extend(self.start_queued_picker_refresh(next));
            }
        }
        update
    }

    fn start_queued_picker_refresh(&mut self, refresh_id: RefreshId) -> Vec<Effect> {
        let mut effects = vec![Effect::RefreshPickerSessions { refresh_id }];
        if let Some(effect) = self.agent_refresh.request(&self.config.agent_bridge) {
            self.agent_refresh_id = Some(refresh_id);
            effects.push(effect);
        }
        effects
    }

    fn animation_frame(&mut self) -> Update {
        if !ui::subscriptions(self).animation_frame {
            return Update::default();
        }
        self.ui.advance_animation();
        Update::redraw()
    }

    fn retain_agents(&mut self) {
        let items = self.agents.items().to_vec();
        if self
            .agents
            .replace(items, &active_session_order(self.sessions.items()))
        {
            self.ui.sync_agent_state(self.agents.items().len());
        }
    }
}

fn active_session_order(sessions: &[Session]) -> Vec<String> {
    sessions
        .iter()
        .filter(|session| session.is_active())
        .map(|session| session.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::SessionLifecycle;
    use agent_core::{Activity, AgentState, AgentTarget};

    fn config(agent_bridge: &str) -> AppConfig {
        AppConfig {
            agent_bridge: agent_bridge.into(),
            worktree: create::worktree::Config::default(),
        }
    }

    fn app() -> App {
        App::new(ui::Theme::test_default(), config("bridge"))
    }

    fn active(name: &str, current: bool) -> Session {
        Session {
            name: name.into(),
            lifecycle: SessionLifecycle::Active { current },
            ..Default::default()
        }
    }

    fn agent(session: &str) -> Agent {
        Agent {
            id: format!("{session}-agent"),
            agent_label: "codex".into(),
            state: AgentState::Idle,
            seen: true,
            target: AgentTarget {
                session_name: session.into(),
                pane_id: 1,
            },
            activity: Activity {
                kind: "stop".into(),
                label: "finished".into(),
                preview: "finished".into(),
            },
            owner_pid: 1,
            process_fingerprint: "process".into(),
            agent_session_id: "agent-session".into(),
            observed_at_ms: 1,
            pending_permissions: Vec::new(),
        }
    }

    fn fetch_id(update: &Update) -> Option<RequestId> {
        update.effects.iter().find_map(|effect| match effect {
            Effect::FetchAgents { request_id, .. } => Some(*request_id),
            _ => None,
        })
    }

    fn agent_count(app: &App) -> usize {
        match ui::view(&app) {
            ui::screens::ScreenView::List(view) => view.agents.len(),
            ui::screens::ScreenView::Create(_) => panic!("expected list view"),
        }
    }

    fn list_view(app: &App) -> ui::screens::list::ListView {
        match ui::view(&app) {
            ui::screens::ScreenView::List(view) => view,
            ui::screens::ScreenView::Create(_) => panic!("expected list view"),
        }
    }

    #[test]
    fn rename_owns_the_original_identity() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("old", true)]));
        app.update(Message::Key(Key::Ctrl('r')));
        app.update(Message::Key(Key::Backspace));
        let update = app.update(Message::Key(Key::Enter));
        assert!(matches!(
            update.effects.as_slice(),
            [Effect::RenameAgentSession { old, .. }, Effect::RenameCurrentSession { .. }] if old == "old"
        ));
    }

    #[test]
    fn initialization_does_not_fetch_or_schedule_animation() {
        let app = app();
        let update = app.initial_update();
        assert!(fetch_id(&update).is_none());
        assert!(!ui::subscriptions(&app).animation_frame);
    }

    #[test]
    fn permission_grant_fetches_even_without_an_initial_visibility_event() {
        let mut app = app();
        let update = app.update(Message::PermissionGranted);
        assert_eq!(fetch_id(&update), Some(RequestId(0)));
        assert!(ui::subscriptions(&app).animation_frame);
    }

    #[test]
    fn external_agent_update_only_refetches_agents() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });
        let update = app.update(Message::ExternalAgentUpdate);
        assert_eq!(fetch_id(&update), Some(RequestId(1)));
        assert!(!update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::RefreshPickerSessions { .. })));
        assert!(!update.redraw);
    }

    #[test]
    fn external_agent_update_is_a_noop_while_a_fetch_is_already_in_flight() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        let update = app.update(Message::ExternalAgentUpdate);
        assert!(fetch_id(&update).is_none());
        assert!(update.effects.is_empty());
    }

    // Regression test: an agent-refresh pipe can start an agent fetch outside of any
    // `picker_refresh` cycle. A later visibility-triggered refresh must not claim to be
    // waiting on that fetch's completion, or it would stay "refreshing" forever once the
    // externally-started fetch finishes without ever notifying `picker_refresh`.
    #[test]
    fn a_picker_refresh_started_while_an_external_agent_fetch_is_in_flight_still_completes() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        app.update(Message::PickerSessionsFinished {
            refresh_id: RefreshId(0),
            result: Ok(Vec::new()),
        });
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });

        // Starts a second agent fetch outside of any picker_refresh cycle.
        app.update(Message::ExternalAgentUpdate);

        let reopened = app.update(Message::VisibilityChanged(true));
        let Some(Effect::RefreshPickerSessions { refresh_id }) = reopened
            .effects
            .iter()
            .find(|effect| matches!(effect, Effect::RefreshPickerSessions { .. }))
        else {
            panic!("expected a RefreshPickerSessions effect");
        };
        // Completing only the session-list half must finish the cycle: it must not still be
        // waiting on the externally-started agent fetch.
        let finished = app.update(Message::PickerSessionsFinished {
            refresh_id: *refresh_id,
            result: Ok(Vec::new()),
        });
        assert!(matches!(
            app.picker_refresh.view(),
            crate::picker_refresh::RefreshView::Ready
        ));
        assert!(finished.effects.is_empty());
    }

    #[test]
    fn repeated_visibility_queues_one_follow_up_refresh() {
        let mut app = app();
        let first = app.update(Message::PermissionGranted);
        assert_eq!(fetch_id(&first), Some(RequestId(0)));
        assert!(fetch_id(&app.update(Message::VisibilityChanged(true))).is_none());
        assert!(fetch_id(&app.update(Message::VisibilityChanged(true))).is_none());
        let follow_up = app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });
        assert_eq!(fetch_id(&follow_up), Some(RequestId(1)));
    }

    #[test]
    fn reopening_after_completion_fetches_a_new_snapshot() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });
        app.update(Message::VisibilityChanged(false));
        let update = app.update(Message::VisibilityChanged(true));
        assert_eq!(fetch_id(&update), Some(RequestId(1)));
    }

    #[test]
    fn hiding_replaces_ui_state_without_clearing_cached_data() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![
            active("alpha", true),
            active("beta", false),
        ]));
        app.update(Message::GitLoaded {
            session_name: "alpha".into(),
            info: GitInfo {
                branch: Some("main".into()),
                repo_root: Some("/repo".into()),
                is_main_worktree: true,
            },
        });
        app.update(Message::PermissionGranted);
        app.update(Message::VisibilityChanged(true));
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(vec![agent("alpha")]),
        });

        app.update(Message::Key(Key::Char('b')));
        app.update(Message::ListAction(ListAction::FocusAgents));
        let before = list_view(&app);
        assert_eq!(before.query, "b");
        assert_eq!(before.focus, ui::screens::list::Focus::Agents);

        app.update(Message::VisibilityChanged(false));
        app.update(Message::VisibilityChanged(true));

        let reopened = list_view(&app);
        assert_eq!(reopened.query, "");
        assert_eq!(reopened.selected_session, Some(0));
        assert_eq!(reopened.selected_agent, Some(0));
        assert_eq!(reopened.focus, ui::screens::list::Focus::Sessions);
        assert_eq!(reopened.filtered_count, reopened.total_count);
        assert_eq!(reopened.agents.len(), 1);
        assert_eq!(
            reopened
                .sessions
                .iter()
                .find(|session| session.name == "alpha")
                .and_then(|session| session.branch.as_deref()),
            Some("main")
        );
    }

    #[test]
    fn dismiss_resets_ui_before_the_host_reports_visibility_change() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![
            active("alpha", true),
            active("beta", false),
        ]));
        app.update(Message::VisibilityChanged(true));
        app.update(Message::Key(Key::Down));
        app.update(Message::Key(Key::Char('b')));

        let update = app.update(Message::Key(Key::Escape));

        assert!(update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::HidePlugin)));
        assert!(update.redraw);
        let view = list_view(&app);
        assert_eq!(view.query, "");
        assert_eq!(view.selected_session, Some(0));
        assert_eq!(view.focus, ui::screens::list::Focus::Sessions);
    }

    #[test]
    fn hiding_create_screen_restores_default_screen_and_title() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("one", true)]));
        app.update(Message::VisibilityChanged(true));
        app.update(Message::Key(Key::Ctrl('n')));

        let update = app.update(Message::VisibilityChanged(false));

        assert!(matches!(app.ui.state.screen, Screen::List));
        assert!(update.effects.iter().any(|effect| matches!(
            effect,
            Effect::RenamePluginPane {
                title: LIST_PANE_TITLE
            }
        )));
    }

    #[test]
    fn hidden_results_update_cache_without_redrawing() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("one", true)]));
        app.update(Message::PermissionGranted);
        app.update(Message::VisibilityChanged(true));
        app.update(Message::VisibilityChanged(false));
        let update = app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(vec![agent("one")]),
        });
        assert!(!update.redraw);
        assert_eq!(agent_count(&app), 1);
    }

    #[test]
    fn unchanged_successful_snapshot_only_redraws_to_clear_refreshing() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });
        app.update(Message::VisibilityChanged(false));
        let reopened = app.update(Message::VisibilityChanged(true));
        assert_eq!(fetch_id(&reopened), Some(RequestId(1)));
        let update = app.update(Message::AgentsFetchFinished {
            request_id: RequestId(1),
            result: Ok(Vec::new()),
        });
        assert!(update.redraw);
        assert!(update.effects.is_empty());
    }

    #[test]
    fn failed_refresh_preserves_cached_agents() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("one", true)]));
        app.update(Message::PermissionGranted);
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(vec![agent("one")]),
        });
        app.update(Message::VisibilityChanged(false));
        let reopened = app.update(Message::VisibilityChanged(true));
        assert_eq!(fetch_id(&reopened), Some(RequestId(1)));
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(1),
            result: Err(()),
        });
        assert_eq!(agent_count(&app), 1);
        assert!(matches!(
            app.picker_refresh.view(),
            crate::picker_refresh::RefreshView::Failed
        ));
    }

    #[test]
    fn recreated_app_starts_with_materialized_cache_and_resets_transient_ui() {
        let mut original = app();
        original.update(Message::SessionsLoaded(vec![active("one", true)]));
        original.update(Message::GitLoaded {
            session_name: "one".into(),
            info: GitInfo {
                branch: Some("cached-branch".into()),
                ..Default::default()
            },
        });
        original.update(Message::PermissionGranted);
        original.update(Message::VisibilityChanged(true));
        original.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(vec![agent("one")]),
        });
        original.update(Message::Key(Key::Char('x')));

        let mut restored = App::restore(
            ui::Theme::test_default(),
            config("bridge"),
            original.persistent_state(),
        );
        let initial = list_view(&restored);
        assert_eq!(initial.query, "");
        assert_eq!(initial.agents.len(), 1);
        assert_eq!(initial.sessions[0].branch.as_deref(), Some("cached-branch"));

        let update = restored.update(Message::SessionsLoaded(vec![active("one", true)]));
        assert!(!update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::LookupGit(_))));
    }

    #[test]
    fn restore_uses_runtime_config_instead_of_snapshot() {
        let runtime_config = AppConfig {
            agent_bridge: "runtime-bridge".into(),
            worktree: create::worktree::Config {
                branch_prefix: "runtime".into(),
                worktree_root: "/tmp/runtime-worktrees".into(),
            },
        };
        let restored = App::restore(
            ui::Theme::test_default(),
            runtime_config.clone(),
            app().persistent_state(),
        );

        assert_eq!(restored.config, runtime_config);
    }

    #[test]
    fn animation_stops_after_refresh_completes() {
        let mut app = app();
        app.update(Message::PermissionGranted);
        app.update(Message::AgentsFetchFinished {
            request_id: RequestId(0),
            result: Ok(Vec::new()),
        });
        let update = app.update(Message::AnimationFrame);
        assert!(!update.redraw);
        assert!(update.effects.is_empty());
    }

    #[test]
    fn create_transition_requests_candidates_and_title() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("one", true)]));
        let update = app.update(Message::Key(Key::Ctrl('n')));
        assert!(matches!(app.ui.state.screen, Screen::Create(_)));
        assert!(update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::FetchDirectoryCandidates)));
    }
}
