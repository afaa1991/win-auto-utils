//! Mouse press left button instruction handler
//!
//! Implements the `press` instruction for pressing and holding the left mouse button.
//!
//! # Execution Behavior
//! Presses and holds the left mouse button without releasing.
//! Use with `release` instruction or `move` to perform drag operations.
//!
//! # Syntax
//! ```text
//! press
//! ```
//!
//! # Examples
//! ```text
//! press                      # Press left button at current position
//! move 500 500               # Drag to new position while holding
//! release                    # Release the button
//! ```
//!
//! # Use Cases
//! - Drag and drop operations
//! - Multi-select with Shift+click patterns
//! - Canvas drawing applications
//! - Game interactions requiring hold

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

#[derive(Clone)]
pub struct PressParams {
    pub input: INPUT,
}

pub struct PressHandler;

impl InstructionHandler for PressHandler {
    fn name(&self) -> &str {
        "press"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if !args.is_empty() {
            return Err(ScriptError::ParseError(
                "press does not accept arguments. Usage: press".into(),
            ));
        }
        Ok(InstructionData::Custom(Box::new(PressParams {
            input: mouse_input::build_press_left(),
        })))
    }

    #[inline]
    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<PressParams>("Invalid press parameters")?;
        mouse_input::execute_single_input(&params.input).map_err(|e| {
            ScriptError::ExecutionError(format!("Press failed: {:?}", e))
        })?;
        Ok(())
    }
}
