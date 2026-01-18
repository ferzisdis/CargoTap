//! Code state management for typing game functionality
//!
//! This module provides the CodeState struct which manages the state of code
//! and tracks the user's typing progress with a cursor position.

use crate::examples::colored_text_demo::ColoredTextDemo;
use crate::text::ColoredText;

/// Represents the state of code in the typing game
#[derive(Debug, Clone)]
pub struct CodeState {
    /// The complete code text
    code: String,
    /// Current cursor position (number of characters typed)
    cursor_position: usize,
    /// Cached syntax-highlighted version of the full code
    cached_colored_text: Option<ColoredText>,
    /// Whether syntax highlighting is enabled
    syntax_highlighting_enabled: bool,
}

impl CodeState {
    /// Creates a new CodeState with the given initial code
    pub fn new(initial_code: String) -> Self {
        Self {
            code: initial_code,
            cursor_position: 0,
            cached_colored_text: None,
            syntax_highlighting_enabled: false,
        }
    }

    /// Types the next character, advancing the cursor
    /// Returns the character that was typed, or None if no more characters
    pub fn type_character(&mut self) -> Option<char> {
        if self.cursor_position < self.code.len() {
            let ch = self.code[self.cursor_position..].chars().next()?;
            self.cursor_position += ch.len_utf8();
            self.cached_colored_text = None;
            Some(ch)
        } else {
            None
        }
    }

    /// Undoes the last typed character (moves cursor back)
    /// Returns the character that was moved back, or None if nothing to undo
    pub fn backspace(&mut self) -> Option<char> {
        if self.cursor_position > 0 {
            let ch = self.code[..self.cursor_position].chars().last()?;
            self.cursor_position -= ch.len_utf8();
            self.cached_colored_text = None;
            Some(ch)
        } else {
            None
        }
    }

    /// Returns the complete code
    pub fn get_full_code(&self) -> &str {
        &self.code
    }

    /// Returns the code that has been typed so far
    pub fn get_printed_code(&self) -> &str {
        &self.code[..self.cursor_position]
    }

    /// Returns the code that still needs to be typed
    pub fn get_current_code(&self) -> &str {
        &self.code[self.cursor_position..]
    }

    /// Returns the complete code as syntax-highlighted ColoredText
    /// Uses cached version if available, otherwise generates and caches it
    pub fn get_full_code_colored(&mut self) -> &ColoredText {
        if self.cached_colored_text.is_none() {
            let colored = if self.syntax_highlighting_enabled {
                ColoredTextDemo::create_syntax_highlighted_rust(&self.code)
            } else {
                ColoredText::from_str_with_color(&self.code, [1.0, 1.0, 1.0, 1.0])
            };
            self.cached_colored_text = Some(colored);
        }
        self.cached_colored_text.as_ref().unwrap()
    }

    /// Sets whether syntax highlighting is enabled
    /// Invalidates cache if the setting changes
    pub fn set_syntax_highlighting(&mut self, enabled: bool) {
        if self.syntax_highlighting_enabled != enabled {
            self.syntax_highlighting_enabled = enabled;
            self.cached_colored_text = None;
        }
    }

    /// Returns the current cursor position (number of bytes typed)
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Returns the line number (1-based) where the cursor is located
    pub fn get_cursor_line(&self) -> usize {
        self.code[..self.cursor_position]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1
    }

    /// Returns the column number (0-based) where the cursor is located on the current line
    pub fn get_cursor_column(&self) -> usize {
        if let Some(last_newline_pos) = self.code[..self.cursor_position].rfind('\n') {
            self.cursor_position - last_newline_pos - 1
        } else {
            self.cursor_position
        }
    }

    /// Returns the total length of all code
    pub fn get_total_length(&self) -> usize {
        self.code.len()
    }

