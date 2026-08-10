pub mod directory;
pub mod discovery;
pub mod form;
pub mod worktree;

use std::path::PathBuf;

use crate::input::Key;
use crate::sessions::Session;

/// The screen create-mode is currently showing. Dispatch only — no field definitions, discovery
/// parsing, or git logic belongs here; that's each flow module's job.
///
/// There's a single variant today because there's a single entry point (`ctrl-n` -> straight
/// into the directory flow, no type-chooser). Future flows (worktree from repo/branch/Jira) each
/// get their own dedicated keybind and add their own variant + match arm here, backed by their
/// own `create/<flow>.rs` module built on `form::Form`/`form::Combobox` — not a shared chooser
/// screen.
pub enum CreateFlow {
    Directory(directory::DirectoryForm),
    Worktree(worktree::Form),
}

impl Default for CreateFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateFlow {
    pub fn new() -> Self {
        CreateFlow::Directory(directory::DirectoryForm::new())
    }
}

/// What the caller (`main.rs`) should do after a keypress while create-mode is active.
pub enum CreateOutcome {
    /// Stay in create-mode; re-render.
    Continue,
    /// The user backed all the way out; exit create-mode back to the session list.
    Cancelled,
    /// The adapter should create and switch to this session, then exit create-mode.
    CreateSession {
        name: String,
        cwd: PathBuf,
    },
    StartWorktree(worktree::Request),
}

pub fn apply_key(flow: &mut CreateFlow, key: Key, existing_names: &[Session]) -> CreateOutcome {
    match flow {
        CreateFlow::Directory(form) => match form.apply_key(key, existing_names) {
            directory::Outcome::Continue => CreateOutcome::Continue,
            // No chooser to step back to anymore — stepping back all the way out just cancels.
            directory::Outcome::Back => CreateOutcome::Cancelled,
            directory::Outcome::Create { name, cwd } => CreateOutcome::CreateSession { name, cwd },
        },
        CreateFlow::Worktree(form) => match form.apply_key(key, existing_names) {
            worktree::Outcome::Continue => CreateOutcome::Continue,
            worktree::Outcome::Back => CreateOutcome::Cancelled,
            worktree::Outcome::Start(request) => CreateOutcome::StartWorktree(request),
        },
    }
}

/// Routes a `RunCommandResult` to whichever flow is waiting on it. Returns `true` if it was
/// consumed (and the picker should re-render).
pub fn set_directory_candidates(flow: &mut CreateFlow, paths: Vec<PathBuf>) {
    match flow {
        CreateFlow::Directory(form) => form.set_candidates(paths),
        CreateFlow::Worktree(_) => {}
    }
}
