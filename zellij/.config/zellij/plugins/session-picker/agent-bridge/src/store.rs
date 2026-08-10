use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_core::Agent;
use fs2::FileExt;

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(crate) fn state_root() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("SESSION_PICKER_AGENT_STATE_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = env::var("HOME").map_err(|_| "HOME is not set")?;
    Ok(PathBuf::from(home).join(".local/state/session-picker/agents"))
}

pub(crate) fn with_lock<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let root = state_root()?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".lock"))
        .map_err(|e| e.to_string())?;
    lock.lock_exclusive().map_err(|e| e.to_string())?;
    let result = f();
    let _ = fs2::FileExt::unlock(&lock);
    result
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .take(180)
        .collect()
}

pub(crate) fn record_path(id: &str) -> Result<PathBuf, String> {
    Ok(state_root()?.join(format!("{}.json", sanitize(id))))
}

pub(crate) fn record_paths() -> Result<Vec<PathBuf>, String> {
    let root = state_root()?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    fs::read_dir(root)
        .map_err(|e| e.to_string())
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("json"))
                .collect()
        })
}

pub(crate) fn read_record(path: &Path) -> Result<Agent, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub(crate) fn write_record(path: &Path, record: &Agent) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?;
    crate::fs::atomic_write(path, &bytes)
}

pub(crate) fn remove_matching(mut predicate: impl FnMut(&Agent) -> bool) -> Result<(), String> {
    for path in record_paths()? {
        if read_record(&path).is_ok_and(|record| predicate(&record)) {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
