pub mod model;
pub mod render;
mod session_browser;

pub use model::{AgentRow, AgentState, Focus, ListView, SessionRow};

use agent_core::{display_state, Agent, DisplayState};

use self::session_browser::SessionBrowser;
use crate::agent_refresh::RefreshView;
use crate::effects::Effect;
use crate::input::{self, ListAction};
use crate::picker_refresh::RefreshView as PickerRefreshView;
use crate::sessions::{Session, SessionsUpdate};

pub enum Destination {
    Rename(String),
    Create,
    CreateWorktree(Option<Session>),
}

#[derive(Default)]
pub struct Transition {
    pub redraw: bool,
    pub dismiss: bool,
    pub effects: Vec<Effect>,
    pub destination: Option<Destination>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Surface {
    Sessions,
    Agents,
}

#[derive(Default)]
struct AgentUiState {
    selected: usize,
    surface: Surface,
    spinner_tick: usize,
}

impl Default for Surface {
    fn default() -> Self {
        Self::Sessions
    }
}

impl AgentUiState {
    fn selected(&self, agent_count: usize) -> Option<usize> {
        (agent_count > 0).then_some(self.selected)
    }

    fn selected_agent<'a>(&self, agents: &'a [Agent]) -> Option<&'a Agent> {
        agents.get(self.selected)
    }

    fn surface(&self) -> Surface {
        self.surface
    }

    fn spinner_tick(&self) -> usize {
        self.spinner_tick
    }

    fn advance_spinner(&mut self) {
        self.spinner_tick = self.spinner_tick.wrapping_add(1);
    }

    fn clamp_to(&mut self, agent_count: usize) {
        self.selected = self.selected.min(agent_count.saturating_sub(1));
        if agent_count == 0 {
            self.surface = Surface::Sessions;
        }
    }

    fn move_down(&mut self, agent_count: usize) {
        if self.selected + 1 < agent_count {
            self.selected += 1;
        }
    }

    fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn focus_agents(&mut self, has_agents: bool) -> bool {
        if !has_agents {
            return false;
        }
        let changed = self.surface != Surface::Agents;
        self.surface = Surface::Agents;
        changed
    }

    fn focus_sessions(&mut self) -> bool {
        let changed = self.surface != Surface::Sessions;
        self.surface = Surface::Sessions;
        changed
    }
}

#[derive(Default)]
pub struct ListScreen {
    browser: SessionBrowser,
}

#[derive(Default)]
pub struct UiState {
    agents: AgentUiState,
}

impl ListScreen {
    pub fn reset_ui(&mut self, ui: &mut UiState) {
        self.browser.reset();
        *ui = UiState::default();
    }

    pub fn sync_agent_state(&self, ui: &mut UiState, agent_count: usize) {
        ui.agents.clamp_to(agent_count);
    }

    pub fn update_sessions(&mut self, sessions: &[Session], update: SessionsUpdate) -> Transition {
        if update.changed {
            self.browser.set_items(sessions);
        }
        Transition {
            redraw: update.changed,
            effects: update.lookups.into_iter().map(Effect::LookupGit).collect(),
            destination: None,
            ..Default::default()
        }
    }

    pub fn advance_animation(&self, ui: &mut UiState) {
        ui.agents.advance_spinner();
    }

