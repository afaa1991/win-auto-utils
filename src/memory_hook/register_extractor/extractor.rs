//! RegisterExtractor core implementation
//!
//! Provides automatic register extraction at hook points using TrampolineHook.

use crate::memory::{read_memory_t, write_memory_t, MemoryError};
use crate::memory_hook::TrampolineHook;
use crate::memory_hook::register_extractor::register::{Register, calculate_storage_size};
use crate::memory_hook::shellcode::ShellcodeBuilder;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};

/// RegisterExtractor - Automatic register value capture at hook points
///
/// This tool automatically captures specified CPU registers when a hook is triggered,
/// storing them in a private memory region for later retrieval.
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_hook::register_extractor::{RegisterExtractor, Register};
///
/// let mut extractor = RegisterExtractor::builder()
///     .handle(process_handle)
///     .target_address(0x1C17E514F90)
///     .bytes_to_overwrite(15)
///     .extract_register(Register::RDI)
///     .x64()
///     .build()?;
///
/// extractor.install()?;
///
/// // Read captured register value
/// let player_ptr: u64 = extractor.read_register::<u64>(Register::RDI)?;
///
/// extractor.uninstall()?;
/// ```
pub struct RegisterExtractor {
    hook: TrampolineHook,
    storage_address: usize,
    extracted_regs: Vec<Register>,
    is_x64: bool,
    is_installed: bool,
}

impl RegisterExtractor {
    /// Create a new builder for RegisterExtractor
    pub fn builder() -> crate::memory_hook::register_extractor::builder::RegisterExtractorBuilder {
        crate::memory_hook::register_extractor::builder::RegisterExtractorBuilder::new()
    }

