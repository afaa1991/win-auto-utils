//! Mouse scroll up instruction handler
//!
//! Implements the `scrollup` instruction for scrolling the mouse wheel upward.
//!
//! # Execution Behavior
//! Moves cursor to (x, y) if coordinates provided, then scrolls the wheel up by the specified number of notches.
//! Each notch scrolls 120 units (one "line" by Windows conventions).
//!
//! # Coordinate System
//!
//! - **Without hwnd**: Coordinates are absolute screen coordinates
//! - **With hwnd set**: Coordinates are window-relative, converted to screen coordinates by adding window_left and window_top
//!
//! # Syntax
//! ```text
//! scrollup [x] [y] [times]
//! ```
//!
//! # Arguments
//! - `x` (optional): X coordinate to move cursor before scrolling
//! - `y` (optional): Y coordinate to move cursor before scrolling
//! - `times` (optional): Number of notches to scroll. Default is 1. Valid range: 1-100
//!
//! # Examples
//! ```text
//! scrollup                    # Scroll up 1 notch at current position
//! scrollup 3                  # Scroll up 3 notches at current position
//! scrollup 100 200            # Move to (100, 200) then scroll up 1 notch
//! scrollup 100 200 5          # Move to (100, 200) then scroll up 5 notches
//! ```
//!
//! # Errors
//! - Negative coordinates are rejected during parse
//! - Times must be between 1 and 100

use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

#[derive(Clone)]
pub struct ScrollParams {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub times: u32,
    pub scroll_input: INPUT,
}

pub struct ScrollUpHandler;

impl InstructionHandler for ScrollUpHandler {
    fn name(&self) -> &str {
        "scrollup"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        let mut times: u32 = 1;

        match args.len() {
            0 => {}
            1 => {
                if let Ok(val) = args[0].parse::<u32>() {
                    times = val;
                } else if let Ok(val) = args[0].parse::<i32>() {
                    if val >= 0 {
                        times = val as u32;
                    } else {
                        return Err(ScriptError::ParseError(
                            "Scroll times must be positive".into(),
                        ));
                    }
                } else {
                    return Err(ScriptError::ParseError(
                        format!("Invalid argument '{}'. Expected: scrollup [x] [y] [times]", args[0]).into(),
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
                        format!("Invalid argument '{}'. Expected: scrollup [x] [y] [times]", args[0]).into(),
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
                        format!("Invalid argument '{}'. Expected: scrollup [x] [y] [times]", args[1]).into(),
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
                        format!("Invalid argument '{}'. Expected: scrollup [x] [y] [times]", args[0]).into(),
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
                        format!("Invalid argument '{}'. Expected: scrollup [x] [y] [times]", args[1]).into(),
                    ));
                }
                if let Ok(val) = args[2].parse::<u32>() {
                    times = val;
                } else if let Ok(val) = args[2].parse::<i32>() {
                    if val >= 0 {
                        times = val as u32;
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

        if times < 1 || times > 100 {
            return Err(ScriptError::ParseError(
                "Scroll times must be between 1 and 100".into(),
            ));
        }

        Ok(InstructionData::Custom(Box::new(ScrollParams {
            x,
            y,
            times,
            scroll_input: mouse_input::build_scroll_up(120),
        })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<ScrollParams>("Invalid scroll parameters")?;

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

        for _ in 0..params.times {
            mouse_input::execute_single_input(&params.scroll_input).map_err(|e| {
                ScriptError::ExecutionError(format!("Scroll up failed: {:?}", e))
            })?;
        }

        Ok(())
    }
}
