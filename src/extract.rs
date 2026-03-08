use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

/// Writes the embedded aria2c bytes to a temporary directory and makes it executable.
/// Returns (TempDir, path_to_aria2c). Caller must hold TempDir to keep the directory alive.
pub fn extract_aria2c(bytes: &[u8]) -> Result<(TempDir, PathBuf)> {
    let dir = TempDir::new().context("Failed to create temp directory")?;
    let bin_path = dir.path().join("aria2c");
    fs::write(&bin_path, bytes).context("Failed to write aria2c binary")?;
    fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))
        .context("Failed to chmod aria2c")?;
    Ok((dir, bin_path))
}
