//! Utility functions and types for Windows automation.
//!
//! This module provides fine-grained feature control:
//! - `utils_key_code`: Key code conversion utilities
//! - `utils_chars`: Character/string conversion utilities
//! - `utils_thread`: Thread utility functions (sleep, etc.)
//!
//! Use the combined `utils` feature to enable all utilities,
//! or enable individual sub-modules for minimal builds.

#[cfg(feature = "utils_key_code")]
pub mod key_code;

#[cfg(feature = "utils_chars")]
pub mod chars;

#[cfg(feature = "utils_thread")]
pub mod thread;

// Re-export utils for convenience (conditional on features)
#[cfg(feature = "utils_key_code")]
pub use key_code::key_code;

#[cfg(feature = "utils_chars")]
pub use chars::{char_array_to_string, string_to_pcwstr};

#[cfg(feature = "utils_thread")]
pub use thread::sleep_ms;
