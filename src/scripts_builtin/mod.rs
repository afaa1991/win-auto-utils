//! Built-in instruction handlers builder
//!
//! This module provides a flexible builder pattern for creating custom instruction sets.
//! Each instruction category is independently feature-gated, allowing users to select
//! only what they need.

//!
//! # Feature Flags
//! - `scripts_control_flow`: Loop, end, if/else, jump instructions
//! - `scripts_keyboard`: Key press/release instructions (requires keyboard input)
//! - `scripts_mouse`: Mouse click/move instructions (requires mouse input)
//! - `scripts_timing`: Sleep/delay instructions
//! - `scripts`: All of the above (convenience feature)
//!
//! # Usage Examples
//!
//! ## All built-in instructions
//! ```no_run
//! use win_auto_utils::scripts_builtin::BuiltinInstructions;
//!
//! let instructions = BuiltinInstructions::all();
//! ```
//!
//! ## Custom selection with builder
//! ```no_run
//! use win_auto_utils::scripts_builtin::BuiltinInstructions;
//!
//! // Select only what you need
//! let instructions = BuiltinInstructions::builder()
//!     .with_control_flow()    // loop, end (requires scripts_control_flow feature)
//!     .with_keyboard()        // key, key_down, key_up (requires scripts_keyboard feature)
//!     .with_timing()          // sleep (requires scripts_timing feature)
//!     // .with_mouse()        // Skip mouse instructions
//!     .build();
//! ```
//!
//! ## Example 3: Minimal set (control flow only)
//! ```no_run
//! use win_auto_utils::scripts_builtin::BuiltinInstructions;
//!
//! let instructions = BuiltinInstructions::minimal();
//! // Returns only control flow instructions
//! ```
//!
//! ## Example 4: Completely custom (mix built-in + custom)
//! ```no_run
//! use win_auto_utils::scripts_builtin::{BuiltinInstructions, InstructionBuilder};
//! use std::sync::Arc;
//!
//! // Start with built-in control flow
//! let mut instructions = BuiltinInstructions::builder()
//!     .with_control_flow()
//!     .build();
//!
//! // Add your custom instruction
//! // instructions.push(Arc::new(MyCustomHandler));
//! ```
//!
//! ## Example 5: Pure engine without any built-ins
//! ```no_run
//! use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig, InstructionRegistry};
//!
//! // Create empty registry
//! let registry = InstructionRegistry::new();
//!
//! // Register only your custom instructions (clean API!)
//! // registry.register(MyHandler)?;
//!
//! let config = ScriptConfig::default();
//! let engine = ScriptEngine::with_registry_and_config(registry, config);
//! ```

use crate::script_engine::InstructionHandler;

// Submodules - each instruction category in its own module
#[cfg(feature = "scripts_control_flow")]
mod control_flow;
#[cfg(feature = "scripts_keyboard")]
mod keyboard;
#[cfg(feature = "scripts_mouse")]
pub mod mouse;
#[cfg(feature = "scripts_clipboard")]
mod clipboard;
#[cfg(feature = "scripts_terminator")]
mod terminator;
#[cfg(feature = "scripts_timing")]
mod timing;
#[cfg(all(feature = "scripts_window", feature = "script_process_context"))]
mod window;

// Re-export for external use
#[cfg(feature = "scripts_terminator")]
pub use terminator::TerminatorHandler;
#[cfg(all(feature = "scripts_window", feature = "script_process_context"))]
pub use window::ActiveHandler;

/// Builder for constructing custom instruction sets
///
/// Provides a fluent API for selecting which instruction categories to include.
/// Each method is feature-gated, so only available categories can be selected.
///
/// # Example
/// ```no_run
/// use win_auto_utils::scripts_builtin::InstructionBuilder;
///
/// let builder = InstructionBuilder::new()
///     .with_control_flow()
///     .with_keyboard()
///     .with_timing();
///
/// let instructions = builder.build();
/// ```
pub struct InstructionBuilder {
    handlers: Vec<Box<dyn InstructionHandler>>,
}

impl InstructionBuilder {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add control flow instructions (loop, end, continue, break)
    ///
    /// # Requirements
    /// Requires the `scripts_control_flow` feature to be enabled.
    #[cfg(feature = "scripts_control_flow")]
    pub fn with_control_flow(mut self) -> Self {
        self.handlers.push(Box::new(control_flow::LoopHandler));
        self.handlers.push(Box::new(terminator::TerminatorHandler));
        self.handlers.push(Box::new(control_flow::ContinueHandler));
        self.handlers.push(Box::new(control_flow::BreakHandler));
        self
    }

