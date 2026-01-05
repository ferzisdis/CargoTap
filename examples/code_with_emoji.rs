// 🦀 Rust Example with Untypeable Characters
// This file demonstrates how CargoTap handles emoji and other Unicode

fn main() {
    // 🚀 Basic printing
    println!("Hello, World!");

    // ✨ Variables and types
    let x = 42;
    let y = 3.14;

    // 🎯 Conditional logic
    if x > 40 {
        println!("Large number!");
    }

    // 🔥 Pattern matching
    match x {
        0..=10 => println!("Small"),
        11..=50 => println!("Medium"),
        _ => println!("Large"),
    }

    // 💯 Functions work great
    let result = add(5, 7);
    println!("Result: {}", result);
}

// ⭐ Helper function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 👍 All done!
