//! Profiling utilities for performance analysis
//!
//! This module provides simple utilities for measuring and tracking performance
//! of different parts of the application.

use std::time::Instant;

/// A simple timer for measuring execution time of code blocks
pub struct ScopedTimer {
    name: &'static str,
    start: Instant,
    threshold_ms: f64,
}

impl ScopedTimer {
    /// Create a new scoped timer that will log when dropped
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold_ms: 0.0,
        }
    }

    /// Create a timer that only logs if execution exceeds the threshold
    pub fn with_threshold(name: &'static str, threshold_ms: f64) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold_ms,
        }
    }

    /// Get the elapsed time without dropping the timer
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed_ms = self.elapsed_ms();
        if elapsed_ms >= self.threshold_ms {
            log::debug!("[TIMING] {} took {:.3}ms", self.name, elapsed_ms);
        }
    }
}

/// Macro for easy scoped timing
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _timer = $crate::profiling::ScopedTimer::new($name);
    };
    ($name:expr, $threshold_ms:expr) => {
        let _timer = $crate::profiling::ScopedTimer::with_threshold($name, $threshold_ms);
    };
}

/// A performance counter that tracks min/max/average timing
#[derive(Debug, Clone)]
pub struct PerfCounter {
    name: String,
    count: u64,
    total_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

impl Default for PerfCounter {
    fn default() -> Self {
        Self {
            name: String::new(),
            count: 0,
            total_ms: 0.0,
            min_ms: f64::MAX,
            max_ms: 0.0,
        }
    }
}

impl PerfCounter {
    /// Create a new performance counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Record a timing sample
    pub fn record(&mut self, duration_ms: f64) {
        self.count += 1;
        self.total_ms += duration_ms;
        self.min_ms = self.min_ms.min(duration_ms);
        self.max_ms = self.max_ms.max(duration_ms);
    }

    /// Get the average time in milliseconds
    pub fn avg_ms(&self) -> f64 {
        if self.count > 0 {
            self.total_ms / self.count as f64
        } else {
            0.0
        }
    }

    /// Get the minimum recorded time
    pub fn min_ms(&self) -> f64 {
        if self.count > 0 { self.min_ms } else { 0.0 }
    }

    /// Get the maximum recorded time
    pub fn max_ms(&self) -> f64 {
        self.max_ms
    }

    /// Get the number of samples
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.count = 0;
        self.total_ms = 0.0;
        self.min_ms = f64::MAX;
        self.max_ms = 0.0;
    }

    /// Print a summary report
    pub fn report(&self) {
        if self.count > 0 {
            log::info!(
                "[PERF] {} - avg: {:.3}ms, min: {:.3}ms, max: {:.3}ms, samples: {}",
                self.name,
                self.avg_ms(),
                self.min_ms(),
                self.max_ms(),
                self.count
            );
        }
    }
}

/// A collection of performance counters
#[derive(Debug, Default)]
pub struct PerfStats {
    pub frame_time: PerfCounter,
    pub key_processing: PerfCounter,
    pub text_update: PerfCounter,
    pub ui_generation: PerfCounter,
    pub render: PerfCounter,
}

impl PerfStats {
    /// Create a new performance statistics collection
    pub fn new() -> Self {
        Self {
            frame_time: PerfCounter::new("Frame Time"),
            key_processing: PerfCounter::new("Key Processing"),
            text_update: PerfCounter::new("Text Update"),
            ui_generation: PerfCounter::new("UI Generation"),
            render: PerfCounter::new("Render"),
        }
    }

    /// Print a full performance report
    pub fn report_all(&self) {
        log::info!("=== Performance Report ===");
        self.frame_time.report();
        self.key_processing.report();
        self.text_update.report();
        self.ui_generation.report();
        self.render.report();
    }

    /// Reset all counters
    pub fn reset_all(&mut self) {
        self.frame_time.reset();
        self.key_processing.reset();
        self.text_update.reset();
        self.ui_generation.reset();
        self.render.reset();
    }
}

/// Helper function to measure execution time of a closure
pub fn measure<F, R>(f: F) -> (R, f64)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    (result, elapsed_ms)
}

/// Helper function to measure and log execution time
pub fn measure_and_log<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let (result, elapsed_ms) = measure(f);
    log::debug!("[TIMING] {} took {:.3}ms", name, elapsed_ms);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_perf_counter() {
        let mut counter = PerfCounter::new("test");
        assert_eq!(counter.count(), 0);

        counter.record(10.0);
        counter.record(20.0);
        counter.record(15.0);

        assert_eq!(counter.count(), 3);
        assert!((counter.avg_ms() - 15.0).abs() < 0.001);
        assert!((counter.min_ms() - 10.0).abs() < 0.001);
        assert!((counter.max_ms() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_measure() {
        let (result, elapsed_ms) = measure(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(elapsed_ms >= 10.0);
        assert!(elapsed_ms < 50.0); // Should be close to 10ms
    }

    #[test]
    fn test_scoped_timer() {
        let timer = ScopedTimer::new("test");
        thread::sleep(Duration::from_millis(5));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 5.0);
    }
}
