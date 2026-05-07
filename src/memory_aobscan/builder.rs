//! AobScanBuilder Module
//!
//! Provides a fluent, chainable API for configuring and executing Array of Bytes (AOB)
//! pattern scans on remote process memory.
//!
//! # Features
//! - Pattern configuration from hex strings or raw byte vectors
//! - Memory range limiting for targeted scans
//! - Optional memory region caching for repeated scans
//! - Configurable to find first match only or all matches
//!
//! # Quick Example
//! ```no_run
//! use win_auto_utils::memory_aobscan::AobScanBuilder;
//! use windows::Win32::Foundation::HANDLE;
//!
//! # fn example(handle: HANDLE) -> Result<(), Box<dyn std::error::Error>> {
//! // Find first match
//! let first_match = AobScanBuilder::new(handle)
//!     .pattern_str("48 ?? 5C 24 ?? 48")?
//!     .find_all(false)
//!     .scan()?;
//!
//! // Find all occurrences in specific range
//! let all_matches = AobScanBuilder::new(handle)
//!     .pattern_bytes(vec![0x48, 0x89, 0x5C])
//!     .start_address(0x10000000)
//!     .length(0x1000000)
//!     .find_all(true)
//!     .scan()?;
//! # Ok(())
//! # }
//! ```

use crate::memory::MemoryError;
use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::scanner::aob_scan_internal;
use windows::Win32::Foundation::HANDLE;

/// Fluent builder for configuring and executing AOB pattern scans.
///
/// Provides a clean, chainable interface for specifying all scan parameters.
///
/// # Example Flow
/// ```
/// // 1. Create with handle
/// // 2. Set pattern
/// // 3. Configure options
/// // 4. Execute scan
/// ```
///
/// # Default Values
/// - `start_address`: 0 (entire address space)
/// - `length`: 0 (scan until end)
/// - `find_all`: false (find first match only)
/// - `use_cache`: true (enable region caching)
pub struct AobScanBuilder {
    /// Handle to the target process (requires PROCESS_VM_READ access)
    handle: HANDLE,
    /// The pattern to search for
    pattern: Option<Pattern>,
    /// Starting memory address for the scan range
    start_address: usize,
    /// Length of memory to scan (0 = scan all)
    length: usize,
    /// Whether to find all matches or stop at first
    find_all: bool,
    /// Whether to use cached memory region information
    use_cache: bool,
}

impl AobScanBuilder {
    /// Creates a new AOB scan builder for the specified process.
    ///
    /// # Arguments
    /// * `handle` - Valid handle to the target process with PROCESS_VM_READ access
    ///
    /// # Returns
    /// Fresh builder instance with default configuration
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            pattern: None,
            start_address: 0,
            length: 0,
            find_all: false,
            use_cache: true,
        }
    }

    /// Configures whether to cache memory region information.
    ///
    /// Caching improves performance for repeated scans by avoiding redundant
    /// calls to `VirtualQueryEx`. Disable if the process memory layout changes
    /// frequently.
    ///
    /// # Arguments
    /// * `use_cache` - `true` to cache regions, `false` for fresh query each time
    ///
    /// # Default
    /// Caching is enabled by default.
    pub fn use_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    /// Sets the search pattern from a hex string with optional wildcards.
    ///
    /// # Pattern Syntax
    /// - Hex bytes: `48 89 5C`
    /// - Wildcards: `48 ?? 5C` or `48 ? 5C`
    ///
    /// # Arguments
    /// * `pattern` - Hex pattern string
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder for chaining
    /// * `Err(String)` - If pattern parsing fails
    pub fn pattern_str(mut self, pattern: &str) -> Result<Self, String> {
        self.pattern = Some(Pattern::from_str(pattern)?);
        Ok(self)
    }

    /// Sets the search pattern from a raw byte vector (no wildcards).
    ///
    /// Creates an exact-match pattern with no wildcards.
    ///
    /// # Arguments
    /// * `bytes` - Vector of bytes to match
    pub fn pattern_bytes(mut self, bytes: Vec<u8>) -> Self {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;

        let mask = vec![true; bytes.len()];
        let mask_bytes = vec![0xFF; bytes.len()];
        let anchor_sequence = find_best_anchor_sequence(&bytes, &mask);

        self.pattern = Some(Pattern {
            bytes,
            mask,
            mask_bytes,
            anchor_sequence,
        });
        self
    }

    /// Sets the starting memory address for the scan.
    ///
    /// # Arguments
    /// * `addr` - Memory address to begin scanning from
    pub fn start_address(mut self, addr: usize) -> Self {
        self.start_address = addr;
        self
    }

    /// Sets the length of memory to scan.
    ///
    /// # Arguments
    /// * `len` - Number of bytes to scan (0 = scan all available memory)
    pub fn length(mut self, len: usize) -> Self {
        self.length = len;
        self
    }

    /// Configures whether to find all matches or stop at first match.
    ///
    /// # Arguments
    /// * `all` - `true` to find all matches, `false` to stop at first
    pub fn find_all(mut self, all: bool) -> Self {
        self.find_all = all;
        self
    }

    /// Builds and executes the pattern scan.
    ///
    /// # Returns
    /// * `Ok(Vec<usize>)` - Vector of matching memory addresses, sorted ascending
    /// * `Err(MemoryError)` - If scan fails or pattern not set
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_aobscan::AobScanBuilder;
    /// use windows::Win32::Foundation::HANDLE;
    ///
    /// # fn example(handle: HANDLE) -> Result<(), Box<dyn std::error::Error>> {
    /// let results = AobScanBuilder::new(handle)
    ///     .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
    ///     .find_all(false)
    ///     .scan()?;
    ///
    /// if let Some(&addr) = results.first() {
    ///     println!("Found at: 0x{:X}", addr);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan(self) -> Result<Vec<usize>, MemoryError> {
        let pattern = self
            .pattern
            .ok_or_else(|| MemoryError::InvalidAddress("Pattern not set".to_string()))?;

        aob_scan_internal(
            self.handle,
            &pattern,
            self.start_address,
            self.length,
            self.find_all,
            self.use_cache,
        )
    }
}
