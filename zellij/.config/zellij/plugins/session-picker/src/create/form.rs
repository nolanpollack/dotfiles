use crate::input::Action;
use crate::picker::Picker;

/// A plain single-line editable string, mirroring the rename-draft input already used
/// elsewhere in this plugin.
#[derive(Default)]
pub struct TextField {
    pub value: String,
}

impl TextField {
    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }
}

/// Whether a combobox is showing its committed value as plain text, or an open fuzzy list.
enum ComboboxMode<T> {
    Collapsed,
    Expanded(Picker<T>),
}

/// A field that picks one `T` from a fuzzy-searchable list. Reuses `Picker<T>`'s engine
/// verbatim while expanded; owns nothing about where candidates come from or what they mean.
pub struct Combobox<T> {
    key_fn: fn(&T) -> &str,
    committed: Option<T>,
    mode: ComboboxMode<T>,
}

impl<T: Clone + 'static> Combobox<T> {
    pub fn new(key_fn: fn(&T) -> &str) -> Self {
        Self { key_fn, committed: None, mode: ComboboxMode::Collapsed }
    }

    pub fn is_expanded(&self) -> bool {
        matches!(self.mode, ComboboxMode::Expanded(_))
    }

    pub fn committed(&self) -> Option<&T> {
        self.committed.as_ref()
    }

    /// Collapsed-state display text: the committed item's key, or empty.
    pub fn display(&self) -> String {
        self.committed.as_ref().map(|t| (self.key_fn)(t).to_string()).unwrap_or_default()
    }

    /// Opens the dropdown, seeding it with whatever candidates are already available (an empty
    /// list while a fetch is still in flight — the flow-specific module owns actually fetching
    /// and later calls `set_candidates` once results arrive).
    pub fn expand(&mut self, candidates: Vec<T>) {
        let mut picker = Picker::new(self.key_fn);
        picker.set_items(candidates);
        self.mode = ComboboxMode::Expanded(picker);
    }

    /// Replaces the candidate list of an already-expanded dropdown, e.g. once an async fetch
    /// completes after the user opened it. No-op if collapsed.
    pub fn set_candidates(&mut self, candidates: Vec<T>) {
        if let ComboboxMode::Expanded(picker) = &mut self.mode {
            picker.set_items(candidates);
        }
    }

    /// Commits the highlighted item (if any) and collapses. Returns `true` if something was
    /// actually committed (vs. an empty list collapsing with nothing selected).
    fn commit_and_collapse(&mut self) -> bool {
        let mut committed_now = false;
        if let ComboboxMode::Expanded(picker) = &self.mode {
            if let Some(item) = picker.selected_item() {
                self.committed = Some(item.clone());
                committed_now = true;
            }
        }
        self.mode = ComboboxMode::Collapsed;
        committed_now
    }

    fn collapse(&mut self) {
        self.mode = ComboboxMode::Collapsed;
    }

    pub fn picker(&self) -> Option<&Picker<T>> {
        match &self.mode {
            ComboboxMode::Expanded(picker) => Some(picker),
            ComboboxMode::Collapsed => None,
        }
    }

    fn picker_mut(&mut self) -> Option<&mut Picker<T>> {
        match &mut self.mode {
            ComboboxMode::Expanded(picker) => Some(picker),
            ComboboxMode::Collapsed => None,
        }
    }
}

/// A form field: either a fuzzy-searchable combobox or a plain text input. There are exactly
/// two kinds and every field in every planned flow (including the deferred worktree ones) is one
/// of them — an enum, not a trait, since the axis that actually varies is the candidate type
/// `T` inside `Combobox`, not the field kind itself.
pub enum Field<T> {
    Combobox(Combobox<T>),
    Text(TextField),
}

/// An ordered list of fields plus which one has focus. Knows how to move focus and how to route
/// an already-interpreted `Action` (never a raw key — that mapping lives once, centrally, in
/// `input::BINDINGS`) to the focused field's own edit behavior. Does NOT know what submitting or
/// cancelling the form means, what a field represents, or how to fetch combobox candidates —
/// those are the flow-specific module's job.
pub struct Form<T> {
    fields: Vec<Field<T>>,
    focus: usize,
}

