//! Timing instruction handlers
//!
//! Provides timing control instructions for script execution:
//! - `sleep`: Pause execution for specified duration
//! - `time`: Execute body for a specified total duration
//!
//! # Instructions
//!
//! ## `sleep` - Delay execution
//!
//! Pauses script execution for the specified number of milliseconds.
//!
//! **Syntax:** `sleep <milliseconds>`
//!
//! **Parameters:**
//! - `<milliseconds>` (required): Duration to sleep (must be >= 0)
//!
//! **Examples:**
//! ```text
//! sleep 100              # Pause for 100ms
//! sleep 1000             # Pause for 1 second
//!
//! # Common use case: Add delay between actions
//! key A
//! sleep 50               # Wait 50ms before next action
//! key B
//! ```
//!
//! **Notes:**
//! - Uses optimized `sleep_ms` utility for high-frequency calls
//! - Actual sleep duration may vary slightly due to OS scheduling
//! - Recommended for rate limiting or waiting for UI updates
//!
//! # Error Messages
//!
//! ```text
//! sleep                  → "Missing duration. Usage: sleep <ms>"
//! sleep abc              → "Invalid duration 'abc': ..."
//! ```

use crate::script_engine::compiler::GenericBlockMetadata;
use crate::script_engine::instruction::{
    BlockStructure, InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;
use crate::scripts_builtin::terminator::TerminatorHandler;
use crate::utils::sleep_ms;
use std::time::Instant;

/// Time runtime state stored in VM execution state
#[derive(Debug)]
pub struct TimeRuntimeState {
    /// Total duration in milliseconds
    pub total_duration_ms: u32,
    /// When the time block started
    pub start_time: Instant,
    /// Start IP of the time body (first instruction after 'time')
    pub body_start_ip: usize,
    /// End IP of the time block (the 'end' instruction)
    pub end_ip: usize,
}

/// Stack key for time state management
pub const TIME_STACK_KEY: &str = "__builtin_time_stack";

/// Initialize time terminator registration (call once at startup)
pub fn init_time_terminator() {
    use crate::script_engine::compiler::TerminatorMetadata;

    TerminatorHandler::register_handler(
        "time",
        |vm: &mut VMContext, _metadata: &TerminatorMetadata| {
            // Get time stack from VM execution state
            let time_stack =
                vm.get_or_create_execution_state::<Vec<TimeRuntimeState>>(TIME_STACK_KEY);

            // Check the current time state (peek without popping)
            if let Some(time_state) = time_stack.last() {
                // Check if time has elapsed
                let elapsed_ms = time_state.start_time.elapsed().as_millis() as u32;

                if elapsed_ms < time_state.total_duration_ms {
                    // Time not yet elapsed, jump back to body start
                    Ok(time_state.body_start_ip)
                } else {
                    // Time elapsed, pop and continue after 'end'
                    time_stack.pop();
                    Ok(vm.ip + 1)
                }
            } else {
                Err(ScriptError::ExecutionError(
                    "Time end without active time block".into(),
                ))
            }
        },
    );
}

/// Sleep handler
///
/// Syntax: `sleep <milliseconds>`
///
/// # Examples
/// ```text
/// sleep 100      # Delay execution for 100 milliseconds
/// sleep 1000     # Delay execution for 1 second
/// ```
pub struct SleepHandler;

impl InstructionHandler for SleepHandler {
    fn name(&self) -> &str {
        "sleep"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.is_empty() {
            return Err(ScriptError::ParseError(
                "Missing duration. Usage: sleep <ms>".into(),
            ));
        }

        let ms = args[0].parse::<u32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid duration '{}': {}", args[0], e))
        })?;

        Ok(InstructionData::U32(ms))
    }

    #[inline]
    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let ms = match data {
            InstructionData::U32(duration) => *duration,
            _ => return Err(ScriptError::ExecutionError("Invalid sleep duration".into())),
        };

        // Use optimized utility function for high-frequency calls
        sleep_ms(ms);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sleep_handler_parse_valid() {
        let handler = SleepHandler;
        let result = handler.parse(&["100"]).unwrap();
        match result {
            InstructionData::U32(ms) => assert_eq!(ms, 100),
            _ => panic!("Expected U32"),
        }
    }

    #[test]
    fn test_sleep_handler_parse_invalid() {
        let handler = SleepHandler;
        assert!(handler.parse(&[]).is_err());
        assert!(handler.parse(&["abc"]).is_err());
    }

    #[test]
    fn test_sleep_handler_execution() {
        let handler = SleepHandler;
        let mut vm = VMContext::new();

        let start = Instant::now();
        handler
            .execute(&mut vm, &InstructionData::U32(50), None)
            .unwrap();
        let elapsed = start.elapsed();

        // Should sleep for at least 50ms (with some tolerance)
        assert!(elapsed.as_millis() >= 50);
        assert!(elapsed.as_millis() < 100); // But not too long
    }
}

