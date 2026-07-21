use std::collections::BTreeMap;
use std::path::PathBuf;

use zellij_tile::prelude::run_command_with_env_variables_and_cwd;

/// Context key tagging a git-info `run_command` call, so its `RunCommandResult` can be matched
/// back to the session it was requested for.
pub const CONTEXT_KEY: &str = "session_git_info";

/// Git branch and worktree-grouping info resolved for a session's cwd.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitInfo {
    pub branch: Option<String>,
    /// Absolute path to the repo's main checkout — the same value for every worktree of the
    /// same repo, since `git rev-parse --git-common-dir` always resolves to the primary repo's
    /// `.git`, worktree or not.
    pub repo_root: Option<String>,
    /// True if this session's cwd IS `repo_root` (the main checkout, not a linked worktree).
    pub is_main_checkout: bool,
}

impl GitInfo {
    /// Parses the `branch\nrepo_root\nis_main` stdout our lookup scripts emit.
    fn parse(stdout: &[u8]) -> Self {
        let text = String::from_utf8_lossy(stdout);
        let mut lines = text.lines();
        let non_empty = |s: &str| (!s.is_empty()).then(|| s.to_string());
        let branch = lines.next().map(str::trim).and_then(non_empty);
        let repo_root = lines.next().map(str::trim).and_then(non_empty);
        let is_main_checkout = lines.next().map(str::trim) == Some("1");
        Self { branch, repo_root, is_main_checkout }
    }
}

/// Shell fragment (expects `dir` to already be set) that prints branch, repo_root, and an
/// is-main-checkout flag for that directory. Shared by both lookup scripts below so the git
/// logic only has to be gotten right once.
fn git_info_for_dir_sh() -> &'static str {
    r#"git -C "$dir" branch --show-current 2>/dev/null
common=$(git -C "$dir" rev-parse --git-common-dir 2>/dev/null) || exit 0
case "$common" in /*) ;; *) common="$dir/$common" ;; esac
repo_root=$(dirname "$common")
printf '%s\n' "$repo_root"
[ "$dir" = "$repo_root" ] && echo 1 || echo 0
"#
}

/// Resolves `$1`'s (a session name's) cwd via the `cwd` Zellij itself records in that session's
/// `session-layout.kdl` cache file, then emits its git info. Tries both the XDG (Linux) and
/// macOS cache-dir conventions since there's no plugin API to ask another session for its cwd
/// directly.
fn lookup_by_session_name_script() -> String {
    format!(
        r#"name="$1"
for base in "${{XDG_CACHE_HOME:-$HOME/.cache}}/zellij" "$HOME/Library/Caches/org.Zellij-Contributors.Zellij"; do
    f="$base/contract_version_1/session_info/$name/session-layout.kdl"
    if [ -f "$f" ]; then
        dir=$(awk -F'"' '/^[[:space:]]*cwd /{{print $2; exit}}' "$f")
        [ -z "$dir" ] && exit 0
        {tail}
        exit 0
    fi
done
"#,
        tail = git_info_for_dir_sh()
    )
}

/// Emits git info for `$1` (a directory) directly, skipping the session-layout.kdl lookup —
/// used when we already know the cwd, e.g. from `Event::HostFolderChanged`.
fn lookup_at_dir_script() -> String {
    format!("dir=\"$1\"\n{tail}", tail = git_info_for_dir_sh())
}

/// Fires a git-info lookup for `name`, resolving its cwd from Zellij's own session cache.
pub fn spawn_lookup_by_name(name: &str) {
    spawn(&lookup_by_session_name_script(), name, name);
}

/// Fires a git-info lookup directly against `dir`, tagging the result for `name`.
pub fn spawn_lookup_at_dir(name: &str, dir: PathBuf) {
    let dir_arg = dir.to_string_lossy().to_string();
    spawn(&lookup_at_dir_script(), &dir_arg, name);
}

/// Runs `script` with `arg` as its `$1`, tagging the eventual `RunCommandResult`'s context with
/// `session_name` so the result can be matched back to the session it was requested for.
fn spawn(script: &str, arg: &str, session_name: &str) {
    let mut context = BTreeMap::new();
    context.insert(CONTEXT_KEY.to_string(), session_name.to_string());
    run_command_with_env_variables_and_cwd(
        &["sh", "-c", script, "sh", arg],
        BTreeMap::new(),
        PathBuf::from("."),
        context,
    );
}

/// Parses a `RunCommandResult`'s context + stdout, returning the session name it was resolved
/// for alongside the parsed git info.
pub fn parse_result(context: &BTreeMap<String, String>, stdout: &[u8]) -> Option<(String, GitInfo)> {
    let name = context.get(CONTEXT_KEY)?;
    Some((name.clone(), GitInfo::parse(stdout)))
}
