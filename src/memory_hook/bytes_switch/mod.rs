//! Bytes Switch Module
//!
//! Provides lightweight bytecode switching for quickly enabling/disabling specific
//! machine instructions at runtime without changing execution flow.
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_hook::BytesSwitch;
//!
//! // Disable a feature with NOP
//! let mut switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
//! switch.enable()?;   // Write NOP
//! switch.disable()?;  // Restore original instructions
//! ```
//!
//! # See Also
//! - [`BytesSwitch`] - Core implementation with automatic state management

pub mod builder;

mod switch;

pub use switch::BytesSwitch;
