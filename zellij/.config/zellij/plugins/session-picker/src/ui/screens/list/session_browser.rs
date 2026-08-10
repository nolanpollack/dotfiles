//! Picker interaction over the session data projection used by the list screen.

use crate::sessions::Session;
use crate::ui::components::picker::{Picker, PickerState, View};

pub struct SessionBrowser {
    picker: Picker<Session>,
    picker_state: PickerState,
}

impl Default for SessionBrowser {
    fn default() -> Self {
        Self {
            picker: Picker::new(|session: &Session| session.name.as_str()),
            picker_state: PickerState::default(),
        }
    }
}

impl SessionBrowser {
    pub fn set_items(&mut self, sessions: &[Session]) {
        self.picker.set_items(sessions.to_vec());
        self.picker.clamp(&mut self.picker_state);
    }

    pub fn selected(&self) -> Option<&Session> {
        self.picker.selected_item(&self.picker_state)
    }

    pub fn view(&self) -> View<'_, Session> {
        self.picker.view(&self.picker_state)
    }

    pub fn move_up(&mut self) {
        self.picker.move_up(&mut self.picker_state);
    }

    pub fn move_down(&mut self) {
        self.picker.move_down(&mut self.picker_state);
    }

    pub fn push_char(&mut self, character: char) {
        self.picker.push_char(&mut self.picker_state, character);
    }

    pub fn pop_char(&mut self) {
        self.picker.pop_char(&mut self.picker_state);
    }

    pub fn clear_query(&mut self) {
        self.picker.clear_query(&mut self.picker_state);
    }

    pub fn reset(&mut self) {
        self.picker_state = PickerState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_clears_transient_picker_state() {
        let mut browser = SessionBrowser::default();
        browser.set_items(&[Session {
            name: "one".into(),
            ..Default::default()
        }]);
        browser.push_char('o');
        assert_eq!(browser.view().query, "o");

        browser.reset();

        let view = browser.view();
        assert_eq!(view.query, "");
        assert_eq!(view.selected, Some(0));
    }
}
