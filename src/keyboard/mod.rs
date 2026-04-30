//! Keyboard input simulation module
//!
//! Provides keyboard input capabilities through two methods:
//! - **PostMessage**: Background input (doesn't require focus)
//! - **SendInput**: System-level input (works with all applications)
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::keyboard::SendInputKeyboard;
//!
//! let mut kb = SendInputKeyboard::new();
//! kb.click("a").unwrap();
//! ```

pub mod keyboard_message;
pub mod keyboard_input;

// Re-export main types for convenience
pub use keyboard_message::{PostMessageKeyboard, PostMessageKeyBoardError};
pub use keyboard_input::{SendInputKeyboard, SendKeyBoardInputError};

// Re-export atomic high-performance functions
pub use keyboard_message::{
    post_key_down_atomic, post_key_up_atomic, post_key_click_atomic,
};

pub use keyboard_input::{
    execute_inputs as send_execute_inputs,
    execute_single_input as send_execute_single_input,
    build_keybd_input, build_key_click_inputs, build_key_down_input, build_key_up_input,
};

// Re-export common key code utilities from utils::key_code
pub use crate::utils::key_code::{get_scan_code, is_extended_key};
