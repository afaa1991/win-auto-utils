//! TrampolineHook Module
//!
//! Provides trampoline hook functionality that preserves the original function
//! while allowing custom code injection.
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_hook::TrampolineHook;
//!
//! // For x86 process
//! let mut hook = TrampolineHook::new_x86(handle, target_addr, shellcode);
//! hook.install()?;
//!
//! // For x64 process
//! let mut hook = TrampolineHook::new_x64(handle, target_addr, shellcode);
//! hook.install()?;
//! ```
//!
//! # See Also
//! - [`TrampolineHook`] - Core implementation with automatic memory management
//! - [`TrampolineHookBuilder`] - Builder pattern for precise control

mod alloc;
mod hook;
pub mod builder;

pub use hook::TrampolineHook;
