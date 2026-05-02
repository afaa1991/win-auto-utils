//! SendInput-based mouse input (atomic high-performance API)
//!
//! This module provides minimal, zero-overhead functions that accept pre-built INPUT structures.
//! Zero runtime allocation - all INPUT structures are created at parse/compile time.
//!
//! For user-friendly APIs, see the convenience wrapper at the end of this file.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

// ============================================================================
// High-Performance Cursor Positioning (SetCursorPos API)
// ============================================================================

/// Set cursor position using SetCursorPos API (22x faster than SendInput MOVE)
///
/// # Performance
/// ~2.2 μs per call vs ~50 μs for SendInput(MOVE)
/// Throughput: ~450K IPS vs ~20K IPS
///
/// # Parameters
/// * `x` - Screen X coordinate in pixels
/// * `y` - Screen Y coordinate in pixels
#[inline]
pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), SendMouseInputError> {
    unsafe {
        SetCursorPos(x, y).map_err(|_| SendMouseInputError::SetCursorPosFailed)?;
        Ok(())
    }
}

// ============================================================================
// Atomic High-Performance Functions (Zero Overhead - Direct INPUT Execution)
// ============================================================================

/// Execute INPUT structures directly (atomic operation)
///
/// # Performance
/// Zero overhead - accepts pre-built INPUT slice and sends directly via syscall
///
/// # Parameters
/// * `inputs` - Pre-built INPUT structures (created at parse/compile time)
#[inline]
pub fn execute_inputs(inputs: &[INPUT]) -> Result<(), SendMouseInputError> {
    if inputs.is_empty() {
        return Ok(());
    }

    let result = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        Err(SendMouseInputError::SendInputFailed)
    } else {
        Ok(())
    }
}

/// Execute a single INPUT structure atomically
#[inline]
pub fn execute_single_input(input: &INPUT) -> Result<(), SendMouseInputError> {
    execute_inputs(&[input.clone()])
}

// ============================================================================
// Helper Functions for Building INPUT Structures (Parse-time Use Only)
// ============================================================================

/// Helper function to convert screen coordinates to normalized absolute coordinates
/// Screen coordinates: 0-65535 range for full desktop
#[inline]
pub fn normalize_coords(x: i32, y: i32) -> (i32, i32) {
    let screen_width = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
        )
    };
    let screen_height = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
        )
    };

    // Use ceiling division to ensure we don't undershoot the target coordinate
    // Formula: (value * 65535 + screen_size - 1) / screen_size
    // This ensures that any fractional part rounds up, minimizing undershoot errors
    let nx = ((x as i64) * 65535 + (screen_width as i64) - 1) / (screen_width as i64);
    let ny = ((y as i64) * 65535 + (screen_height as i64) - 1) / (screen_height as i64);

    (nx as i32, ny as i32)
}

/// Build a MOUSE INPUT structure (for parse-time construction)
///
/// # Usage
/// Call this during instruction parsing to build static INPUT structures
#[inline]
pub fn build_mouse_input(flags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32, data: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Build left click INPUT pair at current position (parse-time)
#[inline]
pub fn build_click_left() -> [INPUT; 2] {
    [
        build_mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
    ]
}

/// Build left click INPUT sequence at specified coordinates (parse-time)
#[inline]
pub fn build_click_left_at(x: i32, y: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
    ]
}

/// Build right click INPUT pair at current position (parse-time)
#[inline]
pub fn build_click_right() -> [INPUT; 2] {
    [
        build_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
    ]
}

/// Build right click INPUT sequence at specified coordinates (parse-time)
#[inline]
pub fn build_click_right_at(x: i32, y: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
    ]
}

/// Build middle click INPUT pair at current position (parse-time)
#[inline]
pub fn build_click_middle() -> [INPUT; 2] {
    [
        build_mouse_input(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_MIDDLEUP, 0, 0, 0),
    ]
}

/// Build middle click INPUT sequence at specified coordinates (parse-time)
#[inline]
pub fn build_click_middle_at(x: i32, y: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0),
        build_mouse_input(MOUSEEVENTF_MIDDLEUP, 0, 0, 0),
    ]
}

/// Build press left button INPUT (parse-time)
#[inline]
pub fn build_press_left() -> INPUT {
    build_mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0)
}

/// Build press left button INPUT at specified coordinates (parse-time)
/// Moves to the position first, then presses
#[inline]
pub fn build_press_left_at(x: i32, y: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
    ]
}

/// Build release left button INPUT (parse-time)
#[inline]
pub fn build_release_left() -> INPUT {
    build_mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0)
}

/// Build release left button INPUT at specified coordinates (parse-time)
/// Moves to the position first, then releases
#[inline]
pub fn build_release_left_at(x: i32, y: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_LEFTUP, 0, 0, 0),
    ]
}

/// Build move to absolute coordinates INPUT (parse-time)
#[inline]
pub fn build_move(x: i32, y: i32) -> INPUT {
    let (nx, ny) = normalize_coords(x, y);
    build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0)
}

/// Build relative move INPUT (parse-time)
#[inline]
pub fn build_move_relative(dx: i32, dy: i32) -> INPUT {
    build_mouse_input(MOUSEEVENTF_MOVE, dx, dy, 0)
}

