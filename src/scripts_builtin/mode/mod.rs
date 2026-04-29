//! Script mode configuration instructions
//!
//! Provides the `mode` instruction for configuring module-specific execution modes.
//! This implements a flexible key-value based configuration system where each module
//! can define its own mode keys and values.

//!
//! # Usage Examples
//!
//! ```text
//! # Set keyboard/mouse input mode to background
//! mode input_mode post
//! key A                  # Uses PostMessage
//! click 100 200          # Uses PostMessage
//!
//! # Switch back to foreground
//! mode input_mode send
//! key B                  # Uses SendInput
//!
//! # Can still override per-instruction
//! mode input_mode post
//! key C send             # Explicitly use SendInput
//! ```
//!
//! # Future Extensions
//! ```text
//! # Debug mode for verbose logging
//! mode debug_level verbose
//!
//! # Performance optimization mode
//! mode performance_level max
//!
//! # Compatibility mode for legacy systems
//! mode compatibility_mode legacy
//! ```

pub mod mode_cmd;

// Re-export handlers for convenience
pub use mode_cmd::ModeHandler;
