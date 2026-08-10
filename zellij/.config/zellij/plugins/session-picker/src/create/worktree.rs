use std::path::PathBuf;

use super::form::TextField;
use crate::input::Key;
use crate::sessions::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub branch_prefix: String,
    pub worktree_root: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            branch_prefix: "nolanpollack".into(),
            worktree_root: PathBuf::from("/Users/nolanpollack/stripe/worktrees"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub session_name: String,
    pub repository: PathBuf,
    pub base_branch: String,
    pub branch_name: String,
    pub worktree: PathBuf,
}

pub fn validation_args(request: &Request) -> Vec<String> {
    command_args(
        r#"repo="$1"; base="$2"; branch="$3"; destination="$4"
git -C "$repo" rev-parse --is-inside-work-tree >/dev/null || { echo "repository is not a Git checkout" >&2; exit 1; }
git -C "$repo" rev-parse --verify "$base^{commit}" >/dev/null || { echo "base branch '$base' does not resolve" >&2; exit 1; }
git -C "$repo" show-ref --verify --quiet "refs/heads/$branch" && { echo "branch '$branch' already exists" >&2; exit 1; }
[ ! -e "$destination" ] || { echo "worktree destination already exists" >&2; exit 1; }
"#,
        request,
    )
}

pub fn creation_args(request: &Request) -> Vec<String> {
    command_args(
        "git -C \"$1\" worktree add -b \"$3\" \"$4\" \"$2\"",
        request,
    )
}

fn command_args(script: &str, request: &Request) -> Vec<String> {
    vec![
        "sh".into(),
        "-c".into(),
        script.into(),
        "sh".into(),
        request.repository.to_string_lossy().into(),
        request.base_branch.clone(),
        request.branch_name.clone(),
        request.worktree.to_string_lossy().into(),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Checking,
    Creating,
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    SessionName,
    Repository,
    BaseBranch,
    BranchName,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::SessionName => Self::Repository,
            Self::Repository => Self::BaseBranch,
            Self::BaseBranch => Self::BranchName,
            Self::BranchName => Self::SessionName,
        }
    }

    fn previous(self) -> Self {
        self.next().next().next()
    }
}

pub enum Outcome {
    Continue,
    Back,
    Start(Request),
}

pub struct Form {
    session_name: TextField,
    repository: TextField,
    base_branch: TextField,
    branch_name: TextField,
    focus: Focus,
    config: Config,
    stage: Option<Stage>,
    pending: Option<Request>,
    error: Option<String>,
}

impl Form {
    pub fn new(config: Config, selected: Option<&Session>) -> Self {
        let mut repository = TextField::default();
        let mut base_branch = TextField::default();
        if let Some(session) = selected {
            if let Some(root) = &session.repo_root {
                repository.set(root.clone());
            }
            if let Some(branch) = &session.branch {
                base_branch.set(branch.clone());
            }
        }
        Self {
            session_name: TextField::default(),
            repository,
            base_branch,
            branch_name: TextField::default(),
            focus: Focus::SessionName,
            config,
            stage: None,
            pending: None,
            error: None,
        }
    }

