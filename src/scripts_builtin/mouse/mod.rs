//! Mouse automation instructions
//!
//! Provides seven mouse instructions for automating mouse operations:
//! - `click`: Click at position (or current position)
//! - `move`: Move to absolute position
//! - `moverel`: Move relatively from current position
//! - `scrollup`: Scroll wheel up
//! - `scrolldown`: Scroll wheel down
//! - `press`: Press and hold left button
//! - `release`: Release left button
//!
//! # Coordinate System and Execution Modes
//!
//! Mouse instructions support two execution modes:
//! - `send` (foreground, default): Direct input simulation to the system
//! - `post` (background): Sends messages to a specific window
//!
//! ## Performance Optimization
//!
//! All mouse instructions pre-build their INPUT structures at parse time for zero runtime overhead.
//! For instructions with coordinates (`click`, `move`, `scrollup`, `scrolldown`), the execution flow is:
//! 1. **Parse phase**: Pre-build the action part of INPUT (e.g., mouse down/up events)
//! 2. **Execute phase**: Use `SetCursorPos` for positioning (if coordinates provided), then execute pre-built INPUT
//!
//! This approach eliminates runtime INPUT construction and HashMap lookup overhead, achieving optimal performance.
//!
//! In contrast, `moverel` performs relative movement and doesn't require coordinate transformation,
//! so it directly uses the pre-built INPUT without additional positioning.
//!
//! # Instructions
//!
//! ## `click` - Click at position
//!
//! Performs a complete mouse click (press + release).
//!
//! **Syntax:** `click [x] [y] [delay_ms] [mode]`
//!
//! **Parameters:**
//! - `[x] [y]` (optional): Coordinates to click at. If omitted in Send mode, uses current cursor position.
//! - `[delay_ms]` (optional): Delay in milliseconds after click (0-1000ms). Useful for applications that ignore rapid consecutive clicks.
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! click 100 200              # Click at (100, 200) in foreground
//! click 100 200 post         # Click at (100, 200) in background
//! click                      # Click at current position (foreground only)
//! click send                 # Click at current position (explicit foreground)
//! click 100 200 50           # Click at (100, 200) with 50ms delay
//! click 100 200 100 post     # Click at (100, 200) in background with 100ms delay
//! ```
//!
//! **Notes:**
//! - PostMessage mode requires both x and y coordinates
//! - PostMessage mode requires `target_hwnd` to be set in VM state
//! - SendInput mode without coordinates uses current cursor position
//!
//! ---
//!
//! ## `move` - Move to absolute position
//!
//! Moves the mouse cursor to an absolute screen position.
//!
//! **Syntax:** `move <x> <y> [mode]`
//!
//! **Parameters:**
//! - `<x> <y>` (required): Target coordinates
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! move 100 200               # Move to (100, 200) in foreground
//! move 100 200 post          # Move to (100, 200) in background
//! ```
//!
//! **Notes:**
//! - PostMessage mode sends WM_MOUSEMOVE message to target window
//!
//! ---
//!
//! ## `moverel` - Move relatively
//!
//! Moves the mouse cursor relative to its current position.
//!
//! **Syntax:** `moverel <dx> <dy> [mode]`
//!
//! **Parameters:**
//! - `<dx> <dy>` (required): Offset from current position (positive = right/down, negative = left/up)
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! moverel 10 -5              # Move 10px right, 5px up (foreground)
//! moverel 10 -5 post         # Move relatively in background
//! ```
//!
//! **Notes:**
//! - Relative movement is primarily designed for SendInput mode
//!
//! ---
//!
//! ## `scrollup` - Scroll wheel up
//!
//! Scrolls the mouse wheel upward by a specified number of notches.
//!
//! **Syntax:** `scrollup [x] [y] [times] [mode]`
//!
//! **Parameters:**
//! - `[x] [y]` (optional): Coordinates to scroll at.
//!   - **PostMessage mode**: Specifies the position within the target window where the scroll occurs (mouse does NOT move).
//!   - **SendInput mode**: If provided, moves the mouse to the specified screen coordinates first, then scrolls. If omitted, scrolls at current cursor position.
//! - `[times]` (optional): Number of scroll notches (default: 1, range: 1-100). Each notch = 120 units (Windows standard).
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! scrollup                   # Scroll up 1 notch at current cursor position (foreground)
//! scrollup 3                 # Scroll up 3 notches at current position
//! scrollup 100 200           # Move to (100, 200) then scroll up 1 notch (foreground)
//! scrollup 100 200 5         # Move to (100, 200) then scroll up 5 notches (foreground)
//! scrollup 100 200 post      # Scroll up 1 notch at position (100, 200) within target window (background)
//! scrollup 100 200 3 post    # Scroll up 3 notches at position (100, 200) within target window (background)
//! ```
//!
//! **Important Notes:**
//! - Uses intuitive "times" parameter instead of raw delta values for better usability
//! - **PostMessage mode**: Does NOT move the mouse cursor. The x,y coordinates indicate where the scroll event occurs within the target window.
//! - **SendInput mode with coordinates**: Moves the mouse to the specified position BEFORE scrolling. This allows scrolling in different areas of an application without manual mouse positioning.
//! - **SendInput mode without coordinates**: Scrolls at the current mouse cursor position without moving.
//! - PostMessage mode requires `target_hwnd` to be set in VM state
//!
//! ---
//!
//! ## `scrolldown` - Scroll wheel down
//!
//! Scrolls the mouse wheel downward by a specified number of notches.
//!
//! **Syntax:** `scrolldown [x] [y] [times] [mode]`
//!
//! **Parameters:**
//! - `[x] [y]` (optional): Coordinates to scroll at.
//!   - **PostMessage mode**: Specifies the position within the target window where the scroll occurs (mouse does NOT move).
//!   - **SendInput mode**: If provided, moves the mouse to the specified screen coordinates first, then scrolls. If omitted, scrolls at current cursor position.
//! - `[times]` (optional): Number of scroll notches (default: 1, range: 1-100). Each notch = 120 units (Windows standard).
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! scrolldown                 # Scroll down 1 notch at current cursor position (foreground)
//! scrolldown 3               # Scroll down 3 notches at current position
//! scrolldown 100 200         # Move to (100, 200) then scroll down 1 notch (foreground)
//! scrolldown 100 200 5       # Move to (100, 200) then scroll down 5 notches (foreground)
//! scrolldown 100 200 post    # Scroll down 1 notch at position (100, 200) within target window (background)
//! scrolldown 100 200 3 post  # Scroll down 3 notches at position (100, 200) within target window (background)
//! ```
//!
//! **Important Notes:**
//! - Uses intuitive "times" parameter instead of raw delta values for better usability
//! - **PostMessage mode**: Does NOT move the mouse cursor. The x,y coordinates indicate where the scroll event occurs within the target window.
//! - **SendInput mode with coordinates**: Moves the mouse to the specified position BEFORE scrolling. This allows scrolling in different areas of an application without manual mouse positioning.
//! - **SendInput mode without coordinates**: Scrolls at the current mouse cursor position without moving.
//! - PostMessage mode requires `target_hwnd` to be set in VM state
//!
//! ---
//!
//! ## `press` - Press and hold left button
//!
//! Presses the left mouse button without releasing it. Use with `release` for drag operations.
//!
//! **Syntax:** `press [mode]`
//!
//! **Parameters:**
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! press                      # Press left button (foreground)
//! press post                 # Press left button (background, uses coordinates 0,0)
//! ```
//!
//! **Notes:**
//! - Use with `release` to create drag operations
//! - In PostMessage mode, uses default coordinates (0, 0)
//!
//! ---
//!
//! ## `release` - Release left button
//!
//! Releases the left mouse button that was previously pressed.
//!
//! **Syntax:** `release [mode]`
//!
//! **Parameters:**
//! - `[mode]` (optional): Execution mode - `send` (foreground, default) or `post` (background)
//!
//! **Examples:**
//! ```text
//! release                    # Release left button (foreground)
//! release post               # Release left button (background, uses coordinates 0,0)
//! ```
//!
//! **Notes:**
//! - Must be paired with `press` to avoid stuck buttons
//! - In PostMessage mode, uses default coordinates (0, 0)
//!
//! ---
//!
//! # Execution Modes
//!
//! ## Foreground Mode (`send`) - Default
//!
//! Simulates input using SendInput API (global system input).
//! - No target window configuration needed
//! - Works with the currently focused application
//! - Supports operations without coordinates (uses current cursor position)
//!
//! ```text
//! click                      # Same as: click send
//! move 100 200               # Same as: move 100 200 send
//! ```
//!
//! ## Background Mode (`post`)
//!
//! Sends messages directly to a specific window using PostMessage API.
//! - Requires `target_hwnd` to be set in VM state before execution
//! - Works even when window is not in focus
//! - Must specify mode explicitly
//! - Requires coordinates for most operations
//!
//! ```text
//! # Set target window first (in your Rust code)
//! # vm.set_persistent_state("target_hwnd", hwnd);
//!
//! click 100 200 post         # Click in background window
//! move 100 200 post          # Move in background window
//! ```
//!
//! ---
//!
//! # Common Use Cases
//!
//! ## Simple Click
//! ```text
//! click 100 200              # Click at specific position
//! click                      # Click at current position
//! ```
//!
//! ## Drag and Drop
//! ```text
//! # Move to source position
//! move 100 100
//! # Press and hold
//! press
//! # Move to destination
//! move 300 300
//! # Release
//! release
//! ```
//!
//! ## Drag and Drop (Background)
//! ```text
//! # Set target window first
//! # vm.set_persistent_state("target_hwnd", hwnd);
//!
//! press post
//! move 300 300 post
//! release post
//! ```
//!
//! ## Scrolling
//! ```text
//! scrollup 2                 # Scroll up two notches
//! scrolldown 1               # Scroll down one notch
//! ```
//!
//! ---
//!
//! # Error Messages
//!
//! The parser provides clear error messages for common mistakes:
//!
//! ```text
//! click post                 → "PostMessage mode requires coordinates"
//! move 100                   → "Missing coordinates"
//! moverel invalid 5          → "Invalid dx offset 'invalid'"
//! scrollup abc               → "Invalid scroll amount 'abc'"
//! ```
//!
//! ---
//!
//! # Important Notes
//!
//! 1. **Background mode requires setup**: Before using `post` mode, you must set the target window handle in your Rust code:
//!    ```rust
//!    vm.set_persistent_state("target_hwnd", some_hwnd);
//!    ```
//!
//! 2. **Coordinates are required in PostMessage mode**: Most operations in background mode require explicit coordinates.
//!
//! 3. **Press/Release pairing**: Always pair `press` with `release` to avoid stuck mouse buttons.
//!
//! 4. **Scroll amounts**: Standard scroll notch is 120 units. Multiply for faster scrolling (e.g., 240 = 2 notches).

