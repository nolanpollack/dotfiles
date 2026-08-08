use std::env;
use std::path::PathBuf;

use super::Agent;

pub(crate) struct Claude;

impl Agent for Claude {
    fn label(&self) -> &'static str {
        "claude"
    }

    fn config_path(&self) -> Result<PathBuf, String> {
        let home = PathBuf::from(env::var("HOME").map_err(|_| "HOME is not set")?);
        let root = env::var("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".claude"));
        Ok(root.join("settings.json"))
    }

    fn supported_events(&self) -> &'static [&'static str] {
        &[
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
            "PermissionRequest",
            "Notification",
            "PreCompact",
            "PostCompact",
            "SubagentStart",
            "SubagentStop",
            "Stop",
            "SessionEnd",
        ]
    }
}
