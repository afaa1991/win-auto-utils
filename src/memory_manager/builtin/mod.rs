//! Built-in modifier handlers
//!
//! Provides ready-to-use handler implementations for common memory modification scenarios.
//!
//! # Available Handlers
//! - [`LockHandler`] - Memory locking with automatic value restoration
//! - [`BytesSwitchHandler`] - Bytecode switching (NOP patches, etc.)
//! - [`TrampolineHookHandler`] - Function hooking with trampoline support
//!
//! # Address Resolution Modes
//!
//! ## Static Addresses (LockHandler, BytesSwitchHandler)
//! Use module-relative addresses and pointer chains. Architecture is auto-detected.
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::LockHandler;
//! use std::time::Duration;
//!
//! // Simple module + offset
//! let lock = LockHandler::new_lock(
//!     "health_lock",
//!     "game.exe+1000",
//!     100i32,
//!     Duration::from_millis(100),
//! )?;
//!
//! // Multi-level pointer chain
//! let lock2 = LockHandler::new_lock(
//!     "ammo_lock",
//!     "game.exe+5000->2FC->30",
//!     999i32,
//!     Duration::from_millis(50),
//! )?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! ## AOB Pattern Scanning (TrampolineHookHandler - Recommended!)
//! Use byte patterns that resolve at runtime. Most robust across game updates.
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
//!
//! let hook = TrampolineHookHandler::new_hook_aob(
//!     "skill_hook",
//!     "48 89 5C 24",  // AOB pattern
//!     vec![0x90, 0x90],
//!     5,
//! )?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_manager::builtin::{LockHandler, BytesSwitchHandler, TrampolineHookHandler};
//! use std::time::Duration;
//!
//! // Method 1: Static address lock
//! let lock_handler = LockHandler::new_lock(
//!     "health_lock",
//!     "game.exe+1000->2FC",
//!     vec![0x64, 0x00, 0x00, 0x00],
//!     Duration::from_millis(100),
//! )?;
//!
//! // Method 2: NOP patch with static address
//! let nop_handler = BytesSwitchHandler::new_nop_switch(
//!     "disable_feature",
//!     "game.exe+2000",
//!     6,
//! )?;
//!
//! // Method 3: Trampoline hook with AOB (recommended for hooks!)
//! let shellcode_bytes = vec![0x90, 0x90];
//! let hook_handler = TrampolineHookHandler::new_hook_aob(
//!     "skill_hook",
//!     "48 89 5C 24",  // AOB pattern - works across updates!
//!     shellcode_bytes,
//!     6,
//! )?;
//!
//! // Performance tip: All handlers cache resolved addresses automatically!
//! // First activate: resolves address (AOB scan or pattern parse)
//! // Subsequent activates: uses cached address (fast!)
//! // Cache is invalidated when PID changes
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod address_strategy;
pub mod bytes_switch;
pub mod lock;
pub mod trampoline_hook;

// Re-export handlers and configs
pub use address_strategy::AddressStrategy;
pub use bytes_switch::{BytesSwitchConfig, BytesSwitchHandler};
pub use lock::{LockConfig, LockHandler};
pub use trampoline_hook::{TrampolineHookConfig, TrampolineHookHandler};
