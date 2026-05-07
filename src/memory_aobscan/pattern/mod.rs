//! Pattern Module
//!
//! Provides pattern parsing, anchor selection, and pattern representation.
//!
//! # Submodules
//! - **parser**: Parses hex+wildcard pattern strings into structured Pattern
//! - **anchor**: Intelligent anchor selection for heuristic searching

pub mod anchor;
mod parser;

pub use parser::Pattern;
// Note: find_rarest_byte_index is used internally by scanner, not exported publicly
