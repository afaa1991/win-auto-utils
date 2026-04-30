//! Mouse release left button instruction handler
//!
//! Implements the `release` instruction for releasing the left mouse button.

use super::{parse_mouse_mode, MouseMode, ReleaseParams};
use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Mouse release left button handler
///
/// Syntax: `release [x] [y] [mode]`
///
/// # Parameters
/// - `[x] [y]` (optional): Coordinates for PostMessage mode. If omitted, defaults to (0, 0).
/// - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
///
/// # Mode Priority
/// The effective mode is determined by the following priority (highest to lowest):
/// 1. Explicit mode in instruction (e.g., `release post`)
/// 2. VM context `input_mode` setting (via `set_input_mode`)
/// 3. Hardcoded default (`Send`)
///
/// # Examples
/// ```text
/// release                # Release left button, mode from VM config or default
/// release 100 200 post   # Release at (100, 200) in background
/// release post           # Release at (0, 0) in background (default coordinates)
/// release send           # Release with explicit foreground mode
/// ```
///
/// # Notes
/// - Must be paired with `press` to avoid stuck buttons
/// - In PostMessage mode, coordinates are converted from window to client coordinates
/// - Requires target_hwnd to be set in VM state for PostMessage mode
pub struct ReleaseHandler;

impl InstructionHandler for ReleaseHandler {
    fn name(&self) -> &str {
        "release"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let (mode, mode_offset) = parse_mouse_mode(args, MouseMode::Send)?;
        let coord_args = &args[..args.len() - mode_offset];

        // Parse optional coordinates
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
                "Release requires either 0 or 2 coordinates. Usage: release [x] [y] [mode]".into(),
            ));
        };

        // Validate PostMessage mode requires coordinates
        if mode == MouseMode::Post && (x.is_none() || y.is_none()) {
            return Err(ScriptError::ParseError(
                "PostMessage mode requires coordinates. Usage: release <x> <y> post".into(),
            ));
        }

        // OPTIMIZATION: Pre-build ACTION-ONLY INPUT at parse time
        let send_inputs = if mode == MouseMode::Send {
            vec![mouse_input::build_release_left()]
        } else {
            vec![]
        };

        Ok(InstructionData::Custom(Box::new(ReleaseParams {
            x,
            y,
            mode,
            mode_specified: mode_offset > 0,
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
        let params = data.extract_custom::<ReleaseParams>("Invalid release parameters")?;

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

                // OPTIMIZATION: Use SetCursorPos for movement + pre-built release for execution
                if let Some((screen_x, screen_y)) = screen_coords {
                    // Step 1: Fast cursor positioning using SetCursorPos (~2.2 μs)
                    mouse_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
                        ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
                    })?;

                    // Step 2: Execute release at current position
                    let inputs = vec![mouse_input::build_release_left()];
                    mouse_input::execute_inputs(&inputs).map_err(|e| {
                        ScriptError::ExecutionError(format!("Release failed: {:?}", e))
                    })?;
                } else {
                    // No coordinates provided, just release at current position using pre-built inputs
                    mouse_input::execute_inputs(&params.send_inputs).map_err(|e| {
                        ScriptError::ExecutionError(format!("Release failed: {:?}", e))
                    })?;
                }
            }
            MouseMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    use crate::mouse::mouse_message;
                    
                    // Use provided coordinates or default to (0, 0)
                    let window_x = params.x.unwrap_or(0);
                    let window_y = params.y.unwrap_or(0);
                    // Convert window coordinates to client coordinates for PostMessage
                    let (client_x, client_y) = super::convert_to_client_coords(vm, window_x, window_y)?;
                    mouse_message::post_release_left_atomic(
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
