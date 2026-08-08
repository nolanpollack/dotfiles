use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Writes `bytes` to `path` via a temp file + rename so readers never observe a partial write.
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("tmp.{}", std::process::id()));
    let mut file = File::create(&temp).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(temp, path).map_err(|e| e.to_string())
}
