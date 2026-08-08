mod claude;
mod codex;

use std::path::Path;

use serde_json::{json, Map, Value};

pub(crate) const OWNED_MARKER: &str = "SESSION_PICKER_AGENT_HOOK=1";

pub(crate) const AGENTS: [&dyn Agent; 2] = [&codex::Codex, &claude::Claude];

pub(crate) fn find(label: &str) -> Option<&'static dyn Agent> {
    AGENTS.iter().copied().find(|a| a.label() == label)
}

/// One coding agent's integration with this bridge: where its hook config lives, which
/// lifecycle events it can fire, and how to write itself into (or out of) that config.
/// The `install`/`uninstall`/`is_installed` defaults assume the shared
/// `hooks.<Event> = [{"hooks": [{...}]}]` shape both Codex and Claude use today; an agent
/// whose config format differs can override any of them.
pub(crate) trait Agent {
    /// Name written into `AgentRecord.agent_label` and matched against `--agent <label>`.
    fn label(&self) -> &'static str;
    fn config_path(&self) -> Result<std::path::PathBuf, String>;
    fn supported_events(&self) -> &'static [&'static str];

    fn install(&self, root: &mut Value, executable: &Path) -> Result<(), String> {
        default_install(root, self.supported_events(), self.label(), executable)
    }

    fn uninstall(&self, root: &mut Value) -> Result<(), String> {
        default_uninstall(root)
    }

    fn is_installed(&self, path: &Path) -> bool {
        default_is_installed(path)
    }
}

fn hooks_object(root: &mut Value) -> Result<&mut Map<String, Value>, String> {
    if !root.is_object() {
        return Err("hook configuration root must be an object".into());
    }
    let hooks = root
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| json!({}));
    hooks.as_object_mut().ok_or_else(|| "hooks must be an object".into())
}

fn remove_owned_groups(value: &mut Value) {
    let Some(groups) = value.as_array_mut() else {
        return;
    };
    groups.retain(|group| {
        !group
            .get("hooks")
            .and_then(Value::as_array)
            .is_some_and(|handlers| {
                handlers.iter().any(|handler| {
                    handler
                        .get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|command| command.contains(OWNED_MARKER))
                })
            })
    });
}

fn default_uninstall(root: &mut Value) -> Result<(), String> {
    let hooks = hooks_object(root)?;
    for value in hooks.values_mut() {
        remove_owned_groups(value);
    }
    Ok(())
}

fn default_install(
    root: &mut Value,
    events: &[&str],
    label: &str,
    executable: &Path,
) -> Result<(), String> {
    default_uninstall(root)?;
    let hooks = hooks_object(root)?;
    let command = format!(
        "{} {} hook --agent {}",
        OWNED_MARKER,
        shell_quote(executable),
        label
    );
    for event in events {
        let groups = hooks
            .entry((*event).to_string())
            .or_insert_with(|| json!([]));
        let groups = groups
            .as_array_mut()
            .ok_or_else(|| format!("hooks.{event} must be an array"))?;
        groups.push(json!({"hooks": [{"type": "command", "command": command, "timeout": 5}]}));
    }
    Ok(())
}

fn default_is_installed(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|content| content.contains(OWNED_MARKER))
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_preserves_other_handlers_and_is_idempotent() {
        let mut value =
            json!({"hooks": {"Stop": [{"hooks": [{"type":"command", "command":"island"}]}]}});
        let executable = Path::new("/tmp/bridge");
        let events = codex::Codex.supported_events();
        default_install(&mut value, events, "codex", executable).unwrap();
        default_install(&mut value, events, "codex", executable).unwrap();
        let stop = value["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2);
        assert!(value.to_string().contains("island"));
        default_uninstall(&mut value).unwrap();
        assert_eq!(value["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }
}
