use anyhow::Result;

// Import the colored text types
use crate::text::{ColoredChar, ColoredText};

/// Demonstrates various colored text effects
pub struct ColoredTextDemo;

impl ColoredTextDemo {
    /// Creates a rainbow text effect
    pub fn create_rainbow_text(text: &str) -> ColoredText {
        let mut colored_text = ColoredText::new();
        let colors = [
            [1.0, 0.0, 0.0, 1.0], // Red
            [1.0, 0.5, 0.0, 1.0], // Orange
            [1.0, 1.0, 0.0, 1.0], // Yellow
            [0.0, 1.0, 0.0, 1.0], // Green
            [0.0, 0.0, 1.0, 1.0], // Blue
            [0.5, 0.0, 1.0, 1.0], // Indigo
            [1.0, 0.0, 1.0, 1.0], // Violet
        ];

        for (i, ch) in text.chars().enumerate() {
            let color = colors[i % colors.len()];
            colored_text.push(ch, color);
        }

        colored_text
    }

    /// Creates syntax highlighted Rust code
    pub fn create_syntax_highlighted_rust(code: &str) -> ColoredText {
        let mut colored_text = ColoredText::new();
        let mut current_word = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let mut comment_type = CommentType::None;
        let mut prev_char = '\0';

        for ch in code.chars() {
            // Handle string literals
            if ch == '"' && !in_comment && prev_char != '\\' {
                in_string = !in_string;
                colored_text.push(ch, [1.0, 0.6, 0.6, 1.0]); // Pink for quotes
                prev_char = ch;
                continue;
            }

            if in_string {
                colored_text.push(ch, [1.0, 0.8, 0.4, 1.0]); // Light orange for string content
                prev_char = ch;
                continue;
            }

            // Handle comments
            if !in_comment && ch == '/' && prev_char == '/' {
                in_comment = true;
                comment_type = CommentType::Line;
                // Remove the previous '/' and add both as comment color
                if let Some(last_char) = colored_text.chars.pop() {
                    colored_text.push('/', [0.5, 0.5, 0.5, 1.0]); // Gray for comments
                }
                colored_text.push(ch, [0.5, 0.5, 0.5, 1.0]);
                prev_char = ch;
                continue;
            }

            if !in_comment && ch == '*' && prev_char == '/' {
                in_comment = true;
                comment_type = CommentType::Block;
                // Remove the previous '/' and add both as comment color
                if let Some(last_char) = colored_text.chars.pop() {
                    colored_text.push('/', [0.5, 0.5, 0.5, 1.0]);
                }
                colored_text.push(ch, [0.5, 0.5, 0.5, 1.0]);
                prev_char = ch;
                continue;
            }

            if in_comment {
                if comment_type == CommentType::Line && ch == '\n' {
                    in_comment = false;
                    comment_type = CommentType::None;
                } else if comment_type == CommentType::Block && ch == '/' && prev_char == '*' {
                    in_comment = false;
                    comment_type = CommentType::None;
                }
                colored_text.push(ch, [0.5, 0.5, 0.5, 1.0]); // Gray for comments
                prev_char = ch;
                continue;
            }

            // Handle words (keywords, identifiers, etc.)
            if ch.is_alphanumeric() || ch == '_' {
                current_word.push(ch);
            } else {
                if !current_word.is_empty() {
                    let color = Self::get_keyword_color(&current_word);
                    for word_ch in current_word.chars() {
                        colored_text.push(word_ch, color);
                    }
                    current_word.clear();
                }

                // Color special characters
                let char_color = match ch {
                    '{' | '}' | '(' | ')' | '[' | ']' => [1.0, 0.8, 0.4, 1.0], // Orange brackets
                    ';' | ',' => [0.8, 0.8, 0.8, 1.0], // Light gray punctuation
                    '=' | '+' | '-' | '*' | '/' | '%' => [1.0, 1.0, 0.4, 1.0], // Yellow operators
                    '!' => [1.0, 0.4, 1.0, 1.0],       // Magenta exclamation
                    '.' => [0.8, 0.8, 1.0, 1.0],       // Light blue dots
                    ':' => [1.0, 0.8, 0.8, 1.0],       // Light red colons
                    _ => [0.8, 0.8, 0.8, 1.0],         // Default light gray
                };
                colored_text.push(ch, char_color);
            }

            prev_char = ch;
        }

        // Handle last word if any
        if !current_word.is_empty() {
            let color = Self::get_keyword_color(&current_word);
            for word_ch in current_word.chars() {
                colored_text.push(word_ch, color);
            }
        }

        colored_text
    }

