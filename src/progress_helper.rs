//! Helper module for integrating progress storage with CodeState
//!
//! This module provides high-level functions to make it easy to save and restore
//! progress in your CargoTap application.

use crate::code_state::CodeState;
use crate::progress_storage::{ProgressStorage, compute_hash};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Initializes a CodeState with saved progress restoration
///
/// # Arguments
/// * `file_path` - Path to the file to load
/// * `storage` - Progress storage instance (should be loaded first)
///
/// # Returns
/// A tuple of (CodeState, content_hash) where the CodeState is positioned
/// at the saved progress point (if any)
pub fn initialize_with_progress(
    file_path: &str,
    storage: &ProgressStorage,
) -> Result<(CodeState, String)> {
    // Load file content
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let current_hash = compute_hash(&content);

    // Create CodeState
    let mut code_state = CodeState::new(content);

    // Try to restore saved progress
    if let Some(progress) = storage.get_progress(file_path) {
        if progress.content_hash == current_hash {
            // File unchanged, restore position
            log::info!(
                "Restoring progress for '{}' at position {}",
                file_path,
                progress.position
            );

            // Advance to saved position
            for _ in 0..progress.position {
                if code_state.type_character().is_none() {
                    log::warn!("Saved position exceeds file length, stopping early");
                    break;
                }
            }
        } else {
            log::info!(
                "File '{}' has changed since last session, starting from beginning",
                file_path
            );
        }
    } else {
        log::info!("No saved progress for '{}', starting fresh", file_path);
    }

    Ok((code_state, current_hash))
}

/// Saves the current progress for a file
///
/// # Arguments
/// * `file_path` - Path to the file
/// * `code_state` - Current code state with cursor position
/// * `content_hash` - Hash of the file content
/// * `storage` - Progress storage instance (will be modified and saved)
pub fn save_progress(
    file_path: &str,
    code_state: &CodeState,
    content_hash: &str,
    storage: &mut ProgressStorage,
) -> Result<()> {
    let position = code_state.get_cursor_position();

    storage.save_progress(file_path.to_string(), content_hash.to_string(), position);

    storage
        .save()
        .with_context(|| "Failed to save progress to disk")?;

    log::debug!("Progress saved: {} at position {}", file_path, position);

    Ok(())
}

/// Loads or creates a ProgressStorage instance
///
/// This is a convenience function that handles errors gracefully.
/// If loading fails, it returns a fresh storage instance.
pub fn load_or_create_storage() -> ProgressStorage {
    let mut storage = ProgressStorage::default();

    match storage.load() {
        Ok(_) => {
            log::info!("Progress loaded successfully");
        }
        Err(e) => {
            log::debug!("Could not load progress ({}), starting fresh", e);
        }
    }

    storage
}

/// Auto-save helper that only saves if enough progress has been made
///
/// This prevents excessive disk writes by only saving when the position
/// has changed by at least `min_delta` characters.
pub struct AutoSaveHelper {
    last_saved_position: usize,
    min_delta: usize,
}

impl AutoSaveHelper {
    /// Creates a new AutoSaveHelper
    ///
    /// # Arguments
    /// * `min_delta` - Minimum number of characters typed before triggering a save
    pub fn new(min_delta: usize) -> Self {
        Self {
            last_saved_position: 0,
            min_delta,
        }
    }

    /// Checks if progress should be saved based on position change
    ///
    /// # Arguments
    /// * `current_position` - Current cursor position
    ///
    /// # Returns
    /// true if enough progress has been made to warrant saving
    pub fn should_save(&self, current_position: usize) -> bool {
        let delta = if current_position > self.last_saved_position {
            current_position - self.last_saved_position
        } else {
            self.last_saved_position - current_position
        };

        delta >= self.min_delta
    }

    /// Saves progress if the delta threshold is met
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `code_state` - Current code state
    /// * `content_hash` - Hash of the file content
    /// * `storage` - Progress storage instance
    ///
    /// # Returns
    /// Ok(true) if saved, Ok(false) if skipped, Err if save failed
    pub fn try_save(
        &mut self,
        file_path: &str,
        code_state: &CodeState,
        content_hash: &str,
        storage: &mut ProgressStorage,
    ) -> Result<bool> {
        let current_position = code_state.get_cursor_position();

        if self.should_save(current_position) {
            save_progress(file_path, code_state, content_hash, storage)?;
            self.last_saved_position = current_position;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Forces a save regardless of delta
    pub fn force_save(
        &mut self,
        file_path: &str,
        code_state: &CodeState,
        content_hash: &str,
        storage: &mut ProgressStorage,
    ) -> Result<()> {
        save_progress(file_path, code_state, content_hash, storage)?;
        self.last_saved_position = code_state.get_cursor_position();
        Ok(())
    }

    /// Resets the last saved position (e.g., when switching files)
    pub fn reset(&mut self) {
        self.last_saved_position = 0;
    }
}

/// Checks if a file exists and is readable
pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    if !path.is_file() {
        anyhow::bail!("Path is not a file: {}", path.display());
    }

    // Try to read the file to ensure we have permissions
    fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path.display()))?;

    Ok(())
}

/// Cleans up old progress entries for files that no longer exist
///
/// This is useful to keep the progress storage file small and organized.
pub fn cleanup_old_progress(storage: &mut ProgressStorage) -> usize {
    let all_files = storage.get_all_files();
    let mut removed_count = 0;

    for file_path in all_files {
        if !Path::new(&file_path).exists() {
            storage.remove_progress(&file_path);
            removed_count += 1;
            log::debug!("Removed progress for non-existent file: {}", file_path);
        }
    }

    if removed_count > 0 {
        log::info!("Cleaned up {} old progress entries", removed_count);
    }

    removed_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_save_helper() {
        let mut helper = AutoSaveHelper::new(10);

        // Should not save with small delta
        assert!(!helper.should_save(5));
        assert!(!helper.should_save(9));

        // Should save when delta >= 10
        assert!(helper.should_save(10));
        assert!(helper.should_save(15));

        // After marking as saved
        helper.last_saved_position = 10;
        assert!(!helper.should_save(15)); // delta = 5
        assert!(helper.should_save(20)); // delta = 10
    }

    #[test]
    fn test_auto_save_helper_backward() {
        let mut helper = AutoSaveHelper::new(10);
        helper.last_saved_position = 50;

        // Should save when going backward too
        assert!(helper.should_save(39)); // delta = 11
        assert!(!helper.should_save(45)); // delta = 5
    }

    #[test]
    fn test_auto_save_reset() {
        let mut helper = AutoSaveHelper::new(10);
        helper.last_saved_position = 100;

        helper.reset();
        assert_eq!(helper.last_saved_position, 0);
    }
}
