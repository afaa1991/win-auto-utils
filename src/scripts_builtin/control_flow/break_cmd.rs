//! Break instruction handler
//!
//! Implements the `break` instruction for exiting loops and time blocks early.

use super::loop_cmd::{LoopRuntimeState, LOOP_STACK_KEY};

#[cfg(feature = "scripts_timing")]
use crate::scripts_builtin::timing::{TimeRuntimeState, TIME_STACK_KEY};

use crate::script_engine::instruction::{InstructionData, InstructionHandler, InstructionMetadata, ScriptError};
use crate::script_engine::VMContext;

/// Break handler - exit current loop or time block
///
/// Syntax: `break`
/// Jumps to the instruction after the 'end' of the current loop/time block.
///
/// Examples:
/// ```text
/// loop 10
///     key A
///     break              # Exit loop immediately
/// end
///
/// time 5s
///     key B
///     break              # Exit time block immediately
/// end
/// ```
pub struct BreakHandler;

impl InstructionHandler for BreakHandler {
    fn name(&self) -> &str {
        "break"
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
        // First, try to break out of a time block (higher priority)
        #[cfg(feature = "scripts_timing")]
        {
            let time_stack = vm.get_or_create_execution_state::<Vec<TimeRuntimeState>>(TIME_STACK_KEY);
            
            if !time_stack.is_empty() {
                let time_state = time_stack.pop().unwrap();
                
                // Cleanup block context
                if !vm.is_block_context_empty() {
                    vm.leave_block();
                }
                
                // Jump to instruction after 'end'
                vm.ip = time_state.end_ip + 1;
                return Ok(());
            }
        }
        
        // Not in a time block, try loop stack
        let loop_stack = vm.get_or_create_execution_state::<Vec<LoopRuntimeState>>(LOOP_STACK_KEY);

        if let Some(loop_state) = loop_stack.pop() {
            // Jump to instruction after 'end'
            vm.ip = loop_state.end_ip + 1;
        } else {
            return Err(ScriptError::ExecutionError(
                "'break' outside of loop or time block".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_break_no_params() {
        let handler = BreakHandler;
        let result = handler.parse(&[]).unwrap();
        assert!(matches!(result, InstructionData::None));
    }
}
