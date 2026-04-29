//! AobScanBuilder module
//!
//! Provides a fluent API for configuring and executing AOB scans.

use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;
use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::scanner::aob_scan_internal;

/// Builder for configuring AOB scans.
///
/// Provides a fluent API for setting up pattern scans with various options.
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_aobscan::AobScanBuilder;
///
/// # fn example(handle: windows::Win32::Foundation::HANDLE) -> Result<(), Box<dyn std::error::Error>> {
/// let results = AobScanBuilder::new(handle)
///     .pattern_str("48 ?? 55")?
///     .start_address(0x10000000)
///     .length(0x100000)
///     .find_all(false)
///     .scan()?;
/// # Ok(())
/// # }
/// ```
pub struct AobScanBuilder {
    handle: HANDLE,
    pattern: Option<Pattern>,
    start_address: usize,
    length: usize,
    find_all: bool,
    use_cache: bool,  // Whether to use cached memory regions
}

impl AobScanBuilder {
    /// Create a new builder with the given process handle.
    ///
    /// # Arguments
    /// * `handle` - Handle to the target process (must have PROCESS_VM_READ)
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            pattern: None,
            start_address: 0,
            length: 0, // 0 means scan all available memory
            find_all: false,
            use_cache: true,  // Enable caching by default for better performance
        }
    }

    /// Configure whether to use cached memory regions.
    ///
    /// Caching significantly improves performance for repeated scans on the same process.
    /// Disable caching if the process memory layout changes frequently.
    ///
    /// # Arguments
    /// * `use_cache` - If true, uses cached regions; if false, always queries fresh regions
    ///
    /// # Default
    /// Caching is enabled by default.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_aobscan::AobScanBuilder;
    ///
    /// # fn example(handle: windows::Win32::Foundation::HANDLE) -> Result<(), Box<dyn std::error::Error>> {
    /// // First scan (builds cache)
    /// let results1 = AobScanBuilder::new(handle)
    ///     .pattern_str("48 ?? 55")?
    ///     .use_cache(true)  // Explicitly enable (already default)
    ///     .scan()?;
    ///
    /// // Second scan (uses cache, much faster)
    /// let results2 = AobScanBuilder::new(handle)
    ///     .pattern_str("90 ?? 90")?
    ///     .use_cache(true)
    ///     .scan()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn use_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    /// Set the byte pattern to search for (from string).
    ///
    /// # Arguments
    /// * `pattern` - Pattern string with hex bytes and wildcards
    ///
    /// # Returns
    /// * `Ok(Self)` - The builder for chaining
    /// * `Err(String)` - If pattern parsing fails
    pub fn pattern_str(mut self, pattern: &str) -> Result<Self, String> {
        self.pattern = Some(Pattern::from_str(pattern)?);
        Ok(self)
    }

    /// Set the byte pattern to search for (from raw bytes).
    ///
    /// # Arguments
    /// * `bytes` - Raw byte vector (all bytes must match, no wildcards)
    pub fn pattern_bytes(mut self, bytes: Vec<u8>) -> Self {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;
        
        let mask = vec![true; bytes.len()];
        let mask_bytes = vec![0xFF; bytes.len()];
        
        // Find best multi-byte anchor sequence for this pattern
        let anchor_sequence = find_best_anchor_sequence(&bytes, &mask);
        
        self.pattern = Some(Pattern { 
            bytes, 
            mask, 
            mask_bytes,
            anchor_sequence,
        });
        self
    }

    /// Set the starting address for the scan.
    ///
    /// # Arguments
    /// * `addr` - The memory address to start scanning from
    pub fn start_address(mut self, addr: usize) -> Self {
        self.start_address = addr;
        self
    }

    /// Set the length of the memory region to scan.
    ///
    /// # Arguments
    /// * `len` - Number of bytes to scan (0 = entire process memory)
    pub fn length(mut self, len: usize) -> Self {
        self.length = len;
        self
    }

    /// Configure whether to find all occurrences or just the first one.
    ///
    /// # Arguments
    /// * `all` - If true, returns all matches; if false, stops at first match
    pub fn find_all(mut self, all: bool) -> Self {
        self.find_all = all;
        self
    }

    /// Build and execute the scan.
    ///
    /// # Returns
    /// * `Ok(Vec<usize>)` - Vector of matching addresses
    /// * `Err(MemoryError)` - If scanning fails
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_aobscan::AobScanBuilder;
    ///
    /// # fn example(handle: windows::Win32::Foundation::HANDLE) -> Result<(), Box<dyn std::error::Error>> {
    /// let results = AobScanBuilder::new(handle)
    ///     .pattern_str("48 ?? 55")?
    ///     .scan()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan(self) -> Result<Vec<usize>, MemoryError> {
        let pattern = self.pattern.ok_or_else(|| {
            MemoryError::InvalidAddress("Pattern not set".to_string())
        })?;
        
        aob_scan_internal(
            self.handle, 
            &pattern, 
            self.start_address, 
            self.length, 
            self.find_all,
            self.use_cache
        )
    }
}
