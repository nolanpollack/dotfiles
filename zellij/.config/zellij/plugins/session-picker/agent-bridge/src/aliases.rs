use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::fs as atomic_fs;
use crate::store;

fn aliases_path() -> Result<PathBuf, String> {
    Ok(store::state_root()?.join("session-aliases"))
}

pub(crate) fn read_aliases() -> BTreeMap<String, String> {
    aliases_path()
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn write_aliases(aliases: &BTreeMap<String, String>) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(aliases).map_err(|e| e.to_string())?;
    atomic_fs::atomic_write(&aliases_path()?, &bytes)
}

pub(crate) fn resolve_session_alias(session: &str) -> String {
    let aliases = read_aliases();
    let mut current = session;
    for _ in 0..16 {
        let Some(next) = aliases.get(current) else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
    }
    current.to_string()
}
