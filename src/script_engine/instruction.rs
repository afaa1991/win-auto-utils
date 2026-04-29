//! Linear instruction set definitions
//!
//! Defines the core instruction format for the script engine's plugin-based architecture.
//! Specific instruction logic is registered externally via the `InstructionHandler` trait.
//! This module provides essential base structures and traits.
//!
//! # Architecture
//! The script engine uses a three-phase execution model:
//! 1. **Parse**: Convert text to structured data via `InstructionHandler::parse()`
//! 2. **Compile**: Analyze instructions and establish relationships (optional, via `CompilationAwareHandler`)
//! 3. **Execute**: Run instructions in the VM with pre-computed metadata
//!
//! # Instruction Data Types
//! The `InstructionData` enum supports multiple parameter types:
//! - `None`: No parameters
//! - `U32`, `U64`, `I32`: Numeric parameters
//! - `String`: Text parameters
//! - `Custom`: Custom types for complex parameters (recommended)
//!
//! # Example
//! ```no_run
//! use win_auto_utils::script_engine::instruction::{InstructionData, CompiledInstruction};
//!
//! // Create an instruction with custom data (recommended approach)
//! #[derive(Debug)]
//! struct MyKeyConfig {
//!     key_name: String,
//! }
//!
//! let instr = CompiledInstruction {
//!     name: "key".to_string(),
//!     data: InstructionData::custom(MyKeyConfig {
//!         key_name: "A".to_string(),
//!     }),
//!     metadata: None,
//! };
//!
//! println!("Instruction: {}", instr);
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::script_engine::VMContext;

/// Instruction metadata wrapper
///
/// Provides type-safe access to compilation lifecycle metadata.
/// This encapsulates the raw `Any` type to prevent accidental misuse.
#[derive(Debug)]
pub struct InstructionMetadata {
    inner: Box<dyn Any + Send + Sync>,
}

impl InstructionMetadata {
    /// Create new metadata from any type
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }

    /// Try to downcast to a specific type
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref::<T>()
    }

    /// Try to downcast to a specific type (mutable)
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.inner.downcast_mut::<T>()
    }

    /// Check if metadata contains a specific type
    pub fn is<T: Any>(&self) -> bool {
        self.inner.is::<T>()
    }
}

/// Instruction data container
///
/// Stores parsed instruction parameters. The type is determined by the specific Handler.
#[derive(Debug)]
pub enum InstructionData {
    /// No parameters
    None,
    /// Simple numeric parameters
    U32(u32),
    U64(u64),
    I32(i32),
    /// String parameter
    String(String),
    /// Custom type (recommended for complex parameters)
    Custom(Box<dyn Any + Send + Sync>),
}

impl InstructionData {
    // === 便捷构造方法 ===

    /// Create None data
    #[inline]
    pub fn none() -> Self {
        InstructionData::None
    }

    /// Create U32 data
    #[inline]
    pub fn u32(value: u32) -> Self {
        InstructionData::U32(value)
    }

    /// Create U64 data
    #[inline]
    pub fn u64(value: u64) -> Self {
        InstructionData::U64(value)
    }

    /// Create I32 data
    #[inline]
    pub fn i32(value: i32) -> Self {
        InstructionData::I32(value)
    }

    /// Create String data
    #[inline]
    pub fn string<S: Into<String>>(value: S) -> Self {
        InstructionData::String(value.into())
    }

    /// Create Custom data (automatically boxes the value)
    #[inline]
    pub fn custom<T: Any + Send + Sync>(value: T) -> Self {
        InstructionData::Custom(Box::new(value))
    }

    /// Try to get a reference to the custom data with type checking
    ///
    /// Returns `None` if the data is not `Custom` or the type doesn't match.
    pub fn get_custom_ref<T: Any>(&self) -> Option<&T> {
        if let InstructionData::Custom(boxed) = self {
            boxed.downcast_ref::<T>()
        } else {
            None
        }
    }

    /// Try to get a mutable reference to the custom data with type checking
    ///
    /// Returns `None` if the data is not `Custom` or the type doesn't match.
    pub fn get_custom_mut<T: Any>(&mut self) -> Option<&mut T> {
        if let InstructionData::Custom(boxed) = self {
            boxed.downcast_mut::<T>()
        } else {
            None
        }
    }

    /// Check if the data contains a specific custom type
    pub fn is_custom_type<T: Any>(&self) -> bool {
        if let InstructionData::Custom(boxed) = self {
            boxed.is::<T>()
        } else {
            false
        }
    }

