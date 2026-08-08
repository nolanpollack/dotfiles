//! Zellij plugin lifecycle wrapper around the host-independent application.

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use zellij_tile::prelude::*;

use session_picker::app::{App, Message, Update};
use session_picker::input::{Key, ListAction};
use session_picker::persistence::{PersistentState, SnapshotStore};
use session_picker::ui;

use crate::zellij;

pub struct State {
    app: App,
    plugin_id: u32,
    themes: zellij::ThemeAdapter,
    store: SnapshotStore,
}

impl Default for State {
    fn default() -> Self {
        let themes = zellij::ThemeAdapter::default();
        Self {
            app: App::new(themes.current(), "session-picker-agent-bridge".into()),
            plugin_id: 0,
            themes,
            store: SnapshotStore::default(),
        }
    }
}

impl State {
    fn apply(&mut self, mut update: Update) -> bool {
        let mut redraw = update.redraw;
        let mut pending: VecDeque<_> = std::mem::take(&mut update.effects).into();
        while let Some(effect) = pending.pop_front() {
            if let Some(message) = zellij::execute(effect, self.plugin_id) {
                let next = self.app.update(message);
                redraw |= next.redraw;
                pending.extend(next.effects);
            }
        }
        self.checkpoint();
        redraw
    }

    fn checkpoint(&mut self) {
        let state = PersistentState::new(
            self.themes.persistent_palette(),
            self.app.persistent_state(),
        );
        if let Err(error) = self.store.save_if_changed(&state) {
            eprintln!("session-picker cache save failed: {error}");
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        let ids = get_plugin_ids();
        self.plugin_id = ids.plugin_id;
        let writer_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        self.store = SnapshotStore::at("/data/session-picker-state-v1.json", writer_id);
        let restored = match self.store.load() {
            Ok(state) => state,
            Err(error) => {
                eprintln!("session-picker cache load failed: {error}");
                None
            }
        };
        let cached_theme = restored.as_ref().and_then(|state| state.theme.clone());
        let cached_app = restored.map(|state| state.app).unwrap_or_default();
        self.themes = zellij::ThemeAdapter::from_config(&configuration, cached_theme);
        let bridge = configuration
            .get("agent_bridge_path")
            .filter(|path| !path.is_empty())
            .cloned()
            .unwrap_or_else(|| "session-picker-agent-bridge".into());
        self.app = App::restore(self.themes.current(), bridge, cached_app);
        self.app
            .set_worktree_config(session_picker::create::worktree::Config {
                branch_prefix: configuration
                    .get("branch_prefix")
                    .cloned()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "nolanpollack".into()),
                worktree_root: configuration
                    .get("worktree_root")
                    .map(Into::into)
                    .unwrap_or_else(|| "/Users/nolanpollack/stripe/worktrees".into()),
            });
        self.apply(self.app.initial_update());

        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::ReadCliPipes,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::PermissionRequestResult,
            EventType::ModeUpdate,
            EventType::SessionUpdate,
            EventType::RunCommandResult,
            EventType::HostFolderChanged,
            EventType::Visible,
            EventType::Timer,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let message = match event {
            Event::PermissionRequestResult(status) => {
                if !matches!(status, PermissionStatus::Granted) {
                    return false;
                }
                Message::PermissionGranted
            }
            Event::ModeUpdate(info) => {
                let Some(theme) = self.themes.accept(info.style) else {
                    return false;
                };
                Message::ThemeChanged(theme)
            }
            Event::Key(key) => Message::Key(zellij::key(key)),
            Event::SessionUpdate(live, resurrectable) => {
                Message::SessionsLoaded(zellij::sessions(live, resurrectable))
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                let Some(message) =
                    zellij::decode_command_result(exit_code, &stdout, &stderr, &context)
                else {
                    return false;
                };
                message
            }
            Event::HostFolderChanged(cwd) => Message::HostFolderChanged(cwd),
            Event::Visible(visible) => Message::VisibilityChanged(visible),
            Event::Timer(_) => Message::AnimationFrame,
            _ => return false,
        };
        let update = self.app.update(message);
        self.apply(update)
    }

    fn pipe(&mut self, pipe: PipeMessage) -> bool {
        // Sent by agent-bridge whenever it observes an agent state change, so the picker's
        // cache stays fresh even while its instance is hidden (Zellij never emits a lifecycle
        // event for a hidden-but-still-running plugin).
        if pipe.name == "agent-refresh" {
            let update = self.app.update(Message::ExternalAgentUpdate);
            return self.apply(update);
        }
        if pipe.name != "nav" {
            return false;
        }
        let update = match pipe.payload.as_deref() {
            // Ctrl-j/k are intercepted by the pane navigator. Route its forwarded messages as
            // keys so create forms receive the same up/down navigation as raw arrow keys.
            Some("down") => self.app.update(Message::Key(Key::Down)),
            Some("up") => self.app.update(Message::Key(Key::Up)),
            Some("right") => self
                .app
                .update(Message::ListAction(ListAction::FocusAgents)),
            Some("left") => self
                .app
                .update(Message::ListAction(ListAction::FocusSessions)),
            _ => return false,
        };
        self.apply(update)
    }

    fn render(&mut self, rows: usize, cols: usize) {
        ui::render(self.app.view(), self.app.theme(), rows, cols);
    }
}
