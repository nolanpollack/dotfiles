pub struct ChoiceRow {
    pub display: String,
    pub matched: Vec<usize>,
}

pub enum CreateView {
    Form {
        directory: String,
        name: String,
        directory_focused: bool,
        error: Option<String>,
    },
    DirectoryChoices {
        query: String,
        rows: Vec<ChoiceRow>,
        selected: Option<usize>,
        filtered_count: usize,
        total_count: usize,
    },
    WorktreeForm {
        session_name: String,
        repository: String,
        base_branch: String,
        branch_name: String,
        focused: usize,
        error: Option<String>,
    },
    WorktreeProgress {
        stage: crate::create::worktree::Stage,
        error: Option<String>,
        spinner_tick: usize,
    },
}
