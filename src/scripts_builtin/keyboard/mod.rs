//! Keyboard automation instructions
//!
//! Provides three keyboard instructions for automating key presses:
//! - `key`: Complete click (press + release)
//! - `key_down`: Press and hold a key
//! - `key_up`: Release a held key
//!
//! # Execution Modes
//!
//! Each keyboard instruction supports two execution modes:
//!
//! ## `send` - Foreground Mode (Default)
//!
//! Simulates input to the active window using SendInput API.
//! - No target window configuration needed
//! - Works with the currently focused application
//!
//! ## `post` - Background Mode
//!
//! Sends messages directly to a specific window using PostMessage API.
//! - Requires `target_hwnd` to be set in VM state before execution
//! - Works even when window is not in focus
//!
//! # Instructions
//!
//! ## `key` - Click a key
//!
//! Performs a complete key click (press followed by release).
//!
//! **Syntax:** `key <key_name> [delay_ms] [mode]`
//!
//! **Examples:**
//! ```text
//! key A                    # Click 'A' in foreground (default)
//! key ENTER 50            # Click ENTER with 50ms delay in foreground
//! key A post               # Click 'A' in background
//! key A 50 post           # Click 'A' with 50ms delay in background
//! ```
//!
//! ---
//!
//! ## `key_down` - Press and hold a key
//!
//! Presses a key without releasing it. Use with `key_up` to create key combinations.
//!
//! **Syntax:** `key_down <key_name> [mode]`
//!
//! **Examples:**
//! ```text
//! key_down SHIFT           # Press SHIFT in foreground (default)
//! key_down CONTROL post    # Press CONTROL in background
//! ```
//!
//! ---
//!
//! ## `key_up` - Release a held key
//!
//! Releases a key that was previously pressed with `key_down`.
//!
//! **Syntax:** `key_up <key_name> [mode]`
//!
//! **Examples:**
//! ```text
//! key_up SHIFT             # Release SHIFT in foreground (default)
//! key_up CONTROL post      # Release CONTROL in background
//! ```
//!
//! # Key Combinations
//!
//! ```text
//! # Ctrl+C (Copy)
//! key_down CONTROL
//! key C
//! key_up CONTROL
//! ```
//!
//! # Background Mode Setup
//!
//! Before using `post` mode, you must set the target window handle in your Rust code:
//!
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};
//! use windows::Win32::Foundation::HWND;
//!
//! let config = ScriptConfig::default();
//! let engine = ScriptEngine::with_builtin(config);
//!
//! let hwnd = HWND(std::ptr::null_mut());
//!
//! engine.compile_and_execute_with_context("key A post", |ctx| {
//!     ctx.set_persistent_state("target_hwnd", hwnd);
//! }).unwrap();
//! ```

pub mod key_click;
pub mod key_down;
pub mod key_up;

pub use key_click::KeyClickHandler;
pub use key_down::KeyDownHandler;
pub use key_up::KeyUpHandler;

// ============================================================================
// Common Types
// ============================================================================

use crate::keyboard::keyboard_input;
use crate::script_engine::instruction::ScriptError;
use crate::utils::key_code;

#[derive(Clone)]
pub struct KeyClickParams {
    pub key_down_input: windows::Win32::UI::Input::KeyboardAndMouse::INPUT,
    pub key_up_input: windows::Win32::UI::Input::KeyboardAndMouse::INPUT,
    pub delay_ms: u32,
}

#[derive(Clone)]
pub struct KeyDownParams {
    pub input: windows::Win32::UI::Input::KeyboardAndMouse::INPUT,
}

#[derive(Clone)]
pub struct KeyUpParams {
    pub input: windows::Win32::UI::Input::KeyboardAndMouse::INPUT,
}

#[derive(Clone)]
pub struct KeyPostClickParams {
    pub vk_code: u8,
    pub scan_code: u16,
    pub delay_ms: u32,
}

#[derive(Clone)]
pub struct KeyPostDownUpParams {
    pub vk_code: u8,
    pub scan_code: u16,
}

#[derive(Clone)]
pub enum KeyParams {
    SendClick(KeyClickParams),
    SendDown(KeyDownParams),
    SendUp(KeyUpParams),
    PostClick(KeyPostClickParams),
    PostDown(KeyPostDownUpParams),
    PostUp(KeyPostDownUpParams),
}

