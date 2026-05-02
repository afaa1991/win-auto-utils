//! Unified Address Resolution Strategy for builtin handlers
//!
//! This module provides a common `AddressStrategy` enum that all builtin handlers
//! (LockHandler, BytesSwitchHandler, TrampolineHookHandler) use to delay address
//! resolution until activation time.
//!
//! # Design Philosophy
//!
//! The `AddressStrategy` acts as a **bridge** between Handlers and two independent modules:
//! - `memory_resolver`: Static address resolution (module base + offsets + pointer chains)
//! - `memory_aobscan`: Dynamic byte pattern scanning (architecture-independent)
//!
//! Each strategy variant stores configuration parameters and uses the appropriate
//! module's builder pattern at activation time.
//!
//! # Why This Design?
//!
//! 1. **Module Independence**: resolver and aobscan are completely decoupled
//! 2. **Delayed Resolution**: Store "how to resolve" without immediately resolving
//! 3. **Architecture Auto-Detection**: Static patterns resolved using context architecture
//! 4. **Builder Pattern**: Each module uses its own builder for flexible configuration
//! 5. **Caching Support**: AOB results can be cached across activations

use crate::memory::MemoryError;
use crate::memory_aobscan;
use crate::memory_hook::Architecture;
use crate::memory_resolver::MemoryAddress;

/// Unified address resolution strategy for delayed address resolution at activation time.
///
/// This enum allows handlers to store address resolution information without immediately
/// resolving the address. The actual resolution happens during `activate()` when the
/// ProcessContext (with architecture info) is available.
///
/// # Design Notes
/// - **StaticPattern**: Offset is embedded in the pattern string itself (e.g., "game.exe+0x1000")
/// - **AobPattern/AobPatternWithRange**: Separate offset field for post-scan address correction
///   (useful when scanning for a signature near the target, but not exactly at it)
#[derive(Clone, Debug)]
pub enum AddressStrategy {
    /// Static address with pattern string - resolved using context architecture at activation time
    /// Example: "game.exe+1000->2FC" (offset is part of the pattern)
    StaticPattern(String),

    /// AOB pattern scan (architecture-independent) - scans entire process memory
    /// Example: "48 89 5C 24 ?? 08"
    ///
    /// # Offset Usage
    /// The `offset` field allows correcting the scanned address to the actual target.
    /// For example, if you scan for a function prologue but want to hook an instruction
    /// 0x42 bytes later, use offset = 0x42.
    AobPattern {
        /// AOB pattern string
        pattern: String,
        /// Offset to add to the scanned address (default: 0)
        /// Can be negative to scan backwards from the match
        offset: i64,
    },

    /// AOB pattern scan with custom range (faster for known regions)
    /// If start_address and length are both 0, scans entire memory
    ///
    /// # Offset Usage
    /// Same as `AobPattern`, allows post-scan address correction.
    AobPatternWithRange {
        /// AOB pattern string
        pattern: String,
        /// Starting address for the scan (0 = from beginning)
        start_address: usize,
        /// Length of the scan range (0 = entire process memory)
        length: usize,
        /// Offset to add to the scanned address (default: 0)
        offset: i64,
    },
}

impl AddressStrategy {
    /// Create a static pattern strategy
    pub fn static_pattern(pattern: impl Into<String>) -> Self {
        AddressStrategy::StaticPattern(pattern.into())
    }

    /// Create an AOB pattern strategy (full memory scan) with default offset = 0
    pub fn aob_pattern(pattern: impl Into<String>) -> Self {
        AddressStrategy::AobPattern {
            pattern: pattern.into(),
            offset: 0,
        }
    }

    /// Create an AOB pattern strategy with custom offset
    ///
    /// # Arguments
    /// * `pattern` - AOB pattern string
    /// * `offset` - Offset to add to the scanned address (can be negative)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::address_strategy::AddressStrategy;
    ///
    /// // Scan for function prologue, but hook 0x42 bytes later
    /// let strategy = AddressStrategy::aob_pattern_with_offset("48 89 5C 24", 0x42);
    /// ```
    pub fn aob_pattern_with_offset(pattern: impl Into<String>, offset: i64) -> Self {
        AddressStrategy::AobPattern {
            pattern: pattern.into(),
            offset,
        }
    }

    /// Create an AOB pattern strategy with custom range and default offset = 0
    pub fn aob_pattern_with_range(
        pattern: impl Into<String>,
        start_address: usize,
        length: usize,
    ) -> Self {
        AddressStrategy::AobPatternWithRange {
            pattern: pattern.into(),
            start_address,
            length,
            offset: 0,
        }
    }

    /// Create an AOB pattern strategy with custom range and offset
    ///
    /// # Arguments
    /// * `pattern` - AOB pattern string
    /// * `start_address` - Starting address for the scan (0 = from beginning)
    /// * `length` - Length of the scan range (0 = entire process memory)
    /// * `offset` - Offset to add to the scanned address (can be negative)
    pub fn aob_pattern_with_range_and_offset(
        pattern: impl Into<String>,
        start_address: usize,
        length: usize,
        offset: i64,
    ) -> Self {
        AddressStrategy::AobPatternWithRange {
            pattern: pattern.into(),
            start_address,
            length,
            offset,
        }
    }

    /// Check if this strategy uses AOB scanning
    pub fn is_aob_scan(&self) -> bool {
        matches!(
            self,
            AddressStrategy::AobPattern { .. } | AddressStrategy::AobPatternWithRange { .. }
        )
    }

