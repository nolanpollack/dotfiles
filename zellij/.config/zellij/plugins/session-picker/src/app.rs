//! Host-independent application state and transitions.

use std::path::PathBuf;

use agent_core::AgentRecord;
use serde::{Deserialize, Serialize};

use crate::agent_refresh::{AgentRefresh, RequestId};
use crate::create::{self, CreateFlow};
use crate::effects::Effect;
use crate::git_info::GitInfo;
use crate::input::{self, Key, ListAction, RenameAction};
use crate::list_screen::{
    Destination, ListScreen, ListSnapshot, Transition, UiState as ListUiState,
};
use crate::picker_refresh::{PickerRefresh, RefreshId};
use crate::sessions::SessionInfo;
use crate::ui;

const LIST_PANE_TITLE: &str = "Session Picker";
const CREATE_PANE_TITLE: &str = "New Session";

#[derive(Default)]
pub enum Screen {
    #[default]
    List,
    Rename {
        original: String,
        draft: String,
    },
    Create(Box<CreateFlow>),
}

#[derive(Default)]
struct UiState {
    screen: Screen,
    list: ListUiState,
}

pub enum Message {
    PermissionGranted,
    ThemeChanged(ui::Theme),
    Key(Key),
    ListAction(ListAction),
    SessionsLoaded(Vec<SessionInfo>),
    PickerSessionsFinished {
        refresh_id: RefreshId,
        result: Result<Vec<SessionInfo>, ()>,
    },
    AgentsFetchFinished {
        request_id: RequestId,
        result: Result<Vec<AgentRecord>, ()>,
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
    list: ListScreen,
    ui: UiState,
    theme: ui::Theme,
    agent_bridge: String,
    visible: bool,
    agent_refresh: AgentRefresh,
    agent_refresh_id: Option<RefreshId>,
    picker_refresh: PickerRefresh,
    animation_timer_armed: bool,
    worktree_spinner_tick: usize,
    worktree_config: create::worktree::Config,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSnapshot {
    #[serde(default)]
    pub list: ListSnapshot,
    #[serde(default)]
    pub agents: Option<Vec<AgentRecord>>,
}

impl App {
    pub fn new(theme: ui::Theme, agent_bridge: String) -> Self {
        Self {
            list: ListScreen::default(),
            ui: UiState::default(),
            theme,
            agent_bridge,
            visible: false,
            agent_refresh: AgentRefresh::default(),
            agent_refresh_id: None,
            picker_refresh: PickerRefresh::default(),
            animation_timer_armed: false,
            worktree_spinner_tick: 0,
            worktree_config: create::worktree::Config::default(),
        }
    }

    pub fn set_worktree_config(&mut self, config: create::worktree::Config) {
        self.worktree_config = config;
    }

    pub fn restore(theme: ui::Theme, agent_bridge: String, snapshot: AppSnapshot) -> Self {
        let mut app = Self::new(theme, agent_bridge);
        app.list = ListScreen::from_snapshot(snapshot.list);
        if let Some(records) = snapshot.agents {
            app.list.restore_agents(&mut app.ui.list, records);
            app.agent_refresh.restore_cached();
        }
        app
    }

    pub fn persistent_state(&self) -> AppSnapshot {
        AppSnapshot {
            list: self.list.snapshot(),
            agents: self
                .agent_refresh
                .has_cached_data()
                .then(|| self.list.agent_records().to_vec()),
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
                self.theme = theme;
                Update::redraw()
            }
            Message::Key(key) => self.apply_key(key),
            Message::ListAction(action) if matches!(self.ui.screen, Screen::List) => {
                self.apply_list_action(action)
            }
            Message::ListAction(_) => Update::default(),
            Message::SessionsLoaded(sessions) => {
                let transition = self.list.replace_sessions(&mut self.ui.list, sessions);
                self.finish_list_transition(transition)
            }
            Message::PickerSessionsFinished { refresh_id, result } => {
                self.finish_picker_sessions(refresh_id, result)
            }
            Message::AgentsFetchFinished { request_id, result } => {
                self.finish_agent_fetch(request_id, result)
            }
            Message::GitLoaded { session_name, info } => {
                let transition = self.list.apply_git(&mut self.ui.list, session_name, info);
                self.finish_list_transition(transition)
            }
            Message::DirectoryCandidatesLoaded(paths) => {
                if let Screen::Create(flow) = &mut self.ui.screen {
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
                .list
                .lookup_current(cwd)
                .map(|lookup| Update::effects([Effect::LookupGit(lookup)]))
                .unwrap_or_default(),
            Message::VisibilityChanged(visible) => self.visibility_changed(visible),
            Message::ExternalAgentUpdate => self.external_agent_update(),
            Message::AnimationFrame => self.animation_frame(),
        }
    }

    pub fn theme(&self) -> &ui::Theme {
        &self.theme
    }

    pub fn view(&self) -> ui::model::ScreenView {
        match &self.ui.screen {
            Screen::List => ui::model::ScreenView::List(self.list.view(
                &self.ui.list,
                self.list.hints(&self.ui.list),
                None,
                self.agent_refresh.view(),
                self.picker_refresh.view(),
            )),
            Screen::Rename { draft, .. } => ui::model::ScreenView::List(self.list.view(
                &self.ui.list,
                vec![("enter", "confirm rename"), ("esc", "cancel")],
                Some(draft),
                self.agent_refresh.view(),
                self.picker_refresh.view(),
            )),
            Screen::Create(flow) => {
                ui::model::ScreenView::Create(create_view(flow, self.worktree_spinner_tick))
            }
        }
    }

    fn apply_key(&mut self, key: Key) -> Update {
        match &mut self.ui.screen {
            Screen::List => input::list_action(key)
                .map(|action| self.apply_list_action(action))
                .unwrap_or_default(),
            Screen::Rename { .. } => input::rename_action(key)
                .map(|action| self.apply_rename_action(action))
                .unwrap_or_default(),
            Screen::Create(flow) => match create::apply_key(flow, key, self.list.sessions()) {
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
                create::CreateOutcome::StartWorktree(request) => self
                    .ensure_animation(Update::redraw_with([Effect::ValidateWorktree { request }])),
            },
        }
    }

    fn apply_list_action(&mut self, action: ListAction) -> Update {
        let transition = self
            .list
            .apply_action(&mut self.ui.list, action, &self.agent_bridge);
        self.finish_list_transition(transition)
    }

    fn finish_list_transition(&mut self, transition: Transition) -> Update {
        if transition.dismiss {
            self.reset_ui();
        }
        if let Some(destination) = transition.destination {
            match destination {
                Destination::Rename(name) => {
                    self.ui.screen = Screen::Rename {
                        original: name.clone(),
                        draft: name,
                    };
                }
                Destination::Create => {
                    let flow = CreateFlow::new();
                    self.ui.screen = Screen::Create(Box::new(flow));
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
                        self.worktree_config.clone(),
                        selected.as_ref(),
                    ));
                    self.ui.screen = Screen::Create(Box::new(flow));
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
        let Screen::Create(flow) = &mut self.ui.screen else {
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
                self.ensure_animation(Update::redraw_with([Effect::CreateWorktree { request }]))
            }
            Err(error) => {
                form.fail(error);
                Update::redraw()
            }
        }
    }

    fn finish_worktree_creation(&mut self, result: Result<(), String>) -> Update {
        let Screen::Create(flow) = &mut self.ui.screen else {
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
                self.ui.screen = Screen::List;
                Update::redraw()
            }
            RenameAction::PopChar => {
                if let Screen::Rename { draft, .. } = &mut self.ui.screen {
                    draft.pop();
                }
                Update::redraw()
            }
            RenameAction::PushChar(character) => {
                if let Screen::Rename { draft, .. } = &mut self.ui.screen {
                    draft.push(character);
                }
                Update::redraw()
            }
            RenameAction::Confirm => {
                let Screen::Rename { original, draft } = &self.ui.screen else {
                    return Update::default();
                };
                let old = original.clone();
                let new = draft.trim().to_string();
                self.ui.screen = Screen::List;
                if new.is_empty() || new == old {
                    return Update::redraw();
                }
                Update::redraw_with([
                    Effect::RenameAgentSession {
                        bridge: self.agent_bridge.clone(),
                        old,
                        new: new.clone(),
                    },
                    Effect::RenameCurrentSession { name: new },
                ])
            }
        }
    }

    fn reset_ui(&mut self) {
        self.ui = UiState::default();
    }

    fn visibility_changed(&mut self, visible: bool) -> Update {
        let changed = self.visible != visible;
        self.visible = visible;
        let was_non_list = !matches!(self.ui.screen, Screen::List);
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
        if visible && matches!(self.ui.screen, Screen::List) {
            update.redraw = true;
            update.effects.extend(self.request_picker_refresh());
        }
        self.ensure_animation(update)
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
        let Some(effect) = self.agent_refresh.request(&self.agent_bridge) else {
            // Already mid-fetch (or not yet ready) — the in-flight fetch will pick up the
            // latest state once it completes, so there's nothing to do here.
            return Update::default();
        };
        self.ensure_animation(Update {
            redraw: false,
            effects: vec![effect],
        })
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
            if let Some(effect) = self.agent_refresh.request(&self.agent_bridge) {
                self.agent_refresh_id = Some(refresh_id);
                effects.push(effect);
            }
        }
        effects
    }

    fn finish_picker_sessions(
        &mut self,
        refresh_id: RefreshId,
        result: Result<Vec<SessionInfo>, ()>,
    ) -> Update {
        let mut update = Update {
            redraw: self.visible,
            effects: Vec::new(),
        };
        let success = result.is_ok();
        if let Ok(sessions) = result {
            let transition = self.list.replace_sessions(&mut self.ui.list, sessions);
            update.redraw |= self.visible && transition.redraw;
            update.effects.extend(transition.effects);
        }
        if let Some(next) = self.picker_refresh.finish_sessions(refresh_id, success) {
            update
                .effects
                .extend(self.start_queued_picker_refresh(next));
        }
        self.ensure_animation(update)
    }

    fn finish_agent_fetch(
        &mut self,
        request_id: RequestId,
        result: Result<Vec<AgentRecord>, ()>,
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
        if let Ok(records) = result {
            let transition = self.list.set_agents(&mut self.ui.list, records);
            update.redraw |= self.visible && transition.redraw;
            update.effects.extend(transition.effects);
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
        self.ensure_animation(update)
    }

    fn start_queued_picker_refresh(&mut self, refresh_id: RefreshId) -> Vec<Effect> {
        let mut effects = vec![Effect::RefreshPickerSessions { refresh_id }];
        if let Some(effect) = self.agent_refresh.request(&self.agent_bridge) {
            self.agent_refresh_id = Some(refresh_id);
            effects.push(effect);
        }
        effects
    }

    fn animation_frame(&mut self) -> Update {
        self.animation_timer_armed = false;
        if !self.animation_needed() {
            return Update::default();
        }
        self.list.advance_animation(&mut self.ui.list);
        self.worktree_spinner_tick = self.worktree_spinner_tick.wrapping_add(1);
        self.ensure_animation(Update::redraw())
    }

    fn ensure_animation(&mut self, mut update: Update) -> Update {
        if self.animation_needed() && !self.animation_timer_armed {
            self.animation_timer_armed = true;
            update.effects.push(Effect::ScheduleAnimationFrame);
        }
        update
    }

    fn animation_needed(&self) -> bool {
        (self.visible && (self.picker_refresh.is_refreshing() || self.list.has_working_agents()))
            || matches!(
                &self.ui.screen,
                Screen::Create(flow)
                    if matches!(
                        &**flow,
                        CreateFlow::Worktree(form)
                            if matches!(form.stage(), Some(create::worktree::Stage::Checking | create::worktree::Stage::Creating))
                    )
            )
    }
}

fn create_view(flow: &CreateFlow, worktree_spinner_tick: usize) -> ui::model::CreateView {
    match flow {
        CreateFlow::Directory(form) => {
            if let Some(view) = form.directory().picker() {
                return ui::model::CreateView::DirectoryChoices {
                    query: view.query.into(),
                    rows: view
                        .items
                        .iter()
                        .map(|(candidate, matched)| ui::model::ChoiceRow {
                            display: candidate.display.clone(),
                            matched: matched.clone(),
                        })
                        .collect(),
                    selected: view.selected,
                    filtered_count: view.filtered_count,
                    total_count: view.total_count,
                };
            }
            ui::model::CreateView::Form {
                directory: form.directory().display(),
                name: form.name().value().into(),
                directory_focused: form.focus() == create::directory::Focus::Directory,
                error: form.error().map(str::to_string),
            }
        }
        CreateFlow::Worktree(form) => {
            if let Some(stage) = form.stage() {
                ui::model::CreateView::WorktreeProgress {
                    stage,
                    error: form.error().map(str::to_string),
                    spinner_tick: worktree_spinner_tick,
                }
            } else {
                ui::model::CreateView::WorktreeForm {
                    session_name: form.session_name().value().into(),
                    repository: form.repository().value().into(),
                    base_branch: form.base_branch().value().into(),
                    branch_name: form.branch_name().value().into(),
                    focused: form.focus_index(),
                    error: form.error().map(str::to_string),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::SessionLifecycle;
    use agent_core::{Activity, AgentState, AgentTarget};

    fn app() -> App {
        App::new(ui::Theme::test_default(), "bridge".into())
    }

    fn active(name: &str, current: bool) -> SessionInfo {
        SessionInfo {
            name: name.into(),
            lifecycle: SessionLifecycle::Active { current },
            ..Default::default()
        }
    }

    fn agent(session: &str) -> AgentRecord {
        AgentRecord {
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
        match app.view() {
            ui::model::ScreenView::List(view) => view.agents.len(),
            ui::model::ScreenView::Create(_) => panic!("expected list view"),
        }
    }

    fn list_view(app: &App) -> ui::model::ListView {
        match app.view() {
            ui::model::ScreenView::List(view) => view,
            ui::model::ScreenView::Create(_) => panic!("expected list view"),
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
        assert!(!update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::ScheduleAnimationFrame)));
    }

    #[test]
    fn permission_grant_fetches_even_without_an_initial_visibility_event() {
        let mut app = app();
        let update = app.update(Message::PermissionGranted);
        assert_eq!(fetch_id(&update), Some(RequestId(0)));
        assert!(update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::ScheduleAnimationFrame)));
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
                is_main_checkout: true,
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
        assert_eq!(before.focus, ui::model::Focus::Agents);

        app.update(Message::VisibilityChanged(false));
        app.update(Message::VisibilityChanged(true));

        let reopened = list_view(&app);
        assert_eq!(reopened.query, "");
        assert_eq!(reopened.selected_session, Some(0));
        assert_eq!(reopened.selected_agent, Some(0));
        assert_eq!(reopened.focus, ui::model::Focus::Sessions);
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
        assert_eq!(view.focus, ui::model::Focus::Sessions);
    }

    #[test]
    fn hiding_create_screen_restores_default_screen_and_title() {
        let mut app = app();
        app.update(Message::SessionsLoaded(vec![active("one", true)]));
        app.update(Message::VisibilityChanged(true));
        app.update(Message::Key(Key::Ctrl('n')));

        let update = app.update(Message::VisibilityChanged(false));

        assert!(matches!(app.ui.screen, Screen::List));
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
    fn failed_refresh_preserves_cached_records() {
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
            "bridge".into(),
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
        assert!(matches!(app.ui.screen, Screen::Create(_)));
        assert!(update
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::FetchDirectoryCandidates)));
    }
}
