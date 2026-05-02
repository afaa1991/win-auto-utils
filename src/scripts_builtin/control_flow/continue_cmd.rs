//! Continue instruction handler
//!
//! Implements the `continue` instruction for skipping to next iteration.

use super::loop_cmd::{LoopRuntimeState, LOOP_STACK_KEY};

#[cfg(feature = "scripts_timing")]
use crate::scripts_builtin::timing::{TimeRuntimeState, TIME_STACK_KEY};

use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Continue handler - skip to next iteration
///
/// Syntax: `continue`
///
/// Skips the remaining loop body and proceeds to the next iteration.
/// Internally jumps to the 'end' instruction, which handles counter decrement
/// and loop continuation checks, ensuring correct behavior even for single-iteration loops.
///
/// Examples:
/// ```text
/// loop 10
///     key A
///     continue           # Skip remaining code, go to next iteration
///     key B              # This won't execute
/// end
/// ```
pub struct ContinueHandler;

impl InstructionHandler for ContinueHandler {
    fn name(&self) -> &str {
        "continue"
    }

    fn parse(&self, _args: &[&str]) -> Result<InstructionData, ScriptError> {
        Ok(InstructionData::None)
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        _data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        // First, try to continue in a time block (higher priority)
        #[cfg(feature = "scripts_timing")]
        {
            let time_stack =
                vm.get_or_create_execution_state::<Vec<TimeRuntimeState>>(TIME_STACK_KEY);

            if !time_stack.is_empty() {
                let time_state = time_stack.last().unwrap();
                // Jump to the 'end' instruction to let it handle time checking
                vm.ip = time_state.end_ip;
                return Ok(());
            }
        }

        // Not in a time block, try loop stack
        let loop_stack = vm.get_or_create_execution_state::<Vec<LoopRuntimeState>>(LOOP_STACK_KEY);

        if let Some(loop_state) = loop_stack.last() {
            // Jump to the 'end' instruction to let it handle counter decrement and loop continuation check
            // This ensures proper loop semantics even with single-iteration loops
            vm.ip = loop_state.end_ip;
            return Ok(());
        }

        Err(ScriptError::ExecutionError(
            "'continue' instruction used outside of any loop or time block".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_continue_no_params() {
        let handler = ContinueHandler;
        let result = handler.parse(&[]).unwrap();
        assert!(matches!(result, InstructionData::None));
    }
}