    /// Internal constructor (use builder pattern instead)
    pub(crate) fn new(
        handle: HANDLE,
        target_address: usize,
        bytes_to_overwrite: usize,
        extracted_regs: Vec<Register>,
        is_x64: bool,
    ) -> Self {
        let storage_size = calculate_storage_size(&extracted_regs, is_x64);
        
        // Allocate storage memory with PAGE_READWRITE
        let storage_address = unsafe {
            VirtualAllocEx(
                handle,
                None,
                storage_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };

        if storage_address.is_null() {
            panic!("Failed to allocate storage memory for RegisterExtractor");
        }

        // Generate detour shellcode that extracts registers
        let detour_code = Self::generate_extraction_shellcode(
            &extracted_regs,
            storage_address as usize,
            is_x64,
        );

        // Create internal TrampolineHook
        let mut hook = TrampolineHook::auto_new(handle, target_address, detour_code);
        
        // Set architecture
        hook.set_architecture(is_x64);

        // Configure bytes to overwrite
        hook.set_bytes_to_overwrite(bytes_to_overwrite);

        Self {
            hook,
            storage_address: storage_address as usize,
            extracted_regs,
            is_x64,
            is_installed: false,
        }
    }

    /// Generate shellcode that extracts registers to storage
    fn generate_extraction_shellcode(
        regs: &[Register],
        storage_base: usize,
        is_x64: bool,
    ) -> Vec<u8> {
        let mut builder = if is_x64 {
            ShellcodeBuilder::new_x64()
        } else {
            ShellcodeBuilder::new_x86()
        };

        if is_x64 {
            // 🚀 OPTIMIZED: Only save registers that are actually used + R11 (temporary pointer)
            // This is much more efficient than saving all 16 registers
            
            // Collect unique registers that need to be saved
            let mut regs_to_save: Vec<u8> = Vec::new();
            
            for &reg in regs {
                let reg_index = reg.register_index();
                
                // Add if not already in list
                if !regs_to_save.contains(&reg_index) {
                    regs_to_save.push(reg_index);
                }
            }
            
            // Always save R11 (used as temporary pointer), unless it's already in the list
            if !regs_to_save.contains(&11) {
                regs_to_save.push(11);
            }
            
            // Sort for consistent PUSH/POP order (ascending for PUSH, descending for POP)
            regs_to_save.sort();
            
            // PUSH registers in ascending order
            for &reg_idx in &regs_to_save {
                builder.push_reg_x64(reg_idx);
            }
            
            // Store specified registers to memory
            for &reg in regs {
                let offset = reg.offset(is_x64);
                let addr = storage_base + offset;
                
                // Check if this is a 32-bit register alias
                if reg.is_32bit_alias() {
                    Self::store_register_x64_32bit(&mut builder, reg, addr);
                } else {
                    Self::store_register_x64(&mut builder, reg, addr);
                }
            }
            
            // POP registers in reverse (descending) order
            for reg_idx in regs_to_save.iter().rev() {
                builder.pop_reg_x64(*reg_idx);
            }
        } else {
            // x86: Use PUSHAD/POPAD (simpler, only 8 registers)
            builder.pushad();

            // Store specified registers to memory
            for &reg in regs {
                let offset = reg.offset(is_x64);
                let addr = storage_base + offset;
                Self::store_register_x86(&mut builder, reg, addr);
            }

            builder.popad();
        }

        builder.build()
    }

    /// Generate MOV instruction to store x64 32-bit register (ESI, EDI, etc.)
    fn store_register_x64_32bit(builder: &mut ShellcodeBuilder, reg: Register, addr: usize) {
        let reg_index = reg.register_index();

        // For 32-bit registers on x64, we use MOV [addr], REG32
        // This automatically zero-extends to 64 bits when stored
        builder.mov_mem32_from_reg(addr, reg_index);
    }

    /// Generate MOV instruction to store x64 register
    fn store_register_x64(builder: &mut ShellcodeBuilder, reg: Register, addr: usize) {
        // MOV [addr], REG
        // For x64, we need to use absolute addressing
        let reg_index = reg.register_index();

        // MOV [rip + offset], reg or MOV [abs_addr], reg
        // Using simple approach: MOV reg, [addr] via RAX as intermediary
        // Actually, let's use direct memory operand
        builder.mov_mem64_from_reg(addr, reg_index);
    }

    /// Generate MOV instruction to store x86 register
    fn store_register_x86(builder: &mut ShellcodeBuilder, reg: Register, addr: usize) {
        let reg_index = match reg {
            Register::RAX => 0, // EAX
            Register::RCX => 1, // ECX
            Register::RDX => 2, // EDX
            Register::RBX => 3, // EBX
            Register::RSP => 4, // ESP
            Register::RBP => 5, // EBP
            Register::RSI => 6, // ESI
            Register::RDI => 7, // EDI
            _ => panic!("x86 only supports RAX-RDI (EAX-EDI)"),
        };

        builder.mov_mem32_from_reg(addr, reg_index);
    }

    /// Install the register extractor hook
    pub fn install(&mut self) -> Result<(), MemoryError> {
        self.hook.install()?;
        self.is_installed = true;
        Ok(())
    }

    /// Uninstall the register extractor hook
    pub fn uninstall(&mut self) -> Result<(), MemoryError> {
        self.hook.uninstall()?;
        self.is_installed = false;
        Ok(())
    }

    /// Read the value of a captured register
    ///
    /// # Type Parameters
    /// * `T` - The type to read (must be Copy and sized appropriately)
    ///
    /// # Example
    /// ```no_run
    /// let player_ptr: u64 = extractor.read_register::<u64>(Register::RDI)?;
    /// let health: f32 = extractor.read_register::<f32>(Register::RAX)?;
    /// ```
    pub fn read_register<T: Copy>(&self, reg: Register) -> Result<T, MemoryError> {
        if !self.is_installed {
            return Err(MemoryError::InvalidAddress(
                "RegisterExtractor is not installed".to_string()
            ));
        }

        let offset = reg.offset(self.is_x64);
        let addr = self.storage_address + offset;
        
        read_memory_t(self.hook.handle.0, addr)
    }

    /// Read a value through a pointer chain starting from a register
    ///
    /// # Arguments
    /// * `reg` - The register containing the base pointer
    /// * `offsets` - Array of offsets to dereference
    ///
    /// # Example
    /// ```no_run
    /// // Read [[RDI + 0x10] + 0x34] as f32
    /// let health = extractor.read_chain::<f32>(Register::RDI, &[0x10, 0x34])?;
    /// ```
    pub fn read_chain<T: Copy>(&self, reg: Register, offsets: &[usize]) -> Result<T, MemoryError> {
        if !self.is_installed {
            return Err(MemoryError::InvalidAddress(
                "RegisterExtractor is not installed".to_string()
            ));
        }

        // Read base pointer from register
        let base_ptr: usize = self.read_register::<usize>(reg)?;
        
        if base_ptr == 0 {
            return Err(MemoryError::InvalidAddress(
                "Null pointer encountered in chain".to_string()
            ));
        }

        // Traverse the pointer chain
        let mut current_addr = base_ptr;
        for (i, &offset) in offsets.iter().enumerate() {
            // If this is the last offset, read the final value
            if i == offsets.len() - 1 {
                current_addr += offset;
                return read_memory_t(self.hook.handle.0, current_addr);
            } else {
                // Otherwise, dereference to get next pointer
                current_addr += offset;
                current_addr = read_memory_t::<usize>(self.hook.handle.0, current_addr)?;
                
                if current_addr == 0 {
                    return Err(MemoryError::InvalidAddress(
                        format!("Null pointer at chain level {}", i)
                    ));
                }
            }
        }

        unreachable!()
    }

    /// Write a value to a memory address (convenience method)
    ///
    /// # Example
    /// ```no_run
    /// extractor.write_memory_t::<f32>(player_ptr + 0x34, 999.0)?;
    /// ```
    pub fn write_memory_t<T: Copy>(&self, address: usize, value: T) -> Result<(), MemoryError> {
        write_memory_t(self.hook.handle.0, address, value)
    }

    /// Get the storage base address (for advanced usage)
    pub fn get_storage_address(&self) -> usize {
        self.storage_address
    }

    /// Get the storage address for a specific register
    ///
    /// This is useful for creating "artificial pointers" that can be used
    /// with MemoryLock's dynamic address resolution.
    ///
    /// # Example
    /// ```no_run
    /// // Extract RDI (skill object base address)
    /// let rdi_addr = extractor.get_register_storage_address(Register::RDI);
    /// 
    /// // Use as base address for MemoryLock with offset [+0x98]
    /// let skill_count_addr = MemoryAddress::new(rdi_addr, vec![0x98], pid);
    /// ```
    pub fn get_register_storage_address(&self, reg: Register) -> usize {
        let offset = reg.offset(self.is_x64);
        self.storage_address + offset
    }

    /// Check if the extractor is currently installed
    pub fn is_installed(&self) -> bool {
        self.is_installed
    }

    /// Get the list of extracted registers
    pub fn get_extracted_registers(&self) -> &[Register] {
        &self.extracted_regs
    }
}

impl Drop for RegisterExtractor {
    fn drop(&mut self) {
        // Automatically uninstall if still installed
        if self.is_installed {
            let _ = self.uninstall();
        }

        // Free storage memory
        if self.storage_address != 0 {
            unsafe {
                let _ = VirtualFreeEx(
                    self.hook.handle.0,
                    self.storage_address as *mut _,
                    0,
                    MEM_RELEASE,
                );
            }
        }
    }
}
