use zellij_tile::prelude::{BareKey, KeyModifier, KeyWithModifier};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Action {
    MoveDown,
    MoveUp,
    PushChar(char),
    PopChar,
    Confirm,
    Delete,
    Rename,
    CreateNew,
    NextField,
    Cancel,
}

pub struct Binding {
    pub key: BareKey,
    pub ctrl: bool,
    pub action: Action,
    /// (key_display, description) shown in the hint bar; None = don't show
    pub hint: Option<(&'static str, &'static str)>,
}

pub static BINDINGS: &[Binding] = &[
    Binding { key: BareKey::Down,      ctrl: false, action: Action::MoveDown, hint: None },
    Binding { key: BareKey::Up,        ctrl: false, action: Action::MoveUp,   hint: None },
    Binding { key: BareKey::Char('j'), ctrl: true,  action: Action::MoveDown, hint: None },
    Binding { key: BareKey::Char('k'), ctrl: true,  action: Action::MoveUp,   hint: None },
    Binding { key: BareKey::Backspace, ctrl: false, action: Action::PopChar,  hint: None },
    Binding { key: BareKey::Char('d'), ctrl: true,  action: Action::Delete,   hint: Some(("ctrl+d", "delete")) },
    Binding { key: BareKey::Char('r'), ctrl: true,  action: Action::Rename,  hint: Some(("ctrl+r", "rename current")) },
    Binding { key: BareKey::Char('n'), ctrl: true,  action: Action::CreateNew, hint: Some(("ctrl+n", "new session")) },
    Binding { key: BareKey::Tab,       ctrl: false, action: Action::NextField, hint: None },
    Binding { key: BareKey::Enter,     ctrl: false, action: Action::Confirm,  hint: Some(("enter",  "switch")) },
    Binding { key: BareKey::Esc,       ctrl: false, action: Action::Cancel,   hint: Some(("esc",    "close"))  },
];

pub fn key_to_action(key: &KeyWithModifier) -> Option<Action> {
    let ctrl = key.key_modifiers.contains(&KeyModifier::Ctrl);
    if let Some(b) = BINDINGS.iter().find(|b| b.key == key.bare_key && b.ctrl == ctrl) {
        return Some(b.action);
    }
    if let (BareKey::Char(c), false) = (key.bare_key, ctrl) {
        return Some(Action::PushChar(c));
    }
    None
}

pub fn hints() -> impl Iterator<Item = (&'static str, &'static str)> {
    BINDINGS.iter().filter_map(|b| b.hint)
}
