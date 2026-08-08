use std::path::PathBuf;

use agent_core::{display_state, AgentRecord, DisplayState};
use serde::{Deserialize, Serialize};

use crate::agent_refresh::RefreshView;
use crate::agents::{AgentTracker, Surface, UiState as AgentUiState};
use crate::effects::{Effect, GitLookup};
use crate::git_info::GitInfo;
use crate::input::{self, ListAction};
use crate::picker::PickerState;
use crate::picker_refresh::RefreshView as PickerRefreshView;
use crate::session_catalog::{CatalogUpdate, SessionCatalog, SessionCatalogSnapshot};
use crate::sessions::SessionInfo;
use crate::ui;

pub enum Destination {
    Rename(String),
    Create,
    CreateWorktree(Option<SessionInfo>),
}

#[derive(Default)]
pub struct Transition {
    pub redraw: bool,
    pub dismiss: bool,
    pub effects: Vec<Effect>,
    pub destination: Option<Destination>,
}

#[derive(Default)]
pub struct ListScreen {
    sessions: SessionCatalog,
    agents: AgentTracker,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSnapshot {
    #[serde(default)]
    pub sessions: SessionCatalogSnapshot,
}

#[derive(Default)]
pub struct UiState {
    sessions: PickerState,
    agents: AgentUiState,
}

impl ListScreen {
    pub fn from_snapshot(snapshot: ListSnapshot) -> Self {
        Self {
            sessions: SessionCatalog::from_snapshot(snapshot.sessions),
            agents: AgentTracker::default(),
        }
    }

    pub fn snapshot(&self) -> ListSnapshot {
        ListSnapshot {
            sessions: self.sessions.snapshot(),
        }
    }

    pub fn agent_records(&self) -> &[AgentRecord] {
        self.agents.records()
    }

    pub fn restore_agents(&mut self, ui: &mut UiState, records: Vec<AgentRecord>) {
        let session_order = self.session_order();
        self.agents
            .set_records(&mut ui.agents, records, &session_order);
    }

    pub fn sessions(&self) -> &[SessionInfo] {
        self.sessions.items()
    }

    pub fn replace_sessions(&mut self, ui: &mut UiState, sessions: Vec<SessionInfo>) -> Transition {
        let update = self.sessions.replace(&mut ui.sessions, sessions);
        self.catalog_transition(ui, update)
    }

    pub fn apply_git(&mut self, ui: &mut UiState, name: String, info: GitInfo) -> Transition {
        let update = self.sessions.apply_git(&mut ui.sessions, name, info);
        self.catalog_transition(ui, update)
    }

    fn catalog_transition(&mut self, ui: &mut UiState, update: CatalogUpdate) -> Transition {
        if update.changed {
            self.retain_agents(ui);
        }
        Transition {
            redraw: update.changed,
            effects: update.lookups.into_iter().map(Effect::LookupGit).collect(),
            destination: None,
            ..Default::default()
        }
    }

    pub fn set_agents(&mut self, ui: &mut UiState, records: Vec<AgentRecord>) -> Transition {
        let changed = self
            .agents
            .set_records(&mut ui.agents, records, &self.session_order());
        Transition {
            redraw: changed,
            effects: Vec::new(),
            destination: None,
            ..Default::default()
        }
    }

    pub fn lookup_current(&self, cwd: PathBuf) -> Option<GitLookup> {
        self.sessions.lookup_current(cwd)
    }

    pub fn has_working_agents(&self) -> bool {
        self.agents.has_working()
    }

    pub fn advance_animation(&self, ui: &mut UiState) {
        self.agents.advance_spinner(&mut ui.agents);
    }

