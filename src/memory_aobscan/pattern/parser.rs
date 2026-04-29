//! Pattern parsing module
//!
//! Handles parsing of pattern strings into structured byte patterns with masks.

use super::anchor::find_best_anchor_sequence;

/// Represents a parsed byte pattern with a mask for wildcard support.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub bytes: Vec<u8>,
    pub mask: Vec<bool>,
    pub mask_bytes: Vec<u8>, // Pre-computed mask: 0xFF for true, 0x00 for false (for SIMD)
    
    // Multi-byte anchor sequence for heuristic optimization
    // Stores (offset, byte) pairs for 2-4 consecutive known bytes
    pub anchor_sequence: Option<Vec<(usize, u8)>>,
}

impl Pattern {
    /// Parses a string like "48 ?? 55 ??" into a Pattern.
    ///
    /// # Arguments
    /// * `pattern_str` - Space-separated hex bytes with optional wildcards
    ///
    /// # Returns
    /// * `Ok(Pattern)` - The parsed pattern with bytes and mask
    /// * `Err(String)` - If parsing fails
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
