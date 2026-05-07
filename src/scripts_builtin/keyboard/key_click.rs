//! Key click instruction handler
//!
//! Implements the `key` instruction for complete keyboard click operations.
//!
//! # Execution Sequence (Send Mode)
//! 1. Press key down (KEYEVENTF_KEYDOWN)
//! 2. Wait for `delay_ms` milliseconds
//! 3. Release key up (KEYEVENTF_KEYUP)
//!
//! # Execution Mode (Post Mode)
//! 1. Send WM_KEYDOWN message
//! 2. Wait for `delay_ms` milliseconds
//! 3. Send WM_KEYUP message
//!
//! # Execution Modes
//!
//! ## `send` - Foreground Mode (Default)
//! Simulates input to the active window using SendInput API.
//! - No target window configuration needed
//! - Works with the currently focused application
//!
//! ## `post` - Background Mode
//! Sends messages directly to a specific window using PostMessage API.
//! - Requires `target_hwnd` to be set via `process.set_hwnd()` before execution
//! - Works even when window is not in focus
//!
//! # Syntax
//! ```text
//! key <key_name> [delay_ms] [mode]
//! ```
//!
//! # Arguments
//! - `key_name` (required): Key name (e.g., A, ENTER, F1, SHIFT, CONTROL)
//! - `delay_ms` (optional): Milliseconds to wait between press and release. Default is 0
//! - `mode` (optional): Either `send` (default) or `post`
//!
//! # Key Names
//! - Letters: A-Z, a-z
//! - Numbers: 0-9
//! - Function keys: F1-F12
//! - Special: ENTER, TAB, ESCAPE, SPACE, BACKSPACE, DELETE
//! - Arrows: UP, DOWN, LEFT, RIGHT, HOME, END, PAGEUP, PAGEDOWN
//! - Modifiers: SHIFT, CONTROL, CTRL, ALT, MENU
//! - Others: INSERT, PRINTSCREEN, PAUSE, CAPSLOCK
//!
//! # Examples
//! ```text
//! key A                        # Press and release 'A' in foreground
//! key ENTER 50                 # Press ENTER with 50ms delay
//! key A post                   # Press 'A' in background (requires hwnd)
//! key SHIFT post               # Press SHIFT in background
//! ```
//!
//! # Key Combinations
//! ```text
//! key_down SHIFT
//! key A
//! key_up SHIFT
//! ```
//!
//! # Errors
//! - Unknown key names are rejected during parse
//! - Invalid mode values are rejected during parse
//! - Post mode without hwnd set fails at execution

use crate::keyboard::keyboard_input;
use crate::keyboard::keyboard_message;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::scripts_builtin::keyboard::KeyParams;
use crate::utils::sleep_ms;

pub struct KeyClickHandler;

impl InstructionHandler for KeyClickHandler {
    fn name(&self) -> &str {
        "key"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let params = crate::scripts_builtin::keyboard::parse_key_click_args(args)?;
        Ok(InstructionData::Custom(Box::new(params)))
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<KeyParams>("Invalid key parameters")?;

        match params {
            KeyParams::SendClick(p) => {
                keyboard_input::execute_single_input(&p.key_down_input).map_err(|e| {
                    ScriptError::ExecutionError(format!("Key down failed: {:?}", e))
                })?;
                if p.delay_ms > 0 {
                    sleep_ms(p.delay_ms);
                }
                keyboard_input::execute_single_input(&p.key_up_input).map_err(|e| {
                    ScriptError::ExecutionError(format!("Key up failed: {:?}", e))
                })?;
            }
            KeyParams::PostClick(p) => {
                let hwnd = vm.process.hwnd.ok_or_else(|| {
                    ScriptError::ExecutionError(
                        "PostMessage mode requires hwnd to be set via process.set_hwnd()".into(),
                    )
                })?;

                keyboard_message::post_key_down_atomic(hwnd, p.vk_code, p.scan_code)
                    .map_err(|e| {
                        ScriptError::ExecutionError(format!("PostMessage key down failed: {:?}", e))
                    })?;
                if p.delay_ms > 0 {
                    sleep_ms(p.delay_ms);
                }
                keyboard_message::post_key_up_atomic(hwnd, p.vk_code, p.scan_code)
                    .map_err(|e| {
                        ScriptError::ExecutionError(format!("PostMessage key up failed: {:?}", e))
                    })?;
            }
            _ => {
                return Err(ScriptError::ExecutionError(
                    "Invalid key parameters for click".into(),
                ));
            }
        }

        Ok(())
    }
}
