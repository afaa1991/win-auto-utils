//! Mouse automation instructions
//!
//! Provides mouse instructions for automating mouse operations:
//! - `click`: Click at position (or current position)
//! - `dbclick`: Double-click at position
//! - `move`: Move to absolute position
//! - `moverel`: Move relatively from current position
//! - `scrollup`: Scroll wheel up
//! - `scrolldown`: Scroll wheel down
//! - `press`: Press and hold left button
//! - `release`: Release left button
//!
//! # Coordinate System
//!
//! All coordinates are in absolute screen space.
//!
//! # Instructions
//!
//! ## `click` - Click at position
//!
//! Performs a complete mouse click (press + release).
//!
//! **Syntax:** `click [x] [y] [delay_ms]`
//!
//! **Examples:**
//! ```text
//! click 100 200              # Click at (100, 200)
//! click                      # Click at current position
//! click 100 200 50           # Click at (100, 200) with 50ms delay
//! ```
//!
//! ---
//!
//! ## `dbclick` - Double-click at position
//!
//! Performs a double-click (two rapid clicks).
//!
//! **Syntax:** `dbclick [x] [y] [delay_ms]`
//!
//! **Examples:**
//! ```text
//! dbclick 100 200            # Double-click at (100, 200)
//! dbclick                    # Double-click at current position
//! dbclick 100 200 30         # Double-click at (100, 200) with 30ms delay
//! ```
//!
//! ---
//!
//! ## `move` - Move to absolute position
//!
//! **Syntax:** `move <x> <y>`
//!
//! **Examples:**
//! ```text
//! move 100 200               # Move to (100, 200)
//! ```
//!
//! ---
//!
//! ## `moverel` - Move relatively
//!
//! **Syntax:** `moverel <dx> <dy>`
//!
//! **Examples:**
//! ```text
//! moverel 10 -5              # Move 10px right, 5px up
//! ```
//!
//! ---
//!
//! ## `scrollup` - Scroll wheel up
//!
//! **Syntax:** `scrollup [x] [y] [times]`
//!
//! **Examples:**
//! ```text
//! scrollup                   # Scroll up 1 notch at current position
//! scrollup 3                 # Scroll up 3 notches
//! scrollup 100 200           # Move to (100, 200) then scroll up
//! ```
//!
//! ---
//!
//! ## `scrolldown` - Scroll wheel down
//!
//! **Syntax:** `scrolldown [x] [y] [times]`
//!
//! **Examples:**
//! ```text
//! scrolldown 2               # Scroll down 2 notches
//! ```
//!
//! ---
//!
//! ## `press` - Press and hold left button
//!
//! **Syntax:** `press`
//!
//! **Examples:**
//! ```text
//! press                      # Press left button
//! ```
//!
//! ---
//!
//! ## `release` - Release left button
//!
//! **Syntax:** `release`
//!
//! **Examples:**
//! ```text
//! release                    # Release left button
//! ```


// Submodules - each instruction handler in its own file
pub mod click;
pub mod dbclick;
pub mod move_cmd;
pub mod moverel;
pub mod press;
pub mod release;
pub mod scrolldown;
pub mod scrollup;

// Re-export handlers for convenience
pub use click::ClickHandler;
pub use dbclick::DbClickHandler;
pub use move_cmd::MoveHandler;
pub use moverel::MoveRelHandler;
pub use press::PressHandler;
pub use release::ReleaseHandler;
pub use scrolldown::ScrollDownHandler;
pub use scrollup::ScrollUpHandler;
