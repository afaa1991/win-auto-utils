//! Clipboard automation instructions
//!
//! Provides clipboard operations via script instructions:
//! - `copy`: Copy text to clipboard
//! - `paste`: Paste clipboard content via Ctrl+V
//!
//! # Instructions
//!
//! ## `copy` - Copy text to clipboard
//!
//! **Syntax:** `copy <text>`
//!
//! **Examples:**
//! ```text
//! copy Hello World        # Copy "Hello World" to clipboard
//! copy "Hello World"      # Copy with quotes
//! copy "Line 1\nLine 2"   # Copy multi-line text
//! ```
//!
//! ## `paste` - Paste clipboard content
//!
//! **Syntax:** `paste [delay_ms]`
//!
//! **Examples:**
//! ```text
//! paste                   # Paste with default 20ms delay
//! paste 20                # Paste with 20ms delay
//! paste 50                # Paste with 50ms delay
//! ```

// Submodules
pub mod copy_cmd;
pub mod paste_cmd;

// Re-export handlers
pub use copy_cmd::CopyHandler;
pub use paste_cmd::PasteHandler;
