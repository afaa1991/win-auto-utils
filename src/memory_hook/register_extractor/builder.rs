//! Builder pattern for RegisterExtractor configuration
//!
//! Provides a fluent API for configuring register extraction with validation.

use crate::memory_hook::register_extractor::extractor::RegisterExtractor;
use crate::memory_hook::register_extractor::register::Register;
use crate::memory::MemoryError;
use windows::Win32::Foundation::HANDLE;

/// Builder for constructing RegisterExtractor instances
pub struct RegisterExtractorBuilder {
    handle: Option<HANDLE>,
    target_address: Option<usize>,
    bytes_to_overwrite: Option<usize>,
    extracted_regs: Vec<Register>,
    is_x64: bool,
}

impl RegisterExtractorBuilder {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self {
            handle: None,
            target_address: None,
            bytes_to_overwrite: None,
            extracted_regs: Vec::new(),
            is_x64: true, // Default to x64
        }
    }

    /// Set the process handle
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set the target address to hook
    pub fn target_address(mut self, addr: usize) -> Self {
        self.target_address = Some(addr);
        self
    }

    /// Set the number of bytes to overwrite at the hook point
    pub fn bytes_to_overwrite(mut self, bytes: usize) -> Self {
        self.bytes_to_overwrite = Some(bytes);
        self
    }

    /// Add a register to extract
    pub fn extract_register(mut self, reg: Register) -> Self {
        if !self.extracted_regs.contains(&reg) {
            self.extracted_regs.push(reg);
        }
        self
    }

    /// Add multiple registers to extract
    pub fn extract_registers(mut self, regs: &[Register]) -> Self {
        for &reg in regs {
            if !self.extracted_regs.contains(&reg) {
                self.extracted_regs.push(reg);
            }
        }
        self
    }

    /// Set architecture to x86 (32-bit)
    pub fn x86(mut self) -> Self {
        self.is_x64 = false;
        self
    }

    /// Set architecture to x64 (64-bit)
    pub fn x64(mut self) -> Self {
        self.is_x64 = true;
        self
    }

    /// Build the RegisterExtractor instance
    ///
    /// # Errors
    /// Returns `MemoryError::InvalidAddress` if required fields are missing
    pub fn build(self) -> Result<RegisterExtractor, MemoryError> {
        let handle = self.handle.ok_or_else(|| {
            MemoryError::InvalidAddress("handle is required".to_string())
        })?;

        let target_address = self.target_address.ok_or_else(|| {
            MemoryError::InvalidAddress("target_address is required".to_string())
        })?;

        let bytes_to_overwrite = self.bytes_to_overwrite.ok_or_else(|| {
            MemoryError::InvalidAddress("bytes_to_overwrite is required".to_string())
        })?;

        if self.extracted_regs.is_empty() {
            return Err(MemoryError::InvalidAddress(
                "At least one register must be specified for extraction".to_string()
            ));
        }

        Ok(RegisterExtractor::new(
            handle,
            target_address,
            bytes_to_overwrite,
            self.extracted_regs,
            self.is_x64,
        ))
    }
}

impl Default for RegisterExtractorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
