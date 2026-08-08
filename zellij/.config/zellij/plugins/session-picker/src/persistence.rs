//! Versioned, host-independent snapshots and a small atomic JSON store.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::app::AppSnapshot;
use crate::ui::ThemePalette;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentState {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub theme: Option<ThemePalette>,
    #[serde(default)]
    pub app: AppSnapshot,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            theme: None,
            app: AppSnapshot::default(),
        }
    }
}

impl PersistentState {
    pub fn new(theme: ThemePalette, app: AppSnapshot) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            theme: Some(theme),
            app,
        }
    }
}

fn schema_version() -> u32 {
    SCHEMA_VERSION
}

#[derive(Default)]
pub struct SnapshotStore {
    path: Option<PathBuf>,
    temp_path: Option<PathBuf>,
    last_bytes: Option<Vec<u8>>,
}

impl SnapshotStore {
    pub fn at(path: impl Into<PathBuf>, writer_id: u128) -> Self {
        let path = path.into();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state.json");
        let temp_path = path.with_file_name(format!(".{file_name}.{writer_id}.tmp"));
        Self {
            path: Some(path),
            temp_path: Some(temp_path),
            last_bytes: None,
        }
    }

    pub fn load(&mut self) -> io::Result<Option<PersistentState>> {
        let Some(path) = self.path.as_ref() else {
            return Ok(None);
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let state: PersistentState = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if state.schema_version != SCHEMA_VERSION {
            return Ok(None);
        }
        self.last_bytes = Some(canonical_bytes(&state)?);
        Ok(Some(state))
    }

    pub fn save_if_changed(&mut self, state: &PersistentState) -> io::Result<bool> {
        let Some(path) = self.path.as_ref() else {
            return Ok(false);
        };
        let bytes = canonical_bytes(state)?;
        if self.last_bytes.as_deref() == Some(bytes.as_slice()) {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp_path = self
            .temp_path
            .as_ref()
            .expect("configured store has temp path");
        fs::write(temp_path, &bytes)?;
        fs::rename(temp_path, path)?;
        self.last_bytes = Some(bytes);
        Ok(true)
    }
}

fn canonical_bytes(state: &PersistentState) -> io::Result<Vec<u8>> {
    serde_json::to_vec(state).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("session-picker-{name}-{nonce}.json"))
    }

    #[test]
    fn round_trip_and_unchanged_write_suppression() {
        let path = test_path("round-trip");
        let mut store = SnapshotStore::at(&path, 1);
        let state = PersistentState::new(ThemePalette::default(), AppSnapshot::default());
        assert!(store.save_if_changed(&state).unwrap());
        assert!(!store.save_if_changed(&state).unwrap());

        let mut restored = SnapshotStore::at(&path, 2);
        assert_eq!(restored.load().unwrap(), Some(state));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn corrupt_and_future_snapshots_are_nonfatal() {
        let corrupt = test_path("corrupt");
        fs::write(&corrupt, b"not-json").unwrap();
        assert_eq!(
            SnapshotStore::at(&corrupt, 1).load().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let _ = fs::remove_file(corrupt);

        let future = test_path("future");
        fs::write(&future, br#"{"schema_version":999}"#).unwrap();
        assert_eq!(SnapshotStore::at(&future, 1).load().unwrap(), None);
        let _ = fs::remove_file(future);
    }
}
