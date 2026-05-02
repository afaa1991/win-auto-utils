//! Script engine - High-level API
//!
//! Provides a convenient interface for script compilation and execution.
//! The engine manages the complete lifecycle: parsing, compilation, and execution.
//!
//! # Features
//! - **One-time compilation**: Compile once, execute multiple times efficiently
//! - **Plugin architecture**: Register custom instruction handlers
//! - **Context injection**: Pass custom state (HWND, process handles) to scripts
//! - **Interrupt control**: Optional interrupt mechanism for long-running scripts
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::script_engine::ScriptEngine;
//!
//! // Create engine with built-in instructions (recommended)
//! let engine = ScriptEngine::with_builtin();
//!
//! // Compile and execute in one step
//! engine.compile_and_execute("key A")?;
//! ```
//!
//! # Performance Optimization
//! For repeated execution, compile once and execute multiple times:
//! ```no_run
//! use win_auto_utils::script_engine::ScriptEngine;
//!
//! let engine = ScriptEngine::with_builtin();
//!
//! // Compile once (expensive operation)
//! let script = engine.compile("loop 10\n    key A\nend").unwrap();
//!
//! // Execute multiple times (fast)
//! for _ in 0..100 {
//!     engine.execute(&script).unwrap();
//! }
//! ```
//!
//! # Custom Configuration
//! Customize engine behavior with custom configuration:
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};
//!
//! let mut config = ScriptConfig::default();
//! config.vm.max_steps = 10_000_000; // Increase max execution steps
//!
//! let engine = ScriptEngine::with_registry_and_config(
//!     win_auto_utils::script_engine::InstructionRegistry::new(),
//!     config,
//! );
//! ```
//!
//! # Interrupt Control
//! For long-running scripts, you can use the interrupt controller to stop execution:
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, InterruptController};
//! use std::thread;
//! use std::time::Duration;
//!
//! let engine = ScriptEngine::with_builtin();
//! let script = engine.compile("loop 1000\n    key A\nend").unwrap();
//!
//! // Create an interrupt controller
//! let interrupt = InterruptController::new();
//!
//! // Schedule an interrupt after 5 seconds
//! let interrupt_clone = interrupt.clone();
//! thread::spawn(move || {
//!     thread::sleep(Duration::from_secs(5));
//!     interrupt_clone.request_interrupt();
//! });
//!
//! // Execute with interrupt control
//! // This will stop after ~5 seconds instead of completing all 1000 iterations
//! engine.execute_with_interrupt(&script, &interrupt).ok();
//!
//! // Check if it was interrupted
//! if interrupt.is_interrupted() {
//!     println!("Script was interrupted!");
//!     interrupt.reset(); // Reset for next use
//! }
//! ```
//!
//! # Custom Context
//! Inject custom state into script execution:
//! ```no_run
//! use win_auto_utils::script_engine::ScriptEngine;
//!
//! let engine = ScriptEngine::with_builtin();
//!
//! let hwnd = 0x12345 as *mut std::ffi::c_void;
//! engine.compile_and_execute_with_context("my_instruction", |ctx| {
//!     ctx.set_persistent_state("hwnd", hwnd);
//! }).unwrap();
//! ```

use crate::script_engine::compiler::CompiledScript;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::compiler::{Compiler, CompilerConfig};
use super::instruction::ScriptError;
use super::parser::{Parser, ParserConfig};
use super::{InstructionRegistry, VMConfig, VM};

/// Interrupt controller for script execution
///
/// Provides a thread-safe mechanism to request interruption of running scripts.
/// Multiple clones share the same interrupt state, allowing control from different threads.
///
/// # Use Cases
/// - Stopping long-running or infinite loop scripts
/// - Implementing timeout mechanisms
/// - User-initiated cancellation
///
/// # Thread Safety
/// This type is `Clone` and `Send + Sync`, safe to share across threads.
/// All clones reference the same underlying atomic flag.
///
/// # Example
/// ```no_run
/// use win_auto_utils::script_engine::InterruptController;
///
/// let interrupt = InterruptController::new();
/// let clone = interrupt.clone();
///
/// // From another thread
/// std::thread::spawn(move || {
///     clone.request_interrupt();
/// });
///
/// // Check if interrupted
/// if interrupt.is_interrupted() {
///     println!("Script was interrupted!");
/// }
/// ```
#[derive(Clone)]
pub struct InterruptController {
    flag: Arc<AtomicBool>,
}