    pub fn apply_action(
        &mut self,
        ui: &mut UiState,
        sessions: &[Session],
        agents: &[Agent],
        action: ListAction,
        bridge: &str,
    ) -> Transition {
        match action {
            ListAction::MoveDown => {
                if ui.agents.surface() == Surface::Agents {
                    ui.agents.move_down(agents.len());
                } else {
                    self.browser.move_down();
                }
                redraw()
            }
            ListAction::MoveUp => {
                if ui.agents.surface() == Surface::Agents {
                    ui.agents.move_up();
                } else {
                    self.browser.move_up();
                }
                redraw()
            }
            ListAction::PushChar(character) if ui.agents.surface() == Surface::Sessions => {
                self.browser.push_char(character);
                redraw()
            }
            ListAction::PopChar if ui.agents.surface() == Surface::Sessions => {
                self.browser.pop_char();
                redraw()
            }
            ListAction::Delete if ui.agents.surface() == Surface::Sessions => {
                let Some(session) = self.browser.selected() else {
                    return Transition::default();
                };
                if session.is_current() {
                    return Transition::default();
                }
                effects([
                    Effect::DeleteSession {
                        name: session.name.clone(),
                        lifecycle: session.lifecycle,
                    },
                    Effect::RefreshSessions,
                ])
            }
            ListAction::Confirm if ui.agents.surface() == Surface::Agents => {
                let Some(agent) = ui.agents.selected_agent(agents).cloned() else {
                    return Transition::default();
                };
                dismiss([
                    Effect::MarkAgentSeen {
                        bridge: bridge.into(),
                        id: agent.id,
                    },
                    Effect::HidePlugin,
                    Effect::SwitchToAgent {
                        session_name: agent.target.session_name,
                        pane_id: agent.target.pane_id,
                    },
                ])
            }
            ListAction::Confirm => {
                let mut commands = vec![Effect::HidePlugin];
                if let Some(session) = self.browser.selected() {
                    commands.push(Effect::SwitchSession {
                        name: session.name.clone(),
                    });
                }
                dismiss(commands)
            }
            ListAction::Rename if ui.agents.surface() == Surface::Sessions => {
                let Some(name) = sessions
                    .iter()
                    .find(|session| session.is_current())
                    .map(|session| session.name.clone())
                else {
                    return Transition::default();
                };
                self.browser.clear_query();
                Transition {
                    redraw: true,
                    destination: Some(Destination::Rename(name)),
                    effects: Vec::new(),
                    ..Default::default()
                }
            }
            ListAction::CreateNew if ui.agents.surface() == Surface::Sessions => Transition {
                redraw: true,
                destination: Some(Destination::Create),
                effects: Vec::new(),
                ..Default::default()
            },
            ListAction::CreateWorktree if ui.agents.surface() == Surface::Sessions => Transition {
                redraw: true,
                destination: Some(Destination::CreateWorktree(
                    self.browser.selected().cloned(),
                )),
                effects: Vec::new(),
                ..Default::default()
            },
            ListAction::NextSurface => Transition {
                redraw: if ui.agents.surface() == Surface::Agents {
                    ui.agents.focus_sessions()
                } else {
                    ui.agents.focus_agents(!agents.is_empty())
                },
                ..Default::default()
            },
            ListAction::FocusAgents => Transition {
                redraw: ui.agents.focus_agents(!agents.is_empty()),
                ..Default::default()
            },
            ListAction::FocusSessions => Transition {
                redraw: ui.agents.focus_sessions(),
                ..Default::default()
            },
            ListAction::Cancel => dismiss([Effect::HidePlugin]),
            _ => Transition::default(),
        }
    }

    pub fn view(
        &self,
        ui: &UiState,
        all_sessions: &[Session],
        agents: &[Agent],
        hints: Vec<(&'static str, &'static str)>,
        rename_draft: Option<&String>,
        agent_refresh: RefreshView,
        refresh: PickerRefreshView,
    ) -> model::ListView {
        let picker = self.browser.view();
        let sessions = picker
            .items
            .iter()
            .enumerate()
            .map(|(index, (session, matched))| {
                let nested = is_nested_worktree(session, all_sessions);
                model::SessionRow {
                    name: session.name.clone(),
                    matched: matched.clone(),
                    active: session.is_active(),
                    current: session.is_current(),
                    branch: session.branch.clone(),
                    nested,
                    last_sibling: nested
                        && picker
                            .items
                            .get(index + 1)
                            .is_none_or(|(next, _)| !is_nested_worktree(next, all_sessions)),
                    agent: session_agent_state(
                        agents
                            .iter()
                            .filter(|agent| agent.target.session_name == session.name)
                            .map(|agent| visual_state(display_state(agent))),
                    ),
                    rename_draft: session
                        .is_current()
                        .then(|| rename_draft.cloned())
                        .flatten(),
                }
            })
            .collect();
        model::ListView {
            query: picker.query.into(),
            sessions,
            selected_session: picker.selected,
            filtered_count: picker.filtered_count,
            total_count: picker.total_count,
            agents: agents
                .iter()
                .map(|agent| model::AgentRow {
                    session_name: agent.target.session_name.clone(),
                    label: agent.agent_label.clone(),
                    pane_id: agent.target.pane_id,
                    preview: agent.activity.preview.clone(),
                    state: visual_state(display_state(agent)),
                })
                .collect(),
            selected_agent: ui.agents.selected(agents.len()),
            focus: match ui.agents.surface() {
                Surface::Sessions => model::Focus::Sessions,
                Surface::Agents => model::Focus::Agents,
            },
            spinner_tick: ui.agents.spinner_tick(),
            agent_refresh,
            refresh,
            hints,
        }
    }

