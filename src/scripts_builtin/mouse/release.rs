//! Mouse release left button instruction handler
//!
//! Implements the `release` instruction for releasing the left mouse button.
//!
//! # Execution Behavior
//! Releases the left mouse button that was previously pressed with `press`.
//! Use after `press` to complete drag operations.
//!
//! # Syntax
//! ```text
//! release
//! ```
//!
//! # Examples
//! ```text
//! press                      # Press left button at starting position
//! move 500 500               # Drag to new position while holding
//! release                    # Release the button
//! ```
//!
//! # Use Cases
//! - Completing drag and drop operations
//! - Releasing multi-select operations
//! - Ending canvas drawing strokes
//! - Game button release

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

#[derive(Clone)]
pub struct ReleaseParams {
    pub input: INPUT,
}

pub struct ReleaseHandler;

impl InstructionHandler for ReleaseHandler {
    fn name(&self) -> &str {
        "release"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if !args.is_empty() {
            return Err(ScriptError::ParseError(
                "release does not accept arguments. Usage: release".into(),
            ));
        }
        Ok(InstructionData::Custom(Box::new(ReleaseParams {
            input: mouse_input::build_release_left(),
        })))
    }

    #[inline]
    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<ReleaseParams>("Invalid release parameters")?;
        mouse_input::execute_single_input(&params.input).map_err(|e| {
            ScriptError::ExecutionError(format!("Release failed: {:?}", e))
        })?;
        Ok(())
    }
}
