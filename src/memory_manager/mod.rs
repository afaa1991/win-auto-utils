//! Memory Manager Module
//!
//! Provides a unified interface for managing memory modifications (Locks, Hooks, etc.)
//! with support for dynamic address resolution and process context binding.

pub mod builtin;
pub mod manager;
pub mod register;

// Re-export core types
pub use manager::ModifierManager;
pub use register::ModifierHandler;
