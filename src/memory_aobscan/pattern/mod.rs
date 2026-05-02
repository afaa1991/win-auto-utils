//! Pattern module for AOB scanning
//!
//! Provides pattern parsing, anchor selection, and related functionality.

pub(crate) mod anchor;
mod parser;

pub use parser::Pattern;
// Note: find_rarest_byte_index is used internally by scanner, not exported publicly
