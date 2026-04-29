//! Keyboard automation instructions
//!
//! Provides three keyboard instructions for automating key presses:
//! - `key`: Complete click (press + release)
//! - `key_down`: Press and hold a key
//! - `key_up`: Release a held key
//!
//! # Instructions
//!
//! ## `key` - Click a key
//!
//! Performs a complete key click (press followed by release).
//!
//! **Syntax:** `key <key_name> [delay_ms] [mode]`
//!
//! **Parameters:**
//! - `<key_name>` (required): Key to press (e.g., A, ENTER, SPACE, F1)
//! - `[delay_ms]` (optional): Delay in milliseconds between press and release (default: 0)
//! - `[mode]` (optional): Execution mode override - `send` (foreground) or `post` (background)
//!   - If omitted, uses the current `input_mode` setting (see Script Mode Configuration below)
//!
//! **Examples:**
//! ```text
//! key A                    # Click 'A' using current input_mode (default: send)
//! key A 50                 # Click 'A' with 50ms delay
//! key ENTER post           # Click ENTER in background (explicit override)
//! key SPACE 100 post       # Click SPACE in background with 100ms delay
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
//! **Parameters:**
//! - `<key_name>` (required): Key to press
//! - `[mode]` (optional): Execution mode override - `send` or `post`
//!   - If omitted, uses the current `input_mode` setting
//!
//! **Examples:**
//! ```text
//! key_down SHIFT           # Hold SHIFT using current input_mode
//! key_down CONTROL post    # Hold CONTROL in background (explicit override)
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
//! **Parameters:**
//! - `<key_name>` (required): Key to release
//! - `[mode]` (optional): Execution mode override - `send` or `post`
//!   - If omitted, uses the current `input_mode` setting
//!
//! **Examples:**
//! ```text
//! key_up SHIFT             # Release SHIFT using current input_mode
//! key_up CONTROL post      # Release CONTROL in background (explicit override)
//! ```
//!
//! ---
//!
//! # Execution Modes
//!
//! ## Foreground Mode (`send`) - Default
//!
//! Simulates input to the active window using SendInput API.
//! - No target window configuration needed
//! - Works with the currently focused application
//! - Supports coordinates for mouse operations (not applicable for keyboard)
//!
//! ```text
//! key A                    # Same as: key A send
//! key ENTER 50             # Same as: key ENTER 50 send
//! ```
//!
//! ## Background Mode (`post`)
//!
//! Sends messages directly to a specific window using PostMessage API.
//! - Requires `target_hwnd` to be set in VM state before execution
//! - Works even when window is not in focus
//! - Must specify mode explicitly
//!
//! ```text
//! # Set target window first (in your Rust code)
//! # vm.set_persistent_state("target_hwnd", hwnd);
//!
//! key A post               # Click 'A' in background window
//! key ENTER 50 post        # Click ENTER with delay in background
//! ```
//!
//! ### Setting Target Window (Rust Code)
//!
//! Before using background mode, you must inject the window handle into the VM context:
//!
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};
//! use windows::Win32::Foundation::HWND;
//!
//! let config = ScriptConfig::default();
//! let engine = ScriptEngine::with_builtin(config);
//!
//! // Obtain a valid HWND from your application logic
//! let hwnd = HWND(std::ptr::null_mut());
//!
//! // Set target window handle before executing scripts
//! engine.compile_and_execute_with_context("key A post", |ctx| {
//!     ctx.set_persistent_state("target_hwnd", hwnd);
//! }).unwrap();
//! ```
//!
//! ---
//!
//! # Script Mode Configuration
//!
//! You can configure the default input mode for all keyboard instructions using the `mode` instruction.
//! This eliminates the need to specify the mode parameter on every instruction.
//!
//! **Priority**: Explicit mode in instruction > Current `input_mode` setting > Hardcoded default ("send")
//!
//! ## Setting Input Mode
//!
//! ```text
//! # Switch to background mode (PostMessage)
//! mode input_mode post
//! key A              # Automatically uses PostMessage (no need to write 'post')
//! key B 50           # Automatically uses PostMessage with 50ms delay
//!
//! # Switch back to foreground mode (SendInput)
//! mode input_mode send
//! key C              # Automatically uses SendInput
//!
//! # Can still override per-instruction
//! mode input_mode post
//! key D send         # Explicitly use SendInput, overriding input_mode setting
//! ```
//!
//! ## Setting Input Mode from Rust Code
//!
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};
//! use windows::Win32::Foundation::HWND;
//!
//! let config = ScriptConfig::default();
//! let engine = ScriptEngine::with_builtin(config);
//!
//! // Assume hwnd is a valid window handle obtained elsewhere
//! let hwnd = HWND(std::ptr::null_mut());
//!
//! // Set target window and input mode to background before executing scripts
//! engine.compile_and_execute_with_context("key A", |ctx| {
//!     ctx.set_persistent_state("target_hwnd", hwnd);
//!     ctx.set_persistent_state("input_mode", "post");
//! }).unwrap();
//!
//! // Now all subsequent keyboard instructions will use PostMessage by default
//! ```
//!
//! **Benefits:**
//! - ✅ Reduces repetition in scripts
//! - ✅ Cleaner and more readable code
//! - ✅ Easy to switch modes for entire script sections
//! - ✅ Still allows per-instruction overrides when needed
//! - ✅ Modular: Different modules can have independent configurations
//!
//! ---
//!
//! # Common Use Cases
//!
//! ## Simple Typing
//! ```text
//! key H
//! key E
//! key L
//! key L
//! key O
//! ```
//!
//! ## Key Combinations (Foreground)
//! ```text
//! # Ctrl+C (Copy)
//! key_down CONTROL
//! key C
//! key_up CONTROL
//!
//! # Shift+A (Capital A)
//! key_down SHIFT
//! key A
//! key_up SHIFT
//! ```
//!
//! ## Key Combinations (Background)
//! ```text
//! # Set target window first
//! # vm.set_persistent_state("target_hwnd", hwnd);
//!
//! # Ctrl+S (Save) in background
//! key_down CONTROL post
//! key S post
//! key_up CONTROL post
//! ```
//!
//! ## Key Combinations (Background) - Simplified with Input Mode
//! ```text
//! # Set target window and input mode first
//! # vm.set_persistent_state("target_hwnd", hwnd);
//! # mode input_mode post
//!
//! # Ctrl+S (Save) in background - no need to specify 'post' on each instruction
//! key_down CONTROL
//! key S
//! key_up CONTROL
//! ```
//!
//! ## Typing with Delays
//! ```text
//! # Type slowly for better reliability
//! key H 50
//! key E 50
//! key L 50
//! key L 50
//! key O 50
//! ```
//!
//! ---
//!
//! # Supported Keys
//!
//! ## Letters & Numbers
//! `A-Z`, `0-9`
//!
//! ## Function Keys
//! `F1` through `F24`
//!
//! ## Special Keys
//! - `ENTER`, `RETURN`, `TAB`
//! - `SPACE`, `BACKSPACE`, `DELETE`
//! - `ESCAPE`, `ESC`
//! - `SHIFT`, `CONTROL`, `CTRL`, `ALT`
//! - `UP`, `DOWN`, `LEFT`, `RIGHT` (arrow keys)
//! - `HOME`, `END`, `PAGEUP`, `PAGEDOWN`
//! - `INSERT`, `PRINTSCREEN`, `CAPSLOCK`
//! - `NUMLOCK`, `SCROLLLOCK`
//!
//! ## Numpad Keys
//! `NUMPAD0` through `NUMPAD9`, `NUMPAD_ADD`, `NUMPAD_SUBTRACT`, etc.
//!
//! ---
//!
//! # Error Messages
//!
//! The parser provides clear error messages for common mistakes:
//!
//! ```text
//! key INVALID_KEY          → "Unknown key name: 'INVALID_KEY'"
//! key A invalid            → "Position 2 must be a number (delay) or 'send'/'post' (mode)"
//! key A 50 invalid         → "Position 3 must be 'send' or 'post', got: 'invalid'"
//! key A post 50            → "Unexpected parameter at position 3"
//! key A 50 post extra      → "Too many parameters"
//! ```
//!
//! ---
//!
//! # Important Notes
//!
//! 1. **Background mode requires setup**: Before using `post` mode, you must set the target window handle in your Rust code:
//!    ```rust
//!    vm.set_persistent_state("target_hwnd", some_hwnd);
//!    ```
//!
//! 2. **Delay affects timing**: The `delay_ms` parameter controls the time between key press and release, not between instructions.
//!
//! 3. **Key names are case-insensitive**: `key A` and `key a` work the same way.
//!
//! //! 4. **Modifier keys**: For combinations like Ctrl+C, use `key_down`/`key_up` instead of trying to pass multiple keys to `key`.

