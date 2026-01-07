//! Progress storage for tracking file reading/typing progress
//!
//! This module provides functionality to persist and restore user progress
//! across sessions, including file path, content hash, and cursor position.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Represents the progress for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProgress {
    /// Path to the file being read/typed
    pub file_path: String,
    /// SHA256 hash of the file content (to detect changes)
    pub content_hash: String,
    /// Position where the user stopped (number of characters typed)
    pub position: usize,
    /// Number of lines scrolled down (view offset)
    #[serde(default)]
    pub scroll_offset: usize,
    /// Optional: timestamp of last access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accessed: Option<u64>,
}

impl FileProgress {
    /// Creates a new FileProgress
    pub fn new(file_path: String, content_hash: String, position: usize) -> Self {
        Self {
            file_path,
            content_hash,
            position,
            scroll_offset: 0,
            last_accessed: None,
        }
    }

    /// Creates a new FileProgress with scroll offset
    pub fn with_scroll_offset(
        file_path: String,
        content_hash: String,
        position: usize,
        scroll_offset: usize,
    ) -> Self {
        Self {
            file_path,
            content_hash,
            position,
            scroll_offset,
            last_accessed: None,
        }
    }

    /// Creates a new FileProgress with timestamp
    pub fn with_timestamp(
        file_path: String,
        content_hash: String,
        position: usize,
        timestamp: u64,
    ) -> Self {
        Self {
            file_path,
            content_hash,
            position,
            scroll_offset: 0,
            last_accessed: Some(timestamp),
        }
    }

    /// Creates a new FileProgress with scroll offset and timestamp
    pub fn with_scroll_offset_and_timestamp(
        file_path: String,
        content_hash: String,
        position: usize,
        scroll_offset: usize,
        timestamp: u64,
    ) -> Self {
        Self {
            file_path,
            content_hash,
            position,
            scroll_offset,
            last_accessed: Some(timestamp),
        }
    }
}

/// Storage manager for file progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStorage {
    /// Map of file paths to their progress
    progress_map: HashMap<String, FileProgress>,
    /// Path to the storage file
    #[serde(skip)]
    storage_path: PathBuf,
}

impl ProgressStorage {
    /// Creates a new ProgressStorage with the given storage file path
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            progress_map: HashMap::new(),
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    /// Creates a ProgressStorage with the default storage path
    /// Default: Uses data directory or falls back to current directory
    pub fn default() -> Self {
        let storage_path = if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("cargo_tap").join("progress.json")
        } else {
            PathBuf::from("cargo_tap_progress.json")
        };
        Self::new(storage_path)
    }

    /// Loads progress from disk
    pub fn load(&mut self) -> io::Result<()> {
        if !self.storage_path.exists() {
            // File doesn't exist yet, start with empty storage
            return Ok(());
        }

        let contents = fs::read_to_string(&self.storage_path)?;
        let loaded: HashMap<String, FileProgress> = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.progress_map = loaded;
        Ok(())
    }

    /// Saves progress to disk
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.progress_map)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.storage_path, json)?;
        Ok(())
    }

    /// Saves or updates progress for a file
    pub fn save_progress(&mut self, file_path: String, content_hash: String, position: usize) {
        let progress = FileProgress::new(file_path.clone(), content_hash, position);
        self.progress_map.insert(file_path, progress);
    }

    /// Saves or updates progress with scroll offset
    pub fn save_progress_with_scroll_offset(
        &mut self,
        file_path: String,
        content_hash: String,
        position: usize,
        scroll_offset: usize,
    ) {
        let progress = FileProgress::with_scroll_offset(
            file_path.clone(),
            content_hash,
            position,
            scroll_offset,
        );
        self.progress_map.insert(file_path, progress);
    }

    /// Saves or updates progress with timestamp
    pub fn save_progress_with_timestamp(
        &mut self,
        file_path: String,
        content_hash: String,
        position: usize,
        timestamp: u64,
    ) {
        let progress =
            FileProgress::with_timestamp(file_path.clone(), content_hash, position, timestamp);
        self.progress_map.insert(file_path, progress);
    }

    /// Saves or updates progress with scroll offset and timestamp
    pub fn save_progress_with_scroll_offset_and_timestamp(
        &mut self,
        file_path: String,
        content_hash: String,
        position: usize,
        scroll_offset: usize,
        timestamp: u64,
    ) {
        let progress = FileProgress::with_scroll_offset_and_timestamp(
            file_path.clone(),
            content_hash,
            position,
            scroll_offset,
            timestamp,
        );
        self.progress_map.insert(file_path, progress);
    }

    /// Gets progress for a specific file
    pub fn get_progress(&self, file_path: &str) -> Option<&FileProgress> {
        self.progress_map.get(file_path)
    }

    /// Removes progress for a specific file
    pub fn remove_progress(&mut self, file_path: &str) -> Option<FileProgress> {
        self.progress_map.remove(file_path)
    }

    /// Clears all stored progress
    pub fn clear(&mut self) {
        self.progress_map.clear();
    }

    /// Returns the number of files being tracked
    pub fn count(&self) -> usize {
        self.progress_map.len()
    }

    /// Checks if a file's content has changed by comparing hashes
    pub fn has_file_changed(&self, file_path: &str, current_hash: &str) -> bool {
        if let Some(progress) = self.get_progress(file_path) {
            progress.content_hash != current_hash
        } else {
            true // No previous progress, consider it changed
        }
    }

    /// Gets all file paths being tracked
    pub fn get_all_files(&self) -> Vec<String> {
        self.progress_map.keys().cloned().collect()
    }
}

