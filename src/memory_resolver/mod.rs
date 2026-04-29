//! Memory Address Resolution Module
//!
//! Provides functionality to resolve symbolic memory addresses (e.g., "game.exe+0x123->456")
//! into actual memory addresses in a target process.

pub mod resolver;
pub mod builder;

pub use resolver::{MemoryAddress, AddressBase, AddressOp, ParseError, ResolveError, PointerSize};
pub use builder::MemoryAddressBuilder;