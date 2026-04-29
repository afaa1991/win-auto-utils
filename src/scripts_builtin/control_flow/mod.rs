//! Control flow instructions
//!
//! Provides four control flow instructions for script execution:
//! - `loop`: Start a loop (finite or infinite)
//! - `end`: End a loop or other block structure
//! - `break`: Exit current loop immediately
//! - `continue`: Skip to next iteration
//!
//! # Instructions
//!
//! ## `loop` - Start a loop
//!
//! Begins a loop block that can run a fixed number of times or infinitely.
//!
//! **Syntax:** `loop [iterations]`
//!
//! **Parameters:**
//! - `[iterations]` (optional): Number of times to repeat (default: infinite)
//!   - If omitted, creates an infinite loop (must use `break` to exit)
//!   - Must be greater than 0 if specified
//!
//! **Examples:**
//! ```text
//! loop 5                 # Run 5 times
//! loop                   # Infinite loop (use break to exit)
//! ```
//!
//! ---
//!
//! ## `end` - End a block
//!
//! Marks the end of a loop or other block structure.
//! For loops, checks counter and jumps back if iterations remain.
//!
//! **Syntax:** `end`
//!
//! **Parameters:** None
//!
//! **Examples:**
//! ```text
//! loop 3
//!     key A
//! end                    # End of loop
//! ```
//!
//! ---
//!
//! ## `break` - Exit loop early
//!
//! Immediately exits the current loop and continues after the `end`.
//!
//! **Syntax:** `break`
//!
//! **Parameters:** None
//!
//! **Examples:**
//! ```text
//! loop 10
//!     key A
//!     break              # Exit loop immediately
//! end
//! ```
//!
//! ---
//!
//! ## `continue` - Skip to next iteration
//!
//! Skips the remaining loop body and proceeds to the next iteration.
//!
//! **Syntax:** `continue`
//!
//! **Parameters:** None
//!
//! **Examples:**
//! ```text
//! loop 10
//!     key A
//!     continue           # Skip remaining body
//!     key B              # This won't execute
//! end
//! ```
//!
//! ---
//!
//! # Common Use Cases
//!
//! ## Fixed Iteration Loop
//! ```text
//! loop 5
//!     key H
//!     key E
//!     key L
//!     key L
//!     key O
//! end
//! ```
//!
//! ## Infinite Loop with Break
//! ```text
//! loop
//!     find_color 0xFF0000
//!     if_found
//!         break
//!     end
//!     sleep 100
//! end
//! ```
//!
//! ## Conditional Skip with Continue
//! ```text
//! loop 10
//!     key A
//!     if_condition
//!         continue       # Skip key B on this iteration
//!     end
//!     key B
//! end
//! ```
//!
//! ## Nested Loops
//! ```text
//! loop 3                 # Outer loop
//!     loop 2             # Inner loop
//!         key X
//!     end
//!     key Y
//! end
//! ```
//!
//! ---
//!
//! # Important Notes
//!
//! 1. **Loop boundaries**: Every `loop` must have a matching `end`. The compiler validates this at compile time.
//!
//! 2. **Break/Continue scope**: These instructions only affect the innermost loop. For nested loops, they don't affect outer loops.
//!
//! 3. **Infinite loops**: When using `loop` without iterations, ensure there's a `break` condition to avoid hanging scripts.
//!
//! 4. **Zero iterations not allowed**: `loop 0` will cause a parse error. Minimum is 1.
//!

// Submodules - each instruction handler in its own file
pub mod loop_cmd;
pub mod break_cmd;
pub mod continue_cmd;

// Re-export handlers for convenience
pub use loop_cmd::{LoopHandler, init_loop_terminator};
pub use break_cmd::BreakHandler;
pub use continue_cmd::ContinueHandler;