use crate::script_engine::{instruction::ScriptError, VMContext};
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

// Submodules - each instruction handler in its own file
pub mod click;
pub mod move_cmd;
pub mod moverel;
pub mod press;
pub mod release;
pub mod scrolldown;
pub mod scrollup;

// Re-export handlers for convenience
pub use click::ClickHandler;
pub use move_cmd::MoveHandler;
pub use moverel::MoveRelHandler;
pub use press::PressHandler;
pub use release::ReleaseHandler;
pub use scrolldown::ScrollDownHandler;
pub use scrollup::ScrollUpHandler;

// ============================================================================
// Common Types and Utilities
// ============================================================================

/// Mouse operation mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseMode {
    /// Foreground mode using SendInput API
    Send,
    /// Background mode using PostMessage API
    Post,
}

impl std::fmt::Display for MouseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseMode::Send => write!(f, "send"),
            MouseMode::Post => write!(f, "post"),
        }
    }
}

/// Pre-compiled mouse click parameters (with window coordinates)
#[derive(Clone)]
pub struct ClickParams {
    /// X coordinate in window coordinates (relative to window rect, including title bar and borders)
    pub x: Option<i32>,
    /// Y coordinate in window coordinates
    pub y: Option<i32>,
    /// Operation mode (Send/Post)
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Delay in milliseconds after click (for applications that require interval between clicks)
    pub delay_ms: u32,
    /// Pre-built INPUT structures for SendInput mode (zero runtime allocation)
    pub send_inputs: Vec<INPUT>,
}