use crate::get_scan_code;
use crate::script_engine::instruction::ScriptError;
use crate::script_engine::VMContext;
use crate::utils::key_code;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

// Submodules - each instruction handler in its own file
pub mod key_click;
pub mod key_down;
pub mod key_up;

// Re-export handlers for convenience
pub use key_click::KeyClickHandler;
pub use key_down::KeyDownHandler;
pub use key_up::KeyUpHandler;

// ============================================================================
// Common Types and Utilities
// ============================================================================

/// Key operation mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyMode {
    /// Foreground mode using SendInput API (default)
    Send,
    /// Background mode using PostMessage API
    Post,
}

impl std::fmt::Display for KeyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyMode::Send => write!(f, "send"),
            KeyMode::Post => write!(f, "post"),
        }
    }
}

/// Keyboard instruction parameters (pre-compiled at parse time)
#[derive(Clone)]
pub struct KeyParams {
    /// Virtual key code
    pub vk_code: u8,
    /// Scan code (pre-computed for PostMessage)
    pub scan_code: u16,
    /// Extended key flag
    pub extended: bool,
    /// Operation mode (Send/Post) - may be overridden by default mode at execute time
    pub mode: KeyMode,
    /// Whether mode was explicitly specified in arguments
    /// If false, the handler should check VM context for default mode
    pub mode_specified: bool,
    /// Delay in milliseconds between key down and up (for click only)
    pub delay_ms: u32,
    /// Pre-built INPUT structures for SendInput mode (zero runtime allocation)
    pub send_inputs: Vec<INPUT>,
}

