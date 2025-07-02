//! Demo module to showcase CodeState functionality
//! This module provides a simple command-line demo of the typing game logic

use crate::code_state::CodeState;
use std::io::{self, Write};

pub fn run_demo() {
    println!("🎮 CargoTap Code State Demo");
    println!("==========================");
    println!("Type the characters as they appear. Use 'q' to quit.");
    println!();

    let demo_code = "fn main() {\n    println!(\"Hello, world!\");\n}".to_string();
    let mut code_state = CodeState::new(demo_code);

    println!("Code to type:");
    println!("{}", code_state.get_full_code());
    println!();

    loop {
        // Show current state
        print_state(&code_state);

        if code_state.is_complete() {
            println!("🎉 Congratulations! You've completed typing the code!");
            break;
        }

        // Get user input
        print!("Type next character (or 'q' to quit, 'b' for backspace): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached
                println!("👋 Goodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();

                if input == "q" {
                    println!("👋 Goodbye!");
                    break;
                }

                if input.is_empty() {
                    println!("Please enter a character or 'q' to quit");
                    continue;
                }

                if input == "\\n" {
                    // Special handling for newline
                    handle_character(&mut code_state, '\n');
                } else if input == "b" {
                    // Backspace functionality
                    if let Some(ch) = code_state.backspace() {
                        println!(
                            "⬅️ Moved '{}' back to current code",
                            if ch == '\n' {
                                "\\n".to_string()
                            } else {
                                ch.to_string()
                            }
                        );
                    } else {
                        println!("❌ Nothing to backspace");
                    }
                } else {
                    let typed_char = input.chars().next().unwrap();
                    handle_character(&mut code_state, typed_char);
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }

        println!();
    }
}

fn handle_character(code_state: &mut CodeState, typed_char: char) {
    if let Some(expected_char) = code_state.peek_next_character() {
        if typed_char == expected_char {
            code_state.type_character();
            println!(
                "✓ Correct! Typed: '{}'",
                if expected_char == '\n' {
                    "\\n".to_string()
                } else {
                    expected_char.to_string()
                }
            );
        } else {
            println!(
                "❌ Incorrect! Expected '{}', got '{}'",
                if expected_char == '\n' {
                    "\\n".to_string()
                } else {
                    expected_char.to_string()
                },
                if typed_char == '\n' {
                    "\\n".to_string()
                } else {
                    typed_char.to_string()
                }
            );
        }
    }
}

fn print_state(code_state: &CodeState) {
    println!(
        "Progress: {:.1}% ({}/{})",
        code_state.get_progress() * 100.0,
        code_state.printed_code.len(),
        code_state.get_total_length()
    );

    // Visual representation
    print!("Typed: ");
    for ch in code_state.printed_code.chars() {
        if ch == '\n' {
            print!("\\n");
        } else {
            print!("{}", ch);
        }
    }
    println!();

    if let Some(next_char) = code_state.peek_next_character() {
        println!(
            "Next: '{}'",
            if next_char == '\n' {
                "\\n".to_string()
            } else {
                next_char.to_string()
            }
        );
    }

    println!("Remaining: {}", code_state.current_code.len());
    println!("---");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_logic() {
        let demo_code = "abc".to_string();
        let mut code_state = CodeState::new(demo_code);

        // Test correct typing
        assert_eq!(code_state.peek_next_character(), Some('a'));
        code_state.type_character();
        assert_eq!(code_state.printed_code, "a");
        assert_eq!(code_state.current_code, "bc");

        // Test backspace
        code_state.backspace();
        assert_eq!(code_state.printed_code, "");
        assert_eq!(code_state.current_code, "abc");

        // Test completion
        code_state.type_character(); // a
        code_state.type_character(); // b
        code_state.type_character(); // c
        assert!(code_state.is_complete());
        assert_eq!(code_state.get_progress(), 1.0);
    }
}
