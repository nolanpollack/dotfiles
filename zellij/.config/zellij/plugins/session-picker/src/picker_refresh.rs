//! Coordinates refresh work triggered when the picker is opened.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshView {
    Ready,
    Refreshing,
    Failed,
}

pub struct PickerRefresh {
    next_id: u64,
    current: Option<Current>,
    queued: bool,
    failed: bool,
}

#[derive(Debug, Clone, Copy)]
struct Current {
    id: RefreshId,
    sessions_pending: bool,
    agents_pending: bool,
}

impl Default for PickerRefresh {
    fn default() -> Self {
        Self {
            next_id: 0,
            current: None,
            queued: false,
            failed: false,
        }
    }
}

impl PickerRefresh {
    /// Starts a refresh, or records one request to run immediately after the current refresh.
    pub fn request(&mut self, include_agents: bool) -> Option<RefreshId> {
        if self.current.is_some() {
            self.queued = true;
            return None;
        }
        Some(self.start(include_agents))
    }

    pub fn finish_sessions(&mut self, id: RefreshId, success: bool) -> Option<RefreshId> {
        let current = self.current.as_mut().filter(|current| current.id == id)?;
        current.sessions_pending = false;
        if !success {
            self.failed = true;
        }
        self.finish_current_if_ready()
    }

    pub fn finish_agents(&mut self, id: RefreshId, success: bool) -> Option<RefreshId> {
        let current = self.current.as_mut().filter(|current| current.id == id)?;
        current.agents_pending = false;
        if !success {
            self.failed = true;
        }
        self.finish_current_if_ready()
    }

    pub fn view(&self) -> RefreshView {
        if self.current.is_some() {
            RefreshView::Refreshing
        } else if self.failed {
            RefreshView::Failed
        } else {
            RefreshView::Ready
        }
    }

    pub fn is_refreshing(&self) -> bool {
        self.current.is_some()
    }

    fn start(&mut self, include_agents: bool) -> RefreshId {
        let id = RefreshId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.failed = false;
        self.current = Some(Current {
            id,
            sessions_pending: true,
            agents_pending: include_agents,
        });
        id
    }

    fn finish_current_if_ready(&mut self) -> Option<RefreshId> {
        let current = self.current?;
        if current.sessions_pending || current.agents_pending {
            return None;
        }
        self.current = None;
        self.queued.then(|| {
            self.queued = false;
            self.start(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_requests_coalesce_to_one_follow_up() {
        let mut refresh = PickerRefresh::default();
        let first = refresh.request(true).unwrap();
        assert!(refresh.request(true).is_none());
        assert!(refresh.request(true).is_none());
        assert!(refresh.finish_sessions(first, true).is_none());
        assert_eq!(refresh.finish_agents(first, true), Some(RefreshId(1)));
    }
}