/// Helper function to parse keyboard instruction arguments (pure parameter extraction)
///
/// This function only extracts and validates parameters from args.
/// It does NOT pre-build INPUT structures - that's the responsibility of each handler.
///
/// Smart parameter parsing with fixed positions but optional skip:
/// - Position 1 (required): Key name (e.g., A, ENTER, SPACE)
/// - Position 2 (optional): Delay in milliseconds OR mode ('send'/'post')
/// - Position 3 (optional): Mode (only if position 2 is delay)
///
/// Examples:
/// - `key A` click A (foreground, no delay)
/// - `key A 50` click A (foreground, 50ms delay)
/// - `key A post` click A (background, no delay)
/// - `key A 50 post` click A (background, 50ms delay)
/// - `key A send` click A (foreground, no delay, explicit)
pub fn parse_key_args(args: &[&str]) -> Result<KeyParams, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ParseError(
            "Missing key name. Usage: key <name> [delay_ms] [mode]".into(),
        ));
    }

    // Position 1: Pre-compile key name to VK code
    let vk = key_code(args[0]);
    if vk == 0 {
        return Err(ScriptError::ParseError(format!(
            "Unknown key name: '{}'. Use standard key names like A, B, SPACE, ENTER, etc.",
            args[0]
        )));
    }

    // Pre-compute scan code for PostMessage (one-time system call at parse time)
    let scan_code = get_scan_code(vk);

    // Default values
    let mut delay_ms = 0u32;
    let mut mode = KeyMode::Send;

    // Parse remaining parameters (max 2)
    if args.len() > 3 {
        return Err(ScriptError::ParseError(format!(
            "Too many parameters. Expected format: key <name> [delay_ms] [mode], got {} parameters",
            args.len()
        )));
    }

    // Position 2: Could be delay OR mode
    if args.len() > 1 {
        match args[1].parse::<u32>() {
            Ok(delay) => {
                // It's a number delay
                delay_ms = delay;

                // Position 3 (if exists): Must be mode
                if args.len() > 2 {
                    mode = match args[2] {
                        "post" => KeyMode::Post,
                        "send" => KeyMode::Send,
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
                // It's not a number must be mode
                mode = match args[1] {
                    "post" => KeyMode::Post,
                    "send" => KeyMode::Send,
                    other => {
                        return Err(ScriptError::ParseError(format!(
                            "Position 2 must be a number (delay) or 'send'/'post' (mode), got: '{}'",
                            other
                        )));
                    }
                };

                // Reject extra parameter when position 2 is mode
                if args.len() > 2 {
                    return Err(ScriptError::ParseError(format!(
                        "Unexpected parameter at position 3: '{}'. When mode is specified at position 2, no further parameters are allowed.",
                        args[2]
                    )));
                }
            }
        }
    }

    // Check if extended key
    let extended = crate::utils::key_code::is_extended_key(vk as u8);

    // Determine if mode was explicitly specified
    let mode_specified = args.len() > 1
        && (
            // Position 2 is mode directly
            (args[1] == "send" || args[1] == "post") ||
        // Position 3 is mode (when position 2 is delay)
        (args.len() > 2 && (args[2] == "send" || args[2] == "post"))
        );

    // Return parameters WITHOUT pre-building INPUT structures
    // Each handler will build its own specific inputs based on instruction type
    Ok(KeyParams {
        vk_code: vk as u8,
        scan_code,
        extended,
        mode,
        mode_specified,
        delay_ms,
        send_inputs: vec![], // Empty by default, handlers will populate as needed
    })
}

const INPUT_MODE_KEY: &str = "input_mode";

#[inline]
pub fn get_input_mode(vm: &VMContext) -> String {
    vm.get_persistent_state::<String>(INPUT_MODE_KEY)
        .cloned()
        .unwrap_or_else(|| "send".to_string())
}
