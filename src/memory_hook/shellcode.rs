//! Shellcode builder for generating machine code instructions
//!
//! Provides utilities for creating x86/x64 shellcode for hooks and injections.

/// Architecture type for shellcode generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X64,
}

/// Shellcode builder for creating machine code sequences
///
/// # Example
/// ```
/// use win_auto_utils::memory_hook::ShellcodeBuilder;
///
/// // Create a JMP instruction for x64
/// let mut builder = ShellcodeBuilder::new_x64();
/// builder.jmp_absolute(0x7FF6A1B2C3D4);
/// let code = builder.build();
/// assert_eq!(code.len(), 12); // MOV RAX (10 bytes) + JMP RAX (2 bytes)
/// ```
pub struct ShellcodeBuilder {
    arch: Architecture,
    bytes: Vec<u8>,
}

impl ShellcodeBuilder {
    /// Create a new builder for x86 architecture
    pub fn new_x86() -> Self {
        Self {
            arch: Architecture::X86,
            bytes: Vec::new(),
        }
    }
    
    /// Create a new builder for x64 architecture
    pub fn new_x64() -> Self {
        Self {
            arch: Architecture::X64,
            bytes: Vec::new(),
        }
    }
    
    /// Get the current architecture
    pub fn architecture(&self) -> Architecture {
        self.arch
    }
    