    /// Creates a gradient text effect
    pub fn create_gradient_text(
        text: &str,
        start_color: [f32; 4],
        end_color: [f32; 4],
    ) -> ColoredText {
        let mut colored_text = ColoredText::new();
        let len = text.chars().count();

        for (i, ch) in text.chars().enumerate() {
            let t = if len > 1 {
                i as f32 / (len - 1) as f32
            } else {
                0.0
            };
            let color = [
                start_color[0] + t * (end_color[0] - start_color[0]),
                start_color[1] + t * (end_color[1] - start_color[1]),
                start_color[2] + t * (end_color[2] - start_color[2]),
                start_color[3] + t * (end_color[3] - start_color[3]),
            ];
            colored_text.push(ch, color);
        }

        colored_text
    }

    /// Creates animated-style text with varying intensity
    pub fn create_pulse_text(text: &str, base_color: [f32; 3], pulse_factor: f32) -> ColoredText {
        let mut colored_text = ColoredText::new();

        for (i, ch) in text.chars().enumerate() {
            let phase = (i as f32 * 0.5).sin();
            let intensity = 0.5 + 0.5 * phase * pulse_factor;
            let color = [
                base_color[0] * intensity,
                base_color[1] * intensity,
                base_color[2] * intensity,
                1.0,
            ];
            colored_text.push(ch, color);
        }

        colored_text
    }

    /// Creates a multi-line header with different colors
    pub fn create_header_demo() -> ColoredText {
        let mut colored_text = ColoredText::new();

        // Title line
        colored_text.push_str("🦀 ", [1.0, 0.5, 0.0, 1.0]); // Orange crab emoji
        colored_text.push_str("CargoTap ", [0.0, 1.0, 1.0, 1.0]); // Cyan
        colored_text.push_str("Colored Text Demo", [1.0, 1.0, 0.0, 1.0]); // Yellow
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Separator line
        colored_text.push_str("═".repeat(50).as_str(), [0.5, 0.8, 1.0, 1.0]); // Light blue
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Description
        colored_text.push_str(
            "Demonstrating per-character color support",
            [0.7, 0.7, 0.7, 1.0],
        );
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        colored_text
    }

    /// Gets the appropriate color for Rust keywords
    fn get_keyword_color(word: &str) -> [f32; 4] {
        match word {
            // Control flow keywords - Blue
            "if" | "else" | "match" | "loop" | "while" | "for" | "break" | "continue"
            | "return" => [0.4, 0.6, 1.0, 1.0],
            // Declaration keywords - Purple
            "fn" | "let" | "mut" | "const" | "static" | "struct" | "enum" | "impl" | "trait" => {
                [0.8, 0.4, 1.0, 1.0]
            }
            // Visibility and module keywords - Magenta
            "pub" | "mod" | "use" | "crate" | "super" | "self" => [1.0, 0.4, 0.8, 1.0],
            // Types - Green
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "bool" | "char" | "str" | "String" | "Vec"
            | "Option" | "Result" => [0.4, 1.0, 0.4, 1.0],
            // Functions and macros - Cyan
            "println" | "print" | "dbg" | "panic" | "todo" | "unimplemented" | "main" | "new"
            | "default" => [0.2, 1.0, 1.0, 1.0],
            // Literals - Orange
            "true" | "false" | "None" | "Some" | "Ok" | "Err" => [1.0, 0.7, 0.2, 1.0],
            // Default identifier color - Light gray
            _ => [0.9, 0.9, 0.9, 1.0],
        }
    }

