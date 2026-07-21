use std::collections::BTreeMap;
use std::path::PathBuf;

use zellij_tile::prelude::run_command_with_env_variables_and_cwd;

/// Context key tagging a create-flow discovery `run_command` call. Deliberately a different key
/// name than `git_info::CONTEXT_KEY` so both can be tried against the same `RunCommandResult`
/// context map without either mistaking the other's result for its own.
pub const CONTEXT_KEY: &str = "create_discovery";

const ZOXIDE_LIST_TAG: &str = "zoxide_list";

/// Fires a `zoxide query -l` lookup. Results come back async via `Event::RunCommandResult`;
/// parse them with `parse_zoxide_result`.
pub fn fetch_zoxide_list() {
    let mut context = BTreeMap::new();
    context.insert(CONTEXT_KEY.to_string(), ZOXIDE_LIST_TAG.to_string());
    run_command_with_env_variables_and_cwd(
        &["sh", "-c", "zoxide query -l"],
        BTreeMap::new(),
        PathBuf::from("."),
        context,
    );
}

/// Returns the directories from a `zoxide query -l` `RunCommandResult`, or `None` if this
/// result isn't tagged as one of ours.
pub fn parse_zoxide_result(context: &BTreeMap<String, String>, stdout: &[u8]) -> Option<Vec<PathBuf>> {
    if context.get(CONTEXT_KEY).map(String::as_str) != Some(ZOXIDE_LIST_TAG) {
        return None;
    }
    let text = String::from_utf8_lossy(stdout);
    Some(text.lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect())
}
