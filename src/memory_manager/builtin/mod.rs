//! Built-in modifier handlers
//!
//! Provides ready-to-use handler implementations for common memory modification scenarios.
//!
//! # Available Handlers
//! - [`LockHandler`] - Memory locking with automatic value restoration
//! - [`BytesSwitchHandler`] - Bytecode switching (NOP patches, etc.)
//! - [`TrampolineHookHandler`] - Function hooking with trampoline support
//!
//! # Address Resolution
//! All handlers support two address resolution modes via `AddressSource`:
//! 
//! ## Static Addresses (Traditional)
//! Use module-relative addresses or pointer chains:
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::LockHandler;
//! use win_auto_utils::memory_resolver::{AddressSource, MemoryAddress};
//! use std::time::Duration;
//!
//! let handler = LockHandler::new_lock_typed(
//!     "health_lock".to_string(),
//!     AddressSource::from_static(MemoryAddress::new_x86("game.exe+1000")?),
//!     100i32,
//!     Duration::from_millis(100),
//! );
//! ```
//!
//! ## AOB Pattern Scanning (Dynamic)
//! Use byte patterns that resolve at runtime:
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::LockHandler;
//! use std::time::Duration;
//!
//! let handler = LockHandler::new_lock_aob_typed(
//!     "health_lock".to_string(),
//!     "48 89 5C ?? 48 89",  // Pattern to scan for
//!     100i32,
//!     Duration::from_millis(100),
//! )?;
//! ```
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::{LockHandler, BytesSwitchHandler, TrampolineHookHandler};
//! use win_auto_utils::memory_resolver::{AddressSource, MemoryAddress};
//! use std::time::Duration;
//!
//! // Method 1: Static address
//! let lock_handler = LockHandler::new_lock(
//!     "health_lock".to_string(),
//!     AddressSource::from_static(MemoryAddress::new_x86("game.exe+1000")?),
//!     vec![0x64, 0x00, 0x00, 0x00],
//!     Duration::from_millis(100),
//! );
//!
//! // Method 2: AOB pattern scanning
//! let nop_handler = BytesSwitchHandler::new_nop_switch_aob(
//!     "disable_feature".to_string(),
//!     "90 90 90 90 90 90",  // NOP pattern
//!     6,
//! )?;
//!
//! // Method 3: Trampoline hook with AOB
//! let hook_handler = TrampolineHookHandler::new_x64_hook_aob(
//!     "skill_hook".to_string(),
//!     "48 89 5C 24",  // Function prologue pattern
//!     shellcode_bytes,
//!     6,
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod lock;
pub mod bytes_switch;
pub mod trampoline_hook;

// Re-export handlers and configs
pub use lock::{LockConfig, LockHandler};
pub use bytes_switch::{BytesSwitchConfig, BytesSwitchHandler};
pub use trampoline_hook::{TrampolineHookConfig, TrampolineHookHandler};
