//! Key up instruction handler
//!
//! Implements the `key_up` instruction for releasing keyboard keys.

use super::KeyParams;
use crate::keyboard::keyboard_input;
#[cfg(feature = "script_process_context")]
use crate::keyboard::keyboard_message;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::scripts_builtin::keyboard::{parse_key_args, KeyMode};

/// Key up handler (release a pressed key)
///
/// Syntax: `key_up <key_name> [mode]`
///
/// # Examples
/// ```text
/// key_up A              # Release 'A' in foreground
/// key_up SHIFT post     # Release SHIFT in background
/// ```
///
/// Note: delay_ms is ignored for key_up operations (always set to 0).
pub struct KeyUpHandler;

impl InstructionHandler for KeyUpHandler {
    fn name(&self) -> &str {
        "key_up"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let mut params = parse_key_args(args)?;
        params.delay_ms = 0; // Ignore delay for up operations

        // Pre-build KEYUP INPUT at parse time (zero runtime overhead)
        if params.mode == KeyMode::Send {
            params.send_inputs = vec![keyboard_input::build_key_up_input(
                params.vk_code,
                params.extended,
            )];
        }
        // PostMessage mode keeps send_inputs empty

        Ok(InstructionData::Custom(Box::new(params)))
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<KeyParams>("Invalid key parameters")?;

        // Determine effective mode with priority: explicit > input_mode from VM > hardcoded default
        let effective_mode = if params.mode_specified {
            params.mode
        } else {
            // Apply input mode from VM context
            match super::get_input_mode(vm).as_str() {
                "post" => KeyMode::Post,
                _ => KeyMode::Send, // Default to send
            }
        };

        match effective_mode {
            KeyMode::Send => {
                // Use pre-built KEYUP INPUT (zero runtime allocation)
                // Note: SendInput mode does NOT require target_hwnd
                if params.send_inputs.len() >= 1 {
                    keyboard_input::execute_single_input(&params.send_inputs[0]).map_err(|e| {
                        ScriptError::ExecutionError(format!("SendInput release failed: {:?}", e))
                    })?;
                } else {
                    return Err(ScriptError::ExecutionError(
                        "Invalid pre-built inputs: expected 1 input for key_up".into(),
                    ));
                }
            }
            KeyMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    // PostMessage mode requires target window handle
                    keyboard_message::post_key_up_atomic(
                        vm.process.get_hwnd_or_err()?,
                        params.vk_code,
                        params.scan_code,
                    );
                }

                #[cfg(not(feature = "script_process_context"))]
                {
                    return Err(ScriptError::ExecutionError(
                        "PostMessage mode requires 'script_process_context' feature. \
                         Enable it in Cargo.toml: features = [\"scripts_keyboard_with_post\"] \
                         or use SendInput mode (default).".into()
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::scripts_builtin::keyboard::KeyMode;

    use super::*;

    #[test]
    fn test_parse_key_up_with_mode() {
        let handler = KeyUpHandler;
        let result = handler.parse(&["A", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x41); // 'A'
                assert_eq!(params.mode, KeyMode::Post);
                assert_eq!(params.delay_ms, 0); // Should be forced to 0
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_up_missing_name() {
        let handler = KeyUpHandler;
        assert!(handler.parse(&[]).is_err());
    }

    #[test]
    fn test_prebuilt_up_input() {
        let handler = KeyUpHandler;

        // Test SendInput mode - should have 1 pre-built input (up only)
        let result = handler.parse(&["SHIFT"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.send_inputs.len(), 1, "Should pre-build KEYUP input");
                assert_eq!(params.delay_ms, 0);
            }
            _ => panic!("Expected Custom data"),
        }

        // Test PostMessage mode - should have 0 inputs
        let result = handler.parse(&["SHIFT", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(
                    params.send_inputs.len(),
                    0,
                    "PostMessage mode should not pre-build INPUT"
                );
            }
            _ => panic!("Expected Custom data"),
        }
    }
}
