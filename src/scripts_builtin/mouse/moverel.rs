//! Mouse relative move instruction handler
//!
//! Implements the `moverel` instruction for moving the mouse relatively from current position.
//!
//! # Execution Behavior
//! Moves the cursor by the specified delta from its current position.
//!
//! # Syntax
//! ```text
//! moverel <dx> <dy>
//! ```
//!
//! # Arguments
//! - `dx` (required): Horizontal delta (positive = right, negative = left)
//! - `dy` (required): Vertical delta (positive = down, negative = up)
//!
//! # Examples
//! ```text
//! moverel 10 20              # Move 10px right and 20px down
//! moverel -50 -30             # Move 50px left and 30px up
//! ```
//!
//! # Errors
//! - Execution errors occur if SendInput fails

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

#[derive(Clone)]
pub struct MoveRelParams {
    pub move_input: INPUT,
}

pub struct MoveRelHandler;

impl InstructionHandler for MoveRelHandler {
    fn name(&self) -> &str {
        "moverel"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.len() != 2 {
            return Err(ScriptError::ParseError(
                "Missing offsets. Usage: moverel <dx> <dy>".into(),
            ));
        }

        let dx = args[0].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid dx offset '{}': {}", args[0], e))
        })?;

        let dy = args[1].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid dy offset '{}': {}", args[1], e))
        })?;

        Ok(InstructionData::Custom(Box::new(MoveRelParams {
            move_input: mouse_input::build_move_relative(dx, dy),
        })))
    }

    #[inline]
    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<MoveRelParams>("Invalid moverel parameters")?;

        mouse_input::execute_single_input(&params.move_input).map_err(|e| {
            ScriptError::ExecutionError(format!("Move relative failed: {:?}", e))
        })?;

        Ok(())
    }
}
