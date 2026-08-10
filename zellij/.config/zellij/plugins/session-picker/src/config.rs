use std::collections::BTreeMap;

use session_picker::app::AppConfig;
use session_picker::create::worktree::Config as WorktreeConfig;
use session_picker::ui::ThemeOverrides;

const DEFAULT_AGENT_BRIDGE_PATH: &str = "session-picker-agent-bridge";

/// Typed boundary for Zellij's string-valued plugin configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginConfig {
    pub app: AppConfig,
    pub theme_overrides: ThemeOverrides,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            app: AppConfig {
                agent_bridge: DEFAULT_AGENT_BRIDGE_PATH.into(),
                worktree: WorktreeConfig::default(),
            },
            theme_overrides: ThemeOverrides::default(),
        }
    }
}

impl PluginConfig {
    // Blank agent and branch values fall back to defaults; a blank worktree root remains empty.
    pub fn parse(configuration: BTreeMap<String, String>) -> Self {
        let mut parsed = Self::default();
        let mut theme_entries = Vec::new();

        for (key, value) in configuration {
            match key.as_str() {
                "agent_bridge_path" => {
                    if !value.is_empty() {
                        parsed.app.agent_bridge = value;
                    }
                }
                "branch_prefix" => {
                    if !value.is_empty() {
                        parsed.app.worktree.branch_prefix = value;
                    }
                }
                "worktree_root" => {
                    parsed.app.worktree.worktree_root = value.into();
                }
                _ => theme_entries.push((key, value)),
            }
        }

        parsed.theme_overrides = ThemeOverrides::from_entries(theme_entries);
        parsed
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use session_picker::ui::ThemeOverrides;

    #[test]
    fn defaults_match_plugin_defaults() {
        let config = PluginConfig::parse(BTreeMap::new());

        assert_eq!(config, PluginConfig::default());
    }

    #[test]
    fn overrides_are_parsed_into_typed_configuration() {
        let config = PluginConfig::parse(BTreeMap::from([
            ("agent_bridge_path".into(), "/tmp/bridge".into()),
            ("branch_prefix".into(), "feature".into()),
            ("worktree_root".into(), "/tmp/worktrees".into()),
            ("query_fg".into(), "1, 2, 3".into()),
        ]));

        assert_eq!(config.app.agent_bridge, "/tmp/bridge");
        assert_eq!(config.app.worktree.branch_prefix, "feature");
        assert_eq!(
            config.app.worktree.worktree_root,
            PathBuf::from("/tmp/worktrees")
        );
        assert_eq!(
            config.theme_overrides,
            ThemeOverrides::from_entries([("query_fg".into(), "1, 2, 3".into())])
        );
    }

    #[test]
    fn blank_values_preserve_existing_fallback_semantics() {
        let config = PluginConfig::parse(BTreeMap::from([
            ("agent_bridge_path".into(), String::new()),
            ("branch_prefix".into(), String::new()),
            ("worktree_root".into(), String::new()),
        ]));

        assert_eq!(config.app.agent_bridge, DEFAULT_AGENT_BRIDGE_PATH);
        assert_eq!(
            config.app.worktree.branch_prefix,
            WorktreeConfig::default().branch_prefix
        );
        assert_eq!(config.app.worktree.worktree_root, PathBuf::new());
    }
}
