//! Pattern Parser
//!
//! Handles parsing of hex pattern strings with wildcard support into structured
//! Pattern instances, with pre-computed SIMD-compatible mask bytes and intelligent
//! anchor selection.

use super::anchor::find_best_anchor_sequence;

/// Parsed byte pattern with wildcard support.
///
/// Includes pre-computed mask bytes for SIMD verification and optional
/// multi-byte anchor sequence for optimized heuristic scanning.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern byte values (wildcards stored as 0x00)
    pub bytes: Vec<u8>,
    /// Pattern mask (true = known byte, false = wildcard)
    pub mask: Vec<bool>,
    /// Pre-computed mask bytes for SIMD (0xFF for true, 0x00 for false)
    pub mask_bytes: Vec<u8>,
    /// Multi-byte anchor sequence for heuristic optimization
    /// Stores (offset, byte) pairs for 2-4 consecutive known bytes
    pub anchor_sequence: Option<Vec<(usize, u8)>>,
}

impl Pattern {
    /// Parses a pattern string like "48 ?? 55" into a structured Pattern.
    ///
    /// Supports both "??" and "?" as wildcard tokens.
    ///
    /// # Arguments
    /// * `pattern_str` - Space-separated hex bytes with optional wildcards
    ///
    /// # Returns
    /// * `Ok(Pattern)` - Structured pattern with bytes, mask, and anchor
    /// * `Err(String)` - If pattern is empty or contains invalid hex bytes
    ///
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_aobscan::Pattern;
    ///
    /// let pattern = Pattern::from_str("48 ?? 55").unwrap();
    /// assert_eq!(pattern.bytes.len(), 3);
    /// assert_eq!(pattern.mask, vec![true, false, true]);
    /// ```
    pub fn from_str(pattern_str: &str) -> Result<Self, String> {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();
        let mut mask_bytes = Vec::new();

        for token in pattern_str.split_whitespace() {
            if token == "??" || token == "?" {
                bytes.push(0);
                mask.push(false);
                mask_bytes.push(0x00);
            } else {
                match u8::from_str_radix(token, 16) {
                    Ok(b) => {
                        bytes.push(b);
                        mask.push(true);
                        mask_bytes.push(0xFF);
                    }
                    Err(_) => return Err(format!("Invalid hex byte: {}", token)),
                }
            }
        }

        if bytes.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        // Find best multi-byte anchor sequence
        let anchor_sequence = find_best_anchor_sequence(&bytes, &mask);

        Ok(Self {
            bytes,
            mask,
            mask_bytes,
            anchor_sequence,
        })
    }
}
