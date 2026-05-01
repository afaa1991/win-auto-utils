//! Register interfaces for memory modifiers

use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;

/// Unified handler interface for all memory modifiers
pub trait ModifierHandler: Send {
    /// Get the name of this handler
    fn name(&self) -> &str;
    
    /// Activate the modifier with runtime context
    fn activate(&mut self, handle: HANDLE, pid: u32) -> Result<(), MemoryError>;
    
    /// Deactivate the modifier (stop but keep state for reuse)
    fn deactivate(&mut self) -> Result<(), MemoryError>;
    
    /// Check if currently active
    fn is_active(&self) -> bool;
}
