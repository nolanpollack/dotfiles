use std::path::{Path, PathBuf};

use super::form::{Combobox, TextField};
use crate::input::{directory_action, DirectoryAction, Key};
use crate::sessions::Session;

/// A zoxide-known directory, offered as a combobox candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    pub display: String,
}

impl Candidate {
    fn from_path(path: PathBuf) -> Self {
        let display = path.to_string_lossy().to_string();
        Self { path, display }
    }
}

fn candidate_key(candidate: &Candidate) -> &str {
    &candidate.display
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Focus {
    Directory,
    Name,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::Directory => Self::Name,
            Self::Name => Self::Directory,
        }
    }

    fn previous(self) -> Self {
        self.next()
    }
}

/// What happened as a result of a keypress; the plugin adapter performs the actual session
/// switch for `Create`.
pub enum Outcome {
    Continue,
    Back,
    Create { name: String, cwd: PathBuf },
}

pub struct DirectoryForm {
    directory: Combobox<Candidate>,
    name: TextField,
    focus: Focus,
    error: Option<String>,
}

impl Default for DirectoryForm {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryForm {
    pub fn new() -> Self {
        Self {
            directory: Combobox::new(candidate_key),
            name: TextField::default(),
            focus: Focus::Directory,
            error: None,
        }
    }

    pub fn directory(&self) -> &Combobox<Candidate> {
        &self.directory
    }

    pub fn name(&self) -> &TextField {
        &self.name
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn apply_key(&mut self, key: Key, existing_names: &[Session]) -> Outcome {
        self.error = None;
        let Some(action) = directory_action(key) else {
            return Outcome::Continue;
        };

        if self.directory.is_expanded() {
            return self.apply_directory_picker_action(action);
        }

        match action {
            DirectoryAction::NextField | DirectoryAction::MoveDown => {
                self.focus = self.focus.next();
                Outcome::Continue
            }
            DirectoryAction::MoveUp => {
                self.focus = self.focus.previous();
                Outcome::Continue
            }
            DirectoryAction::Cancel => Outcome::Back,
            DirectoryAction::Confirm if self.focus == Focus::Directory => {
                self.directory.expand();
                Outcome::Continue
            }
            DirectoryAction::Confirm => self.submit(existing_names),
            DirectoryAction::PopChar if self.focus == Focus::Name => {
                self.name.pop_char();
                Outcome::Continue
            }
            DirectoryAction::PushChar(c) if self.focus == Focus::Directory => {
                self.directory.expand_with_char(c);
                Outcome::Continue
            }
            DirectoryAction::PushChar(c) => {
                self.name.push_char(c);
                Outcome::Continue
            }
            _ => Outcome::Continue,
        }
    }

    pub fn set_candidates(&mut self, paths: Vec<PathBuf>) {
        self.directory
            .set_candidates(paths.into_iter().map(Candidate::from_path).collect());
    }

    fn apply_directory_picker_action(&mut self, action: DirectoryAction) -> Outcome {
        match action {
            DirectoryAction::Confirm => {
                if self.directory.commit() {
                    self.sync_default_name();
                    self.focus = Focus::Name;
                }
            }
            DirectoryAction::Cancel => self.directory.collapse(),
            DirectoryAction::MoveDown => self.directory.move_down(),
            DirectoryAction::MoveUp => self.directory.move_up(),
            DirectoryAction::PopChar => self.directory.pop_char(),
            DirectoryAction::PushChar(c) => self.directory.push_char(c),
            _ => {}
        }
        Outcome::Continue
    }

    fn sync_default_name(&mut self) {
        if let Some(candidate) = self.directory.committed() {
            self.name
                .set_if_empty(default_session_name(&candidate.path));
        }
    }

    fn submit(&mut self, existing_names: &[Session]) -> Outcome {
        let Some(candidate) = self.directory.committed() else {
            self.error = Some("choose a directory".to_string());
            return Outcome::Continue;
        };

        let name = if self.name.is_empty() {
            default_session_name(&candidate.path)
        } else {
            self.name.value().trim().to_string()
        };
        if name.is_empty() {
            self.error = Some("session name is required".to_string());
            return Outcome::Continue;
        }
        if existing_names.iter().any(|session| session.name == name) {
            self.error = Some(format!("session '{name}' already exists"));
            return Outcome::Continue;
        }

        Outcome::Create {
            name,
            cwd: candidate.path.clone(),
        }
    }
}

fn default_session_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Key;

    #[test]
    fn chosen_directory_supplies_a_default_session_name() {
        let mut form = DirectoryForm::new();
        form.set_candidates(vec![PathBuf::from("/tmp/project")]);
        form.apply_key(Key::Enter, &[]);
        form.apply_key(Key::Enter, &[]);
        assert_eq!(form.name().value(), "project");
        assert!(matches!(
            form.apply_key(Key::Enter, &[]),
            Outcome::Create { name, cwd }
                if name == "project" && cwd == PathBuf::from("/tmp/project")
        ));
    }

    #[test]
    fn duplicate_name_stays_in_the_form_with_an_error() {
        let mut form = DirectoryForm::new();
        form.set_candidates(vec![PathBuf::from("/tmp/project")]);
        form.apply_key(Key::Enter, &[]);
        form.apply_key(Key::Enter, &[]);
        let existing = [Session {
            name: "project".into(),
            ..Default::default()
        }];
        assert!(matches!(
            form.apply_key(Key::Enter, &existing),
            Outcome::Continue
        ));
        assert!(form
            .error()
            .is_some_and(|error| error.contains("already exists")));
    }
}
