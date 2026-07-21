pub mod directory;
pub mod discovery;
pub mod form;

use std::collections::BTreeMap;

use zellij_tile::prelude::KeyWithModifier;

use crate::sessions::SessionInfo;

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
    /// A session was created and switched to; exit create-mode back to the session list.
    Created,
}

pub fn apply_key(flow: &mut CreateFlow, key: &KeyWithModifier, existing_names: &[SessionInfo]) -> CreateOutcome {
    match flow {
        CreateFlow::Directory(form) => match form.apply_key(key, existing_names) {
            directory::Outcome::Continue => CreateOutcome::Continue,
            // No chooser to step back to anymore — stepping back all the way out just cancels.
            directory::Outcome::Back => CreateOutcome::Cancelled,
            directory::Outcome::Done => CreateOutcome::Created,
        },
    }
}

/// Routes a `RunCommandResult` to whichever flow is waiting on it. Returns `true` if it was
/// consumed (and the picker should re-render).
pub fn apply_discovery_result(flow: &mut CreateFlow, context: &BTreeMap<String, String>, stdout: &[u8]) -> bool {
    match flow {
        CreateFlow::Directory(form) => form.apply_discovery_result(context, stdout),
    }
}
