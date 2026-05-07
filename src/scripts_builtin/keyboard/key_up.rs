//! Key up instruction handler
//!
//! Implements the `key_up` instruction for releasing pressed keys.
//!
//! # Execution Sequence (Send Mode)
//! 1. Release key up (KEYEVENTF_KEYUP)
//!
//! # Execution Mode (Post Mode)
//! 1. Send WM_KEYUP message
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
//! key_up <key_name> [mode]
//! ```
//!
//! # Arguments
//! - `key_name` (required): Key name (e.g., A, ENTER, F1, SHIFT, CONTROL)
//! - `mode` (optional): Either `send` (default) or `post`
//!
//! # Use Cases
//! - Releasing key combinations (Ctrl+C, Alt+Tab, etc.)
//! - Releasing modifier keys after mouse actions
//! - Games requiring key release for stop actions
//!
//! # Examples
//! ```text
//! key_up SHIFT                 # Release SHIFT in foreground
//! key_up CONTROL send          # Release CONTROL explicitly
//! key_up A post               # Release 'A' in background (requires hwnd)
//! ```
//!
//! # Complete Key Combination Pattern
//! ```text
//! key_down SHIFT               # Hold SHIFT
//! key C                        # Press and release 'C'
//! key_up SHIFT                 # Release SHIFT
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

pub struct KeyUpHandler;

impl InstructionHandler for KeyUpHandler {
    fn name(&self) -> &str {
        "key_up"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let params = crate::scripts_builtin::keyboard::parse_key_up_args(args)?;
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
            KeyParams::SendUp(p) => {
                keyboard_input::execute_single_input(&p.input).map_err(|e| {
                    ScriptError::ExecutionError(format!("Key up failed: {:?}", e))
                })?;
            }
            KeyParams::PostUp(p) => {
                let hwnd = vm.process.hwnd.ok_or_else(|| {
                    ScriptError::ExecutionError(
                        "PostMessage mode requires hwnd to be set via process.set_hwnd()".into(),
                    )
                })?;

                keyboard_message::post_key_up_atomic(hwnd, p.vk_code, p.scan_code)
                    .map_err(|e| {
                        ScriptError::ExecutionError(format!("PostMessage key up failed: {:?}", e))
                    })?;
            }
            _ => {
                return Err(ScriptError::ExecutionError(
                    "Invalid key parameters for key_up".into(),
                ));
            }
        }

        Ok(())
    }
}
