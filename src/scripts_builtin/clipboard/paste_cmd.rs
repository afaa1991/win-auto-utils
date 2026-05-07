//! Paste instruction handler
//!
//! Implements the `paste` instruction for pasting clipboard content via Ctrl+V.

use crate::keyboard::keyboard_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use std::thread;
use std::time::Duration;

/// Paste handler - simulate Ctrl+V paste operation
///
/// Syntax: `paste [delay_ms]`
///
/// # Examples
/// ```text
/// paste                   # Paste with default 20ms delay
/// paste 20                # Paste with 20ms delay between key presses
/// paste 50                # Paste with 50ms delay (slower applications)
/// ```
///
/// # Behavior
/// 1. Press and hold CONTROL key
/// 2. Wait for specified delay (default: 20ms)
/// 3. Press and release V key
/// 4. Wait for specified delay
/// 5. Release CONTROL key
///
/// # Notes
/// The delay parameter is important for applications that need time to process
/// key presses. Some applications may require longer delays to properly receive
/// the paste command.
pub struct PasteHandler;

impl InstructionHandler for PasteHandler {
    fn name(&self) -> &str {
        "paste"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let delay_ms = if args.is_empty() {
            20 // Default delay
        } else {
            args[0].parse::<u32>().map_err(|e| {
                ScriptError::ParseError(format!("Invalid delay '{}': {}", args[0], e))
            })?
        };

        Ok(InstructionData::Custom(Box::new(delay_ms)))
    }

    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let delay_ms = *data.extract_custom::<u32>("Invalid paste delay")?;

        // VK codes
        const VK_CONTROL: u8 = 0x11;
        const VK_V: u8 = 0x56;

        // Step 1: Press CONTROL
        let ctrl_down = keyboard_input::build_key_down_input(VK_CONTROL, false);
        keyboard_input::execute_single_input(&ctrl_down).map_err(|e| {
            ScriptError::ExecutionError(format!("Failed to press CONTROL: {:?}", e))
        })?;

        // Step 2: Wait before pressing V
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms as u64));
        }

        // Step 3: Press and release V
        let v_down = keyboard_input::build_key_down_input(VK_V, false);
        let v_up = keyboard_input::build_key_up_input(VK_V, false);
        
        keyboard_input::execute_inputs(&[v_down, v_up]).map_err(|e| {
            ScriptError::ExecutionError(format!("Failed to press V: {:?}", e))
        })?;

        // Step 4: Wait before releasing CONTROL
        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms as u64));
        }

        // Step 5: Release CONTROL
        let ctrl_up = keyboard_input::build_key_up_input(VK_CONTROL, false);
        keyboard_input::execute_single_input(&ctrl_up).map_err(|e| {
            ScriptError::ExecutionError(format!("Failed to release CONTROL: {:?}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paste_default() {
        let handler = PasteHandler;
        let result = handler.parse(&[]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let delay = boxed.downcast_ref::<u32>().unwrap();
                assert_eq!(*delay, 20);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_paste_with_delay() {
        let handler = PasteHandler;
        let result = handler.parse(&["50"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let delay = boxed.downcast_ref::<u32>().unwrap();
                assert_eq!(*delay, 50);
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_paste_invalid_delay() {
        let handler = PasteHandler;
        assert!(handler.parse(&["abc"]).is_err());
    }
}
