use agent_core::AgentRecord;

use crate::effects::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadState {
    Loading,
    Refreshing { request_id: RequestId, cached: bool },
    Ready,
    Failed { cached: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshView {
    Loading,
    Ready,
    Refreshing { cached: bool },
    Failed { cached: bool },
}

pub struct AgentRefresh {
    permissions_granted: bool,
    state: LoadState,
    next_request_id: u64,
}

impl Default for AgentRefresh {
    fn default() -> Self {
        Self {
            permissions_granted: false,
            state: LoadState::Loading,
            next_request_id: 0,
        }
    }
}

impl AgentRefresh {
    pub fn restore_cached(&mut self) {
        self.state = LoadState::Ready;
    }

    pub fn has_cached_data(&self) -> bool {
        self.state.has_cache()
    }

    pub fn grant_permissions(&mut self) {
        if self.permissions_granted {
            return;
        }
        self.permissions_granted = true;
    }

    pub fn request(&mut self, bridge: &str) -> Option<Effect> {
        self.start_if_ready(bridge)
    }

    pub fn finish(
        &mut self,
        request_id: RequestId,
        result: Result<Vec<AgentRecord>, ()>,
    ) -> Option<Result<Vec<AgentRecord>, ()>> {
        let LoadState::Refreshing {
            request_id: active_request_id,
            cached,
        } = self.state
        else {
            return None;
        };
        if request_id != active_request_id {
            return None;
        }
        Some(match result {
            Ok(records) => {
                self.state = LoadState::Ready;
                Ok(records)
            }
            Err(_) => {
                self.state = LoadState::Failed { cached };
                Err(())
            }
        })
    }

    pub fn permissions_granted(&self) -> bool {
        self.permissions_granted
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.state, LoadState::Refreshing { .. })
    }

    pub fn view(&self) -> RefreshView {
        match self.state {
            LoadState::Loading => RefreshView::Loading,
            LoadState::Refreshing { cached, .. } => RefreshView::Refreshing { cached },
            LoadState::Ready => RefreshView::Ready,
            LoadState::Failed { cached } => RefreshView::Failed { cached },
        }
    }

    fn start_if_ready(&mut self, bridge: &str) -> Option<Effect> {
        if !self.permissions_granted || matches!(self.state, LoadState::Refreshing { .. }) {
            return None;
        }
        let request_id = RequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.state = LoadState::Refreshing {
            request_id,
            cached: self.state.has_cache(),
        };
        Some(Effect::FetchAgents {
            bridge: bridge.to_string(),
            request_id,
        })
    }
}

impl LoadState {
    fn has_cache(self) -> bool {
        matches!(self, Self::Ready | Self::Failed { cached: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_id(effect: Effect) -> RequestId {
        match effect {
            Effect::FetchAgents { request_id, .. } => request_id,
            _ => panic!("expected agent fetch"),
        }
    }

    #[test]
    fn visibility_and_permission_are_both_required() {
        let mut refresh = AgentRefresh::default();
        assert!(refresh.request("bridge").is_none());
        refresh.grant_permissions();
        let effect = refresh.request("bridge").unwrap();
        assert_eq!(request_id(effect), RequestId(0));
        assert_eq!(refresh.view(), RefreshView::Refreshing { cached: false });
    }

    #[test]
    fn stale_results_are_ignored() {
        let mut refresh = AgentRefresh::default();
        refresh.grant_permissions();
        refresh.request("bridge");
        assert!(refresh.finish(RequestId(99), Ok(Vec::new())).is_none());
    }

    #[test]
    fn failure_after_success_keeps_cached_data() {
        let mut refresh = AgentRefresh::default();
        refresh.grant_permissions();
        refresh.request("bridge");
        refresh.finish(RequestId(0), Ok(Vec::new()));
        refresh.request("bridge");
        refresh.finish(RequestId(1), Err(()));
        assert_eq!(refresh.view(), RefreshView::Failed { cached: true });
    }
}