    /// Extract custom data with error handling (convenience method for execute phase)
    ///
    /// This is a convenience wrapper that combines pattern matching and error creation.
    /// Use this in instruction handlers to reduce boilerplate code.
    ///
    /// # Example
    /// ```rust
    /// // Instead of:
    /// let params = match data {
    ///     InstructionData::Custom(boxed) => boxed
    ///         .downcast_ref::<MyParams>()
    ///         .ok_or_else(|| ScriptError::ExecutionError("Invalid params".into()))?,
    ///     _ => return Err(ScriptError::ExecutionError("Invalid data type".into())),
    /// };
    ///
    /// // Use:
    /// let params = data.extract_custom::<MyParams>("Invalid params")?;
    /// ```
    pub fn extract_custom<T: Any>(&self, error_msg: &str) -> Result<&T, ScriptError> {
        self.get_custom_ref::<T>()
            .ok_or_else(|| ScriptError::ExecutionError(error_msg.into()))
    }
}

/// Compiled linear instruction
///
/// Each instruction contains a handler name and pre-parsed data.
#[derive(Debug)]
pub struct CompiledInstruction {
    /// Instruction name (for debugging)
    pub name: String,
    /// Parsed data
    pub data: InstructionData,
    /// Metadata from compilation lifecycle (e.g., end_id for loop/end mapping)
    pub metadata: Option<InstructionMetadata>,
}

impl fmt::Display for CompiledInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Script error
#[derive(Debug)]
pub enum ScriptError {
    ParseError(String),
    CompilationError(String),
    ExecutionError(String),
    Interrupted(String),
    InvalidLoopStructure,
    NotImplemented,
    NoWindowHandle,
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Self::Interrupted(msg) => write!(f, "Interrupted: {}", msg),
            Self::InvalidLoopStructure => write!(f, "Invalid loop structure"),
            Self::NotImplemented => write!(f, "Not implemented"),
            Self::NoWindowHandle => write!(f, "No window handle available"),
        }
    }
}

impl std::error::Error for ScriptError {}

/// The core trait that all instruction handlers must implement.
///
/// This trait defines the essential lifecycle of an instruction:
/// 1. **Parse**: Convert script text to structured data (compile-time)
/// 2. **Execute**: Perform the actual operation (run-time)
/// 3. **Optional Hooks**: Customize compilation behavior (optional, with default implementations)
///
/// # Progressive Complexity
///
/// ## Level 1: Simple Instructions (Most Common - 90%)
/// For most instructions (keyboard, mouse, timing), you only need to implement
/// the three required methods:
///
/// ```rust,no_run
/// impl InstructionHandler for KeyHandler {
///     fn name(&self) -> &str { "key" }
///     fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> { ... }
///     fn execute(&self, vm: &mut VMContext, data: &InstructionData, metadata: Option<&InstructionMetadata>) -> Result<(), ScriptError> { ... }
/// }
/// ```
///
/// ## Level 2: Block Instructions (Control Flow - 5%)
/// If your instruction has a start/end structure (like `loop...end` or `find...endfind`),
/// override `declare_block_structure()` to enable **automatic block pairing**:
///
/// ```rust,no_run
/// impl InstructionHandler for LoopHandler {
///     fn name(&self) -> &str { "loop" }
///     fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> { ... }
///     fn execute(&self, vm: &mut VMContext, data: &InstructionData, metadata: Option<&InstructionMetadata>) -> Result<(), ScriptError> { ... }
///     
///     // Enable automatic block pairing
///     fn declare_block_structure(&self) -> Option<BlockStructure> {
///         Some(BlockStructure::SimplePair {
///             start_name: "loop",
///             end_name: "end",
///         })
///     }
/// }
/// ```
///
/// The compiler will automatically:
/// - Scan all instructions and match start/end pairs
/// - Generate bidirectional metadata (`jump_target`, `block_id`)
/// - Handle nested blocks correctly
///
/// # Key Design Principles
///
/// - **Single Trait**: Users only need to understand `InstructionHandler`
/// - **Zero Runtime Overhead**: All analysis happens at compile time
/// - **Progressive Disclosure**: Start simple, add complexity as needed
/// - **Silent Defaults**: Optional methods have safe default implementations (return None)
pub trait InstructionHandler: Send + Sync {
    /// Instruction name (for script parsing)
    fn name(&self) -> &str;

    /// Parse instruction arguments (compile-time)
    ///
    /// Convert script text arguments into structured `InstructionData`.
    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError>;

    /// Execute instruction (run-time)
    ///
    /// # Key Points
    /// - **Control flow instructions**: If implementing constructs like `loop/end`,
    ///   the instruction should directly manipulate `vm.loop_stack` and `vm.ip`.
    /// - **Regular instructions**: Do not modify `vm.ip`; the VM will automatically increment it.
    /// - **Data storage**:
    ///   - Use `vm.registers.set_named()` for meaningful intermediate results
    ///   - Use `vm.registers.general[N]` for fast temporary values
    ///   - Use `vm.custom_state` to access context provided by upper-layer applications
    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError>;

