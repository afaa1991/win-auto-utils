/// Convert Windows CHAR array to Rust String
///
/// This function converts a null-terminated Windows CHAR array (i8 slice) to a Rust String.
/// It reads bytes until it encounters a null terminator (0), then converts them to UTF-8.
///
/// # Parameters
/// - `char_array`: A slice of i8 values representing a Windows CHAR array (typically ANSI encoded)
///
/// # Returns
/// A Rust String containing the converted text. Invalid UTF-8 sequences are replaced with the Unicode replacement character (U+FFFD).
///
/// # Safety Considerations
/// - The input should be a valid null-terminated CHAR array
/// - The function assumes ANSI or ASCII encoding
/// - For Unicode text, consider using wide character (UTF-16) APIs instead
///
/// # Example
/// ```
/// use win_auto_utils::utils::char_array_to_string;
///
/// // Simulate a Windows CHAR array (null-terminated)
/// let char_array: &[i8] = &[72, 101, 108, 108, 111, 0]; // "Hello\0"
/// let result = char_array_to_string(char_array);
/// assert_eq!(result, "Hello");
///
/// // Array without explicit null terminator (will read until end)
/// let char_array2: &[i8] = &[87, 111, 114, 108, 100]; // "World"
/// let result2 = char_array_to_string(char_array2);
/// assert_eq!(result2, "World");
/// ```
///
/// # Performance Notes
/// - Creates an intermediate `Vec<u8>` for the conversion
/// - Uses `from_utf8_lossy` which handles invalid UTF-8 gracefully
/// - Suitable for occasional conversions; for hot loops, consider caching results
pub fn char_array_to_string(char_array: &[i8]) -> String {
    let bytes: Vec<u8> = char_array
        .iter()
        .take_while(|&&c| c != 0) 
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).to_string()
}

/// Convert Rust string to PCWSTR (UTF-16 null-terminated) - Optimized version
///
/// This function converts a Rust string to a UTF-16 encoded vector with a null terminator,
/// suitable for passing to Windows API functions that expect PCWSTR/LPCWSTR.
///
/// # Performance Optimizations
/// - Pre-allocates capacity to avoid reallocations
/// - Uses `extend()` which is optimized for iterators
///
/// # Example
/// ```
/// use win_auto_utils::utils::string_to_pcwstr;
///
/// let utf16 = string_to_pcwstr("Hello");
/// assert_eq!(utf16.len(), 6); // 5 chars + null terminator
/// assert_eq!(utf16[5], 0);    // null terminated
/// ```
pub fn string_to_pcwstr(s: &str) -> Vec<u16> {
    // Collect UTF-16 code points and add null terminator in one operation
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