    /// Append raw bytes to the shellcode
    pub fn append_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(bytes);
        self
    }
    
    /// Generate a relative JMP instruction (5 bytes on x86, 5 bytes on x64)
    ///
    /// Format: E9 <relative_offset_32>
    ///
    /// # Arguments
    /// * `from` - Source address (where the JMP will be placed)
    /// * `to` - Target address (where to jump to)
    ///
    /// # Note
    /// The offset is calculated as: to - from - 5
    pub fn jmp_relative(&mut self, from: usize, to: usize) -> &mut Self {
        let offset = (to as i64) - (from as i64) - 5;
        
        if offset < i32::MIN as i64 || offset > i32::MAX as i64 {
            // Offset too large for relative jump, use absolute
            return self.jmp_absolute(to);
        }
        
        self.bytes.push(0xE9); // JMP rel32
        self.bytes.extend_from_slice(&(offset as i32).to_le_bytes());
        self
    }
    
    /// Generate Absolute jump instruction (architecture-aware)
    ///
    /// x86: Uses PUSH + RET trick (6 bytes)
    /// x64: Uses MOV R11 + JMP R11 (13 bytes, R11 is caller-saved register)
    ///
    /// # Arguments
    /// * `target` - Absolute target address
    pub fn jmp_absolute(&mut self, target: usize) -> &mut Self {
        match self.arch {
            Architecture::X86 => {
                // PUSH target_address (little-endian)
                // RET
                self.bytes.push(0x68); // PUSH imm32
                self.bytes.extend_from_slice(&(target as u32).to_le_bytes());
                self.bytes.push(0xC3); // RET
            }
            Architecture::X64 => {
                // MOV R11, target_address (R11 is caller-saved, safe to use)
                // JMP R11
                self.bytes.push(0x49); // REX.W + R11 prefix
                self.bytes.push(0xBB); // MOV R11, imm64
                self.bytes.extend_from_slice(&(target as u64).to_le_bytes());
                self.bytes.push(0x41); // REX.B prefix for R11
                self.bytes.push(0xFF); // JMP r/m64
                self.bytes.push(0xE3); // R11
            }
        }
        self
    }
    
    /// Generate NOP instructions
    ///
    /// # Arguments
    /// * `count` - Number of NOP bytes
    pub fn nop(&mut self, count: usize) -> &mut Self {
        for _ in 0..count {
            self.bytes.push(0x90);
        }
        self
    }
    
    /// Generate INT3 breakpoint
    pub fn int3(&mut self) -> &mut Self {
        self.bytes.push(0xCC);
        self
    }
    
    /// Save all general-purpose registers (PUSHAD/PUSHF or equivalent)
    pub fn push_all_registers(&mut self) -> &mut Self {
        match self.arch {
            Architecture::X86 => {
                self.bytes.push(0x60); // PUSHAD
                self.bytes.push(0x9C); // PUSHFD
            }
            Architecture::X64 => {
                // Manually push all 16 general-purpose registers
                // Order: RAX, RCX, RDX, RBX, RSP, RBP, RSI, RDI, R8-R15
                for reg in [0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57] {
                    self.bytes.push(reg); // PUSH RAX-RCX...RDI
                }
                // R8-R15 require REX prefix
                for reg in 0x50..=0x57 {
                    self.bytes.push(0x41); // REX.B
                    self.bytes.push(reg);  // PUSH R8-R15
                }
                self.bytes.push(0x9C); // PUSHFQ
            }
        }
        self
    }
    
    /// Restore all general-purpose registers (POPAD/POPF or equivalent)
    pub fn pop_all_registers(&mut self) -> &mut Self {
        match self.arch {
            Architecture::X86 => {
                self.bytes.push(0x9D); // POPFD
                self.bytes.push(0x61); // POPAD
            }
            Architecture::X64 => {
                self.bytes.push(0x9D); // POPFQ
                // Pop in reverse order: R15-R8, RDI-RAX
                for reg in (0x50..=0x57).rev() {
                    self.bytes.push(0x41); // REX.B
                    self.bytes.push(reg);  // POP R15-R8
                }
                for reg in [0x5F, 0x5E, 0x5D, 0x5C, 0x5B, 0x5A, 0x59, 0x58].iter().rev() {
                    self.bytes.push(*reg); // POP RDI-RAX
                }
            }
        }
        self
    }
    
    /// Generate CALL instruction
    ///
    /// # Arguments
    /// * `target` - Target address to call
    pub fn call_absolute(&mut self, target: usize) -> &mut Self {
        match self.arch {
            Architecture::X86 => {
                // PUSH return_address (implicit in CALL)
                // JMP target
                self.bytes.push(0xE8); // CALL rel32
                let offset = (target as i64) - (self.bytes.len() as i64 + 4) - 4;
                self.bytes.extend_from_slice(&(offset as i32).to_le_bytes());
            }
            Architecture::X64 => {
                // For x64, we'll use the same approach but with proper addressing
                self.bytes.push(0xE8); // CALL rel32
                let offset = (target as i64) - (self.bytes.len() as i64 + 4) - 4;
                self.bytes.extend_from_slice(&(offset as i32).to_le_bytes());
            }
        }
        self
    }

    /// PUSH a specific x64 register by index (0=RAX, 1=RCX, ..., 15=R15)
    ///
    /// # Arguments
    /// * `reg_index` - Register index (0-15)
    pub fn push_reg_x64(&mut self, reg_index: u8) -> &mut Self {
        if reg_index < 8 {
            // RAX-RDI: 0x50 + reg_index
            self.bytes.push(0x50 + reg_index);
        } else {
            // R8-R15: REX.B prefix + (0x50 + (reg_index - 8))
            self.bytes.push(0x41);
            self.bytes.push(0x50 + (reg_index - 8));
        }
        self
    }

    /// POP a specific x64 register by index (0=RAX, 1=RCX, ..., 15=R15)
    ///
    /// # Arguments
    /// * `reg_index` - Register index (0-15)
    pub fn pop_reg_x64(&mut self, reg_index: u8) -> &mut Self {
        if reg_index < 8 {
            // RAX-RDI: 0x58 + reg_index
            self.bytes.push(0x58 + reg_index);
        } else {
            // R8-R15: REX.B prefix + (0x58 + (reg_index - 8))
            self.bytes.push(0x41);
            self.bytes.push(0x58 + (reg_index - 8));
        }
        self
    }

    /// MOV \[absolute_address\], register (32-bit)
    ///
    /// Stores a 32-bit register value to an absolute memory address.
    /// This method automatically handles both x86 and x64 architectures correctly.
    ///
    /// # Architecture Differences:
    /// - **x86**: Uses direct absolute addressing `MOV [addr], REG32`
    /// - **x64**: Uses R11 as temporary pointer to avoid RIP-relative limitations (±2GB range)
    ///
    /// # Arguments
    /// * `address` - Absolute target address
    /// * `reg_index` - Register index (0=EAX/RCX, 1=ECX/RCX, ..., 7=EDI/RDI)
    pub fn mov_mem32_from_reg(&mut self, address: usize, reg_index: u8) -> &mut Self {
        if reg_index > 7 {
            panic!("32-bit registers only support indices 0-7 (EAX/RCX to EDI/RDI), got {}", reg_index);
        }
        
        match self.arch {
            Architecture::X86 => {
                // x86: Direct absolute addressing (safe for 32-bit addresses)
                // MOV [address], REG32
                if reg_index == 0 {
                    // Special case: MOV [address], EAX has dedicated opcode A3
                    self.bytes.push(0xA3);
                    self.bytes.extend_from_slice(&(address as u32).to_le_bytes());
                } else {
                    // General case: MOV [address], REG
                    // Opcode: 89 /r with ModR/M mod=00, r/m=101 (disp32)
                    self.bytes.push(0x89);
                    
                    // ModR/M byte: mod=00, reg=reg_index, r/m=101 (SIB with disp32)
                    let modrm = 0x05 | (reg_index << 3);
                    self.bytes.push(modrm);
                    self.bytes.extend_from_slice(&(address as u32).to_le_bytes());
                }
            }
            Architecture::X64 => {
                // x64: Use R11 as temporary pointer to avoid RIP-relative addressing issues
                // Strategy: MOV R11, addr -> MOV [R11], REG32
                
                // Step 1: Load absolute address into R11
                // MOV R11, imm64
                self.bytes.push(0x49); // REX.W + R11
                self.bytes.push(0xBB); // MOV R11, imm64
                self.bytes.extend_from_slice(&(address as u64).to_le_bytes());
                
                // Step 2: Store 32-bit register via R11
                // MOV [R11], REG32
                // Opcode: 41 89 /r (REX.B + MOV r/m32, r32)
                self.bytes.push(0x41);  // REX.B prefix (for R11 base)
                self.bytes.push(0x89);  // MOV r/m32, r32
                
                // ModR/M: mod=00, reg=reg_index, r/m=011 (R11 with REX.B)
                let modrm = 0x03 | (reg_index << 3);
                self.bytes.push(modrm);
            }
        }
        
        self
    }

    /// MOV [absolute_address], register (x64, 64-bit)
    ///
    /// Stores a 64-bit register value to an absolute memory address.
    /// Strategy: Use R11 as temporary pointer (safe because we're inside PUSH/POP context).
    /// 
    /// IMPORTANT: This method assumes it's called within a saved register context
    /// (e.g., after PUSH all registers). R11 is used as temporary but its original
    /// value is already on the stack.
    ///
    /// # Arguments
    /// * `address` - Absolute target address
    /// * `reg_index` - Register index (0=RAX, 1=RCX, ..., 15=R15)
    pub fn mov_mem64_from_reg(&mut self, address: usize, reg_index: u8) -> &mut Self {
        // Step 1: MOV R11, address (load absolute address into R11)
        self.bytes.push(0x49); // REX.W + R11
        self.bytes.push(0xBB); // MOV R11, imm64
        self.bytes.extend_from_slice(&(address as u64).to_le_bytes());
        
        // Step 2: MOV [R11], reg (store register via R11 pointer)
        if reg_index < 8 {
            // RAX-RDI: Need REX.B prefix to specify R11 as base register
            self.bytes.push(0x49); // REX.WB (W=1 for 64-bit, B=1 for R11)
            self.bytes.push(0x89); // MOV r/m64, r64
            
            // ModR/M: mod=00 (no displacement), reg=reg_index, r/m=011 (R11 with REX.B)
            let modrm = (reg_index << 3) | 0x03;
            self.bytes.push(modrm);
        } else {
            // R8-R15: Need REX.RB prefix (R for source reg >= 8, B for R11)
            let rex = 0x4C | ((reg_index - 8) >> 2 & 0x01); // REX.WRB
            self.bytes.push(rex);
            self.bytes.push(0x89); // MOV r/m64, r64
            
            // ModR/M: mod=00, reg=(reg_index-8), r/m=011 (R11 with REX.B)
            let modrm = ((reg_index - 8) << 3) | 0x03;
            self.bytes.push(modrm);
        }
        
        self
    }

    /// PUSHAD (x86 only) - Push all general-purpose registers
    pub fn pushad(&mut self) -> &mut Self {
        if self.arch != Architecture::X86 {
            panic!("PUSHAD is x86 only");
        }
        self.bytes.push(0x60);
        self
    }

    /// POPAD (x86 only) - Pop all general-purpose registers
    pub fn popad(&mut self) -> &mut Self {
        if self.arch != Architecture::X86 {
            panic!("POPAD is x86 only");
        }
        self.bytes.push(0x61);
        self
    }
    
    /// Build the final shellcode
    pub fn build(self) -> Vec<u8> {
        self.bytes
    }
    
    /// Get the current length of the shellcode
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    
    /// Check if the shellcode is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_x86_jmp_relative() {
        let mut builder = ShellcodeBuilder::new_x86();
        builder.jmp_relative(0x1000, 0x2000);
        let code = builder.build();
        
        assert_eq!(code.len(), 5);
        assert_eq!(code[0], 0xE9);
    }
    
    #[test]
    fn test_x64_jmp_absolute() {
        let mut builder = ShellcodeBuilder::new_x64();
        builder.jmp_absolute(0x7FF6A1B2C3D4);
        let code = builder.build();
        
        assert_eq!(code.len(), 13); // MOV R11 (11 bytes) + JMP R11 (2 bytes)
        assert_eq!(code[0], 0x49); // REX.W + R11 prefix
        assert_eq!(code[1], 0xBB); // MOV R11, imm64
    }
    
    #[test]
    fn test_push_pop_registers() {
        let mut builder = ShellcodeBuilder::new_x64();
        builder.push_all_registers();
        let push_len = builder.len();
        
        builder.pop_all_registers();
        let pop_len = builder.len() - push_len;
        
        // Verify symmetry
        assert!(push_len > 0);
        assert_eq!(push_len, pop_len);
    }
    
    #[test]
    fn test_nop_generation() {
        let mut builder = ShellcodeBuilder::new_x86();
        builder.nop(5);
        let code = builder.build();
        
        assert_eq!(code.len(), 5);
        assert!(code.iter().all(|&b| b == 0x90));
    }
}
