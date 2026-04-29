//! MemoryAddress Builder Module
//!
//! Provides a fluent API for building MemoryAddress with optional configuration.
//! This allows fine-grained control over pointer size and other settings.

use crate::memory_resolver::resolver::{MemoryAddress, ParseError, PointerSize};

/// Builder for configuring MemoryAddress
///
/// Provides a fluent API for building MemoryAddress with optional settings.
/// All configuration is optional - use only what you need.
///
/// # Examples
///
/// ## Set Address and Build
/// ```ignore
/// use win_auto_utils::memory_resolver::MemoryAddress;
///
/// let addr = MemoryAddress::builder()
///     .address("app.dll+100->20")
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## Configure x86 Architecture
/// ```ignore
/// use win_auto_utils::memory_resolver::MemoryAddress;
///
/// let addr = MemoryAddress::builder()
///     .address("lf2.exe+58C94->308")
///     .x86()  // 32-bit process
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## Configure x64 Architecture
/// ```ignore
/// use win_auto_utils::memory_resolver::MemoryAddress;
///
/// let addr = MemoryAddress::builder()
///     .address("target.exe+ABC->10")
///     .x64()  // 64-bit process
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct MemoryAddressBuilder {
    /// Address string to parse (required)
    address_str: Option<String>,
    /// Pointer size configuration (optional, defaults to target architecture)
    pointer_size: Option<PointerSize>,
}

impl MemoryAddressBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            address_str: None,
            pointer_size: None,
        }
    }

    /// Set the address string to parse
    ///
    /// This sets the address string that will be parsed during `build()`.
    /// The actual parsing is deferred until `build()` is called.
    ///
    /// # Arguments
    /// * `input` - The address string to parse (e.g., "game.exe+100->20")
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Example
    /// ```ignore
    /// let addr = MemoryAddress::builder()
    ///     .address("app.dll+100->20")
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn address(mut self, input: &str) -> Self {
        self.address_str = Some(input.to_string());
        self
    }

    /// Shorthand for x86 (32-bit) processes
    ///
    /// Configures the builder to use 4-byte pointers for 32-bit target processes.
    ///
    /// # Example
    /// ```ignore
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// let addr = MemoryAddress::builder()
    ///     .address("lf2.exe+58C94->308")
    ///     .x86()
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn x86(mut self) -> Self {
        self.pointer_size = Some(PointerSize::Bits32);
        self
    }

    /// Shorthand for x64 (64-bit) processes
    ///
    /// Configures the builder to use 8-byte pointers for 64-bit target processes.
    ///
    /// # Example
    /// ```ignore
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// let addr = MemoryAddress::builder()
    ///     .address("game.exe+1000->20")
    ///     .x64()
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn x64(mut self) -> Self {
        self.pointer_size = Some(PointerSize::Bits64);
        self
    }

    /// Build the final MemoryAddress
    ///
    /// This method performs the actual parsing of the address string and applies
    /// all configured settings.
    ///
    /// # Returns
    /// Result containing the configured MemoryAddress or a ParseError
    ///
    /// # Errors
    /// Returns ParseError if:
    /// - `address()` was not called before `build()`
    /// - The address string format is invalid
    ///
    /// # Example
    /// ```ignore
    /// let addr = MemoryAddress::builder()
    ///     .address("game.exe+100->20")
    ///     .x86()
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn build(self) -> Result<MemoryAddress, ParseError> {
        let address_str = self.address_str
            .expect("MemoryAddressBuilder: address() must be called before build()");
        
        let mut addr = MemoryAddress::parse(&address_str)?;
        
        // Apply pointer size if configured
        if let Some(size) = self.pointer_size {
            addr.pointer_size = size;
        }
        
        Ok(addr)
    }
}

impl Default for MemoryAddressBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic用法() {
        let addr = MemoryAddressBuilder::new()
            .address("0x12345678")
            .build()
            .unwrap();
        
        assert!(matches!(addr.base, crate::memory_resolver::resolver::AddressBase::Absolute(_)));
    }

    #[test]
    fn test_builder_x86_shortcut() {
        let addr = MemoryAddressBuilder::new()
            .address("lf2.exe+58C94->308")
            .x86()
            .build()
            .unwrap();
        
        assert_eq!(addr.pointer_size, PointerSize::Bits32);
    }

    #[test]
    fn test_builder_x64_shortcut() {
        let addr = MemoryAddressBuilder::new()
            .address("game.exe+1000->20")
            .x64()
            .build()
            .unwrap();
        
        assert_eq!(addr.pointer_size, PointerSize::Bits64);
    }

    #[test]
    #[should_panic(expected = "address() must be called before build()")]
    fn test_builder_without_address_panics() {
        let _addr = MemoryAddressBuilder::new().build().unwrap();
    }

    #[test]
    fn test_builder_invalid_address() {
        let result = MemoryAddressBuilder::new()
            .address("invalid address string")
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_default_pointer_size() {
        // Without calling x86() or x64(), should use default architecture
        let addr = MemoryAddressBuilder::new()
            .address("test.exe+100")
            .build()
            .unwrap();
        
        // Should match the compilation target architecture
        #[cfg(target_pointer_width = "64")]
        assert_eq!(addr.pointer_size, PointerSize::Bits64);
        
        #[cfg(target_pointer_width = "32")]
        assert_eq!(addr.pointer_size, PointerSize::Bits32);
    }
}
