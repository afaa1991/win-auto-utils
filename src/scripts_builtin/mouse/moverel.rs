//! Mouse relative move instruction handler
//!
//! Implements the `moverel` instruction for moving the mouse relatively from current position.

use super::{parse_mouse_mode, MouseMode, MoveRelParams};
use crate::mouse::mouse_input;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Mouse relative move handler
///
/// Syntax: `moverel <dx> <dy> [mode]`
///
/// # Mode Priority
/// The effective mode is determined by the following priority (highest to lowest):
/// 1. Explicit mode in instruction (e.g., `moverel post`)
/// 2. VM context `input_mode` setting (via `ctx.set_persistent_state(context_keys::INPUT_MODE, "post")`)
/// 3. Hardcoded default (`Send`)
///
/// # Examples
/// ```text
/// moverel 10 -5          # Move 10px right, 5px up, mode from VM config or default
/// moverel 10 -5 send     # Move relatively with explicit foreground mode
/// ```
///
/// # Implementation Notes
/// - Unlike absolute mouse instructions (`click`, `move`, `scrollup`, `scrolldown`), `moverel` can pre-build
///   the INPUT structure during parsing because it doesn't depend on window client area coordinates.
/// - Relative movements are calculated from the current cursor position, so no HWND offset calculation is needed.
/// - This allows for zero-cost execution with pre-built INPUT structures, unlike other mouse instructions
///   that must calculate coordinates at execution time due to potential window position changes between calls.
///
/// # Notes
/// - Relative movement is primarily designed for SendInput mode
/// - PostMessage mode does not support relative movement and will return an error
/// - Use absolute `move` instruction for PostMessage mode
pub struct MoveRelHandler;

impl InstructionHandler for MoveRelHandler {
    fn name(&self) -> &str {
        "moverel"
    }

    #[inline]
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.is_empty() || args.len() > 3 {
            return Err(ScriptError::ParseError(
                "Missing offsets. Usage: moverel <dx> <dy> [send|post]".into(),
            ));
        }

        let (mode, mode_offset) = parse_mouse_mode(args, MouseMode::Send)?;
        let coord_args = &args[..args.len() - mode_offset];

        if coord_args.len() != 2 {
            return Err(ScriptError::ParseError(
                "Missing offsets. Usage: moverel <dx> <dy>".into(),
            ));
        }

        let dx = coord_args[0].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid dx offset '{}': {}", coord_args[0], e))
        })?;

        let dy = coord_args[1].parse::<i32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid dy offset '{}': {}", coord_args[1], e))
        })?;

        // Pre-build INPUT structure for SendInput mode
        let send_input = if mode == MouseMode::Send {
            Some(mouse_input::build_move_relative(dx, dy))
        } else {
            None
        };

        Ok(InstructionData::Custom(Box::new(MoveRelParams {
            dx,
            dy,
            mode,
            mode_specified: mode_offset > 0,
            send_input,
        })))
    }

    #[inline]
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<MoveRelParams>("Invalid moverel parameters")?;

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
                // Execute pre-built INPUT structure directly (zero overhead)
                if let Some(ref input) = params.send_input {
                    mouse_input::execute_single_input(input).map_err(|e| {
                        ScriptError::ExecutionError(format!("Move relative failed: {:?}", e))
                    })?;
                }
            }
            MouseMode::Post => {
                // PostMessage doesn't support relative movement well
                // For now, we'll skip this or could convert to absolute
                return Err(ScriptError::ExecutionError(
                    "PostMessage mode does not support relative movement. Use absolute 'move' instruction instead.".into()
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moverel_handler_parse_basic() {
        let handler = MoveRelHandler;
        let result = handler.parse(&["10", "-5"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<MoveRelParams>().unwrap();
                assert_eq!(params.dx, 10);
                assert_eq!(params.dy, -5);
                assert_eq!(params.mode, MouseMode::Send);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_moverel_handler_parse_with_mode() {
        let handler = MoveRelHandler;
        let result = handler.parse(&["10", "-5", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<MoveRelParams>().unwrap();
                assert_eq!(params.dx, 10);
                assert_eq!(params.dy, -5);
                assert_eq!(params.mode, MouseMode::Post);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_moverel_handler_parse_invalid() {
        let handler = MoveRelHandler;
        assert!(handler.parse(&[]).is_err());
        assert!(handler.parse(&["10"]).is_err());
        assert!(handler.parse(&["invalid", "5"]).is_err());
    }
}
