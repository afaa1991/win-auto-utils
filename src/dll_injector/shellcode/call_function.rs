//! Shellcode for calling remote functions with parameters
//!
//! This module generates architecture-specific shellcode to call arbitrary
//! functions in remote processes, supporting both parameterized and
//! parameterless function calls.

/// Generate x64 shellcode to call a function with raw byte parameter
///
/// This function creates machine code that will execute in a remote x64 process
/// to call a target function with optional parameter data. The parameter is passed
/// according to the Windows x64 calling convention (first integer/pointer in RCX).
///
/// # Arguments
/// * `function_address` - Virtual address of the function to call in target process
/// * `param_addr` - Address of parameter data in target process memory (0 if no param)
/// * `param_size` - Size of parameter data in bytes (0 if no param)
///
/// # Returns
/// Vector of bytes containing executable x64 shellcode
///
/// # Parameter Handling
/// - **4 bytes** (i32/f32): Value loaded into ECX, zero-extended to RCX
/// - **8 bytes** (i64/pointer): Value loaded directly into RCX
/// - **Other sizes**: Pointer to data passed in RCX
/// - **No parameter**: RCX zeroed out
///
/// # Generated Assembly (Conceptual for 4-byte param)
/// ```asm
/// push rax, rbx                   ; Save registers
/// sub rsp, 0x28                   ; Align stack
/// mov rax, param_addr             ; Load parameter address
/// mov ecx, [rax]                  ; Load 4-byte value
/// movsxd rcx, ecx                 ; Zero-extend to RCX
/// mov rax, function_address       ; Load target function
/// call rax                        ; Call function
/// add rsp, 0x28                   ; Restore stack
/// pop rbx, rax                    ; Restore registers
/// ret                             ; Return (result in RAX)
/// ```
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. All addresses must be valid in the target process context
/// 4. Function signature should match parameter passing convention
#[cfg(target_arch = "x86_64")]
pub fn generate_call_function_shellcode_x64(
    function_address: usize,
    param_addr: usize,
    param_size: usize,
) -> Vec<u8> {
    let mut code = Vec::new();
    
    // Save registers
    code.extend_from_slice(&[0x50]); // push rax
    code.extend_from_slice(&[0x53]); // push rbx
    
    // Align stack to 16 bytes
    code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28
    
    if param_size > 0 && param_addr != 0 {
        // Load parameter into appropriate register based on size
        if param_size == 4 {
            // f32 or i32 - load into ECX (first integer arg) or XMM0 (first float arg)
            // For simplicity, we'll use integer convention (RCX)
            code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
            code.extend_from_slice(&(param_addr as u64).to_le_bytes());
            code.extend_from_slice(&[0x8B, 0x08]); // mov ecx, [rax]
            
            // Zero extend to RCX
            code.extend_from_slice(&[0x48, 0x63, 0xC9]); // movsxd rcx, ecx
        } else if param_size == 8 {
            // i64 or pointer - load into RCX
            code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
            code.extend_from_slice(&(param_addr as u64).to_le_bytes());
            code.extend_from_slice(&[0x48, 0x8B, 0x08]); // mov rcx, [rax]
        } else {
            // For other sizes, pass pointer to data in RCX
            code.extend_from_slice(&[0x48, 0xB9]); // mov rcx, imm64
            code.extend_from_slice(&(param_addr as u64).to_le_bytes());
        }
    } else {
        // No parameter - zero out RCX
        code.extend_from_slice(&[0x48, 0x31, 0xC9]); // xor rcx, rcx
    }
    
    // Call the target function
    code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
    code.extend_from_slice(&(function_address as u64).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call rax
    
    // Result is in RAX (will be thread exit code)
    
    // Restore stack
    code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 0x28
    
    // Restore registers
    code.extend_from_slice(&[0x5B]); // pop rbx
    code.extend_from_slice(&[0x58]); // pop rax
    
    // Return
    code.extend_from_slice(&[0xC3]); // ret
    
    code
}

/// Generate x86 shellcode to call a function with raw byte parameter
///
/// This function creates machine code that will execute in a remote x86 process
/// to call a target function with optional parameter data. Parameters are pushed
/// onto the stack according to the __stdcall calling convention.
///
/// # Arguments
/// * `function_address` - Virtual address of the function to call in target process
/// * `param_addr` - Address of parameter data in target process memory (0 if no param)
/// * `param_size` - Size of parameter data in bytes (0 if no param)
///
/// # Returns
/// Vector of bytes containing executable x86 shellcode
///
/// # Parameter Handling
/// - **≤4 bytes**: Value loaded into EAX, then pushed onto stack
/// - **>4 bytes**: Pointer to data pushed onto stack
/// - **No parameter**: Nothing pushed
///
/// # Generated Assembly (Conceptual for ≤4-byte param)
/// ```asm
/// mov eax, param_addr             ; Load parameter address
/// mov eax, [eax]                  ; Load value
/// push eax                        ; Push parameter
/// mov eax, function_address       ; Load target function
/// call eax                        ; Call function (__stdcall, callee cleans stack)
/// ret                             ; Return (result in EAX)
/// ```
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. All addresses must be valid in the target process context
/// 4. Function should use __stdcall calling convention
#[cfg(target_arch = "x86")]
pub fn generate_call_function_shellcode_x86(
    function_address: usize,
    param_addr: usize,
    param_size: usize,
) -> Vec<u8> {
    let mut code = Vec::new();
    
    if param_size > 0 && param_addr != 0 {
        // Push parameter onto stack
        if param_size <= 4 {
            // Load value and push
            code.extend_from_slice(&[0xB8]); // mov eax, imm32
            code.extend_from_slice(&(param_addr as u32).to_le_bytes());
            code.extend_from_slice(&[0x8B, 0x00]); // mov eax, [eax]
            code.extend_from_slice(&[0x50]); // push eax
        } else {
            // For larger params, push pointer
            code.extend_from_slice(&[0x68]); // push imm32
            code.extend_from_slice(&(param_addr as u32).to_le_bytes());
        }
    }
    
    // Call the target function
    code.extend_from_slice(&[0xB8]); // mov eax, imm32
    code.extend_from_slice(&(function_address as u32).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call eax
    
    // Clean up stack if we pushed a parameter (__cdecl)
    // For __stdcall, callee cleans up, so no adjustment needed
    // We'll assume __stdcall for simplicity
    
    // Result is in EAX (will be thread exit code)
    
    // Return
    code.extend_from_slice(&[0xC3]); // ret
    
    code
}

/// Generate x64 shellcode to call a function with NO parameters
///
/// This is an optimized version for calling parameterless functions in remote
/// x64 processes. It generates smaller and faster shellcode than the generic
/// version by skipping parameter handling logic.
///
/// # Arguments
/// * `function_address` - Virtual address of the function to call in target process
///
/// # Returns
/// Vector of bytes containing executable x64 shellcode (typically ~40 bytes)
///
/// # Generated Assembly (Conceptual)
/// ```asm
/// push rax, rbx                   ; Save registers
/// sub rsp, 0x28                   ; Align stack (required by Windows x64 ABI)
/// mov rax, function_address       ; Load target function
/// call rax                        ; Call function
/// add rsp, 0x28                   ; Restore stack
/// pop rbx, rax                    ; Restore registers
/// ret                             ; Return (result in RAX)
/// ```
///
/// # Performance Advantage
/// - Smaller shellcode: ~40 bytes vs ~80 bytes (with params)
/// - Faster execution: No parameter allocation/cleanup overhead
/// - Cleaner intent: Explicitly shows no parameters expected
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. Target function signature should be compatible (no parameters)
#[cfg(target_arch = "x86_64")]
pub fn generate_call_function_no_params_shellcode_x64(
    function_address: usize,
) -> Vec<u8> {
    let mut code = Vec::new();
    
    // Save registers (minimal set for simple call)
    code.extend_from_slice(&[0x50]); // push rax
    code.extend_from_slice(&[0x53]); // push rbx
    
    // Align stack to 16 bytes (required by Windows x64 ABI)
    code.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28
    
    // Call the target function directly
    code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
    code.extend_from_slice(&(function_address as u64).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call rax
    
    // Result is in RAX (will be thread exit code)
    
    // Restore stack and registers
    code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 0x28
    code.extend_from_slice(&[0x5B]); // pop rbx
    code.extend_from_slice(&[0x58]); // pop rax
    
    // Return
    code.extend_from_slice(&[0xC3]); // ret
    
    code
}

/// Generate x86 shellcode to call a function with NO parameters
///
/// This is an optimized version for calling parameterless functions in remote
/// x86 processes. It generates minimal shellcode by directly calling the target
/// function without any parameter setup.
///
/// # Arguments
/// * `function_address` - Virtual address of the function to call in target process
///
/// # Returns
/// Vector of bytes containing executable x86 shellcode (typically ~7 bytes)
///
/// # Generated Assembly (Conceptual)
/// ```asm
/// mov eax, function_address       ; Load target function
/// call eax                        ; Call function
/// ret                             ; Return (result in EAX)
/// ```
///
/// # Performance Advantage
/// - Minimal shellcode: Only ~7 bytes
/// - Fastest execution: Direct call with no overhead
/// - Simplest logic: No stack manipulation needed
///
/// # Safety
/// The generated shellcode must be:
/// 1. Written to executable memory in the target process
/// 2. Executed via CreateRemoteThread or similar mechanism
/// 3. Target function signature should be compatible (no parameters)
#[cfg(target_arch = "x86")]
pub fn generate_call_function_no_params_shellcode_x86(
    function_address: usize,
) -> Vec<u8> {
    let mut code = Vec::new();
    
    // Call the target function directly (no parameters to push)
    code.extend_from_slice(&[0xB8]); // mov eax, imm32
    code.extend_from_slice(&(function_address as u32).to_le_bytes());
    code.extend_from_slice(&[0xFF, 0xD0]); // call eax
    
    // Result is in EAX (will be thread exit code)
    
    // Return
    code.extend_from_slice(&[0xC3]); // ret
    
    code
}
