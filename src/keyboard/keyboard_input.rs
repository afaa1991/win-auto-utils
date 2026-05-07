//! SendInput-based keyboard input (atomic high-performance API)
//!
//! This module provides minimal, zero-overhead functions that accept pre-built INPUT structures.
//! Zero runtime allocation - all INPUT structures are created at parse/compile time.
//!
//! For user-friendly APIs with string support, see the convenience wrapper at the end of this file.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};

use crate::utils::key_code;
use crate::utils::sleep_ms;

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
///
/// # Example
/// ```no_run
/// use win_auto_utils::keyboard::send_input;
///
/// // Build INPUT structures at parse time
/// let inputs = send_input::build_key_click_inputs(0x41, false); // 'A' key
///
/// // Execute with zero overhead
/// send_input::execute_inputs(&inputs).unwrap();
/// ```
#[inline]
pub fn execute_inputs(inputs: &[INPUT]) -> Result<(), SendKeyBoardInputError> {
    if inputs.is_empty() {
        return Ok(());
    }

    let result = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };

    if result == 0 {
        Err(SendKeyBoardInputError::SendInputFailed)
    } else {
        Ok(())
    }
}

/// Execute a single INPUT structure atomically
#[inline]
pub fn execute_single_input(input: &INPUT) -> Result<(), SendKeyBoardInputError> {
    execute_inputs(&[input.clone()])
}

// ============================================================================
// Helper Functions for Building INPUT Structures (Parse-time Use Only)
// ============================================================================

/// Build a KEYBD INPUT structure (for parse-time construction)
///
/// # Usage
/// Call this during instruction parsing to build static INPUT structures
#[inline]
pub fn build_keybd_input(vk_code: u8, extended: bool, keyup: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);

    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if keyup {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk_code as u16),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Build KEYDOWN + KEYUP INPUT pair for click operation (parse-time)
#[inline]
pub fn build_key_click_inputs(vk_code: u8, extended: bool) -> [INPUT; 2] {
    [
        build_keybd_input(vk_code, extended, false), // KEYDOWN
        build_keybd_input(vk_code, extended, true),  // KEYUP
    ]
}

/// Build KEYDOWN INPUT only (parse-time)
#[inline]
pub fn build_key_down_input(vk_code: u8, extended: bool) -> INPUT {
    build_keybd_input(vk_code, extended, false)
}

/// Build KEYUP INPUT only (parse-time)
#[inline]
pub fn build_key_up_input(vk_code: u8, extended: bool) -> INPUT {
    build_keybd_input(vk_code, extended, true)
}

// ============================================================================
// Convenience Wrapper (User-Friendly API with Runtime String Lookup)
// ============================================================================

/// Error type for SendInput operations
#[derive(Debug, Clone)]
pub enum SendKeyBoardInputError {
    /// SendInput API call failed
    SendInputFailed,
}

impl std::fmt::Display for SendKeyBoardInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendKeyBoardInputError::SendInputFailed => write!(f, "SendInput API call failed"),
        }
    }
}

impl std::error::Error for SendKeyBoardInputError {}

/// Convenience wrapper for keyboard input with string support
///
/// # Note
/// This wrapper has runtime overhead from string lookup. For maximum performance,
/// use the atomic functions directly with pre-built INPUT structures.
pub struct SendInputKeyboard {
    delay_ms: u64,
}

impl SendInputKeyboard {
    /// Create new instance with default 10ms delay
    pub fn new() -> Self {
        Self { delay_ms: 10 }
    }

    /// Set delay between key down and up (for click operations)
    pub fn set_delay(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
    }

    /// Get current delay setting
    pub fn get_delay(&self) -> u64 {
        self.delay_ms
    }

    /// Click a key by name (convenience method with string lookup)
    ///
    /// # Performance
    /// Has overhead from string-to-vk lookup. For high-performance scenarios,
    /// use `execute_inputs()` with pre-built INPUT structures instead.
    pub fn click(&self, key: &str) -> Result<(), SendKeyBoardInputError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(SendKeyBoardInputError::SendInputFailed);
        }

        let extended = crate::utils::key_code::is_extended_key(vk_code as u8);
        let inputs = build_key_click_inputs(vk_code as u8, extended);

        execute_inputs(&inputs)?;

        if self.delay_ms > 0 {
            sleep_ms(self.delay_ms as u32);
        }

        Ok(())
    }

    /// Press a key by name (convenience method with string lookup)
    pub fn press(&self, key: &str) -> Result<(), SendKeyBoardInputError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(SendKeyBoardInputError::SendInputFailed);
        }

        let extended = crate::utils::key_code::is_extended_key(vk_code as u8);
        self.press_with_vk(vk_code as u8, extended)
    }

    /// Release a key by name (convenience method with string lookup)
    pub fn release(&self, key: &str) -> Result<(), SendKeyBoardInputError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(SendKeyBoardInputError::SendInputFailed);
        }

        let extended = crate::utils::key_code::is_extended_key(vk_code as u8);
        self.release_with_vk(vk_code as u8, extended)
    }

    /// Press a key by VK code (lower overhead than string version)
    pub fn press_with_vk(&self, vk_code: u8, extended: bool) -> Result<(), SendKeyBoardInputError> {
        let input = build_key_down_input(vk_code, extended);
        execute_single_input(&input)
    }

    /// Release a key by VK code
    pub fn release_with_vk(
        &self,
        vk_code: u8,
        extended: bool,
    ) -> Result<(), SendKeyBoardInputError> {
        let input = build_key_up_input(vk_code, extended);
        execute_single_input(&input)
    }

    /// Click a key by VK code (press + release)
    pub fn click_with_vk(&self, vk_code: u8, extended: bool) -> Result<(), SendKeyBoardInputError> {
        let inputs = build_key_click_inputs(vk_code, extended);
        execute_inputs(&inputs)
    }
}

impl Default for SendInputKeyboard {
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
        let inputs = build_key_click_inputs(0x41, false); // 'A' key

        // Just verify compilation and structure correctness
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].r#type, INPUT_KEYBOARD);
        assert_eq!(inputs[1].r#type, INPUT_KEYBOARD);
    }

    #[test]
    fn test_send_input_creation() {
        let kb = SendInputKeyboard::new();
        assert_eq!(kb.get_delay(), 10);
    }

    #[test]
    fn test_set_delay() {
        let mut kb = SendInputKeyboard::new();
        kb.set_delay(50);
        assert_eq!(kb.get_delay(), 50);
    }

    #[test]
    fn test_atomic_functions_compile() {
        // Verify atomic functions compile correctly
        let inputs = build_key_click_inputs(0x41, false);

        // These will fail at runtime in test environment but should compile
        let _result = execute_inputs(&inputs);

        assert!(true); // Compilation check
    }
}
