//! Mouse click instruction handler
//!
//! Implements the `click` instruction for performing mouse clicks at specified or current position.

use super::{ClickParams, MouseMode};
use crate::mouse::send_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::utils::sleep_ms;

/// Mouse click handler
///
/// Syntax: `click [x] [y] [delay_ms] [mode]`
/// If coordinates are provided, clicks at that position.
/// If no coordinates in Send mode, clicks at current cursor position.
/// If no coordinates in Post mode, returns an error (coordinates required).
///
/// # Mode Priority
/// The effective mode is determined by the following priority (highest to lowest):
/// 1. Explicit mode in instruction (e.g., `click post`)
/// 2. VM context `input_mode` setting (via `set_input_mode`)
/// 3. Hardcoded default (`Send`)
///
/// # Examples
/// ```text
/// click 100 200              # Click at (100, 200), mode from VM config or default
/// click 100 200 post         # Click at (100, 200) in background (explicit mode)
/// click                      # Click at current position, mode from VM config or default
/// click send                 # Click at current position with explicit foreground mode
/// click 100 200 50           # Click at (100, 200) with 50ms delay after click
/// click 100 200 100 post     # Click at (100, 200) in background with 100ms delay
/// ```
///
/// # Notes
/// - PostMessage mode requires both x and y coordinates
/// - PostMessage mode requires target_hwnd to be set in VM state
/// - SendInput mode without coordinates uses current cursor position
/// - Delay is useful for applications that ignore rapid consecutive clicks
pub struct ClickHandler;

