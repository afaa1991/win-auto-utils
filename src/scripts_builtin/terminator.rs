//! End instruction - Universal block terminator with registration system
//!
//! This module provides a generic `end` instruction that can terminate any block type.
//! Block handlers register their termination logic through the EndHandler registry.
//!
//! # Architecture
//! - **Registration-based**: Instructions register their end-handling logic
//! - **Metadata-driven**: Uses compilation metadata to determine block type
//! - **Extensible**: New block types can be added without modifying end_cmd.rs
//!
//! # Usage
//! ```rust
//! // In your instruction handler initialization:
//! EndHandler::register_handler("my_block", |vm, metadata| {
//!     // Custom termination logic
//!     // Return Ok(next_ip) to continue execution
//! });
//! ```

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::script_engine::compiler::TerminatorMetadata;
use crate::script_engine::instruction::{InstructionData, InstructionHandler, InstructionMetadata, ScriptError};
use crate::script_engine::VMContext;

/// Type alias for block termination handler function
/// 
/// Parameters:
/// - `vm`: The VM context
/// - `metadata`: The terminator metadata from compilation (contains start_ip, block_type, etc.)
/// 
/// Returns:
/// - `Ok(next_ip)`: The next instruction pointer to execute
/// - `Err(error)`: Execution error
type TerminatorHandlerFn = Box<dyn Fn(&mut VMContext, &TerminatorMetadata) -> Result<usize, ScriptError> + Send + Sync>;

/// Global registry for block termination handlers
/// 
/// Note: Uses OnceLock with interior mutability to allow initialization after first use.
/// Registration happens during startup (before any script execution).
static TERMINATOR_REGISTRY: OnceLock<RwLock<HashMap<String, TerminatorHandlerFn>>> = OnceLock::new();

/// Get or initialize the global terminator registry
#[inline]
fn get_registry() -> &'static RwLock<HashMap<String, TerminatorHandlerFn>> {
    TERMINATOR_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// End instruction handler - Universal block terminator
///
/// This handler dispatches to registered termination logic based on block type.
/// It supports mixed nesting of different block types through metadata-driven dispatch.
///
/// # Examples
/// ```text
/// loop 3
///     key A
/// end                    # Dispatches to loop terminator
///
/// time 5000
///     key B
/// end                    # Dispatches to time terminator
/// ```
pub struct TerminatorHandler;

impl TerminatorHandler {
    /// Register a termination handler for a specific block type
    ///
    /// # Arguments
    /// * `block_type` - The block type name (e.g., "loop", "time")
    /// * `handler` - Function that handles termination and returns next IP
    ///
    /// # Example
    /// ```rust
    /// EndHandler::register_handler("my_block", |vm, metadata| {
    ///     // Custom logic based on metadata
    ///     Ok(vm.ip + 1)
    /// });
    /// ```
    pub fn register_handler<F>(block_type: &str, handler: F)
    where
        F: Fn(&mut VMContext, &TerminatorMetadata) -> Result<usize, ScriptError> + Send + Sync + 'static,
    {
        let mut registry = get_registry().write().unwrap();
        registry.insert(block_type.to_string(), Box::new(handler));
    }
}

impl InstructionHandler for TerminatorHandler {
    fn name(&self) -> &str {
        "end"
    }

    fn parse(&self, _args: &[&str]) -> Result<InstructionData, ScriptError> {
        Ok(InstructionData::None)
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        _data: &InstructionData,
        metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        // Get terminator metadata from compilation
        let term_metadata = metadata
            .and_then(|m| m.get::<crate::script_engine::compiler::TerminatorMetadata>())
            .ok_or_else(|| {
                ScriptError::ExecutionError("'end' instruction missing terminator metadata".into())
            })?;

        let block_type = &term_metadata.block_type;

        // Look up registered handler (use read lock for concurrent access)
        let registry = get_registry().read().map_err(|e| {
            ScriptError::ExecutionError(format!("Failed to access terminator registry: {}", e))
        })?;

        if let Some(handler) = registry.get(block_type.as_str()) {
            // Call the registered handler
            let next_ip = handler(vm, term_metadata)?;
            vm.ip = next_ip;
        } else {
            // No handler registered - default behavior: continue after 'end'
            vm.ip += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_end_no_params() {
        let handler = TerminatorHandler;
        let result = handler.parse(&[]).unwrap();
        assert!(matches!(result, InstructionData::None));
    }

    #[test]
    fn test_parse_end_ignores_args() {
        let handler = TerminatorHandler;
        let result = handler.parse(&["extra"]).unwrap();
        assert!(matches!(result, InstructionData::None));
    }
}
