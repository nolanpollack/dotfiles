//! The sole adapter between application data and Zellij's plugin API.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zellij_tile::prelude::*;

use session_picker::app::Message;
use session_picker::effects::{Effect, GitLookup};
use session_picker::host_protocol::{self, ResultKind};
use session_picker::input::Key;
use session_picker::sessions::{SessionInfo, SessionLifecycle};
use session_picker::ui::{Theme, ThemeOverrides, ThemePalette};

fn default_style() -> Style {
    Style {
        colors: DEFAULT_STYLES,
        rounded_corners: true,
        hide_session_name: false,
    }
}

pub struct ThemeAdapter {
    overrides: ThemeOverrides,
    fallback: ThemePalette,
    last_style: Option<Style>,
}

impl Default for ThemeAdapter {
    fn default() -> Self {
        Self {
            overrides: ThemeOverrides::default(),
            fallback: theme_palette(&default_style()),
            last_style: None,
        }
    }
}

impl ThemeAdapter {
    pub fn from_config(
        config: &BTreeMap<String, String>,
        cached_theme: Option<ThemePalette>,
    ) -> Self {
        Self {
            overrides: ThemeOverrides::from_config(config),
            fallback: cached_theme.unwrap_or_else(|| theme_palette(&default_style())),
            last_style: None,
        }
    }

    pub fn current(&self) -> Theme {
        Theme::from_palette(self.persistent_palette(), &self.overrides)
    }

    pub fn accept(&mut self, incoming: Style) -> Option<Theme> {
        let configured = self
            .last_style
            .as_ref()
            .map(|style| style.colors != DEFAULT_STYLES)
            .unwrap_or_else(|| self.fallback != theme_palette(&default_style()));
        if incoming.colors == DEFAULT_STYLES && configured {
            return None;
        }
        self.last_style = Some(incoming);
        Some(self.current())
    }

    pub fn persistent_palette(&self) -> ThemePalette {
        self.last_style
            .as_ref()
            .map(theme_palette)
            .unwrap_or_else(|| self.fallback.clone())
    }
}

fn theme_palette(style: &Style) -> ThemePalette {
    let colors = &style.colors;
    let frame = colors
        .frame_unselected
        .map(|frame| frame.base)
        .unwrap_or(colors.frame_selected.base);
    ThemePalette {
        separator_fg: palette(frame),
        query_fg: palette(colors.text_unselected.base),
        list_match_fg: palette(colors.list_unselected.emphasis_0),
        list_normal_fg: palette(colors.list_unselected.base),
        list_inactive_fg: palette(colors.text_unselected.background),
        list_current_marker_fg: palette(colors.list_unselected.emphasis_1),
        list_selected_bg: palette(colors.list_selected.background),
        hint_key_fg: palette(colors.list_unselected.emphasis_1),
        hint_desc_fg: palette(colors.list_unselected.base),
        status_count_fg: palette(colors.text_unselected.background),
        error_fg: palette(colors.exit_code_error.base),
        agent_blocked_fg: palette(colors.exit_code_error.base),
        agent_working_fg: palette(colors.list_unselected.emphasis_2),
        agent_done_fg: palette(colors.exit_code_success.base),
        agent_idle_fg: palette(colors.list_unselected.emphasis_1),
        agent_unknown_fg: palette(colors.text_unselected.background),
        panel_divider_fg: palette(frame),
    }
}

fn palette(color: PaletteColor) -> ratatui::style::Color {
    match color {
        PaletteColor::Rgb((r, g, b)) => ratatui::style::Color::Rgb(r, g, b),
        PaletteColor::EightBit(index) => ratatui::style::Color::Indexed(index),
    }
}

pub fn key(key: KeyWithModifier) -> Key {
    let ctrl = key.key_modifiers.contains(&KeyModifier::Ctrl);
    match (key.bare_key, ctrl) {
        (BareKey::Up, _) => Key::Up,
        (BareKey::Down, _) => Key::Down,
        (BareKey::Enter, _) => Key::Enter,
        (BareKey::Esc, _) => Key::Escape,
        (BareKey::Backspace, _) => Key::Backspace,
        (BareKey::Tab, _) => Key::Tab,
        (BareKey::Char(c), true) => Key::Ctrl(c),
        (BareKey::Char(c), false) => Key::Char(c),
        _ => Key::Other,
    }
}

