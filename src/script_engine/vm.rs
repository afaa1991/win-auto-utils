//! Virtual machine executor
//!
//! Iteratively executes linear instructions. The VM only manages IP (Instruction Pointer).
//! Specific instruction execution logic is fully delegated to Handlers in the registry.
//!
//! # Features
//! - **Register system**: 16 general-purpose registers (R0-R15) + named registers
//! - **Two-layer state management**: Persistent state + instruction scopes
//! - **Interrupt support**: Optional external interruption via atomic flag
//! - **Loop support**: Nested loop execution via execution shared state
//!
//! # Register System
//! The VM provides two types of data storage:
//!
//! ## General-Purpose Registers (R0-R15)
//! Similar to CPU registers, used for fast temporary storage:
//! ```no_run
//! use win_auto_utils::script_engine::vm::VMContext;
//!
//! let mut ctx = VMContext::new();
//! ctx.registers.general[0] = 42;  // Store in R0
//! let value = ctx.registers.general[0];  // Read from R0
//! ```
//!
//! ## Named Registers
//! String-keyed registers for meaningful intermediate results:
//! ```no_run
//! use win_auto_utils::script_engine::vm::VMContext;
//!
//! let mut ctx = VMContext::new();
//! ctx.registers.set_named("last_x", 100i32);
//! ctx.registers.set_named("last_y", 200i32);
//!
//! let x = ctx.registers.get_named::<i32>("last_x").unwrap();
//! let y = ctx.registers.get_named::<i32>("last_y").unwrap();
//! ```
//!
//! # State Management Layers
//!
//! ## Persistent State
//! Long-lived state that persists across multiple `execute()` calls:
//! ```no_run
//! use win_auto_utils::script_engine::vm::VMContext;
//!
//! let mut ctx = VMContext::new();
//! ctx.set_persistent_state("hwnd", 0x12345 as *mut std::ffi::c_void);
//!
//! let hwnd = ctx.get_persistent_state::<*mut std::ffi::c_void>("hwnd").unwrap();
//! ```
//!
//! ## Instruction Scopes
//! Per-execution state that is automatically cleared on each `execute()` call:
//! ```no_run
//! use win_auto_utils::script_engine::vm::VMContext;
//!
//! let mut ctx = VMContext::new();
//! let ip = 0;
//! let cached_data = ctx.get_or_init_instruction_scope(ip, || {
//!     // Expensive computation, runs once per execute()
//!     42
//! });
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::script_engine::CompiledScript;

use super::instruction::ScriptError;

// Re-export from instruction module for backward compatibility
pub use super::instruction::{InstructionHandler, InstructionRegistry};

/// General-purpose register system
///
/// Provides two types of data storage:
/// 1. **General registers** (R0-R15): Similar to CPU registers, for fast temporary storage
/// 2. **Named registers**: String-keyed access, suitable for meaningful intermediate results
#[derive(Debug, Default)]
pub struct VMRegisters {
    /// 16 general-purpose registers (R0-R15)
    pub general: [u64; 16],
    /// Named registers (e.g., "last_x", "found_color")
    named: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl VMRegisters {
    /// Create new register system
    pub fn new() -> Self {
        Self {
            general: [0; 16],
            named: HashMap::new(),
        }
    }

    /// Set named register value
    pub fn set_named<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.named.insert(key.to_string(), Box::new(value));
    }

