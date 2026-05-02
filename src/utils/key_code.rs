// src/input/key_code.rs
//! High-performance keyboard key code conversion
//!
//! Converts key name strings to Windows virtual key codes.
//! Supports all standard keyboard keys including special characters
//! that share the same physical key (e.g., ` and ~).
//!
//! # Performance
//! - Zero system calls - pure memory operations
//! - O(1) lookup for single characters via match table
//! - Compiler-optimized jump tables for pattern matching
//! - No heap allocation except for case conversion

/// Convert key name string to Windows virtual key code
///
/// # Arguments
/// * `name` - Key name (case-insensitive)
///   - Single character: "a", "A", "`", "~", etc.
///   - Special keys: "esc", "f1", "space", "enter", etc.
///
/// # Returns
/// Windows virtual key code (u8), or 0x00 if unknown
///
/// # Examples
/// ```
/// use win_auto_utils::input::key_code;
///
/// // Letters (case-insensitive)
/// assert_eq!(key_code("a"), 0x41);
/// assert_eq!(key_code("A"), 0x41);
///
/// // Shared keys (same physical key)
/// assert_eq!(key_code("`"), 192);  // Backtick
/// assert_eq!(key_code("~"), 192);  // Tilde (Shift + `)
/// assert_eq!(key_code("-"), 189);  // Minus
/// assert_eq!(key_code("_"), 189);  // Underscore (Shift + -)
///
/// // Special keys
/// assert_eq!(key_code("esc"), 0x1B);
/// assert_eq!(key_code("f1"), 0x70);
/// assert_eq!(key_code("space"), 0x20);
/// ```
pub fn key_code(name: &str) -> u8 {
    let name = name.trim().to_uppercase();

    // Single character keys (including Shift variants)
    if name.len() == 1 {
        return match_single_char(name.as_str());
    }

    // Multi-character special keys
    match_special_key(&name)
}

/// Handle single character key conversion
#[inline]
fn match_single_char(ch: &str) -> u8 {
    match ch {
        // Punctuation keys (with/without Shift)
        ";" | ":" => 186,  // Semicolon/Colon
        "=" | "+" => 187,  // Equals/Plus
        "," | "<" => 188,  // Comma/Less-than
        "-" | "_" => 189,  // Minus/Underscore
        "." | ">" => 190,  // Period/Greater-than
        "/" | "?" => 191,  // Slash/Question mark
        "`" | "~" => 192,  // Backtick/Tilde
        "[" | "{" => 219,  // Left bracket/Brace
        "\\" | "|" => 220, // Backslash/Pipe
        "]" | "}" => 221,  // Right bracket/Brace
        "'" | "\"" => 222, // Single quote/Double quote

        // Regular alphanumeric - return uppercase ASCII value
        _ => {
            let byte = ch.as_bytes()[0];
            // Convert lowercase to uppercase for consistent VK codes
            if byte >= b'a' && byte <= b'z' {
                byte - 32 // Convert to uppercase (a=97 -> A=65)
            } else {
                byte
            }
        }
    }
}