pub fn parse_key_click_args(args: &[&str]) -> Result<KeyParams, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ParseError(
            "Missing key name. Usage: key <name> [delay_ms] [mode]".into(),
        ));
    }

    let vk = key_code(args[0]);
    if vk == 0 {
        return Err(ScriptError::ParseError(format!(
            "Unknown key name: '{}'",
            args[0]
        )));
    }

    let scan_code = crate::get_scan_code(vk);
    let extended = crate::utils::key_code::is_extended_key(vk as u8);

    let mut delay_ms = 0u32;
    let mut mode_send = true;

    if args.len() > 1 {
        match args[1].parse::<u32>() {
            Ok(delay) => {
                delay_ms = delay;
                if args.len() > 2 {
                    mode_send = match args[2] {
                        "post" => false,
                        "send" => true,
                        other => {
                            return Err(ScriptError::ParseError(format!(
                                "Position 3 must be 'send' or 'post', got: '{}'",
                                other
                            )));
                        }
                    };
                }
            }
            Err(_) => {
                mode_send = match args[1] {
                    "post" => false,
                    "send" => true,
                    other => {
                        return Err(ScriptError::ParseError(format!(
                            "Position 2 must be a number (delay) or 'send'/'post' (mode), got: '{}'",
                            other
                        )));
                    }
                };
            }
        }
    }

    if mode_send {
        let key_down_input = keyboard_input::build_key_down_input(vk as u8, extended);
        let key_up_input = keyboard_input::build_key_up_input(vk as u8, extended);
        Ok(KeyParams::SendClick(KeyClickParams {
            key_down_input,
            key_up_input,
            delay_ms,
        }))
    } else {
        Ok(KeyParams::PostClick(KeyPostClickParams {
            vk_code: vk as u8,
            scan_code,
            delay_ms,
        }))
    }
}

pub fn parse_key_down_args(args: &[&str]) -> Result<KeyParams, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ParseError(
            "Missing key name. Usage: key_down <name> [mode]".into(),
        ));
    }

    let vk = key_code(args[0]);
    if vk == 0 {
        return Err(ScriptError::ParseError(format!(
            "Unknown key name: '{}'",
            args[0]
        )));
    }

    let scan_code = crate::get_scan_code(vk);
    let extended = crate::utils::key_code::is_extended_key(vk as u8);

    let mut mode_send = true;

    if args.len() > 1 {
        mode_send = match args[1] {
            "post" => false,
            "send" => true,
            other => {
                return Err(ScriptError::ParseError(format!(
                    "Position 2 must be 'send' or 'post', got: '{}'",
                    other
                )));
            }
        };
    }

    if mode_send {
        let input = keyboard_input::build_key_down_input(vk as u8, extended);
        Ok(KeyParams::SendDown(KeyDownParams { input }))
    } else {
        Ok(KeyParams::PostDown(KeyPostDownUpParams {
            vk_code: vk as u8,
            scan_code,
        }))
    }
}

pub fn parse_key_up_args(args: &[&str]) -> Result<KeyParams, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ParseError(
            "Missing key name. Usage: key_up <name> [mode]".into(),
        ));
    }

    let vk = key_code(args[0]);
    if vk == 0 {
        return Err(ScriptError::ParseError(format!(
            "Unknown key name: '{}'",
            args[0]
        )));
    }

    let scan_code = crate::get_scan_code(vk);
    let extended = crate::utils::key_code::is_extended_key(vk as u8);

    let mut mode_send = true;

    if args.len() > 1 {
        mode_send = match args[1] {
            "post" => false,
            "send" => true,
            other => {
                return Err(ScriptError::ParseError(format!(
                    "Position 2 must be 'send' or 'post', got: '{}'",
                    other
                )));
            }
        };
    }

    if mode_send {
        let input = keyboard_input::build_key_up_input(vk as u8, extended);
        Ok(KeyParams::SendUp(KeyUpParams { input }))
    } else {
        Ok(KeyParams::PostUp(KeyPostDownUpParams {
            vk_code: vk as u8,
            scan_code,
        }))
    }
}