/// Pre-compiled mouse move parameters (with pre-built INPUT)
#[derive(Clone)]
pub struct MoveParams {
    /// Target X coordinate in window coordinates
    pub x: i32,
    /// Target Y coordinate in window coordinates
    pub y: i32,
    /// Operation mode (Send/Post)
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Pre-built INPUT structure for SendInput mode (cached after first execution)
    pub send_input: Option<INPUT>,
}

/// Pre-compiled mouse relative move parameters (with pre-built INPUT)
#[derive(Clone)]
pub struct MoveRelParams {
    /// Delta X offset
    pub dx: i32,
    /// Delta Y offset
    pub dy: i32,
    /// Operation mode (Send/Post) - always Send for relative movement
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Pre-built INPUT structure for SendInput mode
    pub send_input: Option<INPUT>,
}

/// Pre-compiled scroll parameters (with pre-built INPUT)
#[derive(Clone)]
pub struct ScrollParams {
    /// X coordinate for PostMessage mode or SendInput with coordinates
    pub x: Option<i32>,
    /// Y coordinate for PostMessage mode or SendInput with coordinates
    pub y: Option<i32>,
    /// Scroll delta amount (default: 120 = one notch)
    pub delta: i32,
    /// Operation mode (Send/Post)
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Pre-built INPUT vector for SendInput mode (zero runtime allocation)
    pub send_inputs: Vec<INPUT>,
}

