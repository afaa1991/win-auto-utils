//! Memory Address Resolution Module
//!
//! Provides functionality to resolve symbolic memory addresses (e.g., "target.exe+0x123->456")
//! into actual memory addresses in a target process.
//!
//! This module combines address parsing and pointer chain resolution.
//!
//! # Feature Flag
//! Enable with: `--features "memory_resolver"`
//!
//! # Quick Start
//!
//! ## Method 1: Quick Parse Methods (Recommended for Simple Cases) ⭐⭐⭐
//! ```no_run
//! use win_auto_utils::memory_resolver::MemoryAddress;
//!
//! // For x86 (32-bit) target processes (e.g., old games)
//! let addr = MemoryAddress::new_x86("lf2.exe+58C94->308")?;
//!
//! // For x64 (64-bit) target processes (modern applications)
//! let addr = MemoryAddress::new_x64("game.exe+1000->20")?;
//!
//! // Default architecture (matches your compiled binary)
//! let addr = MemoryAddress::parse("app.dll+100->20")?;
//! # Ok::<(), win_auto_utils::memory_resolver::ParseError>(())
//! ```
//!
//! ## Method 2: Builder Pattern (For Complex Configuration) ⭐⭐
//! ```no_run
//! use win_auto_utils::memory_resolver::MemoryAddress;
//!
//! let addr = MemoryAddress::builder()
//!     .address("lf2.exe+58C94->308")
//!     .x86()
//!     .build()?;
//! # Ok::<(), win_auto_utils::memory_resolver::ParseError>(())
//! ```
//!
//! ## Resolve and Read Memory
//! ```no_run
//! use win_auto_utils::memory_resolver::MemoryAddress;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     
//!     // Get process handle
//!     let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
//!     let handle = open_process_handle(pid, desired_access)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Parse address string
//!     let addr = MemoryAddress::new_x86("lf2.exe+58C94->308")?;
//!     
//!     // Resolve to actual memory address
//!     let resolved = addr.resolve_address(handle, pid)?;
//!     println!("Resolved address: 0x{:X}", resolved);
//!     
//!     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
//!     Ok(())
//! }
//! ```
//!
//! ## Address Syntax
//! - **Module base**: `module.exe+offset` (e.g., `target.exe+1000`)
//! - **Absolute address**: `0x12345678` or just `12345678` (hex by default)
//! - **Decimal numbers**: Use `#` prefix (e.g., `#100` = decimal 100)
//! - **Direct offset**: `+offset` (adds without dereference)
//! - **Pointer jump**: `->offset` (dereferences then adds offset)
//! - **Underscore separator**: For readability (e.g., `1_0000` = 0x10000)
//!
//! ## More Examples
//! ```no_run
//! use win_auto_utils::memory_resolver::MemoryAddress;
//!
//! // Module-relative with pointer chain
//! let addr1 = MemoryAddress::new_x86("app.dll+1000->20->30")?;
//!
//! // Absolute address with direct offset
//! let addr2 = MemoryAddress::new_x64("0x7FF6A1B2C3D4+100")?;
//!
//! // Decimal numbers
//! let addr3 = MemoryAddress::parse("#1000+#200")?;
//!
//! // Mixed syntax
//! let addr4 = MemoryAddress::parse("target.exe+1_0000->2FC")?;
//! # Ok::<(), win_auto_utils::memory_resolver::ParseError>(())
//! ```

use std::fmt;
use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;

// ============================================================================
// Error Types
// ============================================================================

