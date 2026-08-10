use std::collections::{BTreeMap, BTreeSet};

use agent_core::{Agent, AgentList, AgentState};

#[derive(Default)]
pub struct Agents {
    items: Vec<Agent>,
}

impl Agents {
    pub fn items(&self) -> &[Agent] {
        &self.items
    }
    pub fn has_working(&self) -> bool {
        self.items
            .iter()
            .any(|agent| agent.state == AgentState::Working)
    }
    pub fn replace(&mut self, mut incoming: Vec<Agent>, session_order: &[String]) -> bool {
        let active: BTreeSet<_> = session_order.iter().collect();
        incoming.retain(|agent| active.contains(&agent.target.session_name));
        let order: BTreeMap<_, _> = session_order
            .iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        incoming.sort_by(|a, b| {
            let key = |agent: &Agent| {
                (
                    order
                        .get(&agent.target.session_name)
                        .copied()
                        .unwrap_or(usize::MAX),
                    agent.target.pane_id,
                )
            };
            key(a)
                .cmp(&key(b))
                .then_with(|| a.agent_label.cmp(&b.agent_label))
        });
        if incoming == self.items {
            return false;
        }
        self.items = incoming;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentListParseError {
    InvalidJson,
    UnsupportedSchema { found: u32 },
}

pub fn parse_list(stdout: &[u8]) -> Result<Vec<Agent>, AgentListParseError> {
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

    fn agent(session: &str, pane: u32) -> Agent {
        Agent {
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
        let mut agents = Agents::default();
        agents.replace(
            vec![
                agent("b", 4),
                agent("a", 3),
                agent("gone", 1),
                agent("a", 2),
            ],
            &["a".into(), "b".into()],
        );
        assert_eq!(
            agents
                .items
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a-2", "a-3", "b-4"]
        );
    }
}
