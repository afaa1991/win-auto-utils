//! Mouse move instruction handler (absolute positioning)
//!
//! Implements the `move` instruction for moving the mouse to an absolute position.

use super::{parse_mouse_mode, MouseMode, MoveParams};
use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Mouse move handler (absolute positioning)
///
/// Syntax: `move <x> <y> [mode]`
///
/// # Mode Priority
/// The effective mode is determined by the following priority (highest to lowest):
/// 1. Explicit mode in instruction (e.g., `move post`)
/// 2. VM context `input_mode` setting (via `ctx.set_state(context_keys::INPUT_MODE, "post")`)
/// 3. Hardcoded default (`Send`)
///
/// # Examples
/// ```text
/// move 100 200           # Move to (100, 200), mode from VM config or default
/// move 100 200 post      # Move to (100, 200) in background (explicit mode)
/// move 100 200 send      # Move to (100, 200) with explicit foreground mode
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
/// # Notes
/// - PostMessage mode sends WM_MOUSEMOVE message to target window
/// - Requires target_hwnd to be set in VM state for PostMessage mode
/// - Coordinates are in window coordinates (relative to window rect)
pub struct MoveHandler;

impl InstructionHandler for MoveHandler {
    fn name(&self) -> &str {
        "move"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.is_empty() || args.len() > 3 {
            return Err(ScriptError::ParseError(
                "Missing coordinates. Usage: move <x> <y> [send|post]".into(),
            ));
        }

        let (mode, mode_offset) = parse_mouse_mode(args, MouseMode::Send)?;
        let coord_args = &args[..args.len() - mode_offset];

        if coord_args.len() != 2 {
            return Err(ScriptError::ParseError(
                "Missing coordinates. Usage: move <x> <y>".into(),
            ));
        }

        let x = coord_args[0].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid x coordinate '{}': {}", coord_args[0], e))
        })?;

        let y = coord_args[1].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid y coordinate '{}': {}", coord_args[1], e))
        })?;

        // Don't pre-build INPUT structure - will be built at execute time with proper coordinate conversion
        Ok(InstructionData::Custom(Box::new(MoveParams {
            x,
            y,
            mode,
            mode_specified: mode_offset > 0,
            send_input: None, // Will be built dynamically in execute phase
        })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<MoveParams>("Invalid move parameters")?;

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
                let (screen_x, screen_y) = {
                    #[cfg(feature = "script_process_context")]
                    {
                        if vm.process.has_hwnd() {
                            super::convert_to_window_coords(vm, params.x, params.y)?
                        } else {
                            (params.x, params.y)
                        }
                    }

                    #[cfg(not(feature = "script_process_context"))]
                    {
                        // Without process_context feature, coordinates are treated as screen coordinates
                        (params.x, params.y)
                    }
                };

                // OPTIMIZATION: Use SetCursorPos instead of SendInput(MOVE) for 22x speedup
                // ~2.2 μs vs ~50 μs per call
                mouse_input::set_cursor_pos(screen_x, screen_y).map_err(|e| {
                    ScriptError::ExecutionError(format!("SetCursorPos failed: {:?}", e))
                })?;
            }
            MouseMode::Post => {
                #[cfg(feature = "script_process_context")]
                {
                    use crate::mouse::mouse_message;

                    let (client_x, client_y) =
                        super::convert_to_client_coords(vm, params.x, params.y)?;
                    mouse_message::post_move_atomic(
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
                         or use SendInput mode (default)."
                            .into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_handler_parse_basic() {
        let handler = MoveHandler;
        let result = handler.parse(&["100", "200"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<MoveParams>().unwrap();
                assert_eq!(params.x, 100);
                assert_eq!(params.y, 200);
                assert_eq!(params.mode, MouseMode::Send);
                assert!(!params.mode_specified);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_move_handler_parse_with_mode() {
        let handler = MoveHandler;
        let result = handler.parse(&["100", "200", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<MoveParams>().unwrap();
                assert_eq!(params.x, 100);
                assert_eq!(params.y, 200);
                assert_eq!(params.mode, MouseMode::Post);
                assert!(params.mode_specified);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_move_handler_parse_invalid() {
        let handler = MoveHandler;
        assert!(handler.parse(&[]).is_err());
        assert!(handler.parse(&["100"]).is_err());
        assert!(handler.parse(&["invalid", "200"]).is_err());
        assert!(handler.parse(&["100", "200", "extra", "args"]).is_err());
    }
}