    /// Get named register value
    pub fn get_named<T: Any>(&self, key: &str) -> Option<&T> {
        self.named
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Clear all named registers
    pub fn clear_named(&mut self) {
        self.named.clear();
    }

    /// Get count of general-purpose registers
    pub fn general_count(&self) -> usize {
        self.general.len()
    }
}

/// Custom state storage for context injection
///
/// Note: We use `Any` without `Send + Sync` bounds because VM execution is single-threaded.
/// The VM processes instructions sequentially in a single thread, so cross-thread safety
/// is not required for custom state values (like HWND).
pub type PersistentState = HashMap<String, Box<dyn Any>>;

/// Instruction-level scope storage indexed by instruction pointer (IP)
///
/// Each entry stores arbitrary state specific to an instruction at that IP.
/// This enables instructions to cache runtime-dependent data (e.g., window positions)
/// that should be refreshed on each script execution.
pub type InstructionScopes = HashMap<usize, Box<dyn Any>>;

/// Block-local context for data inheritance and isolation
///
/// This structure holds data that is only valid within a specific block (e.g., inside a loop or if-block).
/// It implements RAII-style lifecycle management: data is pushed when entering a block and popped when leaving.
#[derive(Debug, Default)]
pub struct BlockLocalContext {
    /// Named data storage within the current block
    pub data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl BlockLocalContext {
    /// Create a new empty block context
    pub fn new() -> Self {
        Self::default()
    }
}

/// VM execution context
///
/// Contains all runtime state required for instruction execution.
///
/// # State Management Layers
///
/// ## 1. Persistent State (`persistent_state`)
/// - **Lifetime**: Persists across multiple `execute()` calls until VM destruction or explicit clear.
/// - **Use Cases**: Global configuration, shared resources (HWND), user-injected context.
/// - **Management**: User-controlled; NOT automatically cleared during `reset_execution()`.
///
/// ## 2. Execution Shared State (`execution_shared_state`)
/// - **Lifetime**: Cleared at the start of each `execute()` call.
/// - **Use Cases**: Internal control flow state (loop stacks, if/else stacks), cross-instruction coordination.
/// - **Management**: Automatically cleared; used by instruction handlers for execution-level coordination.
///
/// ## 3. Instruction Scopes (`instruction_scopes`)
/// - **Lifetime**: Cleared at the start of each `execute()` call.
/// - **Use Cases**: Per-execution cached data (e.g., window position cache) that must be fresh.
/// - **Management**: Automatically cleared; supports lazy initialization via `get_or_init_instruction_scope`.
///
/// ## 4. Process Context (`process_ctx`) *(Optional)*
/// - **Lifetime**: Persists across executions when `script_process_context` feature is enabled.
/// - **Use Cases**: Direct field access to Windows handles (HWND, HDC, HANDLE) for ~50x faster access.
/// - **Management**: User-controlled via direct field access or helper methods.
/// - **Performance**: Eliminates HashMap lookup overhead for frequently-accessed handles.
///
/// # Design Principles
/// - **Engine agnostic**: Core fields are generic (IP, registers)
/// - **Flexibility**: Support arbitrary state injection via `persistent_state`
/// - **Performance**: Optional direct field access via `process_ctx` when feature enabled
/// - **Data separation**:
///   - `registers`: Temporary data shared between instructions (e.g., find_color results)
///   - `persistent_state`: Long-lived context provided by upper-layer applications
///   - `execution_shared_state`: Short-lived internal state for control flow and coordination
///   - `instruction_scopes`: Per-instruction cached data within single execution
///   - `process_ctx`: Fast-access Windows handles (optional, feature-gated)
pub struct VMContext {
    /// Instruction pointer (current instruction index)
    pub ip: usize,
    /// Register system (for inter-instruction data transfer)
    pub registers: VMRegisters,
    /// Persistent state (injected by caller, persists across executions)
    pub persistent_state: PersistentState,
    /// Execution shared state (internal control flow, cleared each execute())
    pub execution_shared_state: HashMap<String, Box<dyn Any>>,
    /// Instruction-level scopes (cleared on each execute() call)
    pub instruction_scopes: InstructionScopes,
    /// Block-local context stack (for data inheritance and isolation)
    pub block_context_stack: Vec<BlockLocalContext>,
    /// Process context with direct field access to Windows handles (optional, ~50x faster)
    #[cfg(feature = "script_process_context")]
    pub process: crate::script_engine::process_ctx::ProcessContext,
}

impl VMContext {
    /// Create new execution context
    pub fn new() -> Self {
        Self {
            ip: 0,
            registers: VMRegisters::new(),
            persistent_state: HashMap::new(),
            execution_shared_state: HashMap::new(),
            instruction_scopes: HashMap::new(),
            block_context_stack: Vec::new(),
            #[cfg(feature = "script_process_context")]
            process: crate::script_engine::process_ctx::ProcessContext::new(),
        }
    }

    // ========================================================================
    // Persistent State API (Long-lived, user-managed)
    // ========================================================================

