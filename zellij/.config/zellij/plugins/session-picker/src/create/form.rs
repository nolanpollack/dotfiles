use crate::picker::{Picker, PickerState};

/// A plain single-line editable string.
#[derive(Default)]
pub struct TextField {
    value: String,
}

impl TextField {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn set_if_empty(&mut self, value: String) {
        if self.value.is_empty() {
            self.value = value;
        }
    }

    pub fn set(&mut self, value: String) {
        self.value = value;
    }

    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }
}

/// A fuzzy-searchable choice field. Candidate ownership lives here, rather than in the form
/// that happens to use it, so opening and refreshing the dropdown cannot get out of sync.
pub struct Combobox<T> {
    key_fn: fn(&T) -> &str,
    candidates: Vec<T>,
    committed: Option<T>,
    picker: Option<ExpandedPicker<T>>,
}

struct ExpandedPicker<T> {
    picker: Picker<T>,
    state: PickerState,
}

impl<T: Clone + 'static> Combobox<T> {
    pub fn new(key_fn: fn(&T) -> &str) -> Self {
        Self {
            key_fn,
            candidates: Vec::new(),
            committed: None,
            picker: None,
        }
    }

    pub fn set_candidates(&mut self, candidates: Vec<T>) {
        self.candidates = candidates;
        if let Some(picker) = &mut self.picker {
            picker.picker.set_items(self.candidates.clone());
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.picker.is_some()
    }

    pub fn committed(&self) -> Option<&T> {
        self.committed.as_ref()
    }

    pub fn display(&self) -> String {
        self.committed
            .as_ref()
            .map(|item| (self.key_fn)(item).to_owned())
            .unwrap_or_default()
    }

    pub fn expand(&mut self) {
        let mut picker = Picker::new(self.key_fn);
        picker.set_items(self.candidates.clone());
        self.picker = Some(ExpandedPicker {
            picker,
            state: PickerState::default(),
        });
    }

    pub fn expand_with_char(&mut self, c: char) {
        self.expand();
        self.push_char(c);
    }

    pub fn commit(&mut self) -> bool {
        let selected = self
            .picker
            .as_ref()
            .and_then(|expanded| expanded.picker.selected_item(&expanded.state))
            .cloned();
        self.picker = None;
        if let Some(selected) = selected {
            self.committed = Some(selected);
            true
        } else {
            false
        }
    }

    pub fn collapse(&mut self) {
        self.picker = None;
    }

    pub fn move_up(&mut self) {
        if let Some(expanded) = &mut self.picker {
            expanded.picker.move_up(&mut expanded.state);
        }
    }

    pub fn move_down(&mut self) {
        if let Some(expanded) = &mut self.picker {
            expanded.picker.move_down(&mut expanded.state);
        }
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(expanded) = &mut self.picker {
            expanded.picker.push_char(&mut expanded.state, c);
        }
    }

    pub fn pop_char(&mut self) {
        if let Some(expanded) = &mut self.picker {
            expanded.picker.pop_char(&mut expanded.state);
        }
    }

    pub fn picker(&self) -> Option<crate::picker::View<'_, T>> {
        self.picker
            .as_ref()
            .map(|expanded| expanded.picker.view(&expanded.state))
    }
}
