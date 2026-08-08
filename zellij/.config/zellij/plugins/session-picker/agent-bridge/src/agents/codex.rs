use std::env;
use std::path::PathBuf;

use super::Agent;

pub(crate) struct Codex;

impl Agent for Codex {
    fn label(&self) -> &'static str {
        "codex"
    }

    fn config_path(&self) -> Result<PathBuf, String> {
        let home = PathBuf::from(env::var("HOME").map_err(|_| "HOME is not set")?);
        let root = env::var("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".codex"));
        Ok(root.join("hooks.json"))
    }

    fn supported_events(&self) -> &'static [&'static str] {
        &[
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "PermissionRequest",
            "PreCompact",
            "PostCompact",
            "SubagentStart",
            "SubagentStop",
            "Stop",
        ]
    }
}
