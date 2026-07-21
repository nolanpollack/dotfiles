use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use zellij_tile::prelude::{switch_session_with_cwd, KeyWithModifier};

use super::discovery;
use super::form::{Combobox, Field, Form, TextField};
use crate::input::{key_to_action, Action};
use crate::sessions::SessionInfo;

const DIRECTORY_FIELD: usize = 0;
const NAME_FIELD: usize = 1;

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

fn candidate_key(c: &Candidate) -> &str {
    &c.display
}

/// What happened as a result of a keypress; the caller (`create::mod`) interprets this into a
/// `CreateFlow` transition.
pub enum Outcome {
    /// Stay on this form.
    Continue,
    /// Step back to the type chooser.
    Back,
    /// A session was created and switched to; the caller should exit create-mode entirely.
    Done,
}

pub struct DirectoryForm {
    form: Form<Candidate>,
    error: Option<String>,
    /// Cached separately from the combobox itself: `Combobox::set_candidates` only takes effect
    /// while expanded, but the zoxide fetch usually resolves *before* the user ever opens it —
    /// this is what lets a freshly-opened combobox show real results immediately instead of
    /// starting empty.
    candidates: Vec<Candidate>,
}

impl DirectoryForm {
    pub fn new() -> Self {
        // Fired eagerly (rather than waiting for the combobox to be opened) so the candidate
        // list has a head start on the async round-trip before the user gets there.
        discovery::fetch_zoxide_list();
        let fields = vec![
            Field::Combobox(Combobox::new(candidate_key)),
            Field::Text(TextField::default()),
        ];
        Self { form: Form::new(fields), error: None, candidates: Vec::new() }
    }

    pub fn form(&self) -> &Form<Candidate> {
        &self.form
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn apply_key(&mut self, key: &KeyWithModifier, existing_names: &[SessionInfo]) -> Outcome {
        self.error = None;

        let Some(action) = key_to_action(key) else {
            return Outcome::Continue;
        };

        let consumed = self.form.handle_action(action);
        self.sync_directory_candidates();
        self.sync_default_name();

        if consumed {
            return Outcome::Continue;
        }

        match action {
            Action::Cancel => Outcome::Back,
            Action::Confirm => self.submit(existing_names),
            _ => Outcome::Continue,
        }
    }

    pub fn apply_discovery_result(&mut self, context: &BTreeMap<String, String>, stdout: &[u8]) -> bool {
        let Some(paths) = discovery::parse_zoxide_result(context, stdout) else {
            return false;
        };
        self.candidates = paths.into_iter().map(Candidate::from_path).collect();
        self.sync_directory_candidates();
        true
    }

    /// Pushes the cached candidate list into the directory combobox if it's expanded — a no-op
    /// otherwise. Called after every keypress (cheap: `Picker::set_items` just re-filters) so a
    /// combobox that just opened, or a fetch that just resolved, is never left showing stale or
    /// empty results.
    fn sync_directory_candidates(&mut self) {
        if let Field::Combobox(cb) = self.form.field_mut(DIRECTORY_FIELD) {
            if cb.is_expanded() {
                cb.set_candidates(self.candidates.clone());
            }
        }
    }

    /// Fills the name field with the chosen directory's basename as soon as a directory is
    /// committed, as long as the user hasn't typed their own name already — same "reactive
    /// default until you override it" pattern `gw`'s own form uses for its branch field.
    fn sync_default_name(&mut self) {
        let default_name = match self.form.fields().get(DIRECTORY_FIELD) {
            Some(Field::Combobox(dir)) => dir.committed().map(|c| default_session_name(&c.path)),
            _ => None,
        };
        let Some(default_name) = default_name else {
            return;
        };
        if let Field::Text(name_field) = self.form.field_mut(NAME_FIELD) {
            if name_field.value.is_empty() {
                name_field.value = default_name;
            }
        }
    }

    fn submit(&mut self, existing_names: &[SessionInfo]) -> Outcome {
        let Some(Field::Combobox(directory_field)) = self.form.fields().get(DIRECTORY_FIELD) else {
            return Outcome::Continue;
        };
        let Some(candidate) = directory_field.committed().cloned() else {
            self.error = Some("choose a directory".to_string());
            return Outcome::Continue;
        };

        let typed_name = match self.form.fields().get(NAME_FIELD) {
            Some(Field::Text(t)) => t.value.trim().to_string(),
            _ => String::new(),
        };
        let name = if typed_name.is_empty() { default_session_name(&candidate.path) } else { typed_name };
        if name.is_empty() {
            self.error = Some("session name is required".to_string());
            return Outcome::Continue;
        }
        if existing_names.iter().any(|s| s.name == name) {
            self.error = Some(format!("session '{name}' already exists"));
            return Outcome::Continue;
        }

        switch_session_with_cwd(Some(&name), Some(candidate.path));
        Outcome::Done
    }
}

fn default_session_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}
