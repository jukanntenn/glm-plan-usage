//! Utility functions for file operations.
//!
//! This module provides helper functions for atomic file writes
//! to prevent data corruption during config updates.

use anyhow::Context;
use std::fs;
use std::path::Path;

/// Atomically writes content to a file using a temporary file and rename.
///
/// This prevents data corruption if the process is interrupted during write.
/// The temporary file is written first, then renamed to the target path.
///
/// # Errors
///
/// Returns an error if the temporary file cannot be written or renamed.
pub(crate) fn atomic_write(path: impl AsRef<Path>, content: &str) -> anyhow::Result<()> {
    let path = path.as_ref();
    let new_path = path.with_extension("toml.new");
    fs::write(&new_path, content)
        .with_context(|| format!("Failed to write temporary file: {}", new_path.display()))?;
    fs::rename(&new_path, path)
        .with_context(|| format!("Failed to rename temporary file to {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_write_new_file() {
        let dir = std::env::temp_dir().join("glm-plan-usage-test-atomic-write-new");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.toml");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("toml.new"));

        atomic_write(&path, "hello").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_overwrite() {
        let dir = std::env::temp_dir().join("glm-plan-usage-test-atomic-write-overwrite");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.toml");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("toml.new"));

        atomic_write(&path, "first").unwrap();
        atomic_write(&path, "second").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "second");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }
}
