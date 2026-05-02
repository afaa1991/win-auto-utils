//! Anchor selection module
//!
//! Provides intelligent anchor byte/sequence selection for heuristic searching.

/// Finds the best multi-byte anchor sequence (2-4 consecutive known bytes).
///
/// This optimizes search performance by reducing false positives from memchr.
/// Uses rarity analysis to select the least common byte sequence.
///
/// # Arguments
/// * `bytes` - Pattern byte array
/// * `mask` - Pattern mask array (true = known byte, false = wildcard)
///
/// # Returns
/// * `Some(Vec<(usize, u8)>)` - Best anchor sequence as (offset, byte) pairs
/// * `None` - If no suitable sequence found (less than 2 consecutive known bytes)
pub fn find_best_anchor_sequence(bytes: &[u8], mask: &[bool]) -> Option<Vec<(usize, u8)>> {
    let mut best_sequence: Option<Vec<(usize, u8)>> = None;
    let mut best_score = u32::MAX;

    // Find all consecutive known byte sequences (length 2-4)
    let mut current_seq = Vec::new();

    for (i, &byte) in bytes.iter().enumerate() {
        if mask[i] {
            current_seq.push((i, byte));

            if current_seq.len() >= 2 && current_seq.len() <= 4 {
                // Calculate rarity score for this sequence
                let score = calculate_sequence_rarity(bytes, mask, &current_seq);

                if score < best_score {
                    best_score = score;
                    best_sequence = Some(current_seq.clone());
                }
            }

            if current_seq.len() == 4 {
                // Remove oldest to maintain max length of 4
                current_seq.remove(0);
            }
        } else {
            // Reset on wildcard
            current_seq.clear();
        }
    }

    best_sequence
}

/// Calculate rarity score for a byte sequence (lower = rarer = better).
///
/// Uses sum of individual byte frequencies as a simplified rarity metric.
fn calculate_sequence_rarity(bytes: &[u8], mask: &[bool], sequence: &[(usize, u8)]) -> u32 {
    sequence
        .iter()
        .map(|(_, byte)| get_byte_frequency(bytes, mask, *byte))
        .sum()
}

/// Get frequency of a byte in the pattern (among known bytes only).
fn get_byte_frequency(bytes: &[u8], mask: &[bool], byte: u8) -> u32 {
    bytes
        .iter()
        .zip(mask.iter())
        .filter(|(_, &m)| m)
        .filter(|(&b, _)| b == byte)
        .count() as u32
}

/// Finds the index of the most rare non-wildcard byte for heuristic searching.
///
/// This optimizes search performance by minimizing false positives from memchr.
/// Uses frequency analysis to select the byte that appears least often in the pattern.
///
/// # Arguments
/// * `bytes` - Pattern byte array
/// * `mask` - Pattern mask array
///
/// # Returns
/// * `Some(usize)` - Index of the rarest non-wildcard byte
/// * `None` - If pattern contains only wildcards
///
/// # Example
/// ```ignore
/// // This function is used internally by the scanner module
/// // It finds the rarest byte in a pattern to optimize search performance
/// let bytes = vec![0x48, 0x89, 0x48, 0x55];
/// let mask = vec![true, true, true, true];
/// // 0x48 appears twice, 0x89 and 0x55 appear once
/// // The function will return index of either 0x89 or 0x55 (the rarer ones)
/// ```
pub fn find_rarest_byte_index(bytes: &[u8], mask: &[bool]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }

    // Count frequency of each byte value in the pattern
    let mut freq = [0u32; 256];
    for (i, &byte) in bytes.iter().enumerate() {
        if mask[i] {
            freq[byte as usize] += 1;
        }
    }

    // Find the non-wildcard byte with lowest frequency
    mask.iter()
        .enumerate()
        .filter(|(_, &m)| m)
        .min_by_key(|(i, _)| freq[bytes[*i] as usize])
        .map(|(i, _)| i)
}