    pub fn apply_action(
        &mut self,
        ui: &mut UiState,
        action: ListAction,
        bridge: &str,
    ) -> Transition {
        match action {
            ListAction::MoveDown => {
                if self.agents.surface(&ui.agents) == Surface::Agents {
                    self.agents.move_down(&mut ui.agents);
                } else {
                    self.sessions.move_down(&mut ui.sessions);
                }
                redraw()
            }
            ListAction::MoveUp => {
                if self.agents.surface(&ui.agents) == Surface::Agents {
                    self.agents.move_up(&mut ui.agents);
                } else {
                    self.sessions.move_up(&mut ui.sessions);
                }
                redraw()
            }
            ListAction::PushChar(character)
                if self.agents.surface(&ui.agents) == Surface::Sessions =>
            {
                self.sessions.push_char(&mut ui.sessions, character);
                redraw()
            }
            ListAction::PopChar if self.agents.surface(&ui.agents) == Surface::Sessions => {
                self.sessions.pop_char(&mut ui.sessions);
                redraw()
            }
            ListAction::Delete if self.agents.surface(&ui.agents) == Surface::Sessions => {
                let Some(session) = self.sessions.selected(&ui.sessions) else {
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
            ListAction::Confirm if self.agents.surface(&ui.agents) == Surface::Agents => {
                let Some(agent) = self.agents.selected_agent(&ui.agents).cloned() else {
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
                if let Some(session) = self.sessions.selected(&ui.sessions) {
                    commands.push(Effect::SwitchSession {
                        name: session.name.clone(),
                    });
                }
                dismiss(commands)
            }
            ListAction::Rename if self.agents.surface(&ui.agents) == Surface::Sessions => {
                let Some(name) = self
                    .sessions
                    .items()
                    .iter()
                    .find(|session| session.is_current())
                    .map(|session| session.name.clone())
                else {
                    return Transition::default();
                };
                self.sessions.clear_query(&mut ui.sessions);
                Transition {
                    redraw: true,
                    destination: Some(Destination::Rename(name)),
                    effects: Vec::new(),
                    ..Default::default()
                }
            }
            ListAction::CreateNew if self.agents.surface(&ui.agents) == Surface::Sessions => {
                Transition {
                    redraw: true,
                    destination: Some(Destination::Create),
                    effects: Vec::new(),
                    ..Default::default()
                }
            }
            ListAction::CreateWorktree if self.agents.surface(&ui.agents) == Surface::Sessions => {
                Transition {
                    redraw: true,
                    destination: Some(Destination::CreateWorktree(
                        self.sessions.selected(&ui.sessions).cloned(),
                    )),
                    effects: Vec::new(),
                    ..Default::default()
                }
            }
            ListAction::NextSurface => Transition {
                redraw: if self.agents.surface(&ui.agents) == Surface::Agents {
                    self.agents.focus_sessions(&mut ui.agents)
                } else {
                    self.agents.focus_agents(&mut ui.agents)
                },
                ..Default::default()
            },
            ListAction::FocusAgents => Transition {
                redraw: self.agents.focus_agents(&mut ui.agents),
                ..Default::default()
            },
            ListAction::FocusSessions => Transition {
                redraw: self.agents.focus_sessions(&mut ui.agents),
                ..Default::default()
            },
            ListAction::Cancel => dismiss([Effect::HidePlugin]),
            _ => Transition::default(),
        }
    }

    pub fn view(
        &self,
        ui: &UiState,
        hints: Vec<(&'static str, &'static str)>,
        rename_draft: Option<&String>,
        agent_refresh: RefreshView,
        refresh: PickerRefreshView,
    ) -> ui::model::ListView {
        let picker = self.sessions.view(&ui.sessions);
        let sessions = picker
            .items
            .iter()
            .enumerate()
            .map(|(index, (session, matched))| ui::model::SessionRow {
                name: session.name.clone(),
                matched: matched.clone(),
                active: session.is_active(),
                current: session.is_current(),
                branch: session.branch.clone(),
                nested: session.nested_worktree,
                last_sibling: session.nested_worktree
                    && picker
                        .items
                        .get(index + 1)
                        .is_none_or(|(next, _)| !next.nested_worktree),
                agent: session_agent_state(
                    self.agents
                        .records()
                        .iter()
                        .filter(|agent| agent.target.session_name == session.name)
                        .map(|agent| visual_state(display_state(agent))),
                ),
                rename_draft: session
                    .is_current()
                    .then(|| rename_draft.cloned())
                    .flatten(),
            })
            .collect();
        ui::model::ListView {
            query: picker.query.into(),
            sessions,
            selected_session: picker.selected,
            filtered_count: picker.filtered_count,
            total_count: picker.total_count,
            agents: self
                .agents
                .records()
                .iter()
                .map(|agent| ui::model::AgentRow {
                    session_name: agent.target.session_name.clone(),
                    label: agent.agent_label.clone(),
                    pane_id: agent.target.pane_id,
                    preview: agent.activity.preview.clone(),
                    state: visual_state(display_state(agent)),
                })
                .collect(),
            selected_agent: self.agents.selected(&ui.agents),
            focus: match self.agents.surface(&ui.agents) {
                Surface::Sessions => ui::model::Focus::Sessions,
                Surface::Agents => ui::model::Focus::Agents,
            },
            spinner_tick: self.agents.spinner_tick(&ui.agents),
            agent_refresh,
            refresh,
            hints,
        }
    }

    pub fn hints(&self, ui: &UiState) -> Vec<(&'static str, &'static str)> {
        if self.agents.surface(&ui.agents) == Surface::Agents {
            return vec![
                ("enter", "jump"),
                ("tab/ctrl-h", "sessions"),
                ("esc", "close"),
            ];
        }
        let mut hints = input::LIST_HINTS.to_vec();
        if !self.agents.records().is_empty() {
            hints.insert(1.min(hints.len()), ("tab/ctrl-l", "agents"));
        }
        hints
    }

    fn session_order(&self) -> Vec<String> {
        self.sessions
            .items()
            .iter()
            .filter(|session| session.is_active())
            .map(|session| session.name.clone())
            .collect()
    }

    fn retain_agents(&mut self, ui: &mut UiState) {
        let records = self.agents.records().to_vec();
        self.agents
            .set_records(&mut ui.agents, records, &self.session_order());
    }
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

fn visual_state(state: DisplayState) -> ui::model::AgentState {
    match state {
        DisplayState::Blocked => ui::model::AgentState::Blocked,
        DisplayState::Working => ui::model::AgentState::Working,
        DisplayState::Done => ui::model::AgentState::Done,
        DisplayState::Idle => ui::model::AgentState::Idle,
        DisplayState::Unknown => ui::model::AgentState::Unknown,
    }
}

fn session_agent_state(
    states: impl Iterator<Item = ui::model::AgentState>,
) -> Option<ui::model::AgentState> {
    states.min_by_key(|state| match state {
        ui::model::AgentState::Blocked => 0,
        ui::model::AgentState::Working => 1,
        ui::model::AgentState::Done => 2,
        ui::model::AgentState::Idle => 3,
        ui::model::AgentState::Unknown => 4,
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
        screen.replace_sessions(
            &mut ui,
            vec![SessionInfo {
                name: "destination".into(),
                lifecycle: SessionLifecycle::Active { current: false },
                ..Default::default()
            }],
        );

        let transition = screen.apply_action(&mut ui, ListAction::Confirm, "bridge");

        assert!(matches!(
            transition.effects.as_slice(),
            [Effect::HidePlugin, Effect::SwitchSession { name }] if name == "destination"
        ));
    }

    #[test]
    fn session_status_uses_the_highest_priority_agent() {
        use ui::model::AgentState;

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
}
