//! Memory Manager Module
//!
//! Provides a unified interface for managing memory modifications (Locks, Hooks, etc.)
//! with support for dynamic address resolution and process context binding.

pub mod manager;
pub mod register;
pub mod builtin;

// Re-export core types
pub use manager::ModifierManager;
pub use register::ModifierHandler;
