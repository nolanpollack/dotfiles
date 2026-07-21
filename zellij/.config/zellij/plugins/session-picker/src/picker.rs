use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Generic fuzzy-filterable, navigable list. Holds query/selection state and scoring logic;
/// knows nothing about how it's displayed.
pub struct Picker<T> {
    items: Vec<T>,
    key_fn: Box<dyn for<'a> Fn(&'a T) -> &'a str>,
    /// Indices into `items` that match `query`, ordered by match score.
    filtered: Vec<usize>,
    /// Index into `filtered`, not `items` — resolve via `items[filtered[selected]]`.
    selected: usize,
    query: String,
    matcher: SkimMatcherV2,
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
            filtered: Vec::new(),
            selected: 0,
            query: String::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.refilter();
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.refilter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.refilter();
    }

    /// Resets the filter so every item is visible again.
    pub fn clear_query(&mut self) {
        if !self.query.is_empty() {
            self.query.clear();
            self.refilter();
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.filtered.get(self.selected).map(|&i| &self.items[i])
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered.len()
    }

    pub fn selected_index(&self) -> Option<usize> {
        (!self.filtered.is_empty()).then_some(self.selected)
    }

    pub fn view(&self) -> View<'_, T> {
        View {
            query: &self.query,
            items: self.visible_with_highlights(),
            selected: self.selected_index(),
            filtered_count: self.filtered_count(),
            total_count: self.items.len(),
        }
    }

    fn visible_with_highlights(&self) -> Vec<(&T, Vec<usize>)> {
        let query = &self.query;
        let items = &self.items;
        let key_fn = &*self.key_fn;
        let matcher = &self.matcher;
        self.filtered
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

    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            let mut scored: Vec<(i64, usize)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let key = (self.key_fn)(item);
                    self.matcher.fuzzy_match(key, &self.query).map(|score| (score, i))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered = scored.into_iter().map(|(_, i)| i).collect();
        }
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }
}
