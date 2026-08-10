//! Zellij plugin lifecycle wrapper around the host-independent application.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use zellij_tile::prelude as Zellij;

use session_picker::app::{App, Message, Update};
use session_picker::input::{Key, ListAction};
use session_picker::persistence::{PersistentState, SnapshotStore};
use session_picker::ui;

use crate::config::PluginConfig;
use crate::zellij;

const SNAPSHOT_PATH: &str = "/data/session-picker-state-v1.json";

pub struct State {
    app: App,
    plugin_id: u32,
    themes: zellij::ThemeAdapter,
    store: SnapshotStore,
    animation_timer_armed: bool,
}

impl Default for State {
    fn default() -> Self {
        let themes = zellij::ThemeAdapter::default();
        let config = PluginConfig::default();
        Self {
            app: App::new(themes.current(), config.app),
            plugin_id: 0,
            themes,
            store: SnapshotStore::default(),
            animation_timer_armed: false,
        }
    }
}

impl Zellij::ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        *self = Self::bootstrap(configuration);
        self.apply(self.app.initial_update());

        Zellij::request_permission(&[
            Zellij::PermissionType::ReadApplicationState,
            Zellij::PermissionType::ChangeApplicationState,
            Zellij::PermissionType::RunCommands,
            Zellij::PermissionType::ReadCliPipes,
        ]);
        Zellij::subscribe(&[
            Zellij::EventType::Key,
            Zellij::EventType::PermissionRequestResult,
            Zellij::EventType::ModeUpdate,
            Zellij::EventType::SessionUpdate,
            Zellij::EventType::RunCommandResult,
            Zellij::EventType::HostFolderChanged,
            Zellij::EventType::Visible,
            Zellij::EventType::Timer,
        ]);
    }

    fn update(&mut self, event: Zellij::Event) -> bool {
        let message = match event {
            Zellij::Event::PermissionRequestResult(status) => {
                if !matches!(status, Zellij::PermissionStatus::Granted) {
                    return false;
                }
                Message::PermissionGranted
            }
            Zellij::Event::ModeUpdate(info) => {
                let Some(theme) = self.themes.accept(info.style) else {
                    return false;
                };
                Message::ThemeChanged(theme)
            }
            Zellij::Event::Key(key) => Message::Key(zellij::key(key)),
            Zellij::Event::SessionUpdate(live, resurrectable) => {
                Message::SessionsLoaded(zellij::sessions(live, resurrectable))
            }
            Zellij::Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                let Some(message) =
                    zellij::decode_command_result(exit_code, &stdout, &stderr, &context)
                else {
                    return false;
                };
                message
            }
            Zellij::Event::HostFolderChanged(cwd) => Message::HostFolderChanged(cwd),
            Zellij::Event::Visible(visible) => Message::VisibilityChanged(visible),
            Zellij::Event::Timer(_) => {
                self.animation_timer_armed = false;
                Message::AnimationFrame
            }
            _ => return false,
        };

        let update = self.app.update(message);
        self.apply(update)
    }

    fn pipe(&mut self, pipe: Zellij::PipeMessage) -> bool {
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
        ui::render(ui::view(&self.app), self.app.theme(), rows, cols);
    }
}

impl State {
    // Zellij constructs Default before passing runtime configuration to load.
    fn bootstrap(configuration: BTreeMap<String, String>) -> Self {
        let plugin_id = Zellij::get_plugin_ids().plugin_id;
        let mut store = SnapshotStore::at(SNAPSHOT_PATH);
        let restored = Self::load_snapshot(&mut store);
        let cached_theme = restored.as_ref().and_then(|state| state.theme.clone());
        let cached_app = restored.map(|state| state.app).unwrap_or_default();
        let PluginConfig {
            app: app_config,
            theme_overrides,
        } = PluginConfig::parse(configuration);
        let themes = zellij::ThemeAdapter::from_overrides(theme_overrides, cached_theme);
        let app = App::restore(themes.current(), app_config, cached_app);

        Self {
            app,
            plugin_id,
            themes,
            store,
            animation_timer_armed: false,
        }
    }

    fn load_snapshot(store: &mut SnapshotStore) -> Option<PersistentState> {
        match store.load() {
            Ok(state) => state,
            Err(error) => {
                eprintln!("session-picker cache load failed: {error}");
                None
            }
        }
    }

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
        self.reconcile_subscriptions();
        self.checkpoint();
        redraw
    }

    fn reconcile_subscriptions(&mut self) {
        if !ui::subscriptions(&self.app).animation_frame {
            self.animation_timer_armed = false;
        } else if !self.animation_timer_armed {
            self.animation_timer_armed = true;
            zellij::schedule_animation_frame();
        }
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