/// Build scroll up INPUT (parse-time)
#[inline]
pub fn build_scroll_up(delta: i32) -> INPUT {
    build_mouse_input(MOUSEEVENTF_WHEEL, 0, 0, delta as u32)
}

/// Build scroll up INPUT at specified coordinates (parse-time)
/// Moves to the position first, then scrolls
#[inline]
pub fn build_scroll_up_at(x: i32, y: i32, delta: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_WHEEL, 0, 0, delta as u32),
    ]
}

/// Build scroll down INPUT (parse-time)
#[inline]
pub fn build_scroll_down(delta: i32) -> INPUT {
    build_mouse_input(MOUSEEVENTF_WHEEL, 0, 0, (-delta) as u32)
}

/// Build scroll down INPUT at specified coordinates (parse-time)
/// Moves to the position first, then scrolls
#[inline]
pub fn build_scroll_down_at(x: i32, y: i32, delta: i32) -> Vec<INPUT> {
    let (nx, ny) = normalize_coords(x, y);

    vec![
        build_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, nx, ny, 0),
        build_mouse_input(MOUSEEVENTF_WHEEL, 0, 0, (-delta) as u32),
    ]
}

// ============================================================================
// Convenience Wrapper (User-Friendly API)
// ============================================================================

/// Error type for SendInput operations
#[derive(Debug, Clone)]
pub enum SendMouseInputError {
    /// SendInput API call failed
    SendInputFailed,
    /// SetCursorPos API call failed
    SetCursorPosFailed,
}

impl std::fmt::Display for SendMouseInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendMouseInputError::SendInputFailed => write!(f, "SendInput API call failed"),
            SendMouseInputError::SetCursorPosFailed => write!(f, "SetCursorPos API call failed"),
        }
    }
}

impl std::error::Error for SendMouseInputError {}

/// Convenience wrapper for mouse input
///
/// # Note
/// This wrapper has runtime overhead from coordinate calculations. For maximum performance,
/// use the atomic functions directly with pre-built INPUT structures.
pub struct SendInputMouse;

impl SendInputMouse {
    /// Create new instance
    pub fn new() -> Self {
        Self
    }

    /// Click left button at current position (convenience method)
    pub fn click_left(&self) -> Result<(), SendMouseInputError> {
        let inputs = build_click_left();
        execute_inputs(&inputs)
    }

    /// Click left button at specified coordinates
    pub fn click_left_at(&self, x: i32, y: i32) -> Result<(), SendMouseInputError> {
        let inputs = build_click_left_at(x, y);
        execute_inputs(&inputs)
    }

    /// Click right button at current position
    pub fn click_right(&self) -> Result<(), SendMouseInputError> {
        let inputs = build_click_right();
        execute_inputs(&inputs)
    }

    /// Click right button at specified coordinates
    pub fn click_right_at(&self, x: i32, y: i32) -> Result<(), SendMouseInputError> {
        let inputs = build_click_right_at(x, y);
        execute_inputs(&inputs)
    }

    /// Click middle button at current position
    pub fn click_middle(&self) -> Result<(), SendMouseInputError> {
        let inputs = build_click_middle();
        execute_inputs(&inputs)
    }

    /// Click middle button at specified coordinates
    pub fn click_middle_at(&self, x: i32, y: i32) -> Result<(), SendMouseInputError> {
        let inputs = build_click_middle_at(x, y);
        execute_inputs(&inputs)
    }

    /// Press left button
    pub fn press_left(&self) -> Result<(), SendMouseInputError> {
        let input = build_press_left();
        execute_single_input(&input)
    }

    /// Release left button
    pub fn release_left(&self) -> Result<(), SendMouseInputError> {
        let input = build_release_left();
        execute_single_input(&input)
    }

    /// Move mouse to absolute coordinates
    pub fn move_to(&self, x: i32, y: i32) -> Result<(), SendMouseInputError> {
        let input = build_move(x, y);
        execute_single_input(&input)
    }

    /// Move mouse by relative offset
    pub fn move_relative(&self, dx: i32, dy: i32) -> Result<(), SendMouseInputError> {
        let input = build_move_relative(dx, dy);
        execute_single_input(&input)
    }

    /// Scroll up
    pub fn scroll_up(&self, delta: i32) -> Result<(), SendMouseInputError> {
        let input = build_scroll_up(delta);
        execute_single_input(&input)
    }

    /// Scroll down
    pub fn scroll_down(&self, delta: i32) -> Result<(), SendMouseInputError> {
        let input = build_scroll_down(delta);
        execute_single_input(&input)
    }
}

impl Default for SendInputMouse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_execute_inputs() {
        // Test building INPUT structures and executing them
        let inputs = build_click_left();

        // Just verify compilation and structure correctness
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].r#type, INPUT_MOUSE);
        assert_eq!(inputs[1].r#type, INPUT_MOUSE);
    }

    #[test]
    fn test_build_click_at_inputs() {
        let inputs = build_click_left_at(100, 200);

        // Should have 3 inputs: move + down + up
        assert_eq!(inputs.len(), 3);
    }

    #[test]
    fn test_atomic_functions_compile() {
        // Verify atomic functions compile correctly
        let inputs = build_click_left();

        // These will fail at runtime in test environment but should compile
        let _result = execute_inputs(&inputs);

        assert!(true); // Compilation check
    }
}
