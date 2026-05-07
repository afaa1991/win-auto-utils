//! Mouse move instruction handler (absolute positioning)
//!
//! Implements the `move` instruction for moving the mouse to an absolute position.
//!
//! # Execution Behavior
//! Moves the cursor to the specified absolute screen coordinates.
//!
//! # Coordinate System
//!
//! - **Without hwnd**: Coordinates are absolute screen coordinates
//! - **With hwnd set**: Coordinates are window-relative, converted to screen coordinates by adding window_left and window_top
//!
//! # Syntax
//! ```text
//! move <x> <y>
//! ```
//!
//! # Arguments
//! - `x` (required): X coordinate
//! - `y` (required): Y coordinate
//!
//! # Examples
//! ```text
//! move 100 200              # Move to (100, 200) screen coordinates
//! ```
//!
//! # Errors
//! - Negative coordinates are rejected during parse
//! - Execution errors occur if SetCursorPos fails

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

#[derive(Clone)]
pub struct MoveParams {
    pub x: i32,
    pub y: i32,
}

pub struct MoveHandler;

impl InstructionHandler for MoveHandler {
    fn name(&self) -> &str {
        "move"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.len() != 2 {
            return Err(ScriptError::ParseError(
                "Missing coordinates. Usage: move <x> <y>".into(),
            ));
        }

        let x = args[0].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid x coordinate '{}': {}", args[0], e))
        })?;

        let y = args[1].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid y coordinate '{}': {}", args[1], e))
        })?;

        // Validate coordinates
        if x < 0 || y < 0 {
            return Err(ScriptError::ParseError(
                format!("Coordinates cannot be negative: ({}, {})", x, y).into(),
            ));
        }

        Ok(InstructionData::Custom(Box::new(MoveParams { x, y })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<MoveParams>("Invalid move parameters")?;

        // Apply window offset if window geometry is available
        let (screen_x, screen_y) = match vm.process.window_geometry {
            Some(geo) => (params.x + geo.window_left, params.y + geo.window_top),
            None => (params.x, params.y),
        };

        mouse_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
            ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
        })?;

        Ok(())
    }
}