/// Pre-compiled mouse press parameters
#[derive(Clone)]
pub struct PressParams {
    /// X coordinate for PostMessage mode or SendInput with coordinates (optional)
    pub x: Option<i32>,
    /// Y coordinate for PostMessage mode or SendInput with coordinates (optional)
    pub y: Option<i32>,
    /// Operation mode (Send/Post)
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Pre-built INPUT structures for SendInput mode (empty if built at execute time)
    pub send_inputs: Vec<INPUT>,
}

/// Pre-compiled mouse release parameters
#[derive(Clone)]
pub struct ReleaseParams {
    /// X coordinate for PostMessage mode or SendInput with coordinates (optional)
    pub x: Option<i32>,
    /// Y coordinate for PostMessage mode or SendInput with coordinates (optional)
    pub y: Option<i32>,
    /// Operation mode (Send/Post)
    pub mode: MouseMode,
    /// Whether mode was explicitly specified in the instruction
    pub mode_specified: bool,
    /// Pre-built INPUT structures for SendInput mode (empty if built at execute time)
    pub send_inputs: Vec<INPUT>,
}

/// Helper function to parse mouse mode parameter from instruction arguments.
///
/// Examines the last argument to determine if it's a mode specifier ("send" or "post").
/// Returns the parsed mode and the number of arguments consumed (0 or 1).
///
/// # Arguments
/// * `args` - Instruction arguments slice
/// * `default_mode` - Default mode to use if no mode is specified
///
/// # Returns
/// * `(MouseMode, usize)` - Tuple of (parsed mode, number of args consumed)
///
/// # Examples
/// ```ignore
/// parse_mouse_mode(&["100", "200"], MouseMode::Send)  // → (Send, 0)
/// parse_mouse_mode(&["100", "200", "post"], MouseMode::Send)  // → (Post, 1)
/// ```
#[inline]
pub(crate) fn parse_mouse_mode(
    args: &[&str],
    default_mode: MouseMode,
) -> Result<(MouseMode, usize), ScriptError> {
    if args.is_empty() {
        return Ok((default_mode, 0));
    }

    // Check if last argument is mode
    let last_arg = args[args.len() - 1];
    match last_arg {
        "post" => Ok((MouseMode::Post, 1)),
        "send" => Ok((MouseMode::Send, 1)),
        _ => Ok((default_mode, 0)), // Not a mode parameter
    }
}

/// Constant key for storing/retrieving the default input mode in VM persistent state.
///
/// This key is used by `get_input_mode()` to fetch the current default mode setting.
/// Users can set this via `vm.set_persistent_state(INPUT_MODE_KEY, "post")` to change
/// the default mode for all subsequent mouse/keyboard instructions.
const INPUT_MODE_KEY: &str = "input_mode";

/// Get the current default input mode from VM persistent state.
///
/// Returns the mode configured via `vm.set_persistent_state("input_mode", ...)`.
/// If not set, defaults to "send" (foreground mode).
///
/// # Arguments
/// * `vm` - Virtual machine context reference
///
/// # Returns
/// Current default mode as a string ("send" or "post")
///
/// # Examples
/// ```ignore
/// let mode = get_input_mode(vm);  // → "send" (default)
/// vm.set_persistent_state("input_mode", "post");
/// let mode = get_input_mode(vm);  // → "post"
/// ```
#[inline]
pub(crate) fn get_input_mode(vm: &VMContext) -> String {
    vm.get_persistent_state::<String>(INPUT_MODE_KEY)
        .cloned()
        .unwrap_or_else(|| "send".to_string())
}