/// Utility function to compute SHA256 hash of a string
pub fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Utility function to compute SHA256 hash of a file
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(compute_hash(&content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_progress_creation() {
        let progress = FileProgress::new("test.txt".to_string(), "abc123".to_string(), 42);

        assert_eq!(progress.file_path, "test.txt");
        assert_eq!(progress.content_hash, "abc123");
        assert_eq!(progress.position, 42);
        assert_eq!(progress.scroll_offset, 0);
        assert!(progress.last_accessed.is_none());
    }

    #[test]
    fn test_file_progress_with_timestamp() {
        let progress = FileProgress::with_timestamp(
            "test.txt".to_string(),
            "abc123".to_string(),
            42,
            1234567890,
        );

        assert_eq!(progress.last_accessed, Some(1234567890));
    }

    #[test]
    fn test_progress_storage_new() {
        let storage = ProgressStorage::new("test_progress.json");
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_save_and_get_progress() {
        let mut storage = ProgressStorage::new("test_progress.json");

        storage.save_progress("file1.txt".to_string(), "hash1".to_string(), 100);

        let progress = storage.get_progress("file1.txt");
        assert!(progress.is_some());

        let progress = progress.unwrap();
        assert_eq!(progress.file_path, "file1.txt");
        assert_eq!(progress.content_hash, "hash1");
        assert_eq!(progress.position, 100);
    }

    #[test]
    fn test_remove_progress() {
        let mut storage = ProgressStorage::new("test_progress.json");

        storage.save_progress("file1.txt".to_string(), "hash1".to_string(), 100);

        assert_eq!(storage.count(), 1);

        let removed = storage.remove_progress("file1.txt");
        assert!(removed.is_some());
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_has_file_changed() {
        let mut storage = ProgressStorage::new("test_progress.json");

        storage.save_progress("file1.txt".to_string(), "hash1".to_string(), 100);

        assert!(!storage.has_file_changed("file1.txt", "hash1"));
        assert!(storage.has_file_changed("file1.txt", "different_hash"));
        assert!(storage.has_file_changed("nonexistent.txt", "hash1"));
    }

    #[test]
    fn test_clear() {
        let mut storage = ProgressStorage::new("test_progress.json");

        storage.save_progress("file1.txt".to_string(), "hash1".to_string(), 100);
        storage.save_progress("file2.txt".to_string(), "hash2".to_string(), 200);

        assert_eq!(storage.count(), 2);

        storage.clear();
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("hello world");
        let hash2 = compute_hash("hello world");
        let hash3 = compute_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_save_and_load_from_disk() {
        let temp_path = "test_storage_temp.json";

        // Clean up if exists
        let _ = fs::remove_file(temp_path);

        // Create and save
        {
            let mut storage = ProgressStorage::new(temp_path);
            storage.save_progress("file1.txt".to_string(), "hash1".to_string(), 100);
            storage.save_progress("file2.txt".to_string(), "hash2".to_string(), 200);
            storage.save().expect("Failed to save");
        }

        // Load in new instance
        {
            let mut storage = ProgressStorage::new(temp_path);
            storage.load().expect("Failed to load");

            assert_eq!(storage.count(), 2);

            let progress = storage.get_progress("file1.txt").unwrap();
            assert_eq!(progress.position, 100);

            let progress = storage.get_progress("file2.txt").unwrap();
            assert_eq!(progress.position, 200);
        }

        // Clean up
        let _ = fs::remove_file(temp_path);
    }
}