    /// Creates a comprehensive demo combining multiple effects
    pub fn create_comprehensive_demo() -> ColoredText {
        let mut colored_text = ColoredText::new();

        // Add header
        colored_text.chars.extend(Self::create_header_demo().chars);

        // Add rainbow text demo
        colored_text.push_str("Rainbow Text: ", [1.0, 1.0, 1.0, 1.0]);
        colored_text
            .chars
            .extend(Self::create_rainbow_text("Hello Rainbow World!").chars);
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Add gradient text demo
        colored_text.push_str("Gradient Text: ", [1.0, 1.0, 1.0, 1.0]);
        colored_text.chars.extend(
            Self::create_gradient_text(
                "This text fades from red to blue",
                [1.0, 0.0, 0.0, 1.0], // Red
                [0.0, 0.0, 1.0, 1.0], // Blue
            )
            .chars,
        );
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Add pulse text demo
        colored_text.push_str("Pulse Text: ", [1.0, 1.0, 1.0, 1.0]);
        colored_text.chars.extend(
            Self::create_pulse_text(
                "This text has varying intensity",
                [0.0, 1.0, 0.0], // Green base
                0.8,             // Pulse strength
            )
            .chars,
        );
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Add syntax highlighted code
        colored_text.push_str("Syntax Highlighted Code:\n", [1.0, 1.0, 1.0, 1.0]);
        let sample_code = r#"fn main() {
    let greeting = "Hello, World!";
    println!("{}", greeting);

    match some_value {
        Some(x) => println!("Got: {}", x),
        None => println!("Nothing here"),
    }
}"#;
        colored_text
            .chars
            .extend(Self::create_syntax_highlighted_rust(sample_code).chars);

        colored_text
    }
}

#[derive(PartialEq)]
enum CommentType {
    None,
    Line,
    Block,
}

/// Example usage function
pub fn run_colored_text_examples() -> Result<()> {
    println!("=== Colored Text Examples ===");

    // Example 1: Simple rainbow text
    let rainbow = ColoredTextDemo::create_rainbow_text("RAINBOW");
    println!(
        "Created rainbow text with {} characters",
        rainbow.chars.len()
    );

    // Example 2: Gradient text
    let gradient = ColoredTextDemo::create_gradient_text(
        "Gradient Example",
        [1.0, 0.0, 0.0, 1.0], // Red start
        [0.0, 1.0, 0.0, 1.0], // Green end
    );
    println!(
        "Created gradient text with {} characters",
        gradient.chars.len()
    );

    // Example 3: Syntax highlighted code
    let code = r#"pub fn hello() {
        println!("Hello, colored world!");
    }"#;
    let highlighted = ColoredTextDemo::create_syntax_highlighted_rust(code);
    println!(
        "Created syntax highlighted code with {} characters",
        highlighted.chars.len()
    );

    // Example 4: Comprehensive demo
    let comprehensive = ColoredTextDemo::create_comprehensive_demo();
    println!(
        "Created comprehensive demo with {} characters",
        comprehensive.chars.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rainbow_text_creation() {
        let rainbow = ColoredTextDemo::create_rainbow_text("TEST");
        assert_eq!(rainbow.chars.len(), 4);

        // Check that each character has a different color
        for i in 0..rainbow.chars.len() - 1 {
            assert_ne!(rainbow.chars[i].color, rainbow.chars[i + 1].color);
        }
    }

    #[test]
    fn test_gradient_text_creation() {
        let gradient = ColoredTextDemo::create_gradient_text(
            "ABC",
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        );

        assert_eq!(gradient.chars.len(), 3);

        // First character should be closer to start color (red)
        assert!(gradient.chars[0].color[0] > gradient.chars[2].color[0]); // More red

        // Last character should be closer to end color (green)
        assert!(gradient.chars[2].color[1] > gradient.chars[0].color[1]); // More green
    }

    #[test]
    fn test_syntax_highlighting() {
        let code = "fn main() {}";
        let highlighted = ColoredTextDemo::create_syntax_highlighted_rust(code);

        assert_eq!(highlighted.chars.len(), code.len());

        // Check that 'fn' keyword has the expected color
        let fn_color = ColoredTextDemo::get_keyword_color("fn");
        assert_eq!(highlighted.chars[0].color, fn_color);
        assert_eq!(highlighted.chars[1].color, fn_color);
    }

    #[test]
    fn test_keyword_colors() {
        // Test different keyword categories
        let control_color = ColoredTextDemo::get_keyword_color("if");
        let declaration_color = ColoredTextDemo::get_keyword_color("fn");
        let type_color = ColoredTextDemo::get_keyword_color("i32");

        // All should be different colors
        assert_ne!(control_color, declaration_color);
        assert_ne!(declaration_color, type_color);
        assert_ne!(control_color, type_color);
    }
}
