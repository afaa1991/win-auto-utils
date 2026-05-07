//! Mouse double-click instruction handler
//!
//! Implements the `dbclick` instruction for performing double-clicks.
//!
//! # Execution Sequence
//! 1. Move cursor to (x, y) if coordinates provided (with window offset if hwnd set)
//! 2. Press left button (MOUSEDOWN)
//! 3. Release left button (MOUSEUP)
//! 4. Wait 50ms (inter-click delay)
//! 5. Press left button (MOUSEDOWN)
//! 6. Release left button (MOUSEUP)
//! 7. Wait for `delay_ms` milliseconds (if specified)
//!
//! # Coordinate System
//!
//! - **Without hwnd**: Coordinates are absolute screen coordinates
//! - **With hwnd set**: Coordinates are window-relative, converted to screen coordinates by adding window_left and window_top
//!
//! # Syntax
//! ```text
//! dbclick [x] [y] [delay_ms]
//! ```
//!
//! # Arguments
//! - `x` (optional): X coordinate. If omitted, double-clicks at current position
//! - `y` (optional): Y coordinate. Required if x is provided
//! - `delay_ms` (optional): Milliseconds to wait after the second click. Default is 0
//!
//! # Examples
//! ```text
//! dbclick                      # Double-click at current position
//! dbclick 100 200             # Double-click at (100, 200)
//! dbclick 50 50 100            # Double-click at (50, 50) with 100ms delay after
//! ```
//!
//! # Errors
//! - Negative coordinates are rejected during parse
//! - Execution errors occur if SetCursorPos or SendInput fails

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::utils::sleep_ms;

#[derive(Clone)]
pub struct DbClickParams {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub delay_ms: u32,
}

pub struct DbClickHandler;

impl InstructionHandler for DbClickHandler {
    fn name(&self) -> &str {
        "dbclick"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        let mut delay_ms: u32 = 0;

        match args.len() {
            0 => {}
            1 => {
                if let Ok(val) = args[0].parse::<i32>() {
                    if val < 0 {
                        return Err(ScriptError::ParseError(
                            format!("Coordinates cannot be negative: {}", val).into(),
                        ));
                    }
                    x = Some(val);
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected dbclick [x] [y] [delay_ms]", args[0]).into(),
                    ));
                }
            }
            2 => {
                if let Ok(val) = args[0].parse::<i32>() {
                    if val < 0 {
                        return Err(ScriptError::ParseError(
                            format!("Coordinates cannot be negative: {}", val).into(),
                        ));
                    }
                    x = Some(val);
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected dbclick [x] [y] [delay_ms]", args[0]).into(),
                    ));
                }
                if let Ok(val) = args[1].parse::<i32>() {
                    if val < 0 {
                        return Err(ScriptError::ParseError(
                            format!("Coordinates cannot be negative: {}", val).into(),
                        ));
                    }
                    y = Some(val);
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected dbclick [x] [y] [delay_ms]", args[1]).into(),
                    ));
                }
            }
            _ => {
                if let Ok(val) = args[0].parse::<i32>() {
                    if val < 0 {
                        return Err(ScriptError::ParseError(
                            format!("Coordinates cannot be negative: {}", val).into(),
                        ));
                    }
                    x = Some(val);
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected dbclick [x] [y] [delay_ms]", args[0]).into(),
                    ));
                }
                if let Ok(val) = args[1].parse::<i32>() {
                    if val < 0 {
                        return Err(ScriptError::ParseError(
                            format!("Coordinates cannot be negative: {}", val).into(),
                        ));
                    }
                    y = Some(val);
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected dbclick [x] [y] [delay_ms]", args[1]).into(),
                    ));
                }
                if let Ok(val) = args[2].parse::<u32>() {
                    delay_ms = val;
                } else if let Ok(val) = args[2].parse::<i32>() {
                    if val >= 0 {
                        delay_ms = val as u32;
                    }
                }
            }
        }

        if let (Some(x_val), Some(y_val)) = (x, y) {
            if x_val < 0 || y_val < 0 {
                return Err(ScriptError::ParseError(
                    format!("Coordinates cannot be negative: ({}, {})", x_val, y_val).into(),
                ));
            }
        }

        Ok(InstructionData::Custom(Box::new(DbClickParams { x, y, delay_ms })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<DbClickParams>("Invalid dbclick parameters")?;

        // Move to position if coordinates provided
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

        // Execute first click
        let click_inputs = mouse_input::build_click_left();
        mouse_input::execute_inputs(&click_inputs).map_err(|e| {
            ScriptError::ExecutionError(format!("Double-click failed: {:?}", e))
        })?;

        sleep_ms(50);

        // Execute second click
        mouse_input::execute_inputs(&click_inputs).map_err(|e| {
            ScriptError::ExecutionError(format!("Double-click failed: {:?}", e))
        })?;

        if params.delay_ms > 0 {
            sleep_ms(params.delay_ms);
        }

        Ok(())
    }
}
