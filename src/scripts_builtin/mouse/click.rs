//! Mouse click instruction handler
//!
//! Implements the `click` instruction for performing mouse clicks at specified or current position.
//!
//! # Execution Sequence
//! 1. Convert window-relative coordinates to screen coordinates (if hwnd is set)
//! 2. Move cursor to (x, y) if coordinates provided
//! 3. Press left button (MOUSEDOWN)
//! 4. Wait for `delay_ms` milliseconds
//! 5. Release left button (MOUSEUP)
//!
//! This ensures proper click detection for applications that require a minimum press duration.
//!
//! # Coordinate System
//!
//! - **Without hwnd**: Coordinates are absolute screen coordinates
//! - **With hwnd set**: Coordinates are window-relative, converted to screen coordinates by adding window_left and window_top
//!
//! # Syntax
//! ```text
//! click [x] [y] [delay_ms]
//! ```
//!
//! # Arguments
//! - `x` (optional): X coordinate. If omitted, clicks at current position
//! - `y` (optional): Y coordinate. Required if x is provided
//! - `delay_ms` (optional): Milliseconds to wait between press and release. Default is 0
//!
//! # Examples
//! ```text
//! click                      # Click at current position
//! click 100 200              # Click at (100, 200) screen coordinates
//! click 50 50 100             # Click at (50, 50) with 100ms delay
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
use crate::utils::sleep_ms;

#[derive(Clone)]
pub struct ClickParams {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub delay_ms: u32,
}

pub struct ClickHandler;

impl InstructionHandler for ClickHandler {
    fn name(&self) -> &str {
        "click"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        let mut delay_ms: u32 = 0;
        let mut i = 0;

        while i < args.len() {
            let arg = args[i];

            if let Ok(val) = arg.parse::<i32>() {
                if x.is_none() {
                    x = Some(val);
                } else if y.is_none() {
                    y = Some(val);
                } else {
                    delay_ms = val as u32;
                }
            } else {
                return Err(ScriptError::ParseError(
                    format!("Invalid argument '{}'. Expected: click [x] [y] [delay_ms]", arg).into(),
                ));
            }
            i += 1;
        }

        // Validate coordinates if provided
        if let (Some(x_val), Some(y_val)) = (x, y) {
            if x_val < 0 || y_val < 0 {
                return Err(ScriptError::ParseError(
                    format!("Coordinates cannot be negative: ({}, {})", x_val, y_val).into(),
                ));
            }
        }

        Ok(InstructionData::Custom(Box::new(ClickParams { x, y, delay_ms })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<ClickParams>("Invalid click parameters")?;

        if let (Some(x), Some(y)) = (params.x, params.y) {
            // Apply window offset if window geometry is available
            let (screen_x, screen_y) = match vm.process.window_geometry {
                Some(geo) => (x + geo.window_left, y + geo.window_top),
                None => (x, y),
            };

            mouse_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
                ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
            })?;
        }

        let press_input = mouse_input::build_press_left();
        mouse_input::execute_single_input(&press_input).map_err(|e| {
            ScriptError::ExecutionError(format!("Click press failed: {:?}", e))
        })?;

        if params.delay_ms > 0 {
            sleep_ms(params.delay_ms);
        }

        let release_input = mouse_input::build_release_left();
        mouse_input::execute_single_input(&release_input).map_err(|e| {
            ScriptError::ExecutionError(format!("Click release failed: {:?}", e))
        })?;

        Ok(())
    }
}
