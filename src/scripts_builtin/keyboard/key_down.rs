//! Key down instruction handler
//!
//! Implements the `key_down` instruction for pressing keys without automatic release.
//!
//! # Execution Sequence (Send Mode)
//! 1. Press key down (KEYEVENTF_KEYDOWN)
//! 2. Key remains held until `key_up` is called
//!
//! # Execution Mode (Post Mode)
//! 1. Send WM_KEYDOWN message
//! 2. Key remains held until `key_up` is called
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
//! key_down <key_name> [mode]
//! ```
//!
//! # Arguments
//! - `key_name` (required): Key name (e.g., A, ENTER, F1, SHIFT, CONTROL)
//! - `mode` (optional): Either `send` (default) or `post`
//!
//! # Use Cases
//! - Key combinations (Ctrl+C, Alt+Tab, etc.)
//! - Holding modifier keys while performing mouse actions
//! - Games requiring key hold for sprint/dodge
//!
//! # Examples
//! ```text
//! key_down SHIFT               # Hold SHIFT in foreground
//! key_down CONTROL send         # Hold CONTROL explicitly
//! key_down A post              # Hold 'A' in background (requires hwnd)
//! ```
//!
//! # Key Combinations
//! ```text
//! key_down SHIFT
//! key_down CONTROL
//! key C                        # Type "C" while holding both modifiers
//! key_up SHIFT
//! key_up CONTROL
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

pub struct KeyDownHandler;

impl InstructionHandler for KeyDownHandler {
    fn name(&self) -> &str {
        "key_down"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let params = crate::scripts_builtin::keyboard::parse_key_down_args(args)?;
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
            KeyParams::SendDown(p) => {
                keyboard_input::execute_single_input(&p.input).map_err(|e| {
                    ScriptError::ExecutionError(format!("Key down failed: {:?}", e))
                })?;
            }
            KeyParams::PostDown(p) => {
                let hwnd = vm.process.hwnd.ok_or_else(|| {
                    ScriptError::ExecutionError(
                        "PostMessage mode requires hwnd to be set via process.set_hwnd()".into(),
                    )
                })?;

                keyboard_message::post_key_down_atomic(hwnd, p.vk_code, p.scan_code)
                    .map_err(|e| {
                        ScriptError::ExecutionError(format!("PostMessage key down failed: {:?}", e))
                    })?;
            }
            _ => {
                return Err(ScriptError::ExecutionError(
                    "Invalid key parameters for key_down".into(),
                ));
            }
        }

        Ok(())
    }
}
