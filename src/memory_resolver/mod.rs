//! Memory Address Resolution Module
//!
//! Provides functionality to resolve symbolic memory addresses (e.g., "game.exe+0x123->456")
//! into actual memory addresses in a target process.
//!
//! # Module Responsibility
//! This module is **purely for static address resolution**:
//! - Module base + offset parsing
//! - Pointer chain dereferencing  
//! - Architecture-aware (x86/x64)
//!
//! For AOB pattern scanning, use the separate `memory_aobscan` module.

mod builder;
pub mod resolver;
pub use resolver::{AddressBase, AddressOp, MemoryAddress, ParseError, PointerSize, ResolveError};
