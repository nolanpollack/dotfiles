pub mod components;
pub mod screens;
mod state;
mod theme;

use crate::app::App;
use crate::create::{self, CreateFlow};
use crate::input::ListAction;
use crate::sessions::{Session, SessionsUpdate};
use agent_core::Agent;
use screens::list::{ListScreen, Transition};

pub(crate) use state::{Screen, UiState};
pub use theme::{Theme, ThemeOverrides, ThemePalette};

use ratatui::Terminal;

use crate::backend::StdoutBackend;
use screens::ScreenView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscriptions {
    pub animation_frame: bool,
}

pub fn render(view: ScreenView, theme: &Theme, rows: usize, cols: usize) {
    let Ok(mut terminal) = Terminal::new(StdoutBackend::new(rows as u16, cols as u16)) else {
        return;
    };
    terminal
        .draw(|frame| screens::draw(frame, &view, theme))
        .ok();
}

pub struct Ui {
    pub(crate) state: UiState,
    pub(crate) list: ListScreen,
    theme: Theme,
    worktree_spinner_tick: usize,
}

impl Ui {
    pub fn new(theme: Theme) -> Self {
        Self {
            state: UiState::default(),
            list: ListScreen::default(),
            theme,
            worktree_spinner_tick: 0,
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub(crate) fn apply_list_action(
        &mut self,
        sessions: &[Session],
        agents: &[Agent],
        action: ListAction,
        bridge: &str,
    ) -> Transition {
        self.list
            .apply_action(&mut self.state.list, sessions, agents, action, bridge)
    }

    pub(crate) fn update_sessions(
        &mut self,
        sessions: &[Session],
        update: SessionsUpdate,
    ) -> Transition {
        self.list.update_sessions(sessions, update)
    }

    pub(crate) fn sync_agent_state(&mut self, agent_count: usize) {
        self.list
            .sync_agent_state(&mut self.state.list, agent_count);
    }

    pub(crate) fn reset(&mut self) {
        self.list.reset_ui(&mut self.state.list);
        self.state.screen = Screen::default();
    }

    pub(crate) fn advance_animation(&mut self) {
        self.list.advance_animation(&mut self.state.list);
        self.worktree_spinner_tick = self.worktree_spinner_tick.wrapping_add(1);
    }
}

pub fn view(app: &App) -> ScreenView {
    let ui = &app.ui;
    let sessions = app.sessions.items();
    let agents = app.agents.items();
    let agent_refresh = app.agent_refresh.view();
    let refresh = app.picker_refresh.view();

    match &ui.state.screen {
        Screen::List => ScreenView::List(ui.list.view(
            &ui.state.list,
            sessions,
            agents,
            ui.list.hints(&ui.state.list, agents),
            None,
            agent_refresh,
            refresh,
        )),
        Screen::Rename { draft, .. } => ScreenView::List(ui.list.view(
            &ui.state.list,
            sessions,
            agents,
            vec![("enter", "confirm rename"), ("esc", "cancel")],
            Some(draft),
            agent_refresh,
            refresh,
        )),
        Screen::Create(flow) => ScreenView::Create(create_view(flow, ui.worktree_spinner_tick)),
    }
}

pub fn subscriptions(app: &App) -> Subscriptions {
    Subscriptions {
        animation_frame: (app.visible
            && (app.picker_refresh.is_refreshing() || app.agents.has_working()))
            || matches!(
                &app.ui.state.screen,
                Screen::Create(flow)
                    if matches!(
                        &**flow,
                        CreateFlow::Worktree(form)
                            if matches!(form.stage(), Some(create::worktree::Stage::Checking | create::worktree::Stage::Creating))
                    )
            ),
    }
}

fn create_view(flow: &CreateFlow, worktree_spinner_tick: usize) -> screens::create::CreateView {
    match flow {
        CreateFlow::Directory(form) => {
            if let Some(view) = form.directory().picker() {
                return screens::create::CreateView::DirectoryChoices {
                    query: view.query.into(),
                    rows: view
                        .items
                        .iter()
                        .map(|(candidate, matched)| screens::create::ChoiceRow {
                            display: candidate.display.clone(),
                            matched: matched.clone(),
                        })
                        .collect(),
                    selected: view.selected,
                    filtered_count: view.filtered_count,
                    total_count: view.total_count,
                };
            }
            screens::create::CreateView::Form {
                directory: form.directory().display(),
                name: form.name().value().into(),
                directory_focused: form.focus() == create::directory::Focus::Directory,
                error: form.error().map(str::to_string),
            }
        }
        CreateFlow::Worktree(form) => {
            if let Some(stage) = form.stage() {
                screens::create::CreateView::WorktreeProgress {
                    stage,
                    error: form.error().map(str::to_string),
                    spinner_tick: worktree_spinner_tick,
                }
            } else {
                screens::create::CreateView::WorktreeForm {
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