pub fn sessions(
    live: Vec<zellij_tile::prelude::SessionInfo>,
    resurrectable: Vec<(String, std::time::Duration)>,
) -> Vec<SessionInfo> {
    let mut result: Vec<_> = live
        .into_iter()
        .map(|session| SessionInfo {
            name: session.name,
            lifecycle: SessionLifecycle::Active {
                current: session.is_current_session,
            },
            ..Default::default()
        })
        .collect();
    let mut dead: Vec<_> = resurrectable
        .into_iter()
        .map(|(name, _)| SessionInfo {
            name,
            lifecycle: SessionLifecycle::Resurrectable,
            ..Default::default()
        })
        .collect();
    dead.sort_by(|a, b| a.name.cmp(&b.name));
    result.extend(dead);
    result
}

pub fn decode_command_result(
    exit_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
    context: &BTreeMap<String, String>,
) -> Option<Message> {
    host_protocol::decode_result(exit_code, stdout, stderr, context)
}

pub fn execute(effect: Effect, plugin_id: u32) -> Option<Message> {
    match effect {
        Effect::RefreshSessions => get_session_list().ok().map(|snapshot| {
            Message::SessionsLoaded(sessions(
                snapshot.live_sessions,
                snapshot.resurrectable_sessions,
            ))
        }),
        Effect::RefreshPickerSessions { refresh_id } => {
            let result = get_session_list()
                .map(|snapshot| sessions(snapshot.live_sessions, snapshot.resurrectable_sessions))
                .map_err(|_| ());
            Some(Message::PickerSessionsFinished { refresh_id, result })
        }
        Effect::LookupGit(GitLookup::BySessionName { session_name }) => {
            let args = session_picker::git_info::lookup_by_name_args(&session_name);
            run_tagged(args, ResultKind::Git, Some(session_name));
            None
        }
        Effect::LookupGit(GitLookup::AtDirectory { session_name, cwd }) => {
            let args = session_picker::git_info::lookup_at_dir_args(&cwd);
            run_tagged(args, ResultKind::Git, Some(session_name));
            None
        }
        Effect::FetchDirectoryCandidates => {
            run_tagged(
                session_picker::create::discovery::zoxide_list_args(),
                ResultKind::Directory,
                None,
            );
            None
        }
        Effect::ValidateWorktree { request } => {
            run_tagged(
                session_picker::create::worktree::validation_args(&request),
                ResultKind::WorktreeValidation,
                None,
            );
            None
        }
        Effect::CreateWorktree { request } => {
            run_tagged(
                session_picker::create::worktree::creation_args(&request),
                ResultKind::WorktreeCreation,
                None,
            );
            None
        }
        Effect::SwitchSession { name } => {
            switch_session_with_focus(&name, None, None);
            None
        }
        Effect::SwitchToAgent {
            session_name,
            pane_id,
        } => {
            switch_session_with_focus(&session_name, None, Some((pane_id, false)));
            None
        }
        Effect::FetchAgents { bridge, request_id } => {
            run_tagged(
                vec![bridge, "list".into()],
                ResultKind::Agents { request_id },
                None,
            );
            None
        }
        Effect::MarkAgentSeen { bridge, id } => {
            run_untagged(vec![bridge, "mark-seen".into(), id]);
            None
        }
        Effect::RenameAgentSession { bridge, old, new } => {
            run_untagged(vec![bridge, "rename-session".into(), old, new]);
            None
        }
        Effect::CreateSession { name, cwd } => {
            switch_session_with_cwd(Some(&name), Some(cwd));
            None
        }
        Effect::RenameCurrentSession { name } => {
            rename_session(&name);
            None
        }
        Effect::DeleteSession { name, lifecycle } => {
            if matches!(lifecycle, SessionLifecycle::Active { .. }) {
                kill_sessions(&[name]).ok();
            } else {
                delete_dead_session(&name).ok();
            }
            None
        }
        Effect::RenamePluginPane { title } => {
            rename_plugin_pane(plugin_id, title);
            None
        }
        Effect::HidePlugin => {
            hide_self();
            None
        }
        Effect::ScheduleAnimationFrame => {
            set_timeout(0.09);
            None
        }
    }
}

fn run_tagged(args: Vec<String>, kind: ResultKind, session: Option<String>) {
    run(args, host_protocol::result_context(kind, session));
}

fn run_untagged(args: Vec<String>) {
    run(args, BTreeMap::new());
}

fn run(args: Vec<String>, context: BTreeMap<String, String>) {
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command_with_env_variables_and_cwd(&borrowed, BTreeMap::new(), PathBuf::from("."), context);
}