/// Handle multi-character special key names
#[inline]
fn match_special_key(name: &str) -> u8 {
    match name {
        // Control keys
        "BACK" | "BACKSPACE" => 0x08,
        "TAB" => 0x09,
        "CLEAR" => 0x0C,
        "ENTER" | "RETURN" => 0x0D,
        "SHIFT" => 0x10,
        "CTRL" | "CONTROL" => 0x11,
        "ALT" | "MENU" => 0x12,
        "PAUSE" => 0x13,
        "CAPS" | "CAPITAL" => 0x14,
        "ESC" | "ESCAPE" => 0x1B,
        "SPACE" => 0x20,

        // Navigation keys
        "PAGEUP" | "PRIOR" => 0x21,
        "PAGEDOWN" | "NEXT" => 0x22,
        "END" => 0x23,
        "HOME" => 0x24,
        "LEFT" => 0x25,
        "UP" => 0x26,
        "RIGHT" => 0x27,
        "DOWN" => 0x28,
        "SELECT" => 0x29,
        "PRINT" => 0x2A,
        "EXECUTE" => 0x2B,
        "SNAPSHOT" | "PRINTSCREEN" => 0x2C,
        "INS" | "INSERT" => 0x2D,
        "DEL" | "DELETE" => 0x2E,
        "HELP" => 0x2F,

        // Function keys F1-F24
        _ if name.starts_with('F') && name.len() >= 2 => parse_function_key(&name[1..]),

        // Numpad keys NUM0-NUM9
        _ if name.starts_with("NUM") && name.len() == 4 => parse_numpad_key(&name[3..]),

        // Numpad operators
        "MULTIPLY" => 0x6A,
        "ADD" => 0x6B,
        "SEPARATOR" => 0x6C,
        "SUBTRACT" => 0x6D,
        "DECIMAL" => 0x6E,
        "DIVIDE" => 0x6F,

        // Lock keys
        "NUMLOCK" => 0x90,
        "SCROLL" | "SCROLLLOCK" => 0x91,

        // Shift variants
        "LSHIFT" | "LSHIFTKEY" => 0xA0,
        "RSHIFT" | "RSHIFTKEY" => 0xA1,
        "LCTRL" | "LCONTROL" => 0xA2,
        "RCTRL" | "RCONTROL" => 0xA3,
        "LALT" | "LMENU" => 0xA4,
        "RALT" | "RMENU" => 0xA5,

        // Browser/Media keys (Windows 2000+)
        "BROWSER_BACK" => 0xA6,
        "BROWSER_FORWARD" => 0xA7,
        "BROWSER_REFRESH" => 0xA8,
        "BROWSER_STOP" => 0xA9,
        "BROWSER_SEARCH" => 0xAA,
        "BROWSER_FAVORITES" => 0xAB,
        "BROWSER_HOME" => 0xAC,
        "VOLUME_MUTE" => 0xAD,
        "VOLUME_DOWN" => 0xAE,
        "VOLUME_UP" => 0xAF,
        "MEDIA_NEXT" => 0xB0,
        "MEDIA_PREV" => 0xB1,
        "MEDIA_STOP" => 0xB2,
        "MEDIA_PLAY_PAUSE" => 0xB3,
        "LAUNCH_MAIL" => 0xB4,
        "LAUNCH_MEDIA" => 0xB5,
        "LAUNCH_APP1" => 0xB6,
        "LAUNCH_APP2" => 0xB7,

        // OEM keys (region-specific)
        "OEM_1" => 0xBA,
        "OEM_PLUS" => 0xBB,
        "OEM_COMMA" => 0xBC,
        "OEM_MINUS" => 0xBD,
        "OEM_PERIOD" => 0xBE,
        "OEM_2" => 0xBF,
        "OEM_3" => 0xC0,
        "OEM_4" => 0xDB,
        "OEM_5" => 0xDC,
        "OEM_6" => 0xDD,
        "OEM_7" => 0xDE,
        "OEM_8" => 0xDF,
        "OEM_102" => 0xE2,

        // IME keys
        "PROCESSKEY" => 0xE5,
        "PACKET" => 0xE7,
        "ATTN" => 0xF6,
        "CRSEL" => 0xF7,
        "EXSEL" => 0xF8,
        "EREOF" => 0xF9,
        "PLAY" => 0xFA,
        "ZOOM" => 0xFB,
        "NONAME" => 0xFC,
        "PA1" => 0xFD,
        "OEM_CLEAR" => 0xFE,

        // Unknown key
        _ => 0x00,
    }
}

/// Parse function key number (F1-F24)
#[inline]
fn parse_function_key(num_str: &str) -> u8 {
    if let Ok(num) = num_str.parse::<u8>() {
        if num >= 1 && num <= 24 {
            return 0x6F + num; // F1=0x70, F24=0x87
        }
    }
    0x00
}

/// Parse numpad key number (NUM0-NUM9)
#[inline]
fn parse_numpad_key(num_str: &str) -> u8 {
    if let Ok(num) = num_str.parse::<u8>() {
        if num <= 9 {
            return 0x60 + num; // NUM0=0x60, NUM9=0x69
        }
    }
    0x00
}

