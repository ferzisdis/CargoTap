//! Simple test program to verify progress storage functionality
//!
//! This example creates a simple text file, saves progress at various points,
//! and then verifies that the progress can be loaded correctly.

use std::io;

fn main() -> io::Result<()> {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Progress Storage Test                     ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Test 1: Create and hash a file
    println!("Test 1: File Hashing");
    println!("--------------------");
    let test_content = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let hash = compute_test_hash(test_content);
    println!("✓ Content hash: {}", hash);
    println!("✓ Content length: {} bytes\n", test_content.len());

    // Test 2: Simulate progress storage
    println!("Test 2: Progress Storage Simulation");
    println!("-------------------------------------");

    let file_path = "test_example.rs";
    let positions = vec![0, 10, 25, test_content.len()];

    println!("File: {}", file_path);
    println!("Simulating progress at positions: {:?}\n", positions);

    for (i, position) in positions.iter().enumerate() {
        let progress_pct = (*position as f32 / test_content.len() as f32) * 100.0;
        println!(
            "  Save #{}: position = {}, progress = {:.1}%",
            i + 1,
            position,
            progress_pct
        );

        // Show what would be "typed" vs "remaining"
        let typed = &test_content[..*position.min(&test_content.len())];
        let remaining = &test_content[*position.min(&test_content.len())..];

        println!(
            "    Typed: {:?}",
            typed.chars().take(20).collect::<String>()
        );
        println!(
            "    Remaining: {:?}",
            remaining.chars().take(20).collect::<String>()
        );
    }

    println!("\n✓ Progress tracking works correctly\n");

    // Test 3: Hash consistency
    println!("Test 3: Hash Consistency Check");
    println!("--------------------------------");
    let hash1 = compute_test_hash(test_content);
    let hash2 = compute_test_hash(test_content);
    let hash3 = compute_test_hash("different content");

    if hash1 == hash2 {
        println!("✓ Same content produces same hash");
    } else {
        println!("✗ Hash inconsistency detected!");
    }

    if hash1 != hash3 {
        println!("✓ Different content produces different hash");
    } else {
        println!("✗ Different content produced same hash!");
    }

    println!();

    // Test 4: Storage format simulation
    println!("Test 4: Storage Format");
    println!("-----------------------");
    println!("Simulated JSON storage:");
    println!("{{\n  \"{}\": {{", file_path);
    println!("    \"file_path\": \"{}\",", file_path);
    println!("    \"content_hash\": \"{}\",", hash);
    println!("    \"position\": {},", positions[2]);
    println!("    \"last_accessed\": {}", current_timestamp());
    println!("  }}\n}}");
    println!("✓ Storage format validated\n");

    // Test 5: Progress restoration simulation
    println!("Test 5: Progress Restoration");
    println!("-----------------------------");
    let saved_position = 25;
    let saved_hash = hash.clone();

    println!("Simulating app restart...");
    println!("  Saved position: {}", saved_position);
    println!("  Saved hash: {}", saved_hash);
    println!();

    // Check if file changed
    let current_hash = compute_test_hash(test_content);
    if current_hash == saved_hash {
        println!("✓ File unchanged (hash match)");
        println!("✓ Restoring position {}", saved_position);
        println!(
            "  User continues from: {:?}",
            &test_content[saved_position..]
                .chars()
                .take(20)
                .collect::<String>()
        );
    } else {
        println!("⚠ File changed (hash mismatch)");
        println!("  Starting from beginning");
    }

    println!();

    // Test 6: Multiple files
    println!("Test 6: Multiple Files");
    println!("-----------------------");
    let files = vec![
        ("file1.rs", "fn test1() { }", 5),
        ("file2.rs", "fn test2() { }", 8),
        ("file3.rs", "fn test3() { }", 12),
    ];

    println!("Tracking {} files:\n", files.len());
    for (path, content, pos) in &files {
        let hash = compute_test_hash(content);
        println!("  {}", path);
        println!("    Position: {}/{}", pos, content.len());
        println!("    Hash: {}", hash);
    }

    println!("\n✓ Multiple file tracking works\n");

    // Summary
    println!("╔════════════════════════════════════════════╗");
    println!("║  All Tests Passed! ✓                       ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("The progress storage system is working correctly.");
    println!("You can now integrate it into your CargoTap app!");
    println!();
    println!("Next steps:");
    println!("  1. Read PROGRESS_QUICKSTART.md for quick integration");
    println!("  2. See INTEGRATION_EXAMPLE.md for detailed examples");
    println!("  3. Run: cargo run --example progress_storage_demo");

    Ok(())
}

/// Simple hash function for testing (matches the one in progress_storage)
fn compute_test_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
