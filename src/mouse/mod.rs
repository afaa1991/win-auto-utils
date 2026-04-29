//! Mouse input simulation module
//!
//! Provides mouse control capabilities through two methods:
//! - **PostMessage**: Background input to specific window (requires coordinates)
//! - **SendInput**: System-level input (works with all applications)
//!
//! # Features
//! - Click (left, right, middle, double-click)
//! - Press/Release (button down/up)
//! - Move (absolute and relative)
//! - Scroll (up/down)
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::mouse::SendInputMouse;
//!
//! let mut mouse = SendInputMouse::new();
//! mouse.click_left().unwrap();
//! ```

pub mod post_message;
pub mod send_input;

// Re-export main types for convenience
pub use post_message::{PostMessageMouse, PostMessageError};
pub use send_input::{SendInputMouse, SendInputError};

// Re-export atomic high-performance functions
pub use post_message::{
    post_click_left_atomic, post_click_right_atomic, post_click_middle_atomic,
    post_press_left_atomic, post_release_left_atomic,
    post_move_atomic, post_scroll_up_atomic, post_scroll_down_atomic,
};

pub use send_input::{
    execute_inputs as mouse_execute_inputs,
    execute_single_input as mouse_execute_single_input,
    build_mouse_input,
    build_click_left, build_click_left_at,
    build_click_right, build_click_right_at,
    build_click_middle, build_click_middle_at,
    build_press_left, build_release_left,
    build_move, build_move_relative,
    build_scroll_up, build_scroll_down,
};
