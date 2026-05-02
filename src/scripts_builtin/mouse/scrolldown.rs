//! Mouse scroll down instruction handler
//!
//! Implements the `scrolldown` instruction for scrolling the mouse wheel downward.

use super::{parse_mouse_mode, MouseMode, ScrollParams};
use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Mouse scroll down handler
///
/// Syntax: `scrolldown [x] [y] [times] [mode]`
/// Scrolls the mouse wheel downward by a specified number of times (each time = 120 units = one notch)
/// Coordinates are optional - if omitted, uses current position for SendInput mode
///
/// # Parameters
/// - `[x] [y]`: Optional coordinates for target position
/// - `[times]`: Number of scroll notches (default: 1, range: 1-100)
/// - `[mode]`: Operation mode (send/post)
///
/// # Mode Priority
/// The effective mode is determined by the following priority (highest to lowest):
/// 1. Explicit mode in instruction (e.g., `scrolldown post`)
/// 2. VM context `input_mode` setting (via `ctx.set_persistent_state(context_keys::INPUT_MODE, "post")`)
/// 3. Hardcoded default (`Send`)
///
/// # Examples
/// ```text
/// scrolldown                   # Scroll down 1 notch at current position
/// scrolldown 3                 # Scroll down 3 notches at current position
/// scrolldown 100 200 5         # Scroll down 5 notches at (100, 200)
/// scrolldown 100 200 3 post    # Scroll down 3 notches at (100, 200) in background
/// ```
///
/// # Setting Target Window for PostMessage Mode
/// Before using PostMessage mode, set the target window handle:
/// ```rust,no_run
/// use win_auto_utils::scripts_builtin::context_keys;
/// # let hwnd = 0u32 as windows::Win32::Foundation::HWND;
/// # let mut vm = win_auto_utils::script_engine::vm::VMContext::new();
/// context_keys::set_vmcontext_hwnd(&mut vm, hwnd);
/// ```
///
/// # Error Messages
///
/// ```text
/// scrolldown abc               → "Invalid times 'abc': ..."
/// scrolldown 100 200 abc       → "Invalid times 'abc': ..."
/// scrolldown 100 200 0         → "Scroll times must be between 1 and 100"
/// ```
///
/// # Notes
/// - PostMessage mode requires target_hwnd to be set in VM state
/// - Without coordinates in PostMessage mode, defaults to (0, 0)
/// - PostMessage scroll does NOT move the actual mouse cursor
/// - Each scroll notch = 120 units (Windows standard)
/// - Times parameter controls how many notches to scroll (more intuitive than distance)
pub struct ScrollDownHandler;

impl InstructionHandler for ScrollDownHandler {
    fn name(&self) -> &str {
        "scrolldown"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        let (mode, mode_offset) = parse_mouse_mode(args, MouseMode::Send)?;
        let value_args = &args[..args.len() - mode_offset];

        // Parse coordinates and times
        let (x, y, times) = match value_args.len() {
            0 => (None, None, 1), // Default: 1 notch at current position
            1 => {
                // Could be times only
                let times = value_args[0].parse::<u32>().map_err(|e| {
                    ScriptError::ParseError(format!("Invalid times '{}': {}", value_args[0], e))
                })?;
                if times < 1 || times > 100 {
                    return Err(ScriptError::ParseError(
                        "Scroll times must be between 1 and 100".into(),
                    ));
                }
                (None, None, times)
            }
            2 => {
                // Could be x y or x times - assume x y for backward compatibility
                let x = value_args[0].parse::<i32>().map_err(|e| {
                    ScriptError::ParseError(format!(
                        "Invalid x coordinate '{}': {}",
                        value_args[0], e
                    ))
                })?;
                let y = value_args[1].parse::<i32>().map_err(|e| {
                    ScriptError::ParseError(format!(
                        "Invalid y coordinate '{}': {}",
                        value_args[1], e
                    ))
                })?;
                (Some(x), Some(y), 1)
            }
            3 => {
                // x y times
                let x = value_args[0].parse::<i32>().map_err(|e| {
                    ScriptError::ParseError(format!(
                        "Invalid x coordinate '{}': {}",
                        value_args[0], e
                    ))
                })?;
                let y = value_args[1].parse::<i32>().map_err(|e| {
                    ScriptError::ParseError(format!(
                        "Invalid y coordinate '{}': {}",
                        value_args[1], e
                    ))
                })?;
                let times = value_args[2].parse::<u32>().map_err(|e| {
                    ScriptError::ParseError(format!("Invalid times '{}': {}", value_args[2], e))
                })?;
                if times < 1 || times > 100 {
                    return Err(ScriptError::ParseError(
                        "Scroll times must be between 1 and 100".into(),
                    ));
                }
                (Some(x), Some(y), times)
            }
            _ => {
                return Err(ScriptError::ParseError(
                    "Invalid number of arguments. Usage: scrolldown [x] [y] [times] [mode]".into(),
                ));
            }
        };

        // Validate PostMessage mode requires coordinates if provided
        if mode == MouseMode::Post && x.is_some() && y.is_none() {
            return Err(ScriptError::ParseError(
                "PostMessage scroll requires both x and y coordinates".into(),
            ));
        }

        // OPTIMIZATION: Pre-build SINGLE SCROLL INPUT at parse time (delta=120 fixed)
        // Execute phase will loop N times to achieve N notches - zero runtime construction!
        let send_inputs = if mode == MouseMode::Send {
            vec![mouse_input::build_scroll_down(120)] // Fixed delta, pre-built once
        } else {
            vec![]
        };

        Ok(InstructionData::Custom(Box::new(ScrollParams {
            x,
            y,
            delta: times as i32, // Reuse delta field to store times count
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
        let params = data.extract_custom::<ScrollParams>("Invalid scroll parameters")?;

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
                        use crate::scripts_builtin::mouse::convert_to_window_coords;

                        if vm.process.has_hwnd() {
                            Some(convert_to_window_coords(vm, x, y)?)
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

                // OPTIMIZATION: Use SetCursorPos for movement + pre-built scroll for execution
                if let Some((screen_x, screen_y)) = screen_coords {
                    mouse_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
                        ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
                    })?;

                    // Execute pre-built scroll INPUT N times (zero construction overhead)
                    for _ in 0..params.delta {
                        mouse_input::execute_inputs(&params.send_inputs).map_err(|e| {
                            ScriptError::ExecutionError(format!("Scroll down failed: {:?}", e))
                        })?;
                    }
                } else {
                    // No coordinates: execute pre-built scroll at current position N times
                    for _ in 0..params.delta {
                        mouse_input::execute_inputs(&params.send_inputs).map_err(|e| {
                            ScriptError::ExecutionError(format!("Scroll down failed: {:?}", e))
                        })?;
                    }
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
                    let (client_x, client_y) =
                        super::convert_to_client_coords(vm, window_x, window_y)?;
                    // Execute scroll N times for PostMessage mode
                    for _ in 0..params.delta {
                        mouse_message::post_scroll_down_atomic(
                            vm.process.get_hwnd_or_err()?,
                            client_x,
                            client_y,
                            120,
                        );
                    }
                }

                #[cfg(not(feature = "script_process_context"))]
                {
                    return Err(ScriptError::ExecutionError(
                        "PostMessage mode requires 'script_process_context' feature. \
                         Enable it in Cargo.toml: features = [\"scripts_mouse_with_post\"] \
                         or use SendInput mode (default)."
                            .into(),
                    ));
                }
            }
        }

        Ok(())
    }
}