    /// Check if this strategy uses static pattern
    pub fn is_static_pattern(&self) -> bool {
        matches!(self, AddressStrategy::StaticPattern(_))
    }

    /// Get the pattern string (for all variants)
    pub fn pattern(&self) -> &str {
        match self {
            AddressStrategy::StaticPattern(p) => p,
            AddressStrategy::AobPattern { pattern, .. } => pattern,
            AddressStrategy::AobPatternWithRange { pattern, .. } => pattern,
        }
    }

    /// Build a MemoryAddress from static pattern (for LockHandler usage)
    ///
    /// This method converts the static pattern string into a parsed MemoryAddress
    /// using the provided architecture. Unlike `resolve()`, this returns the
    /// MemoryAddress object itself, allowing LockHandler to manage its own
    /// resolution lifecycle.
    ///
    /// # Arguments
    /// * `architecture` - Target process architecture (from ProcessContext)
    ///
    /// # Returns
    /// Parsed MemoryAddress ready for use with MemoryLock
    ///
    /// # Errors
    /// Returns error if pattern parsing fails or if called on non-static strategy
    pub fn build_memory_address(
        &self,
        architecture: Architecture,
    ) -> Result<MemoryAddress, MemoryError> {
        match self {
            AddressStrategy::StaticPattern(pattern) => {
                // Parse pattern with architecture-aware parser
                match architecture {
                    Architecture::X64 => MemoryAddress::new_x64(pattern).map_err(|e| {
                        MemoryError::InvalidAddress(format!(
                            "Failed to parse x64 pattern '{}': {}",
                            pattern, e
                        ))
                    }),
                    Architecture::X86 => MemoryAddress::new_x86(pattern).map_err(|e| {
                        MemoryError::InvalidAddress(format!(
                            "Failed to parse x86 pattern '{}': {}",
                            pattern, e
                        ))
                    }),
                }
            }
            _ => Err(MemoryError::InvalidAddress(
                "build_memory_address only supports StaticPattern strategy".to_string(),
            )),
        }
    }

    /// Resolve the address using the strategy and provided context
    ///
    /// This method delegates to the appropriate underlying module using their builders:
    /// - StaticPattern → memory_resolver::MemoryAddress (parsed then resolved)
    /// - AobPattern variants → memory_aobscan::AobScanBuilder
    pub fn resolve(
        &self,
        handle: windows::Win32::Foundation::HANDLE,
        pid: u32,
        architecture: Architecture,
    ) -> Result<usize, MemoryError> {
        match self {
            // ==================== Static Pattern ====================
            // Uses memory_resolver::MemoryAddress directly
            AddressStrategy::StaticPattern(pattern) => {
                // Parse pattern with architecture-aware parser
                let mem_addr = match architecture {
                    Architecture::X64 => MemoryAddress::new_x64(pattern).map_err(|e| {
                        MemoryError::InvalidAddress(format!(
                            "Failed to parse x64 pattern '{}': {}",
                            pattern, e
                        ))
                    })?,
                    Architecture::X86 => MemoryAddress::new_x86(pattern).map_err(|e| {
                        MemoryError::InvalidAddress(format!(
                            "Failed to parse x86 pattern '{}': {}",
                            pattern, e
                        ))
                    })?,
                };

                // Resolve the address
                mem_addr.resolve_address(handle, pid).map_err(|e| {
                    MemoryError::InvalidAddress(format!(
                        "Failed to resolve address '{}': {}",
                        pattern, e
                    ))
                })
            }

            // ==================== AOB Pattern (Full Memory) ====================
            // Uses memory_aobscan::AobScanBuilder
            AddressStrategy::AobPattern { pattern, offset } => {
                // Build scanner and execute scan
                let results = memory_aobscan::AobScanBuilder::new(handle)
                    .pattern_str(pattern)
                    .map_err(|e| MemoryError::InvalidAddress(format!("AOB scan failed: {}", e)))?
                    .scan()
                    .map_err(|e| MemoryError::InvalidAddress(format!("AOB scan failed: {}", e)))?;

                if results.is_empty() {
                    return Err(MemoryError::InvalidAddress(format!(
                        "AOB pattern '{}' not found",
                        pattern
                    )));
                }

                // Apply offset correction
                Ok((results[0] as i64 + offset) as usize)
            }

            // ==================== AOB Pattern With Range ====================
            // Uses memory_aobscan::AobScanBuilder with range configuration
            AddressStrategy::AobPatternWithRange {
                pattern,
                start_address,
                length,
                offset,
            } => {
                // Build scanner with range configuration
                let mut builder = memory_aobscan::AobScanBuilder::new(handle)
                    .pattern_str(pattern)
                    .map_err(|e| MemoryError::InvalidAddress(format!("AOB scan failed: {}", e)))?;

                // Apply range if specified
                if *start_address != 0 {
                    builder = builder.start_address(*start_address);
                }
                if *length != 0 {
                    builder = builder.length(*length);
                }

                // Execute scan
                let results = builder
                    .scan()
                    .map_err(|e| MemoryError::InvalidAddress(format!("AOB scan failed: {}", e)))?;

                if results.is_empty() {
                    return Err(MemoryError::InvalidAddress(format!(
                        "AOB pattern '{}' not found",
                        pattern
                    )));
                }

                // Apply offset correction
                Ok((results[0] as i64 + offset) as usize)
            }
        }
    }
}