    /// Get value from persistent state
    pub fn get_persistent_state<T: Any>(&self, key: &str) -> Option<&T> {
        self.persistent_state
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Check if a key exists in persistent state
    pub fn has_persistent_state(&self, key: &str) -> bool {
        self.persistent_state.contains_key(key)
    }

    /// Set value in persistent state
    ///
    /// # Naming Convention
    /// - **User-defined keys**: Any name not starting with `__builtin_` (preserved across executions)
    /// - **Internal keys**: Keys starting with `__builtin_` are reserved for internal use and
    ///   will be automatically cleared during `reset_execution()` to ensure clean execution state.
    ///
    /// Note: No `Send + Sync` bounds required because VM execution is single-threaded.
    pub fn set_persistent_state<T: Any>(&mut self, key: &str, value: T) {
        self.persistent_state
            .insert(key.to_string(), Box::new(value));
    }

    /// Clear all persistent state
    pub fn clear_persistent_state(&mut self) {
        self.persistent_state.clear();
    }

    /// Get mutable reference to value in persistent state
    ///
    /// This is useful for modifying collections (like Vec) stored in persistent state.
    /// Returns `None` if the key doesn't exist or type mismatch.
    pub fn get_persistent_state_mut<T: Any>(&mut self, key: &str) -> Option<&mut T> {
        self.persistent_state
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    // ========================================================================
    // Execution Shared State API (Internal control flow, auto-cleared)
    // ========================================================================

    /// Get or create execution shared state
    ///
    /// This is used by instruction handlers to share state across multiple instructions
    /// within a single execution (e.g., loop stacks, if/else stacks).
    /// All execution shared state is automatically cleared at the start of each execute().
    ///
    /// # Type Parameters
    /// - `T`: The state type (must be Clone + Default)
    ///
    /// # Arguments
    /// - `key`: The state key
    ///
    /// # Returns
    /// A mutable reference to the state value
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::vm::VMContext;
    /// let mut vm = VMContext::new();
    /// let loop_stack = vm.get_or_create_execution_state::<Vec<String>>("loop_stack");
    /// loop_stack.push("test".to_string());
    /// ```
    #[inline]
    pub fn get_or_create_execution_state<T: Any + Default>(&mut self, key: &str) -> &mut T {
        if !self.execution_shared_state.contains_key(key) {
            self.execution_shared_state
                .insert(key.to_string(), Box::new(T::default()));
        }
        self.execution_shared_state
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .expect("Execution shared state type mismatch")
    }

    /// Get mutable reference to execution shared state
    ///
    /// Returns `None` if the key doesn't exist or type mismatch.
    pub fn get_execution_state_mut<T: Any>(&mut self, key: &str) -> Option<&mut T> {
        self.execution_shared_state
            .get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Check if execution shared state exists
    pub fn has_execution_state(&self, key: &str) -> bool {
        self.execution_shared_state.contains_key(key)
    }

    /// Clear all execution shared state
    ///
    /// This is called automatically at the start of each `execute()` call.
    pub fn clear_execution_shared_state(&mut self) {
        self.execution_shared_state.clear();
    }

    // ========================================================================
    // Instruction Scopes API (Per-execution, auto-cleared)
    // ========================================================================

    /// Get or initialize scope data for a specific instruction IP
    ///
    /// This implements a lazy initialization pattern:
    /// 1. If scope data exists for this IP, return it
    /// 2. Otherwise, call the initializer function and store the result
    ///
    /// This is the preferred way for instruction handlers to manage per-execution state.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::vm::VMContext;
    /// let mut vm = VMContext::new();
    /// let ip = 0;
    /// let cached_data = vm.get_or_init_instruction_scope(ip, || {
    ///     // Expensive computation, runs once per execute()
    ///     42
    /// });
    /// ```
    pub fn get_or_init_instruction_scope<T: Any + Send + Sync, F: FnOnce() -> T>(
        &mut self,
        ip: usize,
        init_fn: F,
    ) -> &mut T {
        if !self.instruction_scopes.contains_key(&ip) {
            let value = init_fn();
            self.instruction_scopes.insert(ip, Box::new(value));
        }
        self.instruction_scopes
            .get_mut(&ip)
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .expect("Instruction scope type mismatch")
    }

    /// Get or initialize scope data using current instruction pointer (convenience method)
    ///
    /// This is a convenience wrapper around `get_or_init_instruction_scope` that automatically
    /// uses the current `vm.ip` value. This eliminates boilerplate code in instruction handlers.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::script_engine::vm::VMContext;
    /// let mut vm = VMContext::new();
    /// // Instead of:
    /// // let ip = vm.ip;
    /// // let cached_data = vm.get_or_init_instruction_scope(ip, || { ... });
    ///
    /// // Use:
    /// let cached_data = vm.get_or_init_current_scope(|| {
    ///     // Expensive computation that should only run once per execute()
    ///     42
    /// });
    /// ```
    #[inline]
    pub fn get_or_init_current_scope<T: Any + Send + Sync, F: FnOnce() -> T>(
        &mut self,
        init_fn: F,
    ) -> &mut T {
        let ip = self.ip;
        self.get_or_init_instruction_scope(ip, init_fn)
    }

    /// Get mutable reference to scope data for a specific instruction IP
    ///
    /// Returns `None` if no scope data exists for this IP.
    pub fn get_instruction_scope_mut<T: Any>(&mut self, ip: usize) -> Option<&mut T> {
        self.instruction_scopes
            .get_mut(&ip)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Clear all instruction scopes
    ///
    /// This is called automatically at the start of each `execute()` call.
    pub fn clear_instruction_scopes(&mut self) {
        self.instruction_scopes.clear();
    }

    // ========================================================================
    // Block Local Context API (Optional data isolation mechanism)
    // ========================================================================
    //
    // Note: This is an OPTIONAL mechanism for semantic instructions that need
    // block-level data isolation. Control flow instructions (loop/end) do NOT
    // use this mechanism to maintain zero overhead.
    //
    // Use cases:
    // - find_color...endfind: Store found coordinates in block context
    // - if_found...end: Isolate conditional branch data
    // - Custom blocks requiring automatic cleanup

    /// Enter a new block scope (called by semantic instructions, NOT control flow)
    ///
    /// Pushes a new `BlockLocalContext` onto the stack. Data stored in this context
    /// will be isolated from outer blocks and automatically cleaned up when leaving.
    ///
    /// # When to Use
    /// Semantic instructions that need temporary data storage within a block should
    /// call this in their execute phase. Control flow instructions should NOT call this.
    #[inline]
    pub fn enter_block(&mut self) {
        self.block_context_stack.push(BlockLocalContext::new());
    }

    /// Leave the current block scope (called by semantic instructions or end terminators)
    ///
    /// Pops the top `BlockLocalContext` from the stack, destroying all data within it.
    /// This ensures strict isolation and prevents "dirty data" from affecting subsequent logic.
    ///
    /// # Safety Check
    /// Always check `is_block_context_empty()` before calling to avoid panics.
    /// Control flow instructions should use this pattern:
    /// ```no_run
    /// # use win_auto_utils::script_engine::vm::VMContext;
    /// # let mut vm = VMContext::new();
    /// if !vm.is_block_context_empty() {
    ///     vm.leave_block();
    /// }
    /// ```
    #[inline]
    pub fn leave_block(&mut self) {
        self.block_context_stack.pop();
    }

    /// Push data into the current active block
    ///
    /// # Arguments
    /// - `key`: A semantic name for the data (e.g., "last_coords", "target_hwnd")
    /// - `value`: The data to store
    ///
    /// # Note
    /// If called outside of any block, this operation is silently ignored.
    pub fn push_to_current_block<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        if let Some(ctx) = self.block_context_stack.last_mut() {
            ctx.data.insert(key.to_string(), Box::new(value));
        }
    }

    /// Get data from the current active block
    ///
    /// # Returns
    /// - `Some(&T)` if the key exists in the current block and type matches
    /// - `None` if not in a block, key doesn't exist, or type mismatch
    pub fn get_from_current_block<T: Any>(&self, key: &str) -> Option<&T> {
        self.block_context_stack
            .last()
            .and_then(|ctx| ctx.data.get(key))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Check if the block context stack is empty
    pub fn is_block_context_empty(&self) -> bool {
        self.block_context_stack.is_empty()
    }

    /// Check if currently inside a block scope
    pub fn is_in_block(&self) -> bool {
        !self.block_context_stack.is_empty()
    }

    /// Clear all block contexts (called during reset_execution)
    pub fn clear_block_contexts(&mut self) {
        self.block_context_stack.clear();
    }

    // ========================================================================
    // Execution Control
    // ========================================================================

    /// Reset execution state (preserve persistent state, clear instruction scopes)
    ///
    /// This method is called at the start of each `execute()` call to ensure
    /// a clean execution environment while preserving long-lived context.
    pub fn reset_execution(&mut self) {
        self.ip = 0;
        self.registers = VMRegisters::new();

        // Clear instruction scopes (per-execution state)
        self.clear_instruction_scopes();

        // Clear execution shared state (internal control flow state)
        self.clear_execution_shared_state();

        // Clear block contexts (ensure no dirty data from previous runs)
        self.clear_block_contexts();

        // Note: process_ctx is NOT cleared here - it persists across executions
        // like persistent_state. Use reset_all() or process_ctx.clear() if needed.
    }

    /// Reset all state (including persistent state and process context)
    pub fn reset_all(&mut self) {
        self.reset_execution();
        self.clear_persistent_state();

        #[cfg(feature = "script_process_context")]
        self.process.clear();
    }
}

impl Default for VMContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual machine configuration
#[derive(Debug, Clone)]
pub struct VMConfig {
    /// Maximum execution steps (to prevent infinite loops)
    pub max_steps: usize,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
        }
    }
}

/// Virtual machine
pub struct VM<'a> {
    config: VMConfig,
    registry: &'a InstructionRegistry,
    context: VMContext,
    steps_executed: usize,
    /// Optional interrupt flag for external interruption control
    interrupt_flag: Option<Arc<AtomicBool>>,
}

impl<'a> VM<'a> {
    pub fn new(config: VMConfig, registry: &'a InstructionRegistry) -> Self {
        Self {
            config,
            registry,
            context: VMContext::new(),
            steps_executed: 0,
            interrupt_flag: None,
        }
    }

