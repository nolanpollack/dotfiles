//! Host-independent keyboard input and screen-specific key maps.

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Key {
    Up,
    Down,
    Enter,
    Escape,
    Backspace,
    Tab,
    Char(char),
    Ctrl(char),
    Other,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ListAction {
    MoveDown,
    MoveUp,
    PushChar(char),
    PopChar,
    Confirm,
    Delete,
    Rename,
    CreateNew,
    CreateWorktree,
    NextSurface,
    FocusAgents,
    FocusSessions,
    Cancel,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RenameAction {
    PushChar(char),
    PopChar,
    Confirm,
    Cancel,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DirectoryAction {
    MoveDown,
    MoveUp,
    PushChar(char),
    PopChar,
    Confirm,
    NextField,
    Cancel,
}

pub fn list_action(key: Key) -> Option<ListAction> {
    Some(match key {
        Key::Down | Key::Ctrl('j') => ListAction::MoveDown,
        Key::Up | Key::Ctrl('k') => ListAction::MoveUp,
        Key::Backspace => ListAction::PopChar,
        Key::Ctrl('d') => ListAction::Delete,
        Key::Ctrl('r') => ListAction::Rename,
        Key::Ctrl('n') => ListAction::CreateNew,
        Key::Ctrl('w') => ListAction::CreateWorktree,
        Key::Tab => ListAction::NextSurface,
        Key::Ctrl('l') => ListAction::FocusAgents,
        Key::Ctrl('h') => ListAction::FocusSessions,
        Key::Enter => ListAction::Confirm,
        Key::Escape => ListAction::Cancel,
        Key::Char(c) => ListAction::PushChar(c),
        _ => return None,
    })
}

pub fn rename_action(key: Key) -> Option<RenameAction> {
    Some(match key {
        Key::Escape => RenameAction::Cancel,
        Key::Enter => RenameAction::Confirm,
        Key::Backspace => RenameAction::PopChar,
        Key::Char(c) => RenameAction::PushChar(c),
        _ => return None,
    })
}

pub fn directory_action(key: Key) -> Option<DirectoryAction> {
    Some(match key {
        Key::Down | Key::Ctrl('j') => DirectoryAction::MoveDown,
        Key::Up | Key::Ctrl('k') => DirectoryAction::MoveUp,
        Key::Backspace => DirectoryAction::PopChar,
        Key::Enter => DirectoryAction::Confirm,
        Key::Tab => DirectoryAction::NextField,
        Key::Escape => DirectoryAction::Cancel,
        Key::Char(c) => DirectoryAction::PushChar(c),
        _ => return None,
    })
}

pub const LIST_HINTS: &[(&str, &str)] = &[
    ("enter", "switch"),
    ("ctrl+d", "delete"),
    ("ctrl+r", "rename current"),
    ("ctrl+n", "new session"),
    ("ctrl+w", "new worktree"),
    ("esc", "close"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymaps_are_screen_specific() {
        assert_eq!(list_action(Key::Ctrl('d')), Some(ListAction::Delete));
        assert_eq!(rename_action(Key::Ctrl('d')), None);
        assert_eq!(directory_action(Key::Ctrl('d')), None);
        assert_eq!(
            list_action(Key::Ctrl('w')),
            Some(ListAction::CreateWorktree)
        );
    }

    #[test]
    fn text_is_only_accepted_without_control() {
        assert_eq!(
            rename_action(Key::Char('x')),
            Some(RenameAction::PushChar('x'))
        );
        assert_eq!(rename_action(Key::Ctrl('x')), None);
    }
}