/// Time handler - Execute body for a specified total duration
///
/// Syntax: `time <total_milliseconds>`
///
/// This instruction creates a timed execution block that repeatedly executes
/// its body until the specified total duration has elapsed.
///
/// # Examples
/// ```text
/// # Perform actions for 5 seconds
/// time 5000
///     key A
///     sleep 100
/// end
///
/// # Search with timeout
/// time 3000
///     find_color 0xFF0000
///     if_found
///         break
///     end
///     sleep 50
/// end
/// ```
pub struct TimeHandler;

impl InstructionHandler for TimeHandler {
    fn name(&self) -> &str {
        "time"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.is_empty() {
            return Err(ScriptError::ParseError(
                "Missing duration. Usage: time <ms>".into(),
            ));
        }

        let ms = args[0].parse::<u32>().map_err(|e| {
            ScriptError::ParseError(format!("Invalid duration '{}': {}", args[0], e))
        })?;

        if ms == 0 {
            return Err(ScriptError::ParseError(
                "Time duration must be greater than 0".into(),
            ));
        }

        Ok(InstructionData::U32(ms))
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let total_duration_ms = match data {
            InstructionData::U32(duration) => *duration,
            _ => return Err(ScriptError::ExecutionError("Invalid time duration".into())),
        };

        // Get pre-computed end_ip from metadata
        let end_ip = metadata
            .and_then(|m| m.get::<GenericBlockMetadata>())
            .map(|m| m.end_ip)
            .ok_or_else(|| {
                ScriptError::ExecutionError("Time instruction missing metadata".into())
            })?;

        let body_start_ip = vm.ip + 1; // First instruction after 'time'

        // Push time state to execution state stack
        let time_stack = vm.get_or_create_execution_state::<Vec<TimeRuntimeState>>(TIME_STACK_KEY);
        time_stack.push(TimeRuntimeState {
            total_duration_ms,
            start_time: Instant::now(),
            body_start_ip,
            end_ip,
        });

        Ok(())
    }

    /// Declare block structure for automatic pairing
    fn declare_block_structure(&self) -> Option<BlockStructure> {
        Some(BlockStructure::SimplePair {
            start_name: "time",
            end_name: "end",
        })
    }
}

#[cfg(test)]
mod time_tests {
    use super::*;

    #[test]
    fn test_time_handler_parse_valid() {
        let handler = TimeHandler;
        let result = handler.parse(&["5000"]).unwrap();
        match result {
            InstructionData::U32(ms) => assert_eq!(ms, 5000),
            _ => panic!("Expected U32"),
        }
    }

    #[test]
    fn test_time_handler_parse_zero() {
        let handler = TimeHandler;
        assert!(handler.parse(&["0"]).is_err());
    }

    #[test]
    fn test_time_handler_parse_invalid() {
        let handler = TimeHandler;
        assert!(handler.parse(&[]).is_err());
        assert!(handler.parse(&["abc"]).is_err());
    }
}
