use std::collections::{BTreeMap, BTreeSet};

use agent_core::{AgentList, AgentRecord, AgentState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Sessions,
    Agents,
}

#[derive(Default)]
pub struct AgentTracker {
    records: Vec<AgentRecord>,
}

pub struct UiState {
    selected: usize,
    surface: Surface,
    spinner_tick: usize,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected: 0,
            surface: Surface::Sessions,
            spinner_tick: 0,
        }
    }
}

impl AgentTracker {
    pub fn records(&self) -> &[AgentRecord] {
        &self.records
    }
    pub fn selected(&self, ui: &UiState) -> Option<usize> {
        (!self.records.is_empty()).then_some(ui.selected)
    }
    pub fn selected_agent(&self, ui: &UiState) -> Option<&AgentRecord> {
        self.records.get(ui.selected)
    }
    pub fn surface(&self, ui: &UiState) -> Surface {
        ui.surface
    }
    pub fn spinner_tick(&self, ui: &UiState) -> usize {
        ui.spinner_tick
    }
    pub fn advance_spinner(&self, ui: &mut UiState) {
        ui.spinner_tick = ui.spinner_tick.wrapping_add(1);
    }
    pub fn has_working(&self) -> bool {
        self.records.iter().any(|r| r.state == AgentState::Working)
    }
    pub fn set_records(
        &mut self,
        ui: &mut UiState,
        mut records: Vec<AgentRecord>,
        session_order: &[String],
    ) -> bool {
        let active: BTreeSet<_> = session_order.iter().collect();
        records.retain(|r| active.contains(&r.target.session_name));
        let order: BTreeMap<_, _> = session_order
            .iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        records.sort_by(|a, b| {
            let key = |r: &AgentRecord| {
                (
                    order
                        .get(&r.target.session_name)
                        .copied()
                        .unwrap_or(usize::MAX),
                    r.target.pane_id,
                )
            };
            key(a).cmp(&key(b)).then_with(|| a.agent_label.cmp(&b.agent_label))
        });
        if records == self.records {
            return false;
        }
        self.records = records;
        ui.selected = ui.selected.min(self.records.len().saturating_sub(1));
        if self.records.is_empty() {
            ui.surface = Surface::Sessions;
        }
        true
    }

    pub fn move_down(&self, ui: &mut UiState) {
        if ui.selected + 1 < self.records.len() {
            ui.selected += 1;
        }
    }
    pub fn move_up(&self, ui: &mut UiState) {
        ui.selected = ui.selected.saturating_sub(1);
    }
    pub fn focus_agents(&self, ui: &mut UiState) -> bool {
        if self.records.is_empty() {
            return false;
        }
        let changed = ui.surface != Surface::Agents;
        ui.surface = Surface::Agents;
        changed
    }
    pub fn focus_sessions(&self, ui: &mut UiState) -> bool {
        let changed = ui.surface != Surface::Sessions;
        ui.surface = Surface::Sessions;
        changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentListParseError {
    InvalidJson,
    UnsupportedSchema { found: u32 },
}

pub fn parse_list(stdout: &[u8]) -> Result<Vec<AgentRecord>, AgentListParseError> {
    serde_json::from_slice::<AgentList>(stdout)
        .map_err(|_| AgentListParseError::InvalidJson)
        .and_then(|list| {
            if list.schema_version != agent_core::SCHEMA_VERSION {
                return Err(AgentListParseError::UnsupportedSchema {
                    found: list.schema_version,
                });
            }
            Ok(list.agents)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Activity, AgentTarget};

    fn record(session: &str, pane: u32) -> AgentRecord {
        AgentRecord {
            id: format!("{session}-{pane}"),
            agent_label: "codex".into(),
            state: AgentState::Working,
            seen: true,
            target: AgentTarget {
                session_name: session.into(),
                pane_id: pane,
            },
            activity: Activity {
                kind: "tool".into(),
                label: "using Bash".into(),
                preview: "cargo test".into(),
            },
            owner_pid: 1,
            process_fingerprint: "x".into(),
            agent_session_id: "a".into(),
            observed_at_ms: 1,
            pending_permissions: vec![],
        }
    }

    #[test]
    fn filters_and_orders_by_session_then_pane() {
        let mut tracker = AgentTracker::default();
        let mut ui = UiState::default();
        tracker.set_records(
            &mut ui,
            vec![
                record("b", 4),
                record("a", 3),
                record("gone", 1),
                record("a", 2),
            ],
            &["a".into(), "b".into()],
        );
        assert_eq!(
            tracker
                .records
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a-2", "a-3", "b-4"]
        );
    }
}
