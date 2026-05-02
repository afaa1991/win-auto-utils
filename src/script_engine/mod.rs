//! Script Engine - A lightweight, extensible script execution engine.
//!
//! This module provides a complete script interpretation pipeline:
//! 1. **Parse**: Convert script text into structured instructions via `InstructionHandler::parse()`
//! 2. **Compile**: Analyze instructions and establish relationships (optional, via lifecycle hooks)
//! 3. **Execute**: Run compiled instructions in the virtual machine
//!
//! # Architecture
//!
//! The engine follows a three-stage pipeline:
//! - [`parser`]: Tokenizes and parses script text into instruction sequences
//! - [`compiler`]: Resolves labels, validates structure, and optimizes instructions
//! - [`vm`]: Executes compiled instructions with register-based state management
//!
//! # Key Design Principles
//!
//! - **Single Trait Interface**: All instructions implement only `InstructionHandler`
//! - **Progressive Complexity**: Optional lifecycle hooks with default implementations
//! - **Zero Runtime Overhead**: Compile-time analysis, runtime transparency
//! - **Separation of Concerns**: Most handlers are simple (Parse + Execute). Complex instructions use lifecycle hooks for global context analysis.
//! - **Extensibility**: Lifecycle hooks allow any instruction to perform compile-time analysis (e.g., resource validation).
//!
//! # Modules
//!
//! - [`instruction`]: Core trait (`InstructionHandler`) and registry
//! - [`parser`]: Script text parsing and tokenization
//! - [`compiler`]: Instruction compilation and optimization
//! - [`vm`]: Virtual machine for instruction execution
//!
//! For detailed usage examples and comprehensive documentation, see individual submodule documentation.

pub mod compiler;
pub mod engine;
pub mod instruction;
pub mod parser;
pub mod vm;

#[cfg(feature = "script_process_context")]
pub mod process_ctx;

// Re-export main types for convenience
pub use compiler::CompiledScript;
pub use engine::{InterruptController, ScriptConfig, ScriptEngine};
pub use instruction::{
    CompiledInstruction, InstructionData, InstructionHandler, InstructionRegistry, ScriptError,
};
pub use vm::{VMConfig, VMContext, VMRegisters, VM};

#[cfg(feature = "script_process_context")]
pub use process_ctx::ProcessContext;