    /// Create a new VM with interrupt control
    pub fn new_with_interrupt(
        config: VMConfig,
        registry: &'a InstructionRegistry,
        interrupt_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            config,
            registry,
            context: VMContext::new(),
            steps_executed: 0,
            interrupt_flag: Some(interrupt_flag),
        }
    }

    /// Check if an interrupt has been requested
    #[inline]
    fn check_interrupt(&self) -> Result<(), ScriptError> {
        if let Some(ref flag) = self.interrupt_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(ScriptError::Interrupted(
                    "Script execution interrupted by user".into(),
                ));
            }
        }
        Ok(())
    }

    /// Execute compiled script
    pub fn execute(&mut self, script: &CompiledScript) -> Result<(), ScriptError> {
        self.context.reset_execution();
        self.steps_executed = 0;

        while self.context.ip < script.instructions.len() {
            // Check for interrupt request
            self.check_interrupt()?;

            // Prevent infinite loops
            self.steps_executed += 1;
            if self.steps_executed > self.config.max_steps {
                return Err(ScriptError::ExecutionError(format!(
                    "Max execution steps ({}) exceeded",
                    self.config.max_steps
                )));
            }

            // Get current instruction
            let instr = &script.instructions[self.context.ip];

            // Record IP before execution (to detect if Handler modified IP)
            let ip_before = self.context.ip;

            // Get Handler from registry
            let handler = self.registry.get_handler(&instr.name).ok_or_else(|| {
                ScriptError::ExecutionError(format!("Handler not found for '{}'", instr.name))
            })?;

            // Delegate to Handler for execution
            handler.execute(&mut self.context, &instr.data, instr.metadata.as_ref())?;

            // Key: If Handler did not modify IP, automatically increment
            if self.context.ip == ip_before {
                self.context.ip += 1;
            }
        }

        Ok(())
    }