impl InstructionHandler for ClickHandler {
    fn name(&self) -> &str {
        "click"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let (mode, mode_offset) = super::parse_mouse_mode(args, MouseMode::Send)?;
        let remaining_args = &args[..args.len() - mode_offset];

        // Parse optional delay_ms (if present, it should be before mode but after coordinates)
        let (delay_ms, coord_args) = if !remaining_args.is_empty() {
            // Try to parse the last non-mode argument as delay_ms
            let last_arg = remaining_args[remaining_args.len() - 1];
            if last_arg.parse::<u32>().is_ok() {
                // It's a number, could be delay_ms or coordinate
                // Check if we have enough args for coordinates + delay
                if remaining_args.len() >= 3 {
                    // Format: x y delay_ms [mode]
                    let delay = remaining_args[remaining_args.len() - 1]
                        .parse::<u32>()
                        .map_err(|e| {
                            ScriptError::ParseError(format!(
                                "Invalid delay_ms '{}': {}",
                                last_arg, e
                            ))
                        })?;
                    let coord_slice = &remaining_args[..remaining_args.len() - 1];
                    (delay, coord_slice)
                } else {
                    // Not enough args for x y delay, treat as coordinates only
                    (0, remaining_args)
                }
            } else {
                // Not a number, must be coordinates only
                (0, remaining_args)
            }
        } else {
            (0, remaining_args)
        };

        // Parse coordinates if provided
        let (x, y) = if coord_args.len() == 2 {
            let x = coord_args[0].parse::<i32>().map_err(|e| {
                ScriptError::ParseError(format!("Invalid x coordinate '{}': {}", coord_args[0], e))
            })?;
            let y = coord_args[1].parse::<i32>().map_err(|e| {
                ScriptError::ParseError(format!("Invalid y coordinate '{}': {}", coord_args[1], e))
            })?;
            (Some(x), Some(y))
        } else if coord_args.is_empty() {
            (None, None)
        } else {
            return Err(ScriptError::ParseError(
                format!(
                    "Click requires either 0 or 2 coordinates, got {}",
                    coord_args.len()
                )
                .into(),
            ));
        };

        // Validate PostMessage mode requires coordinates
        if mode == MouseMode::Post && (x.is_none() || y.is_none()) {
            return Err(ScriptError::ParseError(
                "PostMessage mode requires coordinates. Usage: click <x> <y> [delay_ms] post"
                    .into(),
            ));
        }

        // OPTIMIZATION: Pre-build ACTION-ONLY INPUT at parse time for zero runtime overhead
        // Strategy: SetCursorPos (execute time) + pre-built action (parse time)
        // This avoids both HashMap lookup AND runtime INPUT construction
        let send_inputs = if mode == MouseMode::Send {
            // Pre-build the action part only (DOWN + UP), movement handled by SetCursorPos
            send_input::build_click_left().to_vec()
        } else {
            vec![] // PostMessage mode doesn't need INPUT
        };

        Ok(InstructionData::Custom(Box::new(ClickParams {
            x,
            y,
            mode,
            mode_specified: mode_offset > 0,
            delay_ms,
            send_inputs,
        })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<ClickParams>("Invalid click parameters")?;

        // Determine effective mode with priority: explicit > input_mode from VM > hardcoded default
        let effective_mode = if params.mode_specified {
            params.mode
        } else {
            match super::get_input_mode(vm).as_str() {
                "post" => MouseMode::Post,
                _ => MouseMode::Send,
            }
        };

        match effective_mode {
            MouseMode::Send => {
                // Pre-extract window offset to avoid closure error propagation issues
                let screen_coords = if let (Some(x), Some(y)) = (params.x, params.y) {
                    #[cfg(feature = "script_process_context")]
                    {
                        if vm.process.has_hwnd() {
                            Some(super::convert_to_window_coords(vm, x, y)?)
                        } else {
                            Some((x, y))
                        }
                    }
                    
                    #[cfg(not(feature = "script_process_context"))]
                    {
                        // Without process_context feature, coordinates are treated as screen coordinates
                        Some((x, y))
                    }
                } else {
                    None
                };

                // OPTIMIZATION: Use SetCursorPos for movement + pre-built click for execution
                if let Some((screen_x, screen_y)) = screen_coords {
                    // Step 1: Fast cursor positioning using SetCursorPos (~2.2 μs)
                    send_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
                        ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
                    })?;

                    // Step 2: Execute pre-built click sequence (zero allocation, zero build)
                    // The INPUT array was pre-built at parse time with relative coordinates
                    send_input::execute_inputs(&params.send_inputs).map_err(|e| {
                        ScriptError::ExecutionError(format!("Click failed: {:?}", e))
                    })?;

                    // Step 3: Apply delay if specified (for applications that ignore rapid clicks)
                    if params.delay_ms > 0 {
                        sleep_ms(params.delay_ms);
                    }
                } else {
                    // No coordinates: use pre-built INPUT from parse phase (zero allocation)
                    send_input::execute_inputs(&params.send_inputs).map_err(|e| {
                        ScriptError::ExecutionError(format!("Click failed: {:?}", e))
                    })?;

                    // Apply delay if specified
                    if params.delay_ms > 0 {
                        sleep_ms(params.delay_ms);
                    }
                }
            }
            MouseMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    use crate::mouse::post_message;
                    // Coordinates are guaranteed to exist in PostMessage mode (validated during parse)
                    let x = params.x.ok_or_else(|| {
                        ScriptError::ExecutionError("PostMessage click requires x coordinate".into())
                    })?;
                    let y = params.y.ok_or_else(|| {
                        ScriptError::ExecutionError("PostMessage click requires y coordinate".into())
                    })?;
                    // Convert window coordinates to client coordinates for PostMessage
                    let (client_x, client_y) = super::convert_to_client_coords(vm, x, y)?;
                    post_message::post_click_left_atomic(
                        vm.process.get_hwnd_or_err()?,
                        client_x,
                        client_y,
                    );
                }

                #[cfg(not(feature = "script_process_context"))]
                {
                    return Err(ScriptError::ExecutionError(
                        "PostMessage mode requires 'script_process_context' feature. \
                         Enable it in Cargo.toml: features = [\"scripts_mouse_with_post\"] \
                         or use SendInput mode (default).".into()
                    ));
                }
            }
        }

        Ok(())
    }
}
