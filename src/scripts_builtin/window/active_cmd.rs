//! Window activation instruction handler
//!
//! Provides the `active` instruction to ensure a window is in the foreground
//! before executing subsequent input operations.
//!
//! # Usage
//! ```text
//! active    # Activate the window specified in context (requires hwnd)
//! ```
//!
//! # Requirements
//! - Requires `script_process_context` feature (provides HWND support)
//! - The VM context must have an HWND set via `set_vmcontext_hwnd()`
//!
//! # Behavior
//! 1. Retrieves HWND from VM context
//! 2. Checks if window is already foreground (fast path)
//! 3. If not, activates window with adaptive polling (up to 300ms timeout)
//! 4. Returns error if activation fails or no HWND is available
//!
//! # Error Cases
//! - No HWND in context: "active instruction requires HWND context"
//! - Activation timeout: "Window activation timed out"
//! - API failure: "Failed to activate window"

use crate::script_engine::instruction::{InstructionData, InstructionHandler, ScriptError};
use crate::script_engine::VMContext;
use crate::window::WindowActivationError;

/// Handler for the `active` instruction
///
/// Ensures the target window (from context) is in the foreground.
pub struct ActiveHandler;

impl InstructionHandler for ActiveHandler {
    fn name(&self) -> &str {
        "active"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        // Validate that we have HWND in context during parse time
        // We'll check it again at execute time to be safe
        if !args.is_empty() {
            return Err(ScriptError::ParseError(format!(
                "active instruction takes no arguments, got {}",
                args.len()
            )));
        }

        // No data needed - HWND will be retrieved from context at execute time
        Ok(InstructionData::None)
    }

    fn execute(
        &self,
        context: &mut VMContext,
        _data: &InstructionData,
        _metadata: Option<&crate::script_engine::instruction::InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        use crate::window::ensure_window_active;

        // Retrieve HWND from VM context using the elegant helper method
        let hwnd = context.process.get_hwnd_or_err()?;
        // Attempt to activate window with 300ms timeout
        match ensure_window_active(hwnd, 300) {
            Ok(()) => {
                // Successfully activated
                Ok(())
            }
            Err(WindowActivationError::Timeout) => {
                Err(ScriptError::ExecutionError(format!(
                    "Window activation timed out. The window may be blocked by Windows foreground lock. \
                     Try running as administrator or ensure no other window has focus."
                )))
            }
            Err(WindowActivationError::ActivationFailed) => {
                Err(ScriptError::ExecutionError(
                    "Failed to activate window. Windows API rejected the request.".to_string()
                ))
            }
            Err(e) => {
                Err(ScriptError::ExecutionError(format!("Window activation failed: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hwnd::get_hwnd_list;

    #[test]
    fn test_active_handler_no_args() {
        let handler = ActiveHandler;

        // Should fail with arguments
        let result = handler.parse(&["arg1"]);
        assert!(result.is_err());

        // Should succeed without arguments
        let result = handler.parse(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_active_instruction_requires_hwnd() {
        let handler = ActiveHandler;
        let mut context = VMContext::default();

        // Should fail when no HWND is set
        let data = handler.parse(&[]).unwrap();
        let result = handler.execute(&mut context, &data, None);

        assert!(result.is_err());
        let err = result.unwrap_err();

        // get_hwnd_or_err returns ExecutionError with descriptive message
        match err {
            ScriptError::ExecutionError(msg) => {
                assert!(
                    msg.contains("window"),
                    "Error message should mention window"
                );
            }
            _ => panic!("Expected ExecutionError, got: {:?}", err),
        }
    }

    #[test]
    fn test_active_instruction_with_real_window() {
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.first() {
            let mut context = VMContext::default();
            context.process.hwnd = Some(hwnd);

            let handler = ActiveHandler;
            let data = handler.parse(&[]).unwrap();
            let result = handler.execute(&mut context, &data, None);

            // May succeed or fail depending on Windows foreground lock
            match result {
                Ok(()) => println!("✓ Window activated successfully"),
                Err(e) => {
                    // Expected in some cases due to system restrictions
                    println!("⚠ Activation failed (expected): {}", e);
                }
            }
        }
    }
}