    /// Add keyboard instructions (key, key_down, key_up)
    ///
    /// Parameters: `key <name> [delay_ms]`
    ///
    /// # Examples
    /// ```text
    /// key A                    # Click 'A'
    /// key A 50                 # Click 'A' with 50ms delay
    /// key_down SHIFT           # Press SHIFT
    /// key_up SHIFT             # Release SHIFT
    /// ```
    /// - `key_up`: Only releases the key
    ///
    /// # Requirements
    /// Requires the `scripts_keyboard` feature to be enabled (which includes utils + keyboard).
    #[cfg(feature = "scripts_keyboard")]
    pub fn with_keyboard(mut self) -> Self {
        self.handlers.push(Box::new(keyboard::KeyClickHandler));
        self.handlers.push(Box::new(keyboard::KeyDownHandler));
        self.handlers.push(Box::new(keyboard::KeyUpHandler));
        self
    }

    /// Add mouse instructions (click, move, moverel, scrollup, scrolldown, press, release)
    ///
    /// These instructions provide comprehensive mouse automation capabilities:
    /// - click: Click at position or current location
    /// - move: Move to absolute coordinates
    /// - moverel: Move relatively from current position
    /// - scrollup/scrolldown: Scroll wheel operations
    /// - press/release: Button press and release
    ///
    /// # Requirements
    /// Requires the `scripts_mouse` feature to be enabled (which includes mouse).
    #[cfg(feature = "scripts_mouse")]
    pub fn with_mouse(mut self) -> Self {
        self.handlers.push(Box::new(mouse::ClickHandler));
        self.handlers.push(Box::new(mouse::MoveHandler));
        self.handlers.push(Box::new(mouse::MoveRelHandler));
        self.handlers.push(Box::new(mouse::ScrollUpHandler));
        self.handlers.push(Box::new(mouse::ScrollDownHandler));
        self.handlers.push(Box::new(mouse::PressHandler));
        self.handlers.push(Box::new(mouse::ReleaseHandler));
        self
    }

    /// Add clipboard instructions (copy, paste)
    ///
    /// These instructions provide clipboard automation capabilities:
    /// - copy: Copy text to clipboard
    /// - paste: Paste clipboard content via Ctrl+V
    ///
    /// # Examples
    /// ```text
    /// copy Hello World        # Copy text to clipboard
    /// paste                   # Paste with default 20ms delay
    /// paste 50                # Paste with 50ms delay
    /// ```
    ///
    /// # Requirements
    /// Requires the `scripts_clipboard` feature to be enabled (which includes clipboard + keyboard).
    #[cfg(feature = "scripts_clipboard")]
    pub fn with_clipboard(mut self) -> Self {
        self.handlers.push(Box::new(clipboard::CopyHandler));
        self.handlers.push(Box::new(clipboard::PasteHandler));
        self
    }

    /// Add timing instructions (sleep, time)
    ///
    /// Provides execution delay and timed execution block functionality:
    /// - `sleep`: Pause for specified duration
    /// - `time`: Execute body repeatedly for a total duration
    ///
    /// # Requirements
    /// Requires the `scripts_timing` feature to be enabled.
    #[cfg(feature = "scripts_timing")]
    pub fn with_timing(mut self) -> Self {
        self.handlers.push(Box::new(timing::SleepHandler));
        self.handlers.push(Box::new(timing::TimeHandler));
        self
    }

    /// Add window activation instruction (active)
    ///
    /// Ensures the target window (from VM context) is in the foreground before
    /// executing subsequent input operations. This prevents keyboard/mouse input
    /// from being sent to the wrong window.
    ///
    /// # Usage
    /// ```text
    /// active    # Activate window from context (requires HWND)
    /// key A     # Now safe to send input
    /// ```
    ///
    /// # Behavior
    /// 1. Retrieves HWND from VM context (set via `set_vmcontext_hwnd()`)
    /// 2. Checks if already foreground (fast path, ~243ns)
    /// 3. If not, activates with adaptive polling (typically 5-50ms)
    /// 4. Returns error if activation fails or times out (300ms default)
    ///
    /// # Requirements
    /// - Requires `scripts_window` feature
    /// - Requires `script_process_context` feature (provides HWND support)
    /// - VM context must have HWND set before executing this instruction
    ///
    /// # Error Cases
    /// - No HWND in context: Instruction parsing fails
    /// - Activation timeout: Windows foreground lock may be blocking
    /// - API failure: System rejected activation request
    #[cfg(all(feature = "scripts_window", feature = "script_process_context"))]
    pub fn with_window(mut self) -> Self {
        self.handlers.push(Box::new(window::ActiveHandler));
        self
    }