impl<T: Clone + 'static> Form<T> {
    pub fn new(fields: Vec<Field<T>>) -> Self {
        Self { fields, focus: 0 }
    }

    pub fn fields(&self) -> &[Field<T>] {
        &self.fields
    }

    pub fn focus(&self) -> usize {
        self.focus
    }

    /// Direct access to a specific field, for a flow module to push freshly-fetched candidates
    /// into a combobox it knows the index of. Not used for action routing (that's
    /// `handle_action`).
    pub fn field_mut(&mut self, index: usize) -> &mut Field<T> {
        &mut self.fields[index]
    }

    fn focused_mut(&mut self) -> &mut Field<T> {
        &mut self.fields[self.focus]
    }

    fn is_last_field(&self) -> bool {
        self.focus + 1 == self.fields.len()
    }

    fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % self.fields.len();
    }

    fn focus_prev(&mut self) {
        self.focus = if self.focus == 0 { self.fields.len() - 1 } else { self.focus - 1 };
    }

    /// Returns `true` if the action was consumed as ordinary field editing/navigation. Returns
    /// `false` for `Confirm` on the last field and `Cancel` while nothing is expanded — those
    /// are submit/cancel signals the caller must interpret, since this type has no idea what
    /// "submit" or "cancel" should *do*.
    pub fn handle_action(&mut self, action: Action) -> bool {
        if let Field::Combobox(cb) = self.focused_mut() {
            if cb.is_expanded() {
                let mut advance = false;
                match action {
                    Action::Confirm => advance = cb.commit_and_collapse(),
                    Action::Cancel => cb.collapse(),
                    Action::MoveDown => {
                        if let Some(p) = cb.picker_mut() {
                            p.move_down();
                        }
                    }
                    Action::MoveUp => {
                        if let Some(p) = cb.picker_mut() {
                            p.move_up();
                        }
                    }
                    Action::PopChar => {
                        if let Some(p) = cb.picker_mut() {
                            p.pop_char();
                        }
                    }
                    Action::PushChar(c) => {
                        if let Some(p) = cb.picker_mut() {
                            p.push_char(c);
                        }
                    }
                    _ => {} // NextField/Delete/Rename/CreateNew have no meaning while expanded
                }
                // A successful selection moves you on to the next field, same as picking a
                // value from a dropdown normally would.
                if advance {
                    self.focus_next();
                }
                return true; // fully swallowed: an expanded combobox owns all input
            }
        }

        match action {
            // ctrl-j/ctrl-k (MoveDown/MoveUp) and Tab both move between fields when nothing is
            // expanded — they only navigate *within* a field once it's expanded (handled above).
            Action::NextField | Action::MoveDown => {
                self.focus_next();
                true
            }
            Action::MoveUp => {
                self.focus_prev();
                true
            }
            Action::Confirm => {
                if self.is_last_field() {
                    return false; // submit signal, not consumed
                }
                match self.focused_mut() {
                    Field::Combobox(cb) => cb.expand(Vec::new()),
                    Field::Text(_) => self.focus_next(),
                }
                true
            }
            Action::Cancel => false, // step-back/cancel signal, not consumed
            Action::PopChar => {
                if let Field::Text(t) = self.focused_mut() {
                    t.pop_char();
                }
                true
            }
            Action::PushChar(c) => {
                match self.focused_mut() {
                    Field::Text(t) => t.push_char(c),
                    // The char that triggers opening seeds the query rather than being
                    // swallowed — otherwise the first character you type just vanishes.
                    Field::Combobox(cb) => {
                        cb.expand(Vec::new());
                        if let Some(p) = cb.picker_mut() {
                            p.push_char(c);
                        }
                    }
                }
                true
            }
            _ => false, // Delete/Rename/CreateNew have no meaning on a collapsed field
        }
    }
}
