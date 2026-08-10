use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Cached items and fuzzy-matching logic for a navigable list. Query and selection live in a
/// separate [`PickerState`] so presentation state can be replaced independently.
pub struct Picker<T> {
    items: Vec<T>,
    key_fn: Box<dyn for<'a> Fn(&'a T) -> &'a str>,
    matcher: SkimMatcherV2,
}

/// Interaction state for a [`Picker`]. Keeping this separate from the items lets callers reset
/// the visible picker without throwing away cached data.
#[derive(Default)]
pub struct PickerState {
    /// Index into the filtered items, not the picker's full item list.
    selected: usize,
    query: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picker() -> Picker<String> {
        let mut picker = Picker::new(String::as_str);
        picker.set_items(vec!["alpha".into(), "beta".into(), "alpine".into()]);
        picker
    }

    #[test]
    fn filtering_clamps_selection() {
        let picker = picker();
        let mut state = PickerState::default();
        picker.move_down(&mut state);
        picker.move_down(&mut state);
        picker.push_char(&mut state, 'b');
        assert_eq!(
            picker.selected_item(&state).map(String::as_str),
            Some("beta")
        );
        assert_eq!(picker.selected_index(&state), Some(0));
    }

    #[test]
    fn view_contains_highlight_indices_without_exposing_mutation() {
        let picker = picker();
        let mut state = PickerState::default();
        picker.push_char(&mut state, 'a');
        picker.push_char(&mut state, 'l');
        let view = picker.view(&state);
        assert_eq!(view.filtered_count, 2);
        assert!(view.items.iter().all(|(_, indices)| !indices.is_empty()));
    }

    #[test]
    fn default_state_resets_interaction_without_clearing_items() {
        let picker = picker();
        let mut state = PickerState::default();
        picker.push_char(&mut state, 'b');
        state = PickerState::default();

        let view = picker.view(&state);
        assert_eq!(view.query, "");
        assert_eq!(view.filtered_count, 3);
        assert_eq!(view.selected, Some(0));
    }
}

/// Immutable snapshot of picker state for rendering — exposes no mutation, so the render
/// layer can't reach `Picker`'s command methods.
pub struct View<'a, T> {
    pub query: &'a str,
    /// Filtered/scored items paired with the char indices in their key that matched `query`.
    pub items: Vec<(&'a T, Vec<usize>)>,
    /// Index into `items`, not the picker's full item list.
    pub selected: Option<usize>,
    pub filtered_count: usize,
    pub total_count: usize,
}

impl<T> Picker<T> {
    pub fn new(key_fn: impl for<'a> Fn(&'a T) -> &'a str + 'static) -> Self {
        Self {
            items: Vec::new(),
            key_fn: Box::new(key_fn),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn push_char(&self, state: &mut PickerState, c: char) {
        state.query.push(c);
        self.clamp_selection(state);
    }

    pub fn pop_char(&self, state: &mut PickerState) {
        state.query.pop();
        self.clamp_selection(state);
    }

    /// Resets the filter so every item is visible again.
    pub fn clear_query(&self, state: &mut PickerState) {
        if !state.query.is_empty() {
            state.query.clear();
            self.clamp_selection(state);
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn move_up(&self, state: &mut PickerState) {
        state.selected = state.selected.saturating_sub(1);
    }

    pub fn move_down(&self, state: &mut PickerState) {
        let filtered_count = self.filtered_indices(&state.query).len();
        if filtered_count > 0 && state.selected + 1 < filtered_count {
            state.selected += 1;
        }
    }

    pub fn selected_item(&self, state: &PickerState) -> Option<&T> {
        self.filtered_indices(&state.query)
            .get(state.selected)
            .map(|&i| &self.items[i])
    }

    pub fn filtered_count(&self, state: &PickerState) -> usize {
        self.filtered_indices(&state.query).len()
    }

    pub fn selected_index(&self, state: &PickerState) -> Option<usize> {
        (self.filtered_count(state) > 0).then_some(state.selected)
    }

    pub fn view<'a>(&'a self, state: &'a PickerState) -> View<'a, T> {
        let items = self.visible_with_highlights(&state.query);
        View {
            query: &state.query,
            selected: (!items.is_empty()).then_some(state.selected),
            filtered_count: items.len(),
            items,
            total_count: self.items.len(),
        }
    }

    pub fn clamp(&self, state: &mut PickerState) {
        self.clamp_selection(state);
    }

    fn visible_with_highlights(&self, query: &str) -> Vec<(&T, Vec<usize>)> {
        let items = &self.items;
        let key_fn = &*self.key_fn;
        let matcher = &self.matcher;
        self.filtered_indices(query)
            .iter()
            .map(|&i| {
                let item = &items[i];
                let indices = if query.is_empty() {
                    vec![]
                } else {
                    matcher
                        .fuzzy_indices(key_fn(item), query)
                        .map(|(_, idx)| idx)
                        .unwrap_or_default()
                };
                (item, indices)
            })
            .collect()
    }

    fn filtered_indices(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            (0..self.items.len()).collect()
        } else {
            let mut scored: Vec<(i64, usize)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let key = (self.key_fn)(item);
                    self.matcher.fuzzy_match(key, query).map(|score| (score, i))
                })
                .collect();
            scored.sort_by_key(|item| std::cmp::Reverse(item.0));
            scored.into_iter().map(|(_, i)| i).collect()
        }
    }

    fn clamp_selection(&self, state: &mut PickerState) {
        let filtered_count = self.filtered_indices(&state.query).len();
        if filtered_count == 0 {
            state.selected = 0;
        } else if state.selected >= filtered_count {
            state.selected = filtered_count - 1;
        }
    }
}