    /// Optional: Declare block structure for automatic pairing.
    ///
    /// Override this method if your instruction has a **start/end structure**
    /// (e.g., `loop...end`, `if...else...end`, `find...endfind`).
    ///
    /// The compiler will use this declaration to:
    /// - Automatically match start/end instruction pairs using LIFO stack
    /// - Generate bidirectional metadata (`jump_target`, `block_id`, etc.)
    /// - Support nested blocks and complex control flow
    ///
    /// # Default Implementation
    ///
    /// Returns `None`, meaning this is a simple instruction without block structure.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// impl InstructionHandler for LoopHandler {
    ///     fn name(&self) -> &str { "loop" }
    ///     
    ///     fn declare_block_structure(&self) -> Option<BlockStructure> {
    ///         Some(BlockStructure::SimplePair {
    ///             start_name: "loop",
    ///             end_name: "end",
    ///         })
    ///     }
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - **Compile time**: Called once during compilation (microseconds)
    /// - **Runtime**: Zero overhead - metadata is pre-computed
    fn declare_block_structure(&self) -> Option<BlockStructure> {
        None // Default: not a block instruction
    }
}

/// Declaration of block structure for automatic pairing.
#[derive(Debug, Clone)]
pub enum BlockStructure {
    /// Simple two-part block: start...end
    /// Examples: loop...end, find...endfind, while...endwhile
    SimplePair {
        /// Name of the start instruction
        start_name: &'static str,
        /// Name of the end/terminator instruction
        end_name: &'static str,
    },

    /// Three-part conditional block: if...else...end
    /// The else part is optional (can be if...end)
    ConditionalTriple {
        /// Name of the if/start instruction
        if_name: &'static str,
        /// Name of the else/middle instruction
        else_name: &'static str,
        /// Name of the end/terminator instruction
        end_name: &'static str,
    },

    /// Custom multi-part block (for future extensions)
    /// Examples: try...catch...finally...endtry
    Custom {
        /// Name of the start instruction
        start_name: &'static str,
        /// Names of middle sections (optional)
        middle_names: Vec<&'static str>,
        /// Name of the end/terminator instruction
        end_name: &'static str,
    },
}

/// Instruction registry.
///
/// This registry stores all instruction handlers in a unified map.
/// Since `InstructionHandler` is now the single trait for all instructions,
/// there is no need to separate them by capability.
///
/// # Usage Pattern
/// ```rust
/// let mut registry = InstructionRegistry::new();
///
/// // Register handlers with the clean API!
/// registry.register(SleepHandler)?;
/// registry.register(KeyHandler)?;
/// registry.register(LoopHandler)?;
///
/// // VM uses unified interface
/// let handler = registry.get_handler("loop").unwrap();
/// handler.execute(vm, &data, metadata)?;
/// ```
///
/// # Performance
/// - **Registration**: O(1) per handler
/// - **Lookup**: O(1) via HashMap
/// - **Execution**: Minimal overhead (single vtable lookup ~5ns)
#[derive(Default)]
pub struct InstructionRegistry {
    /// All instruction handlers
    handlers: HashMap<String, Box<dyn InstructionHandler>>,
}

impl InstructionRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an instruction handler.
    ///
    /// Accepts any type that implements `InstructionHandler` and automatically boxes it.
    /// This provides a clean API for users.
    ///
    /// # Example
    /// ```rust,no_run
    /// let mut registry = InstructionRegistry::new();
    ///
    /// // Clean and simple!
    /// registry.register(SleepHandler)?;
    /// registry.register(KeyClickHandler)?;
    /// registry.register(LoopHandler)?;
    /// ```
    pub fn register<H: InstructionHandler + 'static>(
        &mut self,
        handler: H,
    ) -> Result<(), ScriptError> {
        let boxed = Box::new(handler);
        let name = boxed.name().to_string();

        if self.handlers.insert(name.clone(), boxed).is_some() {
            Err(ScriptError::CompilationError(format!(
                "Instruction '{}' already registered",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Get a handler by name.
    pub fn get_handler(&self, name: &str) -> Option<&dyn InstructionHandler> {
        self.handlers.get(name).map(|h| h.as_ref())
    }

    /// Check if an instruction with the given name is registered.
    pub fn has_instruction(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// List all registered instruction names.
    pub fn list_instructions(&self) -> Vec<String> {
        let mut names: Vec<String> = self.handlers.keys().cloned().collect();
        names.sort();
        names
    }
}

impl std::fmt::Debug for InstructionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut all_names: Vec<&str> = self.handlers.keys().map(|s| s.as_str()).collect();
        all_names.sort();

        f.debug_struct("InstructionRegistry")
            .field("handlers", &self.handlers.len())
            .field("instructions", &all_names)
            .finish()
    }
}