/// Address parsing errors
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Empty input string
    EmptyInput,
    
    /// Invalid hexadecimal format
    InvalidHex(String),
    
    /// Invalid decimal format
    InvalidDecimal(String),
    
    /// Invalid offset format
    InvalidOffset(String),
    
    /// Invalid module address format
    InvalidModuleFormat(String),
    
    /// Invalid pointer jump syntax (expected '->')
    InvalidPointerSyntax(String),
    
    /// Unexpected character in address string
    UnexpectedCharacter(char),
    
    /// Multiple module names in one address
    MultipleModules,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty address string"),
            ParseError::InvalidHex(s) => write!(f, "Invalid hexadecimal: {}", s),
            ParseError::InvalidDecimal(s) => write!(f, "Invalid decimal: {}", s),
            ParseError::InvalidOffset(s) => write!(f, "Invalid offset: {}", s),
            ParseError::InvalidModuleFormat(s) => write!(f, "Invalid module format: {}", s),
            ParseError::InvalidPointerSyntax(s) => write!(f, "Invalid pointer syntax: {}", s),
            ParseError::UnexpectedCharacter(ch) => write!(f, "Unexpected character: '{}'", ch),
            ParseError::MultipleModules => write!(f, "Multiple module names not allowed"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Address resolution errors
#[derive(Debug)]
pub enum ResolveError {
    /// Module not found by name
    ModuleNotFound(String),
    
    /// Failed to read pointer during chain resolution
    PointerReadFailed(usize, MemoryError),
    
    /// Null pointer encountered
    NullPointer(usize),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::ModuleNotFound(name) => {
                write!(f, "Module '{}' not found in target process", name)
            },
            ResolveError::PointerReadFailed(addr, e) => {
                write!(f, "Failed to read pointer at 0x{:X}: {}", addr, e)
            },
            ResolveError::NullPointer(addr) => {
                write!(f, "Null pointer encountered at 0x{:X}", addr)
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Pointer size configuration for target process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    /// 32-bit pointers (4 bytes) - for x86 processes
    Bits32,
    
    /// 64-bit pointers (8 bytes) - for x64 processes
    Bits64,
}

impl PointerSize {
    /// Get pointer size in bytes
    pub fn bytes(&self) -> usize {
        match self {
            PointerSize::Bits32 => 4,
            PointerSize::Bits64 => 8,
        }
    }
    
    /// Get default pointer size based on compilation target architecture
    /// 
    /// This matches the native `usize` size of the compiled binary:
    /// - x86 (32-bit): Returns `PointerSize::Bits32`
    /// - x64 (64-bit): Returns `PointerSize::Bits64`
    /// 
    /// This is the recommended default for most use cases.
    pub fn default_architecture() -> Self {
        #[cfg(target_pointer_width = "64")]
        return PointerSize::Bits64;
        
        #[cfg(target_pointer_width = "32")]
        return PointerSize::Bits32;
    }
}

impl Default for PointerSize {
    fn default() -> Self {
        Self::default_architecture()
    }
}

/// Represents a parsed memory address with a base and a chain of operations.
#[derive(Debug, Clone)]
pub struct MemoryAddress {
    pub base: AddressBase,
    pub operations: Vec<AddressOp>,
    /// Pointer size configuration. Defaults to compilation target architecture.
    pub pointer_size: PointerSize,
}

/// The base type for a memory address.
#[derive(Debug, Clone)]
pub enum AddressBase {
    /// An absolute virtual memory address.
    Absolute(usize),
    /// A module-relative address (module name + offset).
    Module { name: String, offset: usize },
}

/// Operations that can be performed on an address during resolution.
#[derive(Debug, Clone)]
pub enum AddressOp {
    /// Add an offset directly to the current address.
    DirectOffset(usize),
    /// Dereference the current address as a pointer and add an offset.
    PointerJump(usize),
}

impl MemoryAddress {
    /// Parses a string like "target.exe+100->20" into a [`MemoryAddress`].
    ///
    /// # Syntax Specification
    /// - **Default radix**: Hexadecimal (e.g., `100` = 0x100)
    /// - **Decimal marker**: `#` prefix (e.g., `#100` = decimal 100)
    /// - **Direct offset**: `+offset` (adds to current address without dereference)
    /// - **Pointer jump**: `->offset` (dereferences pointer then adds offset)
    /// - **Underscore separator**: Supported for readability (e.g., `1_0000` = 0x10000)
    ///
    /// # Arguments
    /// * `input` - The address string to parse
    ///
    /// # Returns
    /// * `Ok(MemoryAddress)` - The parsed address structure
    /// * `Err(ParseError)` - If parsing fails
    ///
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// let addr = MemoryAddress::parse("target.exe+1000->20").unwrap();
    /// assert!(matches!(addr.base, win_auto_utils::memory_resolver::AddressBase::Module { .. }));
    /// ```
    /// Create a new builder for configuring MemoryAddress
    ///
    /// This provides a fluent API for building MemoryAddress with optional configuration.
    /// Use this when you need fine-grained control over settings.
    /// For simple cases, prefer `parse()`, `new_x86()`, or `new_x64()`.
    ///
    /// # Returns
    /// A new MemoryAddressBuilder instance
    ///
    /// # Examples
    /// ```ignore
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// let addr = MemoryAddress::builder()
    ///     .address("lf2.exe+58C94->308")
    ///     .x86()
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder() -> crate::memory_resolver::builder::MemoryAddressBuilder {
        crate::memory_resolver::builder::MemoryAddressBuilder::new()
    }

    /// Parses an address string with default pointer size (based on compilation target).
    ///
    /// This is the most common way to parse addresses. The pointer size will match
    /// your compiled binary's architecture (x86 → 32-bit, x64 → 64-bit).
    ///
    /// # Arguments
    /// * `input` - The address string to parse (e.g., "game.exe+100->20")
    ///
    /// # Returns
    /// * `Ok(MemoryAddress)` - The parsed address structure
    /// * `Err(ParseError)` - If parsing fails
    ///
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// // Uses default architecture (matches your binary)
    /// let addr = MemoryAddress::parse("target.exe+1000->20").unwrap();
    /// ```
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut chars = input.chars().peekable();
        let mut base = None;
        let mut operations = Vec::new();
        
        // Check for 0x or 0X prefix (legacy hex format)
        if input.starts_with("0x") || input.starts_with("0X") {
            // Skip the "0x" prefix by consuming two characters
            chars.next(); // consume '0'
            chars.next(); // consume 'x' or 'X'
            
            let address = Self::parse_number(&mut chars)?;
            base = Some(AddressBase::Absolute(address));
        }
        // Check if it starts with a digit (absolute address in hex by default)
        else if let Some(&first_ch) = chars.peek() {
            if first_ch.is_ascii_digit() || first_ch == '#' {
                let address = Self::parse_number(&mut chars)?;
                base = Some(AddressBase::Absolute(address));
                // Continue to parse operations after the base address
            }
        }
        
        // Parse operations (+offset, ->offset, or module name)
        while let Some(&ch) = chars.peek() {
            match ch {
                '-' => {
                    chars.next(); // consume '-'
                    if chars.next() != Some('>') {
                        return Err(ParseError::InvalidPointerSyntax("Expected '>' after '-'".to_string()));
                    }
                    let offset = Self::parse_number(&mut chars)?;
                    operations.push(AddressOp::PointerJump(offset));
                },
                
                '+' => {
                    chars.next(); // consume '+'
                    
                    // Check if this is a 0x prefixed number
                    let next_chars: String = chars.clone().take(2).collect();
                    if next_chars == "0x" || next_chars == "0X" {
                        // Consume the 0x prefix
                        chars.next(); // '0'
                        chars.next(); // 'x' or 'X'
                    }
                    
                    let offset = Self::parse_number(&mut chars)?;
                    operations.push(AddressOp::DirectOffset(offset));
                },
                
                _ if ch.is_alphabetic() || ch == '_' || ch == '.' => {
                    if base.is_none() {
                        base = Some(Self::parse_module_name(&mut chars)?);
                    } else {
                        return Err(ParseError::MultipleModules);
                    }
                },
                
                _ => {
                    return Err(ParseError::UnexpectedCharacter(ch));
                }
            }
        }
        
        Ok(MemoryAddress {
            base: base.ok_or(ParseError::EmptyInput)?,
            operations,
            pointer_size: PointerSize::default_architecture(),
        })
    }

    /// Parses an address string for x86 (32-bit) target processes.
    ///
    /// This is a convenience method that sets the pointer size to 4 bytes (32-bit).
    /// Use this when targeting 32-bit processes like old games or legacy applications.
    ///
    /// # Arguments
    /// * `input` - The address string to parse (e.g., "lf2.exe+58C94->308")
    ///
    /// # Returns
    /// * `Ok(MemoryAddress)` - The parsed address with 32-bit pointer size
    /// * `Err(ParseError)` - If parsing fails
    ///
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// // Parse address for 32-bit process (e.g., old game)
    /// let addr = MemoryAddress::new_x86("lf2.exe+58C94->308").unwrap();
    /// assert_eq!(addr.pointer_size, win_auto_utils::memory_resolver::PointerSize::Bits32);
    /// ```
    pub fn new_x86(input: &str) -> Result<Self, ParseError> {
        let mut addr = Self::parse(input)?;
        addr.pointer_size = PointerSize::Bits32;
        Ok(addr)
    }

    /// Parses an address string for x64 (64-bit) target processes.
    ///
    /// This is a convenience method that sets the pointer size to 8 bytes (64-bit).
    /// Use this when targeting modern 64-bit applications.
    ///
    /// # Arguments
    /// * `input` - The address string to parse (e.g., "game.exe+1000->20")
    ///
    /// # Returns
    /// * `Ok(MemoryAddress)` - The parsed address with 64-bit pointer size
    /// * `Err(ParseError)` - If parsing fails
    ///
    /// # Examples
    /// ```
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// // Parse address for 64-bit process (modern application)
    /// let addr = MemoryAddress::new_x64("game.exe+1000->20").unwrap();
    /// assert_eq!(addr.pointer_size, win_auto_utils::memory_resolver::PointerSize::Bits64);
    /// ```
    pub fn new_x64(input: &str) -> Result<Self, ParseError> {
        let mut addr = Self::parse(input)?;
        addr.pointer_size = PointerSize::Bits64;
        Ok(addr)
    }
    
    /// Parse a number (hex by default, decimal with '#' prefix)
    fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<usize, ParseError> {
        let mut num_str = String::new();
        let mut is_decimal = false;
        
        if let Some(&'#') = chars.peek() {
            chars.next();
            is_decimal = true;
        }
        
        // Check for 0x or 0X prefix (optional, for compatibility)
        let next_two: String = chars.clone().take(2).collect();
        if next_two == "0x" || next_two == "0X" {
            chars.next(); // consume '0'
            chars.next(); // consume 'x' or 'X'
            // Continue to parse hex digits
        }
        
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_hexdigit() || ch == '_' {
                num_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        
        if num_str.is_empty() {
            return Err(ParseError::InvalidOffset("Empty number".to_string()));
        }
        
        num_str.retain(|c| c != '_');
        
        if is_decimal {
            usize::from_str_radix(&num_str, 10)
                .map_err(|_| ParseError::InvalidDecimal(num_str))
        } else {
            usize::from_str_radix(&num_str, 16)
                .map_err(|_| ParseError::InvalidHex(num_str))
        }
    }
    
    /// Parse module name (e.g., "app.dll", "target.exe")
    fn parse_module_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<AddressBase, ParseError> {
        let mut name = String::new();
        
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '-' {
                name.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        
        if !name.contains(".dll") && !name.contains(".exe") {
            return Err(ParseError::InvalidModuleFormat(name));
        }
        
        if let Some(&'+') = chars.peek() {
            chars.next(); // consume '+'
            
            // Check for 0x prefix in module offset
            let next_chars: String = chars.clone().take(2).collect();
            if next_chars == "0x" || next_chars == "0X" {
                chars.next(); // '0'
                chars.next(); // 'x' or 'X'
            }
            
            let offset = Self::parse_number(chars)?;
            Ok(AddressBase::Module { name, offset })
        } else {
            Ok(AddressBase::Module { name, offset: 0 })
        }
    }

    /// Set the pointer size for this address
    ///
    /// # Arguments
    /// * `size` - Pointer size configuration (Bits32 or Bits64)
    ///
    /// # Returns
    /// Modified MemoryAddress with updated pointer size
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_resolver::{MemoryAddress, PointerSize};
    ///
    /// // For x86 (32-bit) process
    /// let addr = MemoryAddress::parse("lf2.exe+58C94->308")?
    ///     .with_pointer_size(PointerSize::Bits32);
    ///
    /// // For x64 (64-bit) process
    /// let addr = MemoryAddress::parse("game.exe+1000->20")?
    ///     .with_pointer_size(PointerSize::Bits64);
    /// # Ok::<_, win_auto_utils::memory_resolver::ParseError>(())
    /// ```
    pub fn with_pointer_size(mut self, size: PointerSize) -> Self {
        self.pointer_size = size;
        self
    }

    /// Shorthand for x86 (32-bit) processes
    pub fn x86(self) -> Self {
        self.with_pointer_size(PointerSize::Bits32)
    }

    /// Shorthand for x64 (64-bit) processes (default)
    pub fn x64(self) -> Self {
        self.with_pointer_size(PointerSize::Bits64)
    }

    /// Resolves this address to a final virtual memory address.
    ///
    /// # Arguments
    /// * `handle` - Handle to the target process (must have PROCESS_VM_READ)
    /// * `pid` - Process ID (required for module-based addresses)
    ///
    /// # Returns
    /// * `Ok(usize)` - The resolved memory address
    /// * `Err(MemoryError)` - If resolution fails (invalid address, null pointer, etc.)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    /// use win_auto_utils::handle::open_process_handle;
    /// use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
    ///
    /// let pid = 12345;
    /// let handle = open_process_handle(pid, PROCESS_VM_READ | PROCESS_QUERY_INFORMATION).unwrap();
    /// let addr = MemoryAddress::parse("target.exe+1000->20").unwrap();
    /// let resolved = addr.resolve_address(handle, pid).unwrap();
    /// ```
    pub fn resolve_address(&self, handle: HANDLE, pid: u32) -> Result<usize, ResolveError> {
        let mut current = match &self.base {
            AddressBase::Absolute(address) => *address,
            AddressBase::Module { name, offset } => {
                let base = crate::snapshot::get_module_base_address(pid, name)
                    .ok_or_else(|| ResolveError::ModuleNotFound(name.clone()))?;
                base + offset
            }
        };

        for op in &self.operations {
            match op {
                AddressOp::DirectOffset(offset) => {
                    current += offset;
                }
                AddressOp::PointerJump(offset) => {
                    // Read pointer based on configured pointer size
                    let ptr_value = match self.pointer_size {
                        PointerSize::Bits32 => {
                            // x86: Read 4-byte pointer
                            crate::memory::read_memory_u32(handle, current)
                                .map_err(|e| ResolveError::PointerReadFailed(current, e))? as u64
                        }
                        PointerSize::Bits64 => {
                            // x64: Read 8-byte pointer
                            crate::memory::read_memory_u64(handle, current)
                                .map_err(|e| ResolveError::PointerReadFailed(current, e))?
                        }
                    };
                    
                    if ptr_value == 0 {
                        return Err(ResolveError::NullPointer(current));
                    }
                    current = ptr_value as usize + offset;
                }
            }
        }

        Ok(current)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_absolute_hex_address() {
        let addr = MemoryAddress::parse("0x1234").unwrap();
        assert!(matches!(addr.base, AddressBase::Absolute(0x1234)));
        assert!(addr.operations.is_empty());
    }

    #[test]
    fn test_parse_absolute_hex_without_prefix() {
        let addr = MemoryAddress::parse("1234").unwrap();
        assert!(matches!(addr.base, AddressBase::Absolute(0x1234)));
    }

    #[test]
    fn test_parse_decimal_address() {
        let addr = MemoryAddress::parse("#100").unwrap();
        assert!(matches!(addr.base, AddressBase::Absolute(100)));
    }

    #[test]
    fn test_parse_module_address() {
        let addr = MemoryAddress::parse("target.exe+1000").unwrap();
        match addr.base {
            AddressBase::Module { name, offset } => {
                assert_eq!(name, "target.exe");
                assert_eq!(offset, 0x1000);
            }
            _ => panic!("Expected Module base"),
        }
    }

    #[test]
    fn test_parse_pointer_chain() {
        let addr = MemoryAddress::parse("target.exe+100->20->30").unwrap();
        assert_eq!(addr.operations.len(), 2);
        assert!(matches!(addr.operations[0], AddressOp::PointerJump(0x20)));
        assert!(matches!(addr.operations[1], AddressOp::PointerJump(0x30)));
    }

    #[test]
    fn test_parse_direct_offset() {
        let addr = MemoryAddress::parse("0x1000+100").unwrap();
        assert_eq!(addr.operations.len(), 1);
        assert!(matches!(addr.operations[0], AddressOp::DirectOffset(0x100)));
    }

    #[test]
    fn test_parse_with_underscore() {
        let addr = MemoryAddress::parse("1_0000").unwrap();
        assert!(matches!(addr.base, AddressBase::Absolute(0x10000)));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(MemoryAddress::parse("").is_err());
        assert!(MemoryAddress::parse("invalid").is_err());
    }

    #[test]
    fn test_parse_mixed_operations() {
        let addr = MemoryAddress::parse("target.exe+100->20+30").unwrap();
        // Module offset is part of the base, not an operation
        assert_eq!(addr.operations.len(), 2);
        assert!(matches!(addr.operations[0], AddressOp::PointerJump(0x20)));
        assert!(matches!(addr.operations[1], AddressOp::DirectOffset(0x30)));
    }

    #[test]
    fn test_new_x86_shortcut() {
        let addr = MemoryAddress::new_x86("lf2.exe+58C94->308").unwrap();
        assert_eq!(addr.pointer_size, PointerSize::Bits32);
        match addr.base {
            AddressBase::Module { name, offset } => {
                assert_eq!(name, "lf2.exe");
                assert_eq!(offset, 0x58C94);
            }
            _ => panic!("Expected Module base"),
        }
    }

    #[test]
    fn test_new_x64_shortcut() {
        let addr = MemoryAddress::new_x64("game.exe+1000->20").unwrap();
        assert_eq!(addr.pointer_size, PointerSize::Bits64);
        match addr.base {
            AddressBase::Module { name, offset } => {
                assert_eq!(name, "game.exe");
                assert_eq!(offset, 0x1000);
            }
            _ => panic!("Expected Module base"),
        }
    }

    #[test]
    fn test_parse_default_architecture() {
        let addr = MemoryAddress::parse("test.exe+100").unwrap();
        
        // Should match compilation target architecture
        #[cfg(target_pointer_width = "64")]
        assert_eq!(addr.pointer_size, PointerSize::Bits64);
        
        #[cfg(target_pointer_width = "32")]
        assert_eq!(addr.pointer_size, PointerSize::Bits32);
    }

    #[test]
    fn test_resolve_null_pointer_error() {
        // Test with null handle - should fail gracefully
        let addr = MemoryAddress::parse("0x1000->10").unwrap();
        let handle = HANDLE(std::ptr::null_mut());
        let result = addr.resolve_address(handle, 0);
        assert!(result.is_err());
    }
}
