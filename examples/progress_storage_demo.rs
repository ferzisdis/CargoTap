//! Example demonstrating the progress storage system
//!
//! This example shows how to:
//! - Create a progress storage instance
//! - Save file reading progress
//! - Load progress from disk
//! - Detect file changes using hashes
//! - Restore user position in a file

use std::fs;
use std::io::{self, Write};
use std::path::Path;

// Note: In a real scenario, you would import these from the crate
// For this example, we'll demonstrate the API usage

fn main() -> io::Result<()> {
    println!("=== CargoTap Progress Storage Demo ===\n");

    // Example 1: Basic usage - creating and saving progress
    example_basic_usage()?;

    println!("\n---\n");

    // Example 2: Loading progress on app startup
    example_loading_progress()?;

    println!("\n---\n");

    // Example 3: Detecting file changes
    example_file_change_detection()?;

    println!("\n---\n");

    // Example 4: Managing multiple files
    example_multiple_files()?;

    Ok(())
}

/// Example 1: Basic usage
fn example_basic_usage() -> io::Result<()> {
    println!("Example 1: Basic Usage");
    println!("----------------------");

    // Create a test file
    let test_file = "demo_code.rs";
    let content = r#"fn main() {
    println!("Hello, world!");
    let x = 42;
    println!("x = {}", x);
}"#;

    fs::write(test_file, content)?;

    // Compute hash of the file content
    let hash = compute_simple_hash(content);

    println!("Created file: {}", test_file);
    println!("Content hash: {}", hash);

    // Simulate user typing up to position 25 (middle of the code)
    let position = 25;
    println!("User typed {} characters", position);

    // Create storage and save progress
    println!("\nSaving progress to storage...");

    // This is how you would use it in your app:
    // let mut storage = ProgressStorage::default();
    // storage.load()?; // Load existing progress
    // storage.save_progress(test_file.to_string(), hash, position);
    // storage.save()?; // Persist to disk

    println!("Progress saved successfully!");
    println!("File: {}", test_file);
    println!("Position: {}", position);
    println!("Hash: {}", hash);

    // Clean up
    fs::remove_file(test_file)?;

    Ok(())
}

/// Example 2: Loading progress on app startup
fn example_loading_progress() -> io::Result<()> {
    println!("Example 2: Loading Progress on Startup");
    println!("---------------------------------------");

    // Simulating app startup scenario
    println!("App starting up...");

    // This is what you would do when your app starts:
    println!("\nStep 1: Initialize storage");
    // let mut storage = ProgressStorage::default();

    println!("Step 2: Load saved progress from disk");
    // storage.load()?;

    println!("Step 3: Check if we have progress for the current file");
    let current_file = "src/main.rs";
    // if let Some(progress) = storage.get_progress(current_file) {
    //     println!("Found saved progress!");
    //     println!("  - File: {}", progress.file_path);
    //     println!("  - Position: {}", progress.position);
    //     println!("  - Hash: {}", progress.content_hash);
    //
    //     // Restore the position in your CodeState
    //     // code_state.restore_position(progress.position);
    // } else {
    //     println!("No saved progress found, starting from beginning");
    // }

    println!("\nIn your actual app, you would:");
    println!("1. Create ProgressStorage::default()");
    println!("2. Call storage.load() to read from disk");
    println!("3. Use storage.get_progress(file_path) to restore position");
    println!("4. Initialize your CodeState with the restored position");

    Ok(())
}

/// Example 3: Detecting file changes
fn example_file_change_detection() -> io::Result<()> {
    println!("Example 3: File Change Detection");
    println!("---------------------------------");

    let test_file = "demo_file.txt";

    // Original content
    let original_content = "Hello, this is the original content!";
    fs::write(test_file, original_content)?;
    let original_hash = compute_simple_hash(original_content);

    println!("Original content: \"{}\"", original_content);
    println!("Original hash: {}", original_hash);

    // Simulate saving progress with original hash
    let saved_position = 20;
    println!("\nUser progress saved at position {}", saved_position);

    // Now the file is modified
    let modified_content = "Hello, this content has been modified!";
    fs::write(test_file, modified_content)?;
    let new_hash = compute_simple_hash(modified_content);

    println!("\nFile was modified!");
    println!("New content: \"{}\"", modified_content);
    println!("New hash: {}", new_hash);

    // When loading, detect the change
    println!("\nDetecting changes:");
    if original_hash != new_hash {
        println!("⚠️  File has changed since last session!");
        println!("Options:");
        println!("  1. Start from beginning (position 0)");
        println!("  2. Try to restore position anyway (might be incorrect)");
        println!("  3. Ask user what to do");
    } else {
        println!(
            "✓ File unchanged, safe to restore position {}",
            saved_position
        );
    }

    // This is how you'd use it in your app:
    // if storage.has_file_changed(test_file, &new_hash) {
    //     println!("File changed! Starting from beginning.");
    //     storage.save_progress(test_file.to_string(), new_hash, 0);
    // } else {
    //     // Safe to use saved position
    // }

    // Clean up
    fs::remove_file(test_file)?;

    Ok(())
}

/// Example 4: Managing multiple files
fn example_multiple_files() -> io::Result<()> {
    println!("Example 4: Managing Multiple Files");
    println!("-----------------------------------");

    // Create multiple test files
    let files = vec![
        ("file1.rs", "fn main() { println!(\"File 1\"); }", 15),
        ("file2.rs", "fn test() { let x = 42; }", 10),
        ("file3.rs", "struct Point { x: i32, y: i32 }", 20),
    ];

    println!("Tracking progress for multiple files:\n");

    for (filename, content, position) in &files {
        let hash = compute_simple_hash(content);
        println!("File: {}", filename);
        println!("  Position: {}", position);
        println!("  Hash: {}", hash);

        // In your app:
        // storage.save_progress(filename.to_string(), hash, *position);
    }

    println!("\nYou can:");
    println!("- storage.get_all_files() - list all tracked files");
    println!("- storage.count() - get number of tracked files");
    println!("- storage.remove_progress(file) - remove specific file");
    println!("- storage.clear() - clear all progress");

    Ok(())
}

/// Simple hash computation for demo purposes
fn compute_simple_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Integration example with CodeState
fn integration_example() {
    println!("\n=== Integration with CodeState ===\n");

    println!("When initializing your app:");
    println!(
        r#"
// 1. Load the file content
let file_path = "src/main.rs";
let content = fs::read_to_string(file_path)?;

// 2. Compute hash
let current_hash = progress_storage::compute_hash(&content);

// 3. Initialize storage and load saved progress
let mut storage = ProgressStorage::default();
storage.load()?;

// 4. Check for saved progress
let starting_position = if let Some(progress) = storage.get_progress(file_path) {
    if progress.content_hash == current_hash {
        // File unchanged, restore position
        println!("Restoring position: {}", progress.position);
        progress.position
    } else {
        // File changed, start from beginning
        println!("File changed, starting from beginning");
        0
    }
} else {
    // No saved progress
    0
};

// 5. Initialize CodeState with proper position
let mut code_state = CodeState::new(content.clone());

// Type characters up to the saved position to restore state
for _ in 0..starting_position {
    code_state.type_character();
}

// 6. During gameplay, periodically save progress
// (e.g., every N characters typed, or on window close)
let current_position = code_state.get_cursor_position();
storage.save_progress(
    file_path.to_string(),
    current_hash,
    current_position
);
storage.save()?;
"#
    );
}