/// Check if a virtual key code requires the extended key flag
///
/// Extended keys include arrow keys, navigation keys, insert/delete,
/// numpad divide, and num lock. These keys need the KEYEVENTF_EXTENDEDKEY
/// flag when using SendInput API.
///
/// # Arguments
/// * `vk_code` - Virtual key code
///
/// # Returns
/// * `true` - Key requires extended flag
/// * `false` - Key does not require extended flag
///
/// # Examples
/// ```
/// use win_auto_utils::utils::key_code::is_extended_key;
///
/// assert!(is_extended_key(0x25));  // Left arrow
/// assert!(is_extended_key(0x26));  // Up arrow
/// assert!(!is_extended_key(0x41)); // 'A'
/// assert!(!is_extended_key(0x0D)); // Enter
/// ```
pub fn is_extended_key(vk_code: u8) -> bool {
    matches!(
        vk_code,
        0x25..=0x28 | // Arrow keys (Left, Up, Right, Down)
        0x21..=0x24 | // Page Up/Down, Home, End
        0x2D..=0x2E | // Insert, Delete
        0x6F |        // Numpad Divide
        0x90          // Num Lock
    )
}

/// Get scan code from virtual key code using Windows MapVirtualKey API
///
/// Scan codes are hardware-dependent key identifiers used by PostMessage API.
/// This function converts a virtual key code (VK) to its corresponding scan code.
///
/// # Arguments
/// * `vk_code` - Virtual key code
///
/// # Returns
/// Scan code (u16) for the given virtual key code
///
/// # Examples
/// ```
/// use win_auto_utils::utils::key_code::get_scan_code;
///
/// let scan = get_scan_code(0x41); // 'A' key
/// assert!(scan > 0);
/// ```
pub fn get_scan_code(vk_code: u8) -> u16 {
    unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(
            vk_code as u32,
            windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC,
        ) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_letters() {
        println!("\n[Testing Single Letters]");

        // Lowercase - should return uppercase VK codes
        for ch in b'a'..=b'z' {
            let key = key_code(&(ch as char).to_string());
            let expected = ch - 32; // Convert to uppercase VK code
            println!(
                "  '{}' → 0x{:02X} (expected 0x{:02X})",
                ch as char, key, expected
            );
            assert_eq!(key, expected);
        }

        // Uppercase - should return uppercase VK codes
        for ch in b'A'..=b'Z' {
            let key = key_code(&(ch as char).to_string());
            println!("  '{}' → 0x{:02X}", ch as char, key);
            assert_eq!(key, ch);
        }
    }

    #[test]
    fn test_numbers() {
        println!("\n[Testing Numbers]");

        for ch in b'0'..=b'9' {
            let key = key_code(&(ch as char).to_string());
            println!("  '{}' → 0x{:02X}", ch as char, key);
            assert_eq!(key, ch);
        }
    }

    #[test]
    fn test_shared_keys() {
        println!("\n[Testing Shared Keys (Shift variants)]");

        let shared_pairs = vec![
            ("`", 192, "~", 192),
            ("-", 189, "_", 189),
            ("=", 187, "+", 187),
            ("[", 219, "{", 219),
            ("]", 221, "}", 221),
            ("\\", 220, "|", 220),
            (";", 186, ":", 186),
            ("'", 222, "\"", 222),
            (",", 188, "<", 188),
            (".", 190, ">", 190),
            ("/", 191, "?", 191),
        ];

        for (key1, code1, key2, code2) in shared_pairs {
            let result1 = key_code(key1);
            let result2 = key_code(key2);
            println!(
                "  '{}' → 0x{:02X}, '{}' → 0x{:02X}",
                key1, result1, key2, result2
            );
            assert_eq!(result1, code1);
            assert_eq!(result2, code2);
            assert_eq!(result1, result2, "Shared keys should have same code");
        }
    }

    #[test]
    fn test_control_keys() {
        println!("\n[Testing Control Keys]");

        let control_keys = vec![
            ("back", 0x08),
            ("tab", 0x09),
            ("enter", 0x0D),
            ("shift", 0x10),
            ("ctrl", 0x11),
            ("alt", 0x12),
            ("esc", 0x1B),
            ("space", 0x20),
        ];

        for (name, expected) in control_keys {
            let result = key_code(name);
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_navigation_keys() {
        println!("\n[Testing Navigation Keys]");

        let nav_keys = vec![
            ("pageup", 0x21),
            ("pagedown", 0x22),
            ("end", 0x23),
            ("home", 0x24),
            ("left", 0x25),
            ("up", 0x26),
            ("right", 0x27),
            ("down", 0x28),
            ("insert", 0x2D),
            ("delete", 0x2E),
        ];

        for (name, expected) in nav_keys {
            let result = key_code(name);
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_function_keys() {
        println!("\n[Testing Function Keys F1-F12]");

        for i in 1..=12 {
            let name = format!("f{}", i);
            let result = key_code(&name);
            let expected = 0x6F + i as u8;
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_numpad_keys() {
        println!("\n[Testing Numpad Keys]");

        for i in 0..=9 {
            let name = format!("num{}", i);
            let result = key_code(&name);
            let expected = 0x60 + i as u8;
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_media_keys() {
        println!("\n[Testing Media/Browser Keys]");

        let media_keys = vec![
            ("volume_mute", 0xAD),
            ("volume_down", 0xAE),
            ("volume_up", 0xAF),
            ("media_next", 0xB0),
            ("media_prev", 0xB1),
            ("media_play_pause", 0xB3),
        ];

        for (name, expected) in media_keys {
            let result = key_code(name);
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_case_insensitive() {
        println!("\n[Testing Case Insensitivity]");

        let test_cases = vec!["esc", "ESC", "Esc", "eSc"];
        let expected = 0x1B;

        for name in test_cases {
            let result = key_code(name);
            println!("  '{}' → 0x{:02X}", name, result);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_unknown_keys() {
        println!("\n[Testing Unknown Keys]");

        let unknown = vec!["UNKNOWN", "XYZ", "F25", "NUM10"];

        for name in unknown {
            let result = key_code(name);
            println!("  '{}' → 0x{:02X} (expected 0x00)", name, result);
            assert_eq!(result, 0x00);
        }
    }

    #[test]
    fn test_is_extended_key() {
        println!("\n[Testing is_extended_key]");

        // Extended keys
        let extended_keys = vec![
            (0x25, "Left arrow"),
            (0x26, "Up arrow"),
            (0x27, "Right arrow"),
            (0x28, "Down arrow"),
            (0x21, "Page Up"),
            (0x22, "Page Down"),
            (0x23, "Home"),
            (0x24, "End"),
            (0x2D, "Insert"),
            (0x2E, "Delete"),
            (0x6F, "Numpad Divide"),
            (0x90, "Num Lock"),
        ];

        for (vk, name) in extended_keys {
            assert!(is_extended_key(vk), "{} should be extended", name);
            println!("  {} (0x{:02X}) is extended ✓", name, vk);
        }

        // Non-extended keys
        let normal_keys = vec![
            (0x41, "'A'"),
            (0x0D, "Enter"),
            (0x20, "Space"),
            (0x1B, "Escape"),
        ];

        for (vk, name) in normal_keys {
            assert!(!is_extended_key(vk), "{} should NOT be extended", name);
            println!("  {} (0x{:02X}) is NOT extended ✓", name, vk);
        }
    }

    #[test]
    fn test_get_scan_code() {
        println!("\n[Testing get_scan_code]");

        // Test common keys have valid scan codes
        let test_keys = vec![
            (0x41, "'A'"),
            (0x42, "'B'"),
            (0x0D, "Enter"),
            (0x20, "Space"),
            (0x1B, "Escape"),
            (0x09, "Tab"),
        ];

        for (vk, name) in test_keys {
            let scan = get_scan_code(vk);
            assert!(scan > 0, "Scan code for {} should be > 0", name);
            println!("  VK 0x{:02X} ({}) → Scan 0x{:04X}", vk, name, scan);
        }

        // Verify scan codes are consistent
        let scan_a = get_scan_code(0x41);
        let scan_b = get_scan_code(0x42);
        assert_ne!(
            scan_a, scan_b,
            "Different keys should have different scan codes"
        );
        println!("  Different keys have different scan codes ✓");
    }

    #[test]
    fn test_performance() {
        println!("\n[Performance Test]");

        let iterations = 1_000_000;
        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let _ = key_code("a");
            let _ = key_code("~");
            let _ = key_code("f12");
            let _ = key_code("space");
        }

        let elapsed = start.elapsed();
        let per_call = elapsed / iterations / 4;

        println!("  {} iterations completed in {:?}", iterations, elapsed);
        println!("  Average per call: {:?}", per_call);
        println!(
            "  Calls per second: ~{:.0}M",
            1_000_000_000.0 / elapsed.as_nanos() as f64 * 4.0
        );
    }
}
