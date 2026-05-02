//! Shellcode for calling GetProcAddress in remote process
//!
//! This module generates architecture-specific shellcode to resolve exported
//! function addresses by calling GetProcAddress in the target process.

/// Generate x64 shellcode to call GetProcAddress and store result
///
/// This function creates machine code that will execute in a remote x64 process
/// to call GetProcAddress(module_handle, func_name) and store the result at
/// a specified memory address.
///
/// # Arguments
/// * `get_proc_address` - Address of GetProcAddress function in target process
/// * `module_handle` - Base address (HMODULE) of the target module
/// * `func_name_addr` - Address of null-terminated function name string in target memory
/// * `result_addr` - Address where the result (function pointer) will be stored
///
/// # Returns
/// Vector of bytes containing executable x64 shellcode
///
/// # Generated Assembly (Conceptual)
/// ```asm
/// push rax, rbx, rcx, rdx        ; Save registers
/// sub rsp, 0x28                   ; Align stack
/// mov rcx, module_handle          ; First parameter
/// mov rdx, func_name_addr         ; Second parameter
/// mov rax, get_proc_address       ; Load GetProcAddress address
/// call rax                        ; Call GetProcAddress
/// mov rbx, result_addr            ; Load result address
/// mov [rbx], rax                  ; Store result
/// add rsp, 0x28                   ; Restore stack
/// pop rdx, rcx, rbx, rax          ; Restore registers
/// ret                             ; Return
/// ```
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. All addresses must be valid in the target process context
#[cfg(target_arch = "x86_64")]
pub fn generate_get_proc_addr_shellcode_x64(
    get_proc_address: usize,
    module_handle: usize,
    func_name_addr: usize,
    result_addr: usize,
) -> Vec<u8> {
    let mut code = Vec::new();

    // Save registers
    code.extend_from_slice(&[0x50]); // push rax
    code.extend_from_slice(&[0x53]); // push rbx
    code.extend_from_slice(&[0x51]); // push rcx
    code.extend_from_slice(&[0x52]); // push rdx

    // Align stack to 16 bytes
    code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28

    // Call GetProcAddress(module_handle, func_name_addr)
    // RCX = module_handle, RDX = func_name_addr
    code.extend_from_slice(&[0x48, 0xB9]); // mov rcx, imm64
    code.extend_from_slice(&(module_handle as u64).to_le_bytes());

    code.extend_from_slice(&[0x48, 0xBA]); // mov rdx, imm64
    code.extend_from_slice(&(func_name_addr as u64).to_le_bytes());

    // Call GetProcAddress
    code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
    code.extend_from_slice(&(get_proc_address as u64).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call rax

    // Store result (RAX) to result_addr
    code.extend_from_slice(&[0x48, 0xBB]); // mov rbx, imm64
    code.extend_from_slice(&(result_addr as u64).to_le_bytes());
    code.extend_from_slice(&[0x48, 0x89, 0x03]); // mov [rbx], rax

    // Restore stack
    code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 0x28

    // Restore registers
    code.extend_from_slice(&[0x5A]); // pop rdx
    code.extend_from_slice(&[0x59]); // pop rcx
    code.extend_from_slice(&[0x5B]); // pop rbx
    code.extend_from_slice(&[0x58]); // pop rax

    // Return
    code.extend_from_slice(&[0xC3]); // ret

    code
}

/// Generate x86 shellcode to call GetProcAddress and store result
///
/// This function creates machine code that will execute in a remote x86 process
/// to call GetProcAddress(module_handle, func_name) and store the result at
/// a specified memory address.
///
/// # Arguments
/// * `get_proc_address` - Address of GetProcAddress function in target process
/// * `module_handle` - Base address (HMODULE) of the target module
/// * `func_name_addr` - Address of null-terminated function name string in target memory
/// * `result_addr` - Address where the result (function pointer) will be stored
///
/// # Returns
/// Vector of bytes containing executable x86 shellcode
///
/// # Generated Assembly (Conceptual)
/// ```asm
/// push func_name_addr             ; Second parameter (right-to-left for __stdcall)
/// push module_handle              ; First parameter
/// mov eax, get_proc_address       ; Load GetProcAddress address
/// call eax                        ; Call GetProcAddress
/// mov ebx, result_addr            ; Load result address
/// mov [ebx], eax                  ; Store result
/// ret                             ; Return
/// ```
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. All addresses must be valid in the target process context
#[cfg(target_arch = "x86")]
pub fn generate_get_proc_addr_shellcode_x86(
    get_proc_address: usize,
    module_handle: usize,
    func_name_addr: usize,
    result_addr: usize,
) -> Vec<u8> {
    let mut code = Vec::new();

    // Push parameters for GetProcAddress(module_handle, func_name_addr)
    // __stdcall: right-to-left
    code.extend_from_slice(&[0x68]); // push imm32
    code.extend_from_slice(&(func_name_addr as u32).to_le_bytes());

    code.extend_from_slice(&[0x68]); // push imm32
    code.extend_from_slice(&(module_handle as u32).to_le_bytes());

    // Call GetProcAddress
    code.extend_from_slice(&[0xB8]); // mov eax, imm32
    code.extend_from_slice(&(get_proc_address as u32).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call eax

    // Store result (EAX) to result_addr
    code.extend_from_slice(&[0xBB]); // mov ebx, imm32
    code.extend_from_slice(&(result_addr as u32).to_le_bytes());
    code.extend_from_slice(&[0x89, 0x03]); // mov [ebx], eax

    // Return
    code.extend_from_slice(&[0xC3]); // ret

    code
}