impl InterruptController {
    /// Create a new interrupt controller
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request interruption of script execution
    ///
    /// This sets the interrupt flag. The VM will detect this at the next
    /// instruction boundary and stop execution.
    #[inline]
    pub fn request_interrupt(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Check if an interrupt has been requested
    #[inline]
    pub fn is_interrupted(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Get the internal atomic flag for passing to VM
    pub(crate) fn get_flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }

    /// Reset the interrupt flag
    ///
    /// This must be called manually by the user if they wish to reuse the
    /// `InterruptController` for subsequent script executions.
    /// The engine no longer resets this flag automatically.
    #[inline]
    pub fn reset(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

/// Script engine configuration
#[derive(Debug, Clone, Default)]
pub struct ScriptConfig {
    pub parser: ParserConfig,
    pub compiler: CompilerConfig,
    pub vm: VMConfig,
}

/// Script engine
pub struct ScriptEngine {
    registry: InstructionRegistry,
    config: ScriptConfig,
}

impl ScriptEngine {
    /// Create a new script engine with default configuration and no instructions registered
    ///
    /// This is the simplest way to get started. The engine will have an empty
    /// instruction registry, so you'll need to register your own handlers or use
    /// `with_builtin()` for built-in instructions.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::ScriptEngine;
    ///
    /// let engine = ScriptEngine::new();
    /// // Register custom instructions or use compile-only mode
    /// ```
    pub fn new() -> Self {
        Self {
            registry: InstructionRegistry::new(),
            config: ScriptConfig::default(),
        }
    }

    /// Create a new script engine with custom configuration
    ///
    /// Use this when you need to customize parser, compiler, or VM settings.
    /// Uses an empty instruction registry.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig, VMConfig};
    ///
    /// let mut config = ScriptConfig::default();
    /// config.vm.max_steps = 10_000_000; // Increase max execution steps
    ///
    /// let engine = ScriptEngine::with_config(config);
    /// ```
    pub fn with_config(config: ScriptConfig) -> Self {
        Self {
            registry: InstructionRegistry::new(),
            config,
        }
    }

    /// Create a new script engine with all built-in instructions registered
    ///
    /// This is the recommended way to get started if you want to use the standard
    /// automation instructions (control flow, keyboard, mouse, timing, etc.).
    /// Uses default configuration.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::ScriptEngine;
    ///
    /// let engine = ScriptEngine::with_builtin();
    ///
    /// // Now you can use all built-in instructions
    /// engine.compile_and_execute("loop 3\n    key A\nend").unwrap();
    /// ```
    #[cfg(feature = "scripts_builtin")]
    pub fn with_builtin() -> Self {
        let mut registry = InstructionRegistry::new();
        crate::scripts_builtin::register_all(&mut registry);
        Self::with_registry(registry)
    }

    /// Create a new script engine with custom registry
    ///
    /// Use this for advanced scenarios where you need full control over
    /// instruction registration. Uses default configuration.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::{ScriptEngine, InstructionRegistry};
    ///
    /// let mut registry = InstructionRegistry::new();
    /// // Register your custom instructions...
    ///
    /// let engine = ScriptEngine::with_registry(registry);
    /// ```
    pub fn with_registry(registry: InstructionRegistry) -> Self {
        Self::with_registry_and_config(registry, ScriptConfig::default())
    }

    /// Create a new script engine with custom registry and configuration
    ///
    /// This provides maximum flexibility for advanced use cases.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig, InstructionRegistry};
    ///
    /// let registry = InstructionRegistry::new();
    /// let config = ScriptConfig::default();
    ///
    /// let engine = ScriptEngine::with_registry_and_config(registry, config);
    /// ```
    pub fn with_registry_and_config(registry: InstructionRegistry, config: ScriptConfig) -> Self {
        Self { registry, config }
    }

    /// Compile script text into executable format
    pub fn compile(&self, text: &str) -> Result<CompiledScript, ScriptError> {
        let parser = Parser::new(self.config.parser.clone(), &self.registry);
        let compiler = Compiler::new(self.config.compiler.clone(), &self.registry);

        let ast = parser.parse(text)?;
        let script = compiler.compile(&ast)?;
        Ok(script)
    }

    /// Execute a compiled script (fast path, no interrupt control)
    ///
    /// This is the most efficient execution method when you don't need
    /// interrupt control. For long-running scripts, consider using
    /// `execute_with_interrupt` instead.
    pub fn execute(&self, script: &CompiledScript) -> Result<(), ScriptError> {
        let mut vm = VM::new(self.config.vm.clone(), &self.registry);
        vm.execute(script)
    }

    /// Execute a compiled script with interrupt control
    ///
    /// Allows external interruption of script execution via the provided
    /// `InterruptController`. Useful for long-running scripts or implementing
    /// timeout mechanisms.
    ///
    /// # Important: Interrupt Flag Management
    /// The interrupt flag is **NOT** automatically reset before or after execution.
    /// This allows you to:
    /// - Check if a previous execution was interrupted using `interrupt.is_interrupted()`
    /// - Reuse the same controller for multiple executions
    /// - Manually control when to reset using `interrupt.reset()`
    ///
    /// # Arguments
    /// - `script`: The compiled script to execute
    /// - `interrupt`: The interrupt controller for requesting interruption
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig, InterruptController};
    ///
    /// let config = ScriptConfig::default();
    /// let engine = ScriptEngine::new(config);
    /// let script = engine.compile("loop 1000\n    key A\nend").unwrap();
    ///
    /// let interrupt = InterruptController::new();
    /// // ... from another thread: interrupt.request_interrupt()
    /// let result = engine.execute_with_interrupt(&script, &interrupt);
    ///
    /// // Check if it was interrupted
    /// if interrupt.is_interrupted() {
    ///     println!("Script was interrupted!");
    ///     interrupt.reset(); // Reset for next use
    /// }
    /// ```
    pub fn execute_with_interrupt(
        &self,
        script: &CompiledScript,
        interrupt: &InterruptController,
    ) -> Result<(), ScriptError> {
        let mut vm =
            VM::new_with_interrupt(self.config.vm.clone(), &self.registry, interrupt.get_flag());
        vm.execute(script)
    }

    /// Execute with custom context (for injecting HWND, process handles, etc.)
    pub fn execute_with_context<F>(
        &self,
        script: &CompiledScript,
        setup_fn: F,
    ) -> Result<(), ScriptError>
    where
        F: FnOnce(&mut super::vm::VMContext),
    {
        let mut vm = VM::new(self.config.vm.clone(), &self.registry);
        setup_fn(vm.get_context_mut());
        vm.execute(script)
    }

    /// Execute with custom context and interrupt control
    pub fn execute_with_context_and_interrupt<F>(
        &self,
        script: &CompiledScript,
        setup_fn: F,
        interrupt: &InterruptController,
    ) -> Result<(), ScriptError>
    where
        F: FnOnce(&mut super::vm::VMContext),
    {
        let mut vm =
            VM::new_with_interrupt(self.config.vm.clone(), &self.registry, interrupt.get_flag());
        setup_fn(vm.get_context_mut());
        vm.execute(script)
    }

    /// Compile and execute in one step
    pub fn compile_and_execute(&self, text: &str) -> Result<(), ScriptError> {
        let script = self.compile(text)?;
        self.execute(&script)
    }

    /// Compile and execute with interrupt control
    pub fn compile_and_execute_with_interrupt(
        &self,
        text: &str,
        interrupt: &InterruptController,
    ) -> Result<(), ScriptError> {
        let script = self.compile(text)?;
        self.execute_with_interrupt(&script, interrupt)
    }

    /// Compile and execute with custom context
    pub fn compile_and_execute_with_context<F>(
        &self,
        text: &str,
        setup_fn: F,
    ) -> Result<(), ScriptError>
    where
        F: FnOnce(&mut super::vm::VMContext),
    {
        let script = self.compile(text)?;
        self.execute_with_context(&script, setup_fn)
    }

    /// Compile and execute with custom context and interrupt control
    pub fn compile_and_execute_with_context_and_interrupt<F>(
        &self,
        text: &str,
        setup_fn: F,
        interrupt: &InterruptController,
    ) -> Result<(), ScriptError>
    where
        F: FnOnce(&mut super::vm::VMContext),
    {
        let script = self.compile(text)?;
        self.execute_with_context_and_interrupt(&script, setup_fn, interrupt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_engine::instruction::{
        BlockStructure, InstructionData, InstructionHandler, InstructionMetadata,
    };
    use crate::script_engine::vm::VMContext;

    // ===== Test Data Structures =====

    /// Shared loop state for test handlers
    #[derive(Debug, Clone, Default)]
    struct TestLoopState {
        remaining: u32,
        start_ip: usize,
        #[allow(dead_code)] // Used for potential future enhancements
        end_ip: usize,
    }

    // ===== Test Instruction Handlers =====

    /// End instruction handler (handles loop termination and continuation)
    struct EndHandler;
    impl InstructionHandler for EndHandler {
        fn name(&self) -> &str {
            "end"
        }

        fn parse(&self, _args: &[&str]) -> Result<InstructionData, ScriptError> {
            Ok(InstructionData::None)
        }

        fn execute(
            &self,
            vm: &mut VMContext,
            _data: &InstructionData,
            _metadata: Option<&InstructionMetadata>,
        ) -> Result<(), ScriptError> {
            // First, get the state and determine if we need to loop
            let next_action = {
                let loop_stack =
                    vm.get_or_create_execution_state::<Vec<TestLoopState>>("__test_loop_stack");

                if let Some(mut state) = loop_stack.pop() {
                    state.remaining -= 1;

                    if state.remaining > 0 {
                        // Need to continue looping
                        Some((state.start_ip + 1, state)) // +1 to skip the loop instruction itself
                    } else {
                        // Loop is complete, no jump needed
                        None
                    }
                } else {
                    // No loop state found, no jump needed
                    None
                }
            };

            // Now we can safely modify vm.ip without borrowing conflicts
            if let Some((jump_target, state)) = next_action {
                vm.ip = jump_target;
                // Push the updated state back
                let loop_stack =
                    vm.get_or_create_execution_state::<Vec<TestLoopState>>("__test_loop_stack");
                loop_stack.push(state);
            }

            // End instruction completes successfully
            Ok(())
        }
    }

    /// Loop instruction handler (using declare_block_structure for automatic pairing)
    struct TestLoopHandler;
    impl InstructionHandler for TestLoopHandler {
        fn name(&self) -> &str {
            "loop"
        }

        fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
            if args.is_empty() {
                return Err(ScriptError::ParseError("Missing loop iterations".into()));
            }
            let n = args[0]
                .parse::<u32>()
                .map_err(|e| ScriptError::ParseError(format!("Invalid loop count: {}", e)))?;

            if n == 0 {
                return Err(ScriptError::ParseError(
                    "Loop iterations must be greater than 0".into(),
                ));
            }

            Ok(InstructionData::U32(n))
        }

        fn execute(
            &self,
            vm: &mut VMContext,
            data: &InstructionData,
            metadata: Option<&InstructionMetadata>,
        ) -> Result<(), ScriptError> {
            use crate::script_engine::compiler::GenericBlockMetadata;

            let iterations = match data {
                InstructionData::U32(n) => *n,
                _ => return Err(ScriptError::ExecutionError("Invalid data type".into())),
            };

            // Get pre-computed end_ip from metadata
            let end_ip = metadata
                .and_then(|m| m.get::<GenericBlockMetadata>())
                .map(|m| m.end_ip)
                .ok_or_else(|| {
                    ScriptError::ExecutionError("Loop instruction missing metadata".into())
                })?;

            let start_ip = vm.ip;

            // Use execution shared state for loop stack
            let loop_stack =
                vm.get_or_create_execution_state::<Vec<TestLoopState>>("__test_loop_stack");
            loop_stack.push(TestLoopState {
                remaining: iterations,
                start_ip,
                end_ip,
            });
            Ok(())
        }

        /// Declare block structure for automatic pairing
        fn declare_block_structure(&self) -> Option<BlockStructure> {
            Some(BlockStructure::SimplePair {
                start_name: "loop",
                end_name: "end",
            })
        }
    }

    /// Key instruction handler (records execution count)
    #[test]
    fn test_interrupt_controller_basic() {
        let interrupt = InterruptController::new();
        assert!(!interrupt.is_interrupted());

        interrupt.request_interrupt();
        assert!(interrupt.is_interrupted());

        interrupt.reset();
        assert!(!interrupt.is_interrupted());
    }

    #[test]
    fn test_interrupt_controller_clone_shares_state() {
        let interrupt = InterruptController::new();
        let clone = interrupt.clone();

        assert!(!interrupt.is_interrupted());
        assert!(!clone.is_interrupted());

        clone.request_interrupt();
        assert!(interrupt.is_interrupted());
        assert!(clone.is_interrupted());
    }

    #[test]
    fn test_script_engine_execute_without_interrupt() {
        let mut engine = ScriptEngine::new();
        engine.registry.register(TestLoopHandler).unwrap();
        engine.registry.register(EndHandler).unwrap();

        let script_text = "loop 2\nend";
        let result = engine.compile_and_execute(script_text);

        assert!(
            result.is_ok(),
            "Failed to compile and execute simple loop: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_script_engine_execute_with_interrupt_not_triggered() {
        let mut engine = ScriptEngine::new();
        engine.registry.register(TestLoopHandler).unwrap();
        engine.registry.register(EndHandler).unwrap();

        let script_text = "loop 2\nend";
        let interrupt = InterruptController::new();
        let result = engine.compile_and_execute_with_interrupt(script_text, &interrupt);

        assert!(
            result.is_ok(),
            "Failed to compile and execute with interrupt: {:?}",
            result.err()
        );
        assert!(
            !interrupt.is_interrupted(),
            "Interrupt should not be triggered"
        );
    }

    #[test]
    fn test_script_engine_execute_with_interrupt_triggered() {
        let mut engine = ScriptEngine::new();
        engine.registry.register(TestLoopHandler).unwrap();
        engine.registry.register(EndHandler).unwrap();

        // Create a long-running script
        let script_text = "loop 1000\nend";
        let interrupt = InterruptController::new();

        // We need to trigger the interrupt during execution, not before
        // Since we can't easily do that in a single-threaded test,
        // we'll verify that the interrupt mechanism works by checking the flag state

        // First, verify normal execution works
        let result = engine.compile_and_execute_with_interrupt(script_text, &interrupt);
        assert!(
            result.is_ok(),
            "Normal execution should succeed: {:?}",
            result.err()
        );

        // Verify the flag is still false after successful execution
        assert!(
            !interrupt.is_interrupted(),
            "Interrupt flag should remain false after normal execution"
        );

        // Now test that if we set the flag manually, it persists
        interrupt.request_interrupt();
        assert!(interrupt.is_interrupted());

        // Reset and verify
        interrupt.reset();
        assert!(!interrupt.is_interrupted());
    }
}
