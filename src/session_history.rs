//! Session history storage and analysis
//!
//! This module provides functionality to persist session statistics across
//! application runs and analyze performance trends over time.

use crate::session_state::SessionStats;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Represents a collection of session statistics with analysis capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    /// List of all completed sessions, ordered by timestamp
    sessions: Vec<SessionStats>,
    /// Path to the storage file
    #[serde(skip)]
    storage_path: PathBuf,
}

/// Summary statistics across multiple sessions
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Total characters typed across all sessions
    pub total_chars: usize,
    /// Total time spent typing (in seconds)
    pub total_time: f64,
    /// Average CPM across all sessions
    pub avg_cpm: f64,
    /// Average WPM across all sessions
    pub avg_wpm: f64,
    /// Average accuracy across all sessions
    pub avg_accuracy: f64,
    /// Best CPM achieved
    pub best_cpm: f64,
    /// Best WPM achieved
    pub best_wpm: f64,
    /// Best accuracy achieved
    pub best_accuracy: f64,
    /// Total errors made
    pub total_errors: usize,
}

impl SessionHistory {
    /// Creates a new SessionHistory with the given storage file path
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            sessions: Vec::new(),
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    /// Creates a SessionHistory with the default storage path
    pub fn default() -> Self {
        let storage_path = if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("cargo_tap").join("session_history.json")
        } else {
            PathBuf::from("cargo_tap_session_history.json")
        };
        Self::new(storage_path)
    }

    /// Loads session history from disk
    pub fn load(&mut self) -> io::Result<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.storage_path)?;
        let loaded: Vec<SessionStats> = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.sessions = loaded;
        Ok(())
    }

    /// Saves session history to disk
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.sessions)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.storage_path, json)?;
        Ok(())
    }

    /// Adds a new session to the history
    pub fn add_session(&mut self, stats: SessionStats) {
        self.sessions.push(stats);
    }

    /// Gets all sessions
    pub fn get_all_sessions(&self) -> &[SessionStats] {
        &self.sessions
    }

    /// Gets the most recent N sessions
    pub fn get_recent_sessions(&self, count: usize) -> Vec<&SessionStats> {
        self.sessions.iter().rev().take(count).collect()
    }

    /// Gets sessions for a specific file
    pub fn get_sessions_for_file(&self, file_path: &str) -> Vec<&SessionStats> {
        self.sessions
            .iter()
            .filter(|s| s.file_path == file_path)
            .collect()
    }

    /// Gets the session with the best CPM
    pub fn get_best_cpm_session(&self) -> Option<&SessionStats> {
        self.sessions
            .iter()
            .max_by(|a, b| a.chars_per_minute.partial_cmp(&b.chars_per_minute).unwrap())
    }

    /// Gets the session with the best WPM
    pub fn get_best_wpm_session(&self) -> Option<&SessionStats> {
        self.sessions
            .iter()
            .max_by(|a, b| a.words_per_minute.partial_cmp(&b.words_per_minute).unwrap())
    }

    /// Gets the session with the best accuracy
    pub fn get_best_accuracy_session(&self) -> Option<&SessionStats> {
        self.sessions
            .iter()
            .max_by(|a, b| a.accuracy.partial_cmp(&b.accuracy).unwrap())
    }

    /// Computes summary statistics for all sessions
    pub fn get_summary(&self) -> SessionSummary {
        if self.sessions.is_empty() {
            return SessionSummary {
                total_sessions: 0,
                total_chars: 0,
                total_time: 0.0,
                avg_cpm: 0.0,
                avg_wpm: 0.0,
                avg_accuracy: 0.0,
                best_cpm: 0.0,
                best_wpm: 0.0,
                best_accuracy: 0.0,
                total_errors: 0,
            };
        }

        let total_sessions = self.sessions.len();
        let total_chars = self.sessions.iter().map(|s| s.chars_typed).sum();
        let total_time = self.sessions.iter().map(|s| s.time_elapsed_secs).sum();
        let total_errors = self.sessions.iter().map(|s| s.errors).sum();

        let avg_cpm = self
            .sessions
            .iter()
            .map(|s| s.chars_per_minute)
            .sum::<f64>()
            / total_sessions as f64;
        let avg_wpm = self
            .sessions
            .iter()
            .map(|s| s.words_per_minute)
            .sum::<f64>()
            / total_sessions as f64;
        let avg_accuracy =
            self.sessions.iter().map(|s| s.accuracy).sum::<f64>() / total_sessions as f64;

        let best_cpm = self
            .get_best_cpm_session()
            .map(|s| s.chars_per_minute)
            .unwrap_or(0.0);
        let best_wpm = self
            .get_best_wpm_session()
            .map(|s| s.words_per_minute)
            .unwrap_or(0.0);
        let best_accuracy = self
            .get_best_accuracy_session()
            .map(|s| s.accuracy)
            .unwrap_or(0.0);

        SessionSummary {
            total_sessions,
            total_chars,
            total_time,
            avg_cpm,
            avg_wpm,
            avg_accuracy,
            best_cpm,
            best_wpm,
            best_accuracy,
            total_errors,
        }
    }

    /// Computes summary statistics for the last N sessions
    pub fn get_recent_summary(&self, count: usize) -> SessionSummary {
        if self.sessions.is_empty() {
            return SessionSummary {
                total_sessions: 0,
                total_chars: 0,
                total_time: 0.0,
                avg_cpm: 0.0,
                avg_wpm: 0.0,
                avg_accuracy: 0.0,
                best_cpm: 0.0,
                best_wpm: 0.0,
                best_accuracy: 0.0,
                total_errors: 0,
            };
        }

        let recent_sessions: Vec<_> = self.get_recent_sessions(count);
        let total_sessions = recent_sessions.len();

        if total_sessions == 0 {
            return SessionSummary {
                total_sessions: 0,
                total_chars: 0,
                total_time: 0.0,
                avg_cpm: 0.0,
                avg_wpm: 0.0,
                avg_accuracy: 0.0,
                best_cpm: 0.0,
                best_wpm: 0.0,
                best_accuracy: 0.0,
                total_errors: 0,
            };
        }

        let total_chars = recent_sessions.iter().map(|s| s.chars_typed).sum();
        let total_time = recent_sessions.iter().map(|s| s.time_elapsed_secs).sum();
        let total_errors = recent_sessions.iter().map(|s| s.errors).sum();

        let avg_cpm = recent_sessions
            .iter()
            .map(|s| s.chars_per_minute)
            .sum::<f64>()
            / total_sessions as f64;
        let avg_wpm = recent_sessions
            .iter()
            .map(|s| s.words_per_minute)
            .sum::<f64>()
            / total_sessions as f64;
        let avg_accuracy =
            recent_sessions.iter().map(|s| s.accuracy).sum::<f64>() / total_sessions as f64;

        let best_cpm = recent_sessions
            .iter()
            .map(|s| s.chars_per_minute)
            .fold(0.0_f64, |acc, cpm| acc.max(cpm));
        let best_wpm = recent_sessions
            .iter()
            .map(|s| s.words_per_minute)
            .fold(0.0_f64, |acc, wpm| acc.max(wpm));
        let best_accuracy = recent_sessions
            .iter()
            .map(|s| s.accuracy)
            .fold(0.0_f64, |acc, acc_val| acc.max(acc_val));

        SessionSummary {
            total_sessions,
            total_chars,
            total_time,
            avg_cpm,
            avg_wpm,
            avg_accuracy,
            best_cpm,
            best_wpm,
            best_accuracy,
            total_errors,
        }
    }

    /// Clears all session history
    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    /// Returns the number of sessions stored
    pub fn count(&self) -> usize {
        self.sessions.len()
    }

    /// Checks if there is any improvement trend in recent sessions
    /// Returns (has_improved, improvement_percentage)
    pub fn analyze_improvement(&self, recent_count: usize) -> (bool, f64) {
        if self.sessions.len() < recent_count * 2 {
            return (false, 0.0);
        }

        let recent = self.get_recent_summary(recent_count);
        let previous_start = self.sessions.len().saturating_sub(recent_count * 2);
        let previous_end = self.sessions.len().saturating_sub(recent_count);

        let previous_sessions: Vec<_> =
            self.sessions[previous_start..previous_end].iter().collect();
        let previous_avg_cpm = if !previous_sessions.is_empty() {
            previous_sessions
                .iter()
                .map(|s| s.chars_per_minute)
                .sum::<f64>()
                / previous_sessions.len() as f64
        } else {
            0.0
        };

        if previous_avg_cpm > 0.0 {
            let improvement = ((recent.avg_cpm - previous_avg_cpm) / previous_avg_cpm) * 100.0;
            (improvement > 0.0, improvement)
        } else {
            (false, 0.0)
        }
    }

    /// Formats a detailed statistics report
    pub fn format_statistics_report(&self) -> String {
        if self.sessions.is_empty() {
            return "No sessions recorded yet. Start typing to track your progress!".to_string();
        }

        let summary = self.get_summary();
        let recent_summary = self.get_recent_summary(5);
        let (improved, improvement) = self.analyze_improvement(5);

        let mut report = String::new();
        report.push_str("╔═══════════════════════════════════════════════╗\n");
        report.push_str("║          SESSION STATISTICS REPORT            ║\n");
        report.push_str("╚═══════════════════════════════════════════════╝\n\n");

        report.push_str(&format!(
            "📊 ALL-TIME STATS ({} sessions)\n",
            summary.total_sessions
        ));
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        report.push_str(&format!("  Total Characters: {}\n", summary.total_chars));
        report.push_str(&format!(
            "  Total Time: {:.1} minutes\n",
            summary.total_time / 60.0
        ));
        report.push_str(&format!(
            "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
            summary.avg_cpm, summary.avg_wpm
        ));
        report.push_str(&format!("  Avg Accuracy: {:.1}%\n", summary.avg_accuracy));
        report.push_str(&format!("  Total Errors: {}\n\n", summary.total_errors));

        report.push_str("🏆 BEST PERFORMANCES\n");
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        report.push_str(&format!(
            "  Best Speed: {:.0} CPM / {:.0} WPM\n",
            summary.best_cpm, summary.best_wpm
        ));
        report.push_str(&format!(
            "  Best Accuracy: {:.1}%\n\n",
            summary.best_accuracy
        ));

        if recent_summary.total_sessions > 0 {
            report.push_str(&format!(
                "📈 RECENT PERFORMANCE (last {} sessions)\n",
                recent_summary.total_sessions
            ));
            report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            report.push_str(&format!(
                "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
                recent_summary.avg_cpm, recent_summary.avg_wpm
            ));
            report.push_str(&format!(
                "  Avg Accuracy: {:.1}%\n",
                recent_summary.avg_accuracy
            ));

            if improved {
                report.push_str(&format!("  📊 Improvement: +{:.1}% 🎉\n", improvement));
            } else if improvement < 0.0 {
                report.push_str(&format!("  📊 Change: {:.1}%\n", improvement));
            }
            report.push_str("\n");
        }

        report.push_str("📝 RECENT SESSIONS\n");
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        for (i, session) in self.get_recent_sessions(5).iter().enumerate() {
            report.push_str(&format!(
                "  {}. {:.0} CPM / {:.0} WPM | {:.1}% accuracy | {} chars\n",
                i + 1,
                session.chars_per_minute,
                session.words_per_minute,
                session.accuracy,
                session.chars_typed
            ));
        }

        report.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        report.push_str("Press ESC to return to typing\n");

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stats(cpm: f64, accuracy: f64, chars: usize, errors: usize) -> SessionStats {
        let wpm = cpm / 5.0;
        let time = (chars as f64 / cpm) * 60.0;
        SessionStats {
            chars_typed: chars,
            time_elapsed_secs: time,
            chars_per_minute: cpm,
            words_per_minute: wpm,
            start_position: 0,
            end_position: chars,
            errors,
            accuracy,
            timestamp: 0,
            file_path: "test.rs".to_string(),
        }
    }

    #[test]
    fn test_new_history() {
        let history = SessionHistory::new("test_history.json");
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_add_session() {
        let mut history = SessionHistory::new("test_history.json");
        let stats = create_test_stats(300.0, 95.0, 150, 5);
        history.add_session(stats);
        assert_eq!(history.count(), 1);
    }

    #[test]
    fn test_get_summary() {
        let mut history = SessionHistory::new("test_history.json");
        history.add_session(create_test_stats(300.0, 95.0, 150, 5));
        history.add_session(create_test_stats(400.0, 90.0, 200, 10));

        let summary = history.get_summary();
        assert_eq!(summary.total_sessions, 2);
        assert_eq!(summary.total_chars, 350);
        assert_eq!(summary.best_cpm, 400.0);
        assert_eq!(summary.best_accuracy, 95.0);
    }

    #[test]
    fn test_get_recent_sessions() {
        let mut history = SessionHistory::new("test_history.json");
        history.add_session(create_test_stats(300.0, 95.0, 150, 5));
        history.add_session(create_test_stats(400.0, 90.0, 200, 10));
        history.add_session(create_test_stats(350.0, 92.0, 175, 7));

        let recent = history.get_recent_sessions(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].chars_per_minute, 350.0);
        assert_eq!(recent[1].chars_per_minute, 400.0);
    }

    #[test]
    fn test_best_sessions() {
        let mut history = SessionHistory::new("test_history.json");
        history.add_session(create_test_stats(300.0, 95.0, 150, 5));
        history.add_session(create_test_stats(400.0, 90.0, 200, 10));
        history.add_session(create_test_stats(350.0, 98.0, 175, 2));

        let best_cpm = history.get_best_cpm_session().unwrap();
        assert_eq!(best_cpm.chars_per_minute, 400.0);

        let best_acc = history.get_best_accuracy_session().unwrap();
        assert_eq!(best_acc.accuracy, 98.0);
    }

    #[test]
    fn test_analyze_improvement() {
        let mut history = SessionHistory::new("test_history.json");

        for _ in 0..5 {
            history.add_session(create_test_stats(300.0, 95.0, 150, 5));
        }

        for _ in 0..5 {
            history.add_session(create_test_stats(330.0, 96.0, 165, 4));
        }

        let (improved, improvement) = history.analyze_improvement(5);
        assert!(improved);
        assert!(improvement > 9.0 && improvement < 11.0);
    }
}