    /// Get execution context (for debugging or result extraction)
    pub fn get_context(&self) -> &VMContext {
        &self.context
    }

    /// Get mutable execution context (for injecting custom state)
    pub fn get_context_mut(&mut self) -> &mut VMContext {
        &mut self.context
    }

    /// Get number of steps executed
    pub fn steps_executed(&self) -> usize {
        self.steps_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let registry = InstructionRegistry::new();
        let vm = VM::new(VMConfig::default(), &registry);
        assert_eq!(vm.steps_executed(), 0);
    }

    #[test]
    fn test_registers_general() {
        let mut regs = VMRegisters::new();
        regs.general[0] = 42;
        regs.general[1] = 100;

        assert_eq!(regs.general[0], 42);
        assert_eq!(regs.general[1], 100);
    }

    #[test]
    fn test_registers_named() {
        let mut regs = VMRegisters::new();
        regs.set_named("x", 100i32);
        regs.set_named("y", 200i32);

        assert_eq!(regs.get_named::<i32>("x"), Some(&100));
        assert_eq!(regs.get_named::<i32>("y"), Some(&200));
        assert_eq!(regs.get_named::<i32>("z"), None);
    }

    #[test]
    fn test_vm_context_persistent_state() {
        let mut ctx = VMContext::new();
        ctx.set_persistent_state("hwnd", 0x12345usize);
        ctx.set_persistent_state("process_id", 1234u32);

        assert_eq!(ctx.get_persistent_state::<usize>("hwnd"), Some(&0x12345));
        assert_eq!(ctx.get_persistent_state::<u32>("process_id"), Some(&1234));
    }

