//! Register interfaces for memory modifiers

use super::manager::ProcessContext;
use crate::memory::MemoryError;

/// Unified handler interface for all memory modifiers
pub trait ModifierHandler: Send {
    /// Get the name of this handler
    fn name(&self) -> &str;

    /// Activate the modifier with process context
    /// All handlers should use this unified interface to get handle, pid, and architecture
    fn activate(&mut self, ctx: &ProcessContext) -> Result<(), MemoryError>;

    /// Deactivate the modifier (stop but keep state for reuse)
    fn deactivate(&mut self) -> Result<(), MemoryError>;

    /// Check if currently active
    fn is_active(&self) -> bool;
}