    pub fn hints(&self, ui: &UiState, agents: &[Agent]) -> Vec<(&'static str, &'static str)> {
        if ui.agents.surface() == Surface::Agents {
            return vec![
                ("enter", "jump"),
                ("tab/ctrl-h", "sessions"),
                ("esc", "close"),
            ];
        }
        let mut hints = input::LIST_HINTS.to_vec();
        if !agents.is_empty() {
            hints.insert(1.min(hints.len()), ("tab/ctrl-l", "agents"));
        }
        hints
    }
}

fn is_nested_worktree(session: &Session, all_sessions: &[Session]) -> bool {
    !session.is_main_worktree
        && session.repo_root.as_ref().is_some_and(|repo_root| {
            all_sessions.iter().any(|candidate| {
                candidate.is_main_worktree && candidate.repo_root.as_ref() == Some(repo_root)
            })
        })
}

fn redraw() -> Transition {
    Transition {
        redraw: true,
        ..Default::default()
    }
}

fn effects(effects: impl IntoIterator<Item = Effect>) -> Transition {
    Transition {
        effects: effects.into_iter().collect(),
        ..Default::default()
    }
}

fn dismiss(effects: impl IntoIterator<Item = Effect>) -> Transition {
    Transition {
        dismiss: true,
        effects: effects.into_iter().collect(),
        ..Default::default()
    }
}

fn visual_state(state: DisplayState) -> model::AgentState {
    match state {
        DisplayState::Blocked => model::AgentState::Blocked,
        DisplayState::Working => model::AgentState::Working,
        DisplayState::Done => model::AgentState::Done,
        DisplayState::Idle => model::AgentState::Idle,
        DisplayState::Unknown => model::AgentState::Unknown,
    }
}

fn session_agent_state(
    states: impl Iterator<Item = model::AgentState>,
) -> Option<model::AgentState> {
    states.min_by_key(|state| match state {
        model::AgentState::Blocked => 0,
        model::AgentState::Working => 1,
        model::AgentState::Done => 2,
        model::AgentState::Idle => 3,
        model::AgentState::Unknown => 4,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::SessionLifecycle;

    #[test]
    fn switching_sessions_hides_the_picker_before_switching() {
        let mut screen = ListScreen::default();
        let mut ui = UiState::default();
        let sessions = vec![Session {
            name: "destination".into(),
            lifecycle: SessionLifecycle::Active { current: false },
            ..Default::default()
        }];
        screen.update_sessions(
            &sessions,
            SessionsUpdate {
                changed: true,
                lookups: Vec::new(),
            },
        );

        let transition =
            screen.apply_action(&mut ui, &sessions, &[], ListAction::Confirm, "bridge");

        assert!(matches!(
            transition.effects.as_slice(),
            [Effect::HidePlugin, Effect::SwitchSession { name }] if name == "destination"
        ));
    }

    #[test]
    fn session_status_uses_the_highest_priority_agent() {
        use model::AgentState;

        assert_eq!(
            session_agent_state([AgentState::Done, AgentState::Working].into_iter()),
            Some(AgentState::Working)
        );
        assert_eq!(
            session_agent_state([AgentState::Idle, AgentState::Blocked].into_iter()),
            Some(AgentState::Blocked)
        );
        assert_eq!(session_agent_state(std::iter::empty::<AgentState>()), None);
    }

    #[test]
    fn worktree_nesting_is_derived_from_repository_facts() {
        let sessions = vec![
            Session {
                name: "main".into(),
                repo_root: Some("/repo".into()),
                is_main_worktree: true,
                ..Default::default()
            },
            Session {
                name: "feature".into(),
                repo_root: Some("/repo".into()),
                ..Default::default()
            },
            Session {
                name: "orphan".into(),
                repo_root: Some("/other".into()),
                ..Default::default()
            },
        ];

        assert!(!is_nested_worktree(&sessions[0], &sessions));
        assert!(is_nested_worktree(&sessions[1], &sessions));
        assert!(!is_nested_worktree(&sessions[2], &sessions));
    }
}
