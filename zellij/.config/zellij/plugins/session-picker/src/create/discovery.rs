use std::path::PathBuf;

/// Fires a `zoxide query -l` lookup. Results come back async via `Event::RunCommandResult`;
/// parse them with `parse_zoxide_result`.
pub fn zoxide_list_args() -> Vec<String> {
    vec![
        "sh".to_string(),
        "-c".to_string(),
        "zoxide query -l".to_string(),
    ]
}

/// Returns the directories from a `zoxide query -l` `RunCommandResult`, or `None` if this
/// result isn't tagged as one of ours.
pub fn parse_zoxide_result(stdout: &[u8]) -> Vec<PathBuf> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_blank_discovery_lines() {
        assert_eq!(
            parse_zoxide_result(b"/one\n\n/two\n"),
            vec![PathBuf::from("/one"), PathBuf::from("/two")]
        );
    }
}
