//! Register definitions and offset mappings for RegisterExtractor
//!
//! This module defines the supported registers and their storage offsets
//! in the private memory region.

/// Supported CPU registers for extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    // x64 64-bit registers
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    
    // x86 32-bit registers (aliases to lower 32 bits of x64 registers)
    // These can be used on x64 processes to extract 32-bit register values
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
}

impl Register {
    /// Get the storage offset for this register (in bytes)
    /// 
    /// For x64: each register occupies 8 bytes
    /// For x86: each register occupies 4 bytes
    pub fn offset(&self, is_x64: bool) -> usize {
        let reg_size = if is_x64 { 8 } else { 4 };
        let index = self.register_index() as usize;
        index * reg_size
    }
    
    /// Get the internal register index (0-15 for x64, 0-7 for x86 aliases)
    pub fn register_index(&self) -> u8 {
        match self {
            // x64 registers
            Register::RAX => 0,
            Register::RCX => 1,
            Register::RDX => 2,
            Register::RBX => 3,
            Register::RSP => 4,
            Register::RBP => 5,
            Register::RSI => 6,
            Register::RDI => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
            Register::R12 => 12,
            Register::R13 => 13,
            Register::R14 => 14,
            Register::R15 => 15,
            
            // x86 32-bit register aliases (map to same index as their 64-bit counterparts)
            Register::EAX => 0,  // Lower 32 bits of RAX
            Register::ECX => 1,  // Lower 32 bits of RCX
            Register::EDX => 2,  // Lower 32 bits of RDX
            Register::EBX => 3,  // Lower 32 bits of RBX
            Register::ESP => 4,  // Lower 32 bits of RSP
            Register::EBP => 5,  // Lower 32 bits of RBP
            Register::ESI => 6,  // Lower 32 bits of RSI
            Register::EDI => 7,  // Lower 32 bits of RDI
        }
    }

    /// Get the size of this register in bytes
    pub fn size(&self, is_x64: bool) -> usize {
        // Check if this is a 32-bit register alias
        let is_32bit_alias = matches!(
            self,
            Register::EAX | Register::ECX | Register::EDX | Register::EBX |
            Register::ESP | Register::EBP | Register::ESI | Register::EDI
        );
        
        if is_32bit_alias {
            4  // Always 4 bytes for 32-bit registers
        } else if is_x64 {
            8  // 64-bit registers on x64
        } else {
            4  // 32-bit registers on x86
        }
    }
    
    /// Check if this is a 32-bit register alias (EAX, ECX, etc.)
    pub fn is_32bit_alias(&self) -> bool {
        matches!(
            self,
            Register::EAX | Register::ECX | Register::EDX | Register::EBX |
            Register::ESP | Register::EBP | Register::ESI | Register::EDI
        )
    }

    /// Get all available x64 registers
    pub fn all_x64_registers() -> Vec<Register> {
        vec![
            Register::RAX, Register::RCX, Register::RDX, Register::RBX,
            Register::RSP, Register::RBP, Register::RSI, Register::RDI,
            Register::R8, Register::R9, Register::R10, Register::R11,
            Register::R12, Register::R13, Register::R14, Register::R15,
        ]
    }
    
    /// Get all available x86 registers
    pub fn all_x86_registers() -> Vec<Register> {
        vec![
            Register::EAX, Register::ECX, Register::EDX, Register::EBX,
            Register::ESP, Register::EBP, Register::ESI, Register::EDI,
        ]
    }
}

/// Calculate total storage size needed for extracted registers
pub fn calculate_storage_size(registers: &[Register], is_x64: bool) -> usize {
    if registers.is_empty() {
        return 0;
    }
    
    let reg_size = if is_x64 { 8 } else { 4 };
    // Use fixed layout: always allocate space for all 16 registers for simplicity
    // This ensures consistent offsets regardless of which registers are extracted
    16 * reg_size
}
