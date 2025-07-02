//! Code state management for typing game functionality
//!
//! This module provides the CodeState struct which manages the transition
//! of code from "to be typed" to "already typed" state.

/// Represents the state of code in the typing game
#[derive(Debug, Clone)]
pub struct CodeState {
    /// Code that still needs to be typed
    pub current_code: String,
    /// Code that has already been typed by the user
    pub printed_code: String,
}

impl CodeState {
    /// Creates a new CodeState with the given initial code
    pub fn new(initial_code: String) -> Self {
        Self {
            current_code: initial_code,
            printed_code: String::new(),
        }
    }

    /// Types the next character from current_code to printed_code
    /// Returns the character that was typed, or None if no more characters
    pub fn type_character(&mut self) -> Option<char> {
        if let Some(ch) = self.current_code.chars().next() {
            // Remove the first character from current_code
            self.current_code = self.current_code.chars().skip(1).collect();
            // Add it to printed_code
            self.printed_code.push(ch);
            Some(ch)
        } else {
            None
        }
    }

    /// Undoes the last typed character (moves it back from printed_code to current_code)
    /// Returns the character that was moved back, or None if nothing to undo
    pub fn backspace(&mut self) -> Option<char> {
        if let Some(ch) = self.printed_code.chars().last() {
            // Remove the last character from printed_code
            self.printed_code.pop();
            // Add it to the beginning of current_code
            self.current_code = format!("{}{}", ch, self.current_code);
            Some(ch)
        } else {
            None
        }
    }

    /// Returns the complete code (printed + current)
    pub fn get_full_code(&self) -> String {
        format!("{}|{}", self.printed_code, self.current_code)
    }

    /// Returns the current typing position (length of printed_code)
    pub fn get_cursor_position(&self) -> usize {
        self.printed_code.len()
    }

    /// Returns the total length of all code
    pub fn get_total_length(&self) -> usize {
        self.printed_code.len() + self.current_code.len()
    }

    /// Returns the progress as a percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        let total = self.get_total_length();
        if total == 0 {
            1.0
        } else {
            self.printed_code.len() as f32 / total as f32
        }
    }

    /// Checks if all code has been typed
    pub fn is_complete(&self) -> bool {
        self.current_code.is_empty()
    }

    /// Resets the state with new code
    pub fn reset(&mut self, new_code: String) {
        self.current_code = new_code;
        self.printed_code.clear();
    }

    /// Returns the next character that should be typed (without removing it)
    pub fn peek_next_character(&self) -> Option<char> {
        self.current_code.chars().next()
    }

    /// Returns a slice of the next N characters to be typed
    pub fn peek_next_chars(&self, count: usize) -> String {
        self.current_code.chars().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_code_state() {
        let code_state = CodeState::new("hello".to_string());
        assert_eq!(code_state.current_code, "hello");
        assert_eq!(code_state.printed_code, "");
    }

    #[test]
    fn test_type_character() {
        let mut code_state = CodeState::new("hello".to_string());

        assert_eq!(code_state.type_character(), Some('h'));
        assert_eq!(code_state.printed_code, "h");
        assert_eq!(code_state.current_code, "ello");

        assert_eq!(code_state.type_character(), Some('e'));
        assert_eq!(code_state.printed_code, "he");
        assert_eq!(code_state.current_code, "llo");
    }

    #[test]
    fn test_backspace() {
        let mut code_state = CodeState::new("hello".to_string());

        // Type some characters first
        code_state.type_character();
        code_state.type_character();
        assert_eq!(code_state.printed_code, "he");
        assert_eq!(code_state.current_code, "llo");

        // Now backspace
        assert_eq!(code_state.backspace(), Some('e'));
        assert_eq!(code_state.printed_code, "h");
        assert_eq!(code_state.current_code, "ello");

        assert_eq!(code_state.backspace(), Some('h'));
        assert_eq!(code_state.printed_code, "");
        assert_eq!(code_state.current_code, "hello");
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

        // Peeking shouldn't change state
        assert_eq!(code_state.current_code, "hello");
        assert_eq!(code_state.printed_code, "");
    }
}