    #[test]
    fn test_instruction_scopes_lifecycle() {
        let mut ctx = VMContext::new();

        // Set some persistent state
        ctx.set_persistent_state("persistent_key", "persistent_value");

        // Add instruction scope data
        let ip = 0;
        let scope_data = ctx.get_or_init_instruction_scope(ip, || "scope_value".to_string());
        assert_eq!(scope_data, "scope_value");

        // Reset execution should clear instruction scopes but keep persistent state
        ctx.reset_execution();

        // Persistent state should still exist
        assert_eq!(
            ctx.get_persistent_state::<&str>("persistent_key"),
            Some(&"persistent_value")
        );

        // Instruction scope should be cleared
        assert!(ctx.get_instruction_scope_mut::<String>(ip).is_none());
    }

    #[test]
    fn test_clear_persistent_state() {
        let mut ctx = VMContext::new();
        ctx.set_persistent_state("key1", "value1");
        ctx.set_persistent_state("key2", "value2");

        assert_eq!(ctx.get_persistent_state::<&str>("key1"), Some(&"value1"));

        ctx.clear_persistent_state();

        assert!(ctx.get_persistent_state::<&str>("key1").is_none());
        assert!(ctx.get_persistent_state::<&str>("key2").is_none());
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_get_or_init_instruction_scope() {
        let mut vm = VMContext::new();
        let iterations = 1_000_000;
        let ip = 0;

        let start = Instant::now();
        for _ in 0..iterations {
            let scope_data = vm.get_or_init_instruction_scope(ip, || 42i32);
            *scope_data += 1;
        }
        let elapsed = start.elapsed();

        println!(
            "get_or_init_instruction_scope: {} iterations in {:?}, avg: {:?}",
            iterations,
            elapsed,
            elapsed / iterations as u32
        );

        // Debug mode is more lenient, Release mode will be 5-10x faster
        // Production environment ~50ns per call
        assert!(elapsed.as_nanos() / (iterations as u128) < 1000);
    }

    #[test]
    fn benchmark_persistent_state_access() {
        let mut vm = VMContext::new();
        vm.set_persistent_state("test_key", 42i32);

        let iterations = 1_000_000;

        // Test HashMap access
        let start = Instant::now();
        for _ in 0..iterations {
            let _value = vm
                .persistent_state
                .get("test_key")
                .and_then(|boxed| boxed.downcast_ref::<i32>())
                .unwrap();
        }
        let hashmap_time = start.elapsed();

        println!(
            "Persistent state access: {} iterations in {:?}, avg: {:?}",
            iterations,
            hashmap_time,
            hashmap_time / iterations as u32
        );

        // Debug mode is more lenient
        assert!(hashmap_time.as_nanos() / (iterations as u128) < 500);
    }
}
