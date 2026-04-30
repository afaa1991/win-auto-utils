//! Key click instruction handler
//!
//! Implements the `key` instruction for complete keyboard click operations.
//! Performs press → delay (optional) → release sequence.

use super::KeyParams;
use crate::keyboard::keyboard_input;
#[cfg(feature = "script_process_context")]
use crate::keyboard::keyboard_message;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::scripts_builtin::keyboard::{parse_key_args, KeyMode};

use crate::utils::sleep_ms;

/// Key click handler (complete press + delay + release operation)
///
/// Syntax: `key <key_name> [delay_ms] [mode]`
///
/// # Parameter Parsing Rules
///
/// The parser uses intelligent type inference for optimal performance:
/// - **Position 1** (required): Key name (e.g., A, ENTER, SPACE)
/// - **Position 2** (optional): Delay in milliseconds (number) OR mode ('send'/'post')
/// - **Position 3** (optional): Mode (only if position 2 is a number)
///
/// # Examples
/// ```text
/// key A                    # Click 'A' in foreground (default mode: send, no delay)
/// key A 50                 # Click 'A' with 50ms delay between press and release
/// key A post               # Click 'A' in background (no delay)
/// key A 50 post            # Click 'A' in background with 50ms delay
/// key A send               # Click 'A' in foreground (explicit mode)
/// ```
///
/// # Smart Type Inference
///
/// The parser automatically detects parameter types:
/// - If position 2 is a **number** treated as delay, position 3 can be mode
/// - If position 2 is **'send'/'post'** treated as mode, no position 3 allowed
///
/// Valid patterns:
/// ```text
/// key A 50 post     Valid: 50 is number (delay), post is mode
/// key A post        Valid: post is mode (position 2)
/// key A post 50     Invalid: Cannot have parameter after mode at position 2
/// ```
///
/// # Execution Behavior
/// This instruction performs a complete click operation:
/// 1. Press the key (KEYDOWN)
/// 2. Wait for delay_ms milliseconds (if specified)
/// 3. Release the key (KEYUP)
///
/// For holding keys without automatic release, use `key_down` and `key_up` separately.
pub struct KeyClickHandler;

impl InstructionHandler for KeyClickHandler {
    fn name(&self) -> &str {
        "key"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let mut params = parse_key_args(args)?;

        // Pre-build INPUT structures based on mode (zero runtime overhead)
        if params.mode == KeyMode::Send {
            // For key_click: pre-build complete click sequence [KEYDOWN, KEYUP]
            params.send_inputs =
                keyboard_input::build_key_click_inputs(params.vk_code, params.extended).to_vec();
        }
        // PostMessage mode keeps send_inputs empty

        // Store parameters in Custom data
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
                // SendInput mode does NOT require target_hwnd
                if params.delay_ms > 0 {
                    // Execute with delay: use pre-built inputs separately
                    // params.send_inputs contains [down_input, up_input] (pre-built at parse time)
                    if params.send_inputs.len() >= 2 {
                        // Execute KEYDOWN (first input)
                        keyboard_input::execute_single_input(&params.send_inputs[0]).map_err(|e| {
                            ScriptError::ExecutionError(format!("SendInput press failed: {:?}", e))
                        })?;

                        // Apply delay using optimized utility function
                        sleep_ms(params.delay_ms);

                        // Execute KEYUP (second input)
                        keyboard_input::execute_single_input(&params.send_inputs[1]).map_err(|e| {
                            ScriptError::ExecutionError(format!(
                                "SendInput release failed: {:?}",
                                e
                            ))
                        })?;
                    } else {
                        return Err(ScriptError::ExecutionError(
                            "Invalid pre-built inputs: expected 2 inputs for delayed click".into(),
                        ));
                    }
                } else {
                    // No delay - execute pre-built inputs atomically (zero overhead)
                    keyboard_input::execute_inputs(&params.send_inputs).map_err(|e| {
                        ScriptError::ExecutionError(format!("SendInput click failed: {:?}", e))
                    })?;
                }
            }
            KeyMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    // PostMessage mode requires target window handle
                    let hwnd = vm.process.get_hwnd_or_err()?;

                    if params.delay_ms > 0 {
                        // Execute KEYDOWN
                        keyboard_message::post_key_down_atomic(hwnd, params.vk_code, params.scan_code);

                        // Apply delay using optimized utility function
                        sleep_ms(params.delay_ms);

                        // Execute KEYUP
                        keyboard_message::post_key_up_atomic(hwnd, params.vk_code, params.scan_code);
                    } else {
                        // No delay - atomic click
                        keyboard_message::post_key_click_atomic(hwnd, params.vk_code, params.scan_code);
                    }
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
    use super::*;

    #[test]
    fn test_parse_key_with_delay_and_mode() {
        let handler = KeyClickHandler;

        // Test: key A 50 post (delay at position 2, mode at position 3)
        let result = handler.parse(&["A", "50", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x41); // 'A' key
                assert!(params.scan_code > 0); // Scan code should be pre-computed
                assert_eq!(params.mode, KeyMode::Post);
                assert_eq!(params.delay_ms, 50);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_default_values() {
        let handler = KeyClickHandler;
        let result = handler.parse(&["ENTER"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x0D); // VK_RETURN
                assert!(params.scan_code > 0); // Pre-computed scan code
                assert_eq!(params.mode, KeyMode::Send); // Default foreground
                assert_eq!(params.delay_ms, 0); // Default no delay
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_only_delay() {
        let handler = KeyClickHandler;
        let result = handler.parse(&["SPACE", "100"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x20); // VK_SPACE
                assert_eq!(params.mode, KeyMode::Send); // Default foreground
                assert_eq!(params.delay_ms, 100);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_only_mode() {
        let handler = KeyClickHandler;
        let result = handler.parse(&["TAB", "post"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x09); // VK_TAB
                assert_eq!(params.mode, KeyMode::Post);
                assert_eq!(params.delay_ms, 0);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_key_explicit_send_mode() {
        let handler = KeyClickHandler;
        let result = handler.parse(&["A", "send"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(params.vk_code, 0x41);
                assert_eq!(params.mode, KeyMode::Send);
                assert_eq!(params.delay_ms, 0);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_prebuilt_inputs_are_complete() {
        let handler = KeyClickHandler;

        // Test case 1: No delay - should have 2 inputs (down + up)
        let result = handler.parse(&["A"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(
                    params.send_inputs.len(),
                    2,
                    "Should pre-build both down and up inputs"
                );
            }
            _ => panic!("Expected Custom data"),
        }

        // Test case 2: With delay - should ALSO have 2 inputs (down + up) for zero runtime allocation
        let result = handler.parse(&["A", "50"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(
                    params.send_inputs.len(),
                    2,
                    "Should pre-build both down and up inputs even with delay"
                );
                assert_eq!(params.delay_ms, 50);
            }
            _ => panic!("Expected Custom data"),
        }

        // Test case 3: PostMessage mode - should have 0 inputs
        let result = handler.parse(&["A", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<KeyParams>().unwrap();
                assert_eq!(
                    params.send_inputs.len(),
                    0,
                    "PostMessage mode should not pre-build INPUT structures"
                );
            }
            _ => panic!("Expected Custom data"),
        }
    }
}