    /// Returns the progress as a percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        let total = self.code.len();
        if total == 0 {
            1.0
        } else {
            self.cursor_position as f32 / total as f32
        }
    }

    /// Checks if all code has been typed
    pub fn is_complete(&self) -> bool {
        self.cursor_position >= self.code.len()
    }

    /// Resets the state with new code
    pub fn reset(&mut self, new_code: String) {
        self.code = new_code;
        self.cursor_position = 0;
        self.cached_colored_text = None;
    }

    /// Returns the next character that should be typed (without removing it)
    pub fn peek_next_character(&self) -> Option<char> {
        self.code[self.cursor_position..].chars().next()
    }

    /// Returns a slice of the next N characters to be typed
    pub fn peek_next_chars(&self, count: usize) -> String {
        self.code[self.cursor_position..]
            .chars()
            .take(count)
            .collect()
    }

    /// Consumes all whitespace characters (space, tab, newline) until the next non-whitespace character
    /// Returns the number of whitespace characters consumed
    pub fn consume_whitespace(&mut self) -> usize {
        let mut consumed = 0;

        while self.cursor_position < self.code.len() {
            if let Some(ch) = self.code[self.cursor_position..].chars().next() {
                if ch.is_whitespace() {
                    self.cursor_position += ch.len_utf8();
                    consumed += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if consumed > 0 {
            self.cached_colored_text = None;
        }

        consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_code_state() {
        let code_state = CodeState::new("hello".to_string());
        assert_eq!(code_state.get_current_code(), "hello");
        assert_eq!(code_state.get_printed_code(), "");
    }

    #[test]
    fn test_type_character() {
        let mut code_state = CodeState::new("hello".to_string());

        assert_eq!(code_state.type_character(), Some('h'));
        assert_eq!(code_state.get_printed_code(), "h");
        assert_eq!(code_state.get_current_code(), "ello");

        assert_eq!(code_state.type_character(), Some('e'));
        assert_eq!(code_state.get_printed_code(), "he");
        assert_eq!(code_state.get_current_code(), "llo");
    }

    #[test]
    fn test_backspace() {
        let mut code_state = CodeState::new("hello".to_string());

        code_state.type_character();
        code_state.type_character();
        assert_eq!(code_state.get_printed_code(), "he");
        assert_eq!(code_state.get_current_code(), "llo");

        assert_eq!(code_state.backspace(), Some('e'));
        assert_eq!(code_state.get_printed_code(), "h");
        assert_eq!(code_state.get_current_code(), "ello");

        assert_eq!(code_state.backspace(), Some('h'));
        assert_eq!(code_state.get_printed_code(), "");
        assert_eq!(code_state.get_current_code(), "hello");
    }

    #[test]
    fn test_progress() {
        let mut code_state = CodeState::new("hello".to_string());

        assert_eq!(code_state.get_progress(), 0.0);

        code_state.type_character(); // h
        assert_eq!(code_state.get_progress(), 0.2);

        code_state.type_character(); // e
        code_state.type_character(); // l
        assert_eq!(code_state.get_progress(), 0.6);

        code_state.type_character(); // l
        code_state.type_character(); // o
        assert_eq!(code_state.get_progress(), 1.0);
        assert!(code_state.is_complete());
    }

    #[test]
    fn test_peek_methods() {
        let code_state = CodeState::new("hello".to_string());

        assert_eq!(code_state.peek_next_character(), Some('h'));
        assert_eq!(code_state.peek_next_chars(3), "hel");

        assert_eq!(code_state.get_current_code(), "hello");
        assert_eq!(code_state.get_printed_code(), "");
    }

    #[test]
    fn test_cursor_line_and_column() {
        let code = "fn main() {\n    println!(\"Hello\");\n}".to_string();
        let mut code_state = CodeState::new(code);

        assert_eq!(code_state.get_cursor_line(), 1);
        assert_eq!(code_state.get_cursor_column(), 0);

        for _ in 0..11 {
            code_state.type_character();
        }
        assert_eq!(code_state.get_cursor_line(), 1);
        assert_eq!(code_state.get_cursor_column(), 11);

        code_state.type_character();
        assert_eq!(code_state.get_cursor_line(), 2);
        assert_eq!(code_state.get_cursor_column(), 0);

        for _ in 0..4 {
            code_state.type_character();
        }
        assert_eq!(code_state.get_cursor_line(), 2);
        assert_eq!(code_state.get_cursor_column(), 4);
    }

    #[test]
    fn test_consume_whitespace() {
        let mut code_state = CodeState::new("   \t\n  hello world".to_string());

        let consumed = code_state.consume_whitespace();
        assert_eq!(consumed, 7);
        assert_eq!(code_state.get_current_code(), "hello world");
        assert_eq!(code_state.get_printed_code(), "   \t\n  ");

        assert_eq!(code_state.peek_next_character(), Some('h'));
    }

    #[test]
    fn test_consume_whitespace_no_whitespace() {
        let mut code_state = CodeState::new("hello".to_string());

        let consumed = code_state.consume_whitespace();
        assert_eq!(consumed, 0);
        assert_eq!(code_state.get_current_code(), "hello");
        assert_eq!(code_state.get_printed_code(), "");
    }

    #[test]
    fn test_consume_whitespace_after_typing() {
        let mut code_state = CodeState::new("fn   main()".to_string());

        code_state.type_character();
        code_state.type_character();
        assert_eq!(code_state.get_printed_code(), "fn");
        assert_eq!(code_state.get_current_code(), "   main()");

        let consumed = code_state.consume_whitespace();
        assert_eq!(consumed, 3);
        assert_eq!(code_state.get_printed_code(), "fn   ");
        assert_eq!(code_state.get_current_code(), "main()");
    }
}
