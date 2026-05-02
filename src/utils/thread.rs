//! Thread utility functions for high-frequency operations.
//!
//! This module provides optimized thread operations commonly used in automation scripts,
//! such as sleep/delay functions that are called frequently during instruction execution.

/// Sleep for the specified number of milliseconds.
///
/// This is a convenience wrapper around `std::thread::sleep` optimized for
/// high-frequency calls in script instruction execution (e.g., key click delays).
///
/// # Arguments
/// * `ms` - Duration in milliseconds to sleep
///
/// # Example
/// ```no_run
/// use win_auto_utils::utils::thread::sleep_ms;
///
/// // Sleep for 50 milliseconds
/// sleep_ms(50);
/// ```
///
/// # Performance Notes
/// - Uses standard `std::thread::sleep` internally
/// - Future optimizations may include:
///   - High-resolution timers for sub-millisecond precision
///   - Adaptive sleep strategies based on system load
///   - Performance profiling hooks
#[inline]
pub fn sleep_ms(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sleep_ms_basic() {
        let start = Instant::now();
        sleep_ms(10);
        let elapsed = start.elapsed();

        // Should sleep at least 10ms (with some tolerance for OS scheduling)
        assert!(
            elapsed.as_millis() >= 10,
            "Slept for {:?}, expected at least 10ms",
            elapsed
        );
        assert!(
            elapsed.as_millis() < 50,
            "Slept for {:?}, expected less than 50ms",
            elapsed
        );
    }

    #[test]
    fn test_sleep_ms_zero() {
        let start = Instant::now();
        sleep_ms(0);
        let elapsed = start.elapsed();

        // Zero sleep should return almost immediately
        assert!(
            elapsed.as_millis() < 10,
            "Zero sleep took too long: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_sleep_ms_accuracy() {
        let test_cases = vec![1, 5, 10, 20, 50];

        for ms in test_cases {
            let start = Instant::now();
            sleep_ms(ms);
            let elapsed = start.elapsed().as_millis();

            // Windows timer granularity is typically 1-16ms, allow generous tolerance
            let min_expected = ms as u128;
            // For small delays (< 10ms), Windows may sleep longer due to timer resolution
            let max_expected = if ms < 10 {
                (ms as f64 * 5.0) as u128 // 5x tolerance for sub-10ms delays
            } else {
                (ms as f64 * 2.0) as u128 // 2x tolerance for larger delays
            };

            assert!(
                elapsed >= min_expected && elapsed <= max_expected,
                "sleep_ms({}) resulted in {}ms (expected {}-{}ms)",
                ms,
                elapsed,
                min_expected,
                max_expected
            );
        }
    }
}
