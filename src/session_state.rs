//! Session state management for timed typing sessions
//!
//! This module provides the SessionState struct which manages timed typing sessions,
//! tracking progress, statistics, and time remaining.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Represents the current state of a typing session
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    /// Session has not started yet (waiting for first input)
    NotStarted,
    /// Session is currently active
    Active,
    /// Session time has expired
    Finished,
}

/// Statistics for a completed or active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total characters typed during the session
    pub chars_typed: usize,
    /// Total time elapsed in seconds
    pub time_elapsed_secs: f64,
    /// Characters per minute
    pub chars_per_minute: f64,
    /// Words per minute (assuming 5 chars = 1 word)
    pub words_per_minute: f64,
    /// Starting position in the code
    pub start_position: usize,
    /// Ending position in the code
    pub end_position: usize,
    /// Number of errors (backspaces/corrections made)
    pub errors: usize,
    /// Accuracy percentage (0.0 to 100.0)
    pub accuracy: f64,
    /// Unix timestamp when session completed
    pub timestamp: u64,
    /// File path that was being typed
    pub file_path: String,
}

impl SessionStats {
    /// Create new session statistics
    pub fn new(
        chars_typed: usize,
        time_elapsed_secs: f64,
        start_position: usize,
        end_position: usize,
        errors: usize,
        file_path: String,
    ) -> Self {
        let chars_per_minute = if time_elapsed_secs > 0.0 {
            (chars_typed as f64 / time_elapsed_secs) * 60.0
        } else {
            0.0
        };

        let words_per_minute = chars_per_minute / 5.0; // Standard: 5 chars = 1 word

        let accuracy = if chars_typed + errors > 0 {
            (chars_typed as f64 / (chars_typed + errors) as f64) * 100.0
        } else {
            100.0
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            chars_typed,
            time_elapsed_secs,
            chars_per_minute,
            words_per_minute,
            start_position,
            end_position,
            errors,
            accuracy,
            timestamp,
            file_path,
        }
    }

    /// Format statistics for display
    pub fn format_summary(&self) -> String {
        format!(
            "Session Complete!\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Time: {:.1}s\n\
             Characters: {} (pos {} → {})\n\
             Speed: {:.0} CPM / {:.0} WPM\n\
             Accuracy: {:.1}% ({} errors)\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Press SPACE to start new session",
            self.time_elapsed_secs,
            self.chars_typed,
            self.start_position,
            self.end_position,
            self.chars_per_minute,
            self.words_per_minute,
            self.accuracy,
            self.errors
        )
    }
}

/// Manages a timed typing session
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Current status of the session
    status: SessionStatus,
    /// When the session started
    start_time: Option<Instant>,
    /// Duration of the session in seconds
    duration_secs: f64,
    /// Position where the session started
    start_position: usize,
    /// Number of characters typed in this session
    chars_typed_in_session: usize,
    /// Number of errors in this session (backspaces)
    errors_in_session: usize,
    /// Statistics from the last completed session
    last_session_stats: Option<SessionStats>,
    /// File path being typed
    file_path: String,
}

impl SessionState {
    /// Create a new session state with the given duration in minutes
    pub fn new(duration_minutes: f64) -> Self {
        Self {
            status: SessionStatus::NotStarted,
            start_time: None,
            duration_secs: duration_minutes * 60.0,
            start_position: 0,
            chars_typed_in_session: 0,
            errors_in_session: 0,
            last_session_stats: None,
            file_path: String::new(),
        }
    }

    /// Start the session (called when first character is typed)
    pub fn start(&mut self, current_position: usize, file_path: String) {
        if self.status == SessionStatus::NotStarted {
            self.status = SessionStatus::Active;
            self.start_time = Some(Instant::now());
            self.start_position = current_position;
            self.chars_typed_in_session = 0;
            self.errors_in_session = 0;
            self.file_path = file_path;
            log::info!(
                "🎯 Session started! Duration: {:.1} minutes (starting at position {})",
                self.duration_secs / 60.0,
                current_position
            );
        }
    }

    /// Record a character being typed
    pub fn record_char_typed(&mut self) {
        if self.status == SessionStatus::Active {
            self.chars_typed_in_session += 1;
        }
    }

    /// Record a backspace (decrements typed count and increments errors)
    pub fn record_backspace(&mut self) {
        if self.status == SessionStatus::Active {
            if self.chars_typed_in_session > 0 {
                self.chars_typed_in_session -= 1;
            }
            self.errors_in_session += 1;
        }
    }

    /// Update the session state and check if time has expired
    /// Returns true if the session just finished
    pub fn update(&mut self, current_position: usize) -> bool {
        if self.status != SessionStatus::Active {
            return false;
        }

        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64();

            if elapsed >= self.duration_secs {
                // Session finished!
                self.status = SessionStatus::Finished;
                self.last_session_stats = Some(SessionStats::new(
                    self.chars_typed_in_session,
                    elapsed,
                    self.start_position,
                    current_position,
                    self.errors_in_session,
                    self.file_path.clone(),
                ));

                log::info!("⏰ Session time expired!");
                if let Some(stats) = &self.last_session_stats {
                    log::info!("{}", stats.format_summary());
                }

                return true;
            }
        }