    pub fn session_name(&self) -> &TextField {
        &self.session_name
    }
    pub fn repository(&self) -> &TextField {
        &self.repository
    }
    pub fn base_branch(&self) -> &TextField {
        &self.base_branch
    }
    pub fn branch_name(&self) -> &TextField {
        &self.branch_name
    }
    pub fn focus_index(&self) -> usize {
        match self.focus {
            Focus::SessionName => 0,
            Focus::Repository => 1,
            Focus::BaseBranch => 2,
            Focus::BranchName => 3,
        }
    }
    pub fn stage(&self) -> Option<Stage> {
        self.stage
    }
    pub fn pending_request(&self) -> Option<&Request> {
        self.pending.as_ref()
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn apply_key(&mut self, key: Key, existing: &[Session]) -> Outcome {
        if self.stage.is_some() {
            if key == Key::Escape && self.stage == Some(Stage::Failed) {
                self.stage = None;
                self.error = None;
            }
            return Outcome::Continue;
        }
        self.error = None;
        match key {
            Key::Escape => Outcome::Back,
            Key::Down | Key::Ctrl('j') | Key::Tab => {
                self.focus = self.focus.next();
                Outcome::Continue
            }
            Key::Up | Key::Ctrl('k') => {
                self.focus = self.focus.previous();
                Outcome::Continue
            }
            Key::Enter if self.focus != Focus::BranchName => {
                self.focus = self.focus.next();
                Outcome::Continue
            }
            Key::Enter => self.submit(existing),
            Key::Backspace => {
                self.active_field().pop_char();
                self.sync_default_branch();
                Outcome::Continue
            }
            Key::Char(c) => {
                self.active_field().push_char(c);
                self.sync_default_branch();
                Outcome::Continue
            }
            _ => Outcome::Continue,
        }
    }

    pub fn begin_create(&mut self) {
        self.stage = Some(Stage::Creating);
    }
    pub fn fail(&mut self, error: String) {
        self.stage = Some(Stage::Failed);
        self.error = Some(error);
    }

    fn active_field(&mut self) -> &mut TextField {
        match self.focus {
            Focus::SessionName => &mut self.session_name,
            Focus::Repository => &mut self.repository,
            Focus::BaseBranch => &mut self.base_branch,
            Focus::BranchName => &mut self.branch_name,
        }
    }

    fn sync_default_branch(&mut self) {
        if self.focus == Focus::SessionName {
            self.branch_name.set(format!(
                "{}/{}",
                self.config.branch_prefix,
                self.session_name.value()
            ));
        }
    }

    fn submit(&mut self, existing: &[Session]) -> Outcome {
        let session_name = self.session_name.value().trim().to_string();
        let repository = self.repository.value().trim().to_string();
        let base_branch = self.base_branch.value().trim().to_string();
        let branch_name = self.branch_name.value().trim().to_string();
        if session_name.is_empty()
            || repository.is_empty()
            || base_branch.is_empty()
            || branch_name.is_empty()
        {
            self.error = Some("all fields are required".into());
            return Outcome::Continue;
        }
        if existing.iter().any(|session| session.name == session_name) {
            self.error = Some(format!("session '{session_name}' already exists"));
            return Outcome::Continue;
        }
        if session_name.contains('/') || session_name.contains("..") {
            self.error = Some("session name cannot contain '/' or '..'".into());
            return Outcome::Continue;
        }
        let repo_name = PathBuf::from(&repository)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        if repo_name.is_empty() {
            self.error = Some("repository must name a directory".into());
            return Outcome::Continue;
        }
        let request = Request {
            session_name,
            repository: PathBuf::from(repository),
            base_branch,
            branch_name,
            worktree: self
                .config
                .worktree_root
                .join(repo_name)
                .join(self.session_name.value().trim()),
        };
        self.stage = Some(Stage::Checking);
        self.pending = Some(request.clone());
        Outcome::Start(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Key;

    #[test]
    fn selected_worktree_uses_main_checkout_and_branch() {
        let session = Session {
            repo_root: Some("/stripe/pay-server".into()),
            branch: Some("master".into()),
            ..Default::default()
        };
        let form = Form::new(Config::default(), Some(&session));
        assert_eq!(form.repository().value(), "/stripe/pay-server");
        assert_eq!(form.base_branch().value(), "master");
    }

    #[test]
    fn enter_advances_then_starts_from_branch_name() {
        let mut form = Form::new(Config::default(), None);
        for c in "work".chars() {
            form.apply_key(Key::Char(c), &[]);
        }
        form.apply_key(Key::Enter, &[]);
        for c in "/repo".chars() {
            form.apply_key(Key::Char(c), &[]);
        }
        form.apply_key(Key::Enter, &[]);
        for c in "master".chars() {
            form.apply_key(Key::Char(c), &[]);
        }
        form.apply_key(Key::Enter, &[]);
        assert!(matches!(form.apply_key(Key::Enter, &[]), Outcome::Start(_)));
    }
}
