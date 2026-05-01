//! Memory Address Resolution Module
//!
//! Provides functionality to resolve symbolic memory addresses (e.g., "game.exe+0x123->456")
//! into actual memory addresses in a target process.

pub mod resolver;
mod builder; 
pub use resolver::{MemoryAddress, AddressBase, AddressOp, ParseError, ResolveError, PointerSize};

// Re-export for convenience
pub use crate::memory_aobscan::AobScanBuilder;

/// Unified address source that supports both static addresses and AOB scanning
#[derive(Clone, Debug)]
pub enum AddressSource {
    /// Static address with optional pointer chain
    Static(MemoryAddress),
    
    /// Dynamic address found via AOB pattern scanning
    AobScan {
        /// Pattern string (e.g., "48 89 5C ?? 90")
        pattern: String,
        /// Optional start address for scan (0 = entire process)
        start_address: usize,
        /// Optional scan length (0 = all available memory)
        length: usize,
        /// Find first match only (true) or all matches (false)
        find_first: bool,
    },
}

impl AddressSource {
    /// Create a static address source from a parsed MemoryAddress
    pub fn from_static(address: MemoryAddress) -> Self {
        AddressSource::Static(address)
    }
    
    /// Quick creation from static address pattern string (uses default architecture)
    /// 
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::AddressSource;
    /// 
    /// // Uses default architecture (matches your binary)
    /// let addr = AddressSource::from_pattern("game.exe+1000->2FC")?;
    /// ```
    pub fn from_pattern(pattern: &str) -> Result<Self, ParseError> {
        MemoryAddress::parse(pattern).map(AddressSource::Static)
    }
    
    /// Quick creation from static address pattern string with x86 architecture
    /// 
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::AddressSource;
    /// 
    /// // For 32-bit target processes (old games)
    /// let addr = AddressSource::from_pattern_x86("lf2.exe+58C94->2FC")?;
    /// ```
    pub fn from_pattern_x86(pattern: &str) -> Result<Self, ParseError> {
        MemoryAddress::new_x86(pattern).map(AddressSource::Static)
    }
    
    /// Quick creation from static address pattern string with x64 architecture
    /// 
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::AddressSource;
    /// 
    /// // For 64-bit target processes (modern applications)
    /// let addr = AddressSource::from_pattern_x64("game.exe+1000->2FC")?;
    /// ```
    pub fn from_pattern_x64(pattern: &str) -> Result<Self, ParseError> {
        MemoryAddress::new_x64(pattern).map(AddressSource::Static)
    }
    
    /// Quick creation from AOB pattern string (finds first match)
    /// 
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::AddressSource;
    /// 
    /// let addr = AddressSource::from_aob("48 89 5C ?? 48")?;
    /// ```
    pub fn from_aob(pattern: &str) -> Result<Self, String> {
        Ok(AddressSource::AobScan {
            pattern: pattern.to_string(),
            start_address: 0,
            length: 0,
            find_first: true,
        })
    }
    
    /// Create an AOB scan with custom range and options
    /// 
    /// # Arguments
    /// * `pattern` - AOB pattern string
    /// * `start` - Start address for scan (0 = entire process)
    /// * `length` - Scan length (0 = all available memory)
    /// * `find_first` - Find first match only (true) or all matches (false)
    pub fn from_aob_with_options(pattern: &str, start: usize, length: usize, find_first: bool) -> Result<Self, String> {
        Ok(AddressSource::AobScan {
            pattern: pattern.to_string(),
            start_address: start,
            length,
            find_first,
        })
    }
}

/// Convenience implementation to convert string to AddressSource (defaults to static parsing)
impl std::convert::TryFrom<&str> for AddressSource {
    type Error = ParseError;
    
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // Try to parse as static address first
        MemoryAddress::parse(s).map(AddressSource::Static)
    }
}

impl AddressSource {
    /// Create an AOB scan address source
    pub fn from_aob_scan(pattern: &str) -> Result<Self, String> {
        Ok(AddressSource::AobScan {
            pattern: pattern.to_string(),
            start_address: 0,
            length: 0,
            find_first: true,
        })
    }
    
    /// Create an AOB scan with custom range
    pub fn from_aob_scan_range(pattern: &str, start: usize, length: usize) -> Result<Self, String> {
        Ok(AddressSource::AobScan {
            pattern: pattern.to_string(),
            start_address: start,
            length,
            find_first: true,
        })
    }
    
    /// Resolve the address using the appropriate method
    /// 
    /// # Arguments
    /// * `handle` - Process handle with read permissions
    /// * `pid` - Process ID (for module-based addresses)
    /// 
    /// # Returns
    /// The resolved memory address
    pub fn resolve(&self, handle: windows::Win32::Foundation::HANDLE, pid: u32) -> Result<usize, Box<dyn std::error::Error>> {
        match self {
            AddressSource::Static(addr) => {
                addr.resolve_address(handle, pid)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            AddressSource::AobScan { pattern, start_address, length, find_first } => {
                let results = AobScanBuilder::new(handle)
                    .pattern_str(pattern)?
                    .start_address(*start_address)
                    .length(*length)
                    .find_all(!find_first)
                    .scan()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                
                if results.is_empty() {
                    Err(format!("AOB pattern '{}' not found", pattern).into())
                } else {
                    Ok(results[0])
                }
            }
        }
    }
}
