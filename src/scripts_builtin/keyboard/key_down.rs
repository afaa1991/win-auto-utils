//! Key down instruction handler
//!
//! Implements the `key_down` instruction for pressing keys without automatic release.

use super::KeyParams;
use crate::keyboard::keyboard_input;
#[cfg(feature = "script_process_context")]
use crate::keyboard::keyboard_message;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::scripts_builtin::keyboard::{parse_key_args, KeyMode};

/// Key down handler (press only, no release)
///
/// Syntax: `key_down <key_name> [mode]`
///
/// # Examples
/// ```text
/// key_down SHIFT           # Press SHIFT in foreground (default)
/// key_down SHIFT post      # Press SHIFT in background
/// key_down CONTROL send    # Press CONTROL in foreground (explicit)
/// ```
///
/// # Notes
/// Use with `key_up` to create custom key combinations.
/// Remember to release the key to avoid stuck keys!
pub struct KeyDownHandler;

impl InstructionHandler for KeyDownHandler {
    fn name(&self) -> &str {
        "key_down"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        // For key_down/up, we don't need delay parameter
        if args.is_empty() {
            return Err(ScriptError::ParseError(
                "Missing key name. Usage: key_down <name> [mode:send|post]".into(),
            ));
        }

        let mut params = parse_key_args(args)?;
        params.delay_ms = 0; // Ignore delay for down/up operations

        // Pre-build KEYDOWN INPUT at parse time (zero runtime overhead)
        if params.mode == KeyMode::Send {
            params.send_inputs = vec![keyboard_input::build_key_down_input(
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
                // Use pre-built KEYDOWN INPUT (zero runtime allocation)
                // Note: SendInput mode does NOT require target_hwnd
                if params.send_inputs.len() >= 1 {
                    keyboard_input::execute_single_input(&params.send_inputs[0]).map_err(|e| {
                        ScriptError::ExecutionError(format!("SendInput press failed: {:?}", e))
                    })?;
                } else {
                    return Err(ScriptError::ExecutionError(
                        "Invalid pre-built inputs: expected 1 input for key_down".into(),
                    ));
                }
            }
            KeyMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    // PostMessage mode requires target window handle
                    keyboard_message::post_key_down_atomic(
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
                         or use SendInput mode (default)."
                            .into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_down_with_mode() {
        let handler = KeyDownHandler;
        let result = handler.parse(&["SHIFT", "post"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x10); // VK_SHIFT
                assert!(params.scan_code > 0); // Pre-computed
                assert_eq!(params.mode, KeyMode::Post);
                assert_eq!(params.delay_ms, 0); // Delay ignored for down/up
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_down_missing_name() {
        let handler = KeyDownHandler;
        assert!(handler.parse(&[]).is_err());
    }

    #[test]
    fn test_prebuilt_down_input() {
        let handler = KeyDownHandler;

        // Test SendInput mode - should have 1 pre-built input (down only)
        let result = handler.parse(&["SHIFT"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(
                    params.send_inputs.len(),
                    1,
                    "Should pre-build KEYDOWN input"
                );
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