/// Convert window-relative coordinates to screen coordinates for SendInput mode.
///
/// This function adds the window's screen offset to convert coordinates that are relative
/// to the window rectangle (including title bar and borders) into absolute screen coordinates.
///
/// # Coordinate System
/// - **Input**: Window-relative coordinates (0,0 = top-left corner of window including non-client area)
/// - **Output**: Screen coordinates (absolute position on the desktop)
///
/// # Arguments
/// * `vm` - Virtual machine context containing process/window information
/// * `window_rel_x` - X coordinate relative to window rectangle
/// * `window_rel_y` - Y coordinate relative to window rectangle
///
/// # Returns
/// * `Ok((screen_x, screen_y))` - Converted screen coordinates
/// * `Err(ScriptError::NoWindowHandle)` - If window geometry is not available
///
/// # Formula
/// ```text
/// screen_x = window_rel_x + window_left_offset
/// screen_y = window_rel_y + window_top_offset
/// ```
///
/// # Examples
/// ```ignore
/// // If window is at screen position (100, 200) with title bar height 30px
/// let (sx, sy) = convert_to_window_coords(vm, 50, 80)?;
/// // → sx = 150, sy = 280 (50+100, 80+200)
/// ```
///
/// # Notes
/// - Only available when `script_process_context` feature is enabled
/// - Requires `vm.process.window_geometry` to be set (via HWND tracking)
#[cfg(feature = "script_process_context")]
#[inline]
pub(crate) fn convert_to_window_coords(
    vm: &mut VMContext,
    window_rel_x: i32,
    window_rel_y: i32,
) -> Result<(i32, i32), ScriptError> {
    let geo = vm
        .process
        .window_geometry
        .ok_or(ScriptError::NoWindowHandle)?;

    let client_x = window_rel_x + geo.window_left;
    let client_y = window_rel_y + geo.window_top;

    Ok((client_x, client_y))
}

/// Convert window-relative coordinates to client area coordinates for PostMessage mode.
///
/// This function subtracts the non-client area offset (title bar, borders) to convert
/// coordinates that are relative to the window rectangle into coordinates relative to
/// the client area (the actual content area of the window).
///
/// # Coordinate System
/// - **Input**: Window-relative coordinates (0,0 = top-left corner of window including title bar/borders)
/// - **Output**: Client area coordinates (0,0 = top-left corner of the client/content area)
///
/// # Arguments
/// * `vm` - Virtual machine context containing process/window information
/// * `window_rel_x` - X coordinate relative to window rectangle
/// * `window_rel_y` - Y coordinate relative to window rectangle
///
/// # Returns
/// * `Ok((client_x, client_y))` - Converted client area coordinates
/// * `Err(ScriptError::NoWindowHandle)` - If window geometry is not available
///
/// # Formula
/// ```text
/// client_x = window_rel_x - offset_x  (offset_x = distance from window edge to client area)
/// client_y = window_rel_y - offset_y  (offset_y = title bar height + border width)
/// ```
///
/// # Examples
/// ```ignore
/// // If window has 8px border and 30px title bar (offset_x=8, offset_y=38)
/// let (cx, cy) = convert_to_client_coords(vm, 50, 80)?;
/// // → cx = 42, cy = 42 (50-8, 80-38)
/// ```
///
/// # Notes
/// - Only available when `script_process_context` feature is enabled
/// - Requires `vm.process.window_geometry` to be set (via HWND tracking)
/// - Essential for PostMessage API which uses client area coordinates
#[cfg(feature = "script_process_context")]
#[inline]
pub(crate) fn convert_to_client_coords(
    vm: &mut VMContext,
    window_rel_x: i32,
    window_rel_y: i32,
) -> Result<(i32, i32), ScriptError> {
    let geo = vm
        .process
        .window_geometry
        .ok_or(ScriptError::NoWindowHandle)?;

    let client_x = window_rel_x - geo.offset_x;
    let client_y = window_rel_y - geo.offset_y;

    Ok((client_x, client_y))
}