        false
    }

    /// Get the current status of the session
    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    /// Get time remaining in seconds
    pub fn time_remaining(&self) -> f64 {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f64();
            (self.duration_secs - elapsed).max(0.0)
        } else {
            self.duration_secs
        }
    }

    /// Get time elapsed in seconds
    pub fn time_elapsed(&self) -> f64 {
        if let Some(start) = self.start_time {
            start.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get formatted time remaining as MM:SS
    pub fn format_time_remaining(&self) -> String {
        let remaining = self.time_remaining();
        let minutes = (remaining / 60.0).floor() as u32;
        let seconds = (remaining % 60.0).floor() as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Get statistics from the last completed session
    pub fn last_stats(&self) -> Option<&SessionStats> {
        self.last_session_stats.as_ref()
    }

    /// Start a new session, continuing from the current position
    pub fn start_new_session(&mut self, current_position: usize, file_path: String) {
        self.status = SessionStatus::NotStarted;
        self.start_time = None;
        self.start_position = current_position;
        self.chars_typed_in_session = 0;
        self.errors_in_session = 0;
        self.file_path = file_path;
        // Note: last_session_stats is kept so it can be displayed until next session completes
        log::info!(
            "🔄 Ready for new session (will start at position {})",
            current_position
        );
    }

    /// Reset the session completely
    pub fn reset(&mut self) {
        self.status = SessionStatus::NotStarted;
        self.start_time = None;
        self.start_position = 0;
        self.chars_typed_in_session = 0;
        self.errors_in_session = 0;
        self.last_session_stats = None;
        self.file_path = String::new();
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    /// Check if the session is finished
    pub fn is_finished(&self) -> bool {
        self.status == SessionStatus::Finished
    }

    /// Get current session statistics (even if not finished)
    pub fn current_stats(&self, current_position: usize) -> SessionStats {
        SessionStats::new(
            self.chars_typed_in_session,
            self.time_elapsed(),
            self.start_position,
            current_position,
            self.errors_in_session,
            self.file_path.clone(),
        )
    }

    /// Get the session duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    /// Set a new duration for future sessions (in minutes)
    pub fn set_duration_minutes(&mut self, minutes: f64) {
        self.duration_secs = minutes * 60.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_session() {
        let session = SessionState::new(3.0);
        assert_eq!(session.status(), &SessionStatus::NotStarted);
        assert_eq!(session.time_remaining(), 180.0);
        assert!(!session.is_active());
        assert!(!session.is_finished());
    }

    #[test]
    fn test_start_session() {
        let mut session = SessionState::new(1.0);
        session.start(0, "test.rs".to_string());
        assert_eq!(session.status(), &SessionStatus::Active);
        assert!(session.is_active());
    }

    #[test]
    fn test_record_chars() {
        let mut session = SessionState::new(1.0);
        session.start(0, "test.rs".to_string());
        session.record_char_typed();
        session.record_char_typed();
        session.record_char_typed();

        let stats = session.current_stats(3);
        assert_eq!(stats.chars_typed, 3);
    }

    #[test]
    fn test_backspace() {
        let mut session = SessionState::new(1.0);
        session.start(0, "test.rs".to_string());
        session.record_char_typed();
        session.record_char_typed();
        session.record_backspace();

        let stats = session.current_stats(1);
        assert_eq!(stats.chars_typed, 1);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_time_remaining() {
        let mut session = SessionState::new(1.0 / 60.0); // 1 second
        session.start(0, "test.rs".to_string());
        thread::sleep(Duration::from_millis(100));
        let remaining = session.time_remaining();
        assert!(remaining < 1.0 && remaining > 0.8);
    }

    #[test]
    fn test_format_time() {
        let session = SessionState::new(3.5);
        assert_eq!(session.format_time_remaining(), "03:30");
    }

    #[test]
    fn test_session_stats() {
        let stats = SessionStats::new(120, 60.0, 0, 120, 5, "test.rs".to_string());
        assert_eq!(stats.chars_typed, 120);
        assert_eq!(stats.chars_per_minute, 120.0);
        assert_eq!(stats.words_per_minute, 24.0);
        assert_eq!(stats.errors, 5);
        assert!((stats.accuracy - 96.0).abs() < 0.1);
    }

    #[test]
    fn test_new_session_continuation() {
        let mut session = SessionState::new(1.0);
        session.start(0, "test.rs".to_string());
        session.record_char_typed();
        session.record_char_typed();

        // Finish session
        session.status = SessionStatus::Finished;
        session.last_session_stats =
            Some(SessionStats::new(2, 60.0, 0, 2, 0, "test.rs".to_string()));

        // Start new session
        session.start_new_session(2, "test.rs".to_string());
        assert_eq!(session.status(), &SessionStatus::NotStarted);
        assert!(session.last_stats().is_some());
    }
}