    /// Build and return the instruction list
    pub fn build(self) -> Vec<Box<dyn InstructionHandler>> {
        self.handlers
    }
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-configured instruction sets for common use cases
pub struct BuiltinInstructions;

impl BuiltinInstructions {
    /// Get all available built-in instructions (based on enabled features)
    ///
    /// This is a convenience method that includes all instruction categories
    /// that are compiled into the binary. Use this for quick setup when you
    /// want maximum functionality.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::scripts_builtin::BuiltinInstructions;
    ///
    /// let instructions = BuiltinInstructions::all();
    /// // Contains: control_flow + keyboard + mouse + timing (if features enabled)
    /// ```
    pub fn all() -> Vec<Box<dyn InstructionHandler>> {
        let mut builder = InstructionBuilder::new();

        #[cfg(feature = "scripts_control_flow")]
        {
            builder = builder.with_control_flow();
        }

        #[cfg(feature = "scripts_keyboard")]
        {
            builder = builder.with_keyboard();
        }

        #[cfg(feature = "scripts_mouse")]
        {
            builder = builder.with_mouse();
        }

        #[cfg(feature = "scripts_clipboard")]
        {
            builder = builder.with_clipboard();
        }

        #[cfg(feature = "scripts_timing")]
        {
            builder = builder.with_timing();
        }

        #[cfg(all(feature = "scripts_window", feature = "script_process_context"))]
        {
            builder = builder.with_window();
        }

        #[cfg(feature = "scripts_terminator")]
        {
            // terminator is automatically included when scripts_control_flow or scripts_timing is enabled
            // No need to add it separately here
        }

        builder.build()
    }

    /// Get minimal instruction set (control flow only)
    ///
    /// Returns only the essential control flow instructions (loop, end).
    /// This is useful for lightweight script execution without I/O operations.
    ///
    /// # Requirements
    /// Requires the `scripts_control_flow` feature to be enabled.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::scripts_builtin::BuiltinInstructions;
    ///
    /// let instructions = BuiltinInstructions::minimal();
    /// // Contains only: loop, end
    /// ```
    #[cfg(feature = "scripts_control_flow")]
    pub fn minimal() -> Vec<Box<dyn InstructionHandler>> {
        InstructionBuilder::new().with_control_flow().build()
    }

    /// Create a new builder for custom instruction sets
    ///
    /// This is the most flexible way to construct an instruction set.
    /// You can selectively enable only the categories you need.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::scripts_builtin::BuiltinInstructions;
    ///
    /// let instructions = BuiltinInstructions::builder()
    ///     .with_control_flow()
    ///     .with_keyboard()
    ///     // .with_mouse()  // Skip mouse
    ///     .with_timing()
    ///     .build();
    /// ```
    pub fn builder() -> InstructionBuilder {
        InstructionBuilder::new()
    }
}

/// Convenience function to register all built-in instructions to a registry
///
/// This function registers all built-in instruction handlers using the clean API.
/// Just call this function to register all available built-in handlers!
///
/// # Example
/// ```no_run
/// use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig, InstructionRegistry};
/// use win_auto_utils::scripts_builtin::register_all;
///
/// let mut registry = InstructionRegistry::new();
/// register_all(&mut registry);
///
/// let config = ScriptConfig::default();
/// let engine = ScriptEngine::with_registry(registry, config);
/// ```
pub fn register_all(registry: &mut crate::script_engine::instruction::InstructionRegistry) {
    // Register all handlers directly - using the clean API!

    #[cfg(feature = "scripts_control_flow")]
    {
        // Initialize terminators BEFORE registering handlers
        control_flow::init_loop_terminator();

        registry.register(control_flow::LoopHandler).ok();
        registry.register(terminator::TerminatorHandler).ok();
        registry.register(control_flow::ContinueHandler).ok();
        registry.register(control_flow::BreakHandler).ok();
    }

    #[cfg(feature = "scripts_keyboard")]
    {
        registry.register(keyboard::KeyClickHandler).ok();
        registry.register(keyboard::KeyDownHandler).ok();
        registry.register(keyboard::KeyUpHandler).ok();
    }

    #[cfg(feature = "scripts_clipboard")]
    {
        registry.register(clipboard::CopyHandler).ok();
        registry.register(clipboard::PasteHandler).ok();
    }

    #[cfg(feature = "scripts_mouse")]
    {
        registry.register(mouse::ClickHandler).ok();
        registry.register(mouse::DbClickHandler).ok();
        registry.register(mouse::MoveHandler).ok();
        registry.register(mouse::MoveRelHandler).ok();
        registry.register(mouse::ScrollUpHandler).ok();
        registry.register(mouse::ScrollDownHandler).ok();
        registry.register(mouse::PressHandler).ok();
        registry.register(mouse::ReleaseHandler).ok();
    }

    #[cfg(feature = "scripts_timing")]
    {
        // Initialize time terminator
        timing::init_time_terminator();

        registry.register(timing::SleepHandler).ok();
        registry.register(timing::TimeHandler).ok();
    }
}
