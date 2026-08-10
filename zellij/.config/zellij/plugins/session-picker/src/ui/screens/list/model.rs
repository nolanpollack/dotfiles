use crate::agent_refresh::RefreshView as AgentRefreshView;
use crate::picker_refresh::RefreshView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sessions,
    Agents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Blocked,
    Working,
    Done,
    Idle,
    Unknown,
}

pub struct SessionRow {
    pub name: String,
    pub matched: Vec<usize>,
    pub active: bool,
    pub current: bool,
    pub branch: Option<String>,
    pub nested: bool,
    pub last_sibling: bool,
    pub agent: Option<AgentState>,
    pub rename_draft: Option<String>,
}

pub struct AgentRow {
    pub session_name: String,
    pub label: String,
    pub pane_id: u32,
    pub preview: String,
    pub state: AgentState,
}

pub struct ListView {
    pub query: String,
    pub sessions: Vec<SessionRow>,
    pub selected_session: Option<usize>,
    pub filtered_count: usize,
    pub total_count: usize,
    pub agents: Vec<AgentRow>,
    pub selected_agent: Option<usize>,
    pub focus: Focus,
    pub spinner_tick: usize,
    pub agent_refresh: AgentRefreshView,
    pub refresh: RefreshView,
    pub hints: Vec<(&'static str, &'static str)>,
}
