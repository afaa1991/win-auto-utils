//! Remote Function Calling APIs
//!
//! This module provides functions to call exported functions in remote processes
//! after DLL injection. It supports passing raw byte parameters and retrieving
//! return values through thread exit codes.
//!
//! # Usage Pattern
//! 1. Inject a DLL using [`inject_dll`](super::inject_dll)
//! 2. Get function address using [`get_exported_function_address`]
//! 3. Call the function using [`call_function_with_raw_bytes`] or [`call_function_no_params`]
//!
//! # Example: Calling a Function with Parameters
//! ```no_run
//! use win_auto_utils::dll_injector::{
//!     inject_dll,
//!     get_exported_function_address,
//!     call_function_with_raw_bytes
//! };
//!
//! // Step 1: Inject DLL
//! inject_dll(12345, "C:\\mods\\game_mod.dll")?;
//!
//! // Step 2: Get function address
//! let func_addr = get_exported_function_address(12345, "game_mod.dll", "set_health")?;
//!
//! // Step 3: Call with i32 parameter (999 health)
//! let health: i32 = 999;
//! let result = call_function_with_raw_bytes(12345, func_addr, Some(&health.to_ne_bytes()))?;
//! println!("Function returned: {}", result);
//! # Ok::<(), win_auto_utils::dll_injector::DllInjectorError>(())
//! ```
//!
//! # Example: Calling a Parameterless Function
//! ```no_run
//! use win_auto_utils::dll_injector::{
//!     get_exported_function_address,
//!     call_function_no_params
//! };
//!
//! let func_addr = get_exported_function_address(12345, "game_mod.dll", "reset_stats")?;
//! let result = call_function_no_params(12345, func_addr)?;
//! println!("Reset stats returned: {}", result);
//! # Ok::<(), win_auto_utils::dll_injector::DllInjectorError>(())
//! ```
//!
//! # Parameter Passing
//! - **Basic types**: Use `.to_ne_bytes()` to convert (i8/i16/i32/i64/u8/u16/u32/u64/f32/f64)
//! - **Structs**: Use `#[repr(C)]` and convert to bytes via `std::slice::from_raw_parts`
//! - **Strings/Pointers**: Must allocate memory in target process first
//!
//! # Return Value Limitations
//! Only the thread exit code (u32) can be retrieved directly. For complex return values,
//! use shared memory or modify target process memory instead.

use std::ffi::c_void;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::CloseHandle,
        System::{
            Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE},
            Threading::{CreateRemoteThread, GetExitCodeThread, WaitForSingleObject},
        },
    },
};

use super::helpers::{get_module_base_address, open_process_full};
use super::shellcode;
use super::DllInjectorError;

/// Get the virtual address of an exported function in a remote DLL
///
/// This function uses shellcode to call `GetProcAddress` in the target process,
/// allowing you to resolve function addresses without hardcoding them.
///
/// # Arguments
/// - `pid`: Target process ID
/// - `module_name`: Name of the loaded module (e.g., "game_mod.dll")
/// - `function_name`: Name of the exported function (e.g., "set_health")
///
/// # Returns
/// - `Ok(address)` - The virtual address of the function in target process
/// - `Err(DllInjectorError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::dll_injector::{inject_dll, get_exported_function_address};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // First inject the DLL
///     inject_dll(12345, "C:\\mods\\game_mod.dll")?;
///
///     // Then get the function address
///     let addr = get_exported_function_address(12345, "game_mod.dll", "set_health")?;
///     println!("Function address: 0x{:X}", addr);
///     Ok(())
/// }
/// ```

/// # Performance
/// Typical execution time: 10-50ms (depends on target process responsiveness)
pub fn get_exported_function_address(
    pid: u32,
    module_name: &str,
    function_name: &str,
) -> Result<usize, DllInjectorError> {
    unsafe {
        let handle = open_process_full(pid)?;
        let module_addr = get_module_base_address(pid, module_name)?;

        // Get GetProcAddress from kernel32
        let h_kernel32 = GetModuleHandleA(PCSTR::from_raw("kernel32.dll\0".as_ptr()))
            .map_err(|e| DllInjectorError::GetProcAddressFailed(e.to_string()))?;

        let get_proc_address_fn = match GetProcAddress(h_kernel32, PCSTR::from_raw("GetProcAddress\0".as_ptr())) {
            Some(addr) => addr,
            None => {
                let _ = CloseHandle(handle);
                return Err(DllInjectorError::GetProcAddressFailed(
                    "GetProcAddress not found".to_string()
                ));
            }
        };

        // Allocate memory for function name
        let func_name_cstr = format!("{}\0", function_name);
        let func_name_bytes = func_name_cstr.as_bytes();
        let func_name_addr = VirtualAllocEx(handle, None, func_name_bytes.len(), MEM_COMMIT, PAGE_READWRITE);

        if func_name_addr.is_null() {
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        let mut bytes_written = 0;
        if WriteProcessMemory(handle, func_name_addr, func_name_bytes.as_ptr() as *const c_void, func_name_bytes.len(), Some(&mut bytes_written)).is_err() {
            let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::WriteFailed("Failed to write function name".to_string()));
        }

        // Allocate memory for result
        let result_size = std::mem::size_of::<usize>();
        let result_addr = VirtualAllocEx(handle, None, result_size, MEM_COMMIT, PAGE_READWRITE);

        if result_addr.is_null() {
            let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        // Generate shellcode based on architecture
        #[cfg(target_arch = "x86_64")]
        let shellcode = shellcode::get_proc_addr::generate_get_proc_addr_shellcode_x64(
            get_proc_address_fn as usize,
            module_addr,
            func_name_addr as usize,
            result_addr as usize,
        );

        #[cfg(target_arch = "x86")]
        let shellcode = shellcode::get_proc_addr::generate_get_proc_addr_shellcode_x86(
            get_proc_address_fn as usize,
            module_addr,
            func_name_addr as usize,
            result_addr as usize,
        );

        // Allocate and write shellcode
        let shellcode_addr = VirtualAllocEx(handle, None, shellcode.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);
        if shellcode_addr.is_null() {
            let _ = VirtualFreeEx(handle, result_addr, 0, MEM_RELEASE);
            let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        let mut bytes_written = 0;
        if WriteProcessMemory(handle, shellcode_addr, shellcode.as_ptr() as *const c_void, shellcode.len(), Some(&mut bytes_written)).is_err() {
            let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
            let _ = VirtualFreeEx(handle, result_addr, 0, MEM_RELEASE);
            let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::WriteFailed("Failed to write shellcode".to_string()));
        }

        // Execute shellcode
        let thread_result = CreateRemoteThread(handle, None, 0, Some(std::mem::transmute(shellcode_addr)), None, 0, None);

        match thread_result {
            Ok(thread) => {
                WaitForSingleObject(thread, u32::MAX);

                // Read result
                let mut func_address: usize = 0;
                let mut bytes_read = 0;
                let read_result = ReadProcessMemory(
                    handle,
                    result_addr,
                    &mut func_address as *mut usize as *mut c_void,
                    result_size,
                    Some(&mut bytes_read),
                );

                // Cleanup
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                let _ = VirtualFreeEx(handle, result_addr, 0, MEM_RELEASE);
                let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(thread);
                let _ = CloseHandle(handle);

                if read_result.is_err() || bytes_read != result_size {
                    return Err(DllInjectorError::Other("Failed to read result".to_string()));
                }

                if func_address == 0 {
                    return Err(DllInjectorError::GetProcAddressFailed(format!(
                        "Function '{}' not found in '{}'", function_name, module_name
                    )));
                }

                Ok(func_address)
            }
            Err(e) => {
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                let _ = VirtualFreeEx(handle, result_addr, 0, MEM_RELEASE);
                let _ = VirtualFreeEx(handle, func_name_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(handle);
                Err(DllInjectorError::CreateThreadFailed(e.to_string()))
            }
        }
    }
}

/// Call a remote function with raw byte parameters
///
/// This is the most flexible API for calling remote functions. It allows you to pass
/// arbitrary data by serializing it to bytes. The parameter is passed as the first
/// argument according to the platform's calling convention (RCX on x64, stack on x86).
///
/// # Arguments
/// - `pid`: Target process ID
/// - `function_address`: Virtual address of the function to call
/// - `param_data`: Optional raw bytes to pass as parameter
///
/// # Returns
/// - `Ok(exit_code)` - The thread exit code (function return value)
/// - `Err(DllInjectorError)` on failure
///
/// # Example: Passing i32 Parameter
/// ```no_run
/// use win_auto_utils::dll_injector::{get_exported_function_address, call_function_with_raw_bytes};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let func_addr = get_exported_function_address(12345, "game_mod.dll", "set_health")?;
///
///     // Pass i32 value 999
///     let health: i32 = 999;
///     let result = call_function_with_raw_bytes(12345, func_addr, Some(&health.to_ne_bytes()))?;
///     println!("Result: {}", result);
///     Ok(())
/// }
/// ```
///
/// # Example: Passing f32 Parameter
/// ```no_run
/// use win_auto_utils::dll_injector::{get_exported_function_address, call_function_with_raw_bytes};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let func_addr = get_exported_function_address(12345, "game_mod.dll", "set_speed")?;
///
///     // Pass f32 value 2.5
///     let speed: f32 = 2.5;
///     let result = call_function_with_raw_bytes(12345, func_addr, Some(&speed.to_ne_bytes()))?;
///     Ok(())
/// }
/// ```
///
/// # Example: Passing Struct Parameter
/// ```no_run
/// use win_auto_utils::dll_injector::{get_exported_function_address, call_function_with_raw_bytes};
///
/// #[repr(C)]
/// struct PlayerStats {
///     health: i32,
///     mana: i32,
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let func_addr = get_exported_function_address(12345, "game_mod.dll", "update_stats")?;
///
///     let stats = PlayerStats { health: 999, mana: 500 };
///     let bytes = unsafe {
///         std::slice::from_raw_parts(
///             &stats as *const PlayerStats as *const u8,
///             std::mem::size_of::<PlayerStats>()
///         )
///     };
///     let result = call_function_with_raw_bytes(12345, func_addr, Some(bytes))?;
///     Ok(())
/// }
/// ```

/// # Parameter Size Handling
/// - **4 bytes** (i32/f32): Loaded into ECX/RCX register
/// - **8 bytes** (i64/pointer): Loaded into RCX register
/// - **Other sizes**: Pointer to data is passed in RCX
///
/// # Performance
/// Typical execution time: 5-30ms per call
pub fn call_function_with_raw_bytes(
    pid: u32,
    function_address: usize,
    param_data: Option<&[u8]>,
) -> Result<u32, DllInjectorError> {
    unsafe {
        let handle = open_process_full(pid)?;

        // Allocate memory for parameter if provided
        let param_addr = if let Some(data) = param_data {
            let param_size = data.len();
            let param_addr_raw = VirtualAllocEx(handle, None, param_size, MEM_COMMIT, PAGE_READWRITE);

            if param_addr_raw.is_null() {
                let _ = CloseHandle(handle);
                return Err(DllInjectorError::AllocationFailed);
            }

            let mut bytes_written = 0;
            if WriteProcessMemory(handle, param_addr_raw, data.as_ptr() as *const c_void, param_size, Some(&mut bytes_written)).is_err() {
                let _ = VirtualFreeEx(handle, param_addr_raw, 0, MEM_RELEASE);
                let _ = CloseHandle(handle);
                return Err(DllInjectorError::WriteFailed("Failed to write parameter".to_string()));
            }

            Some((param_addr_raw as usize, param_size))
        } else {
            None
        };

        // Generate shellcode
        let param_addr_val = param_addr.as_ref().map(|(addr, _)| *addr).unwrap_or(0);
        let param_size_val = param_addr.as_ref().map(|(_, size)| *size).unwrap_or(0);

        #[cfg(target_arch = "x86_64")]
        let shellcode = shellcode::call_function::generate_call_function_shellcode_x64(
            function_address,
            param_addr_val,
            param_size_val,
        );

        #[cfg(target_arch = "x86")]
        let shellcode = shellcode::call_function::generate_call_function_shellcode_x86(
            function_address,
            param_addr_val,
            param_size_val,
        );

        // Allocate and write shellcode
        let shellcode_addr = VirtualAllocEx(handle, None, shellcode.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);
        if shellcode_addr.is_null() {
            if let Some((addr, _)) = param_addr {
                let _ = VirtualFreeEx(handle, addr as *mut c_void, 0, MEM_RELEASE);
            }
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        let mut bytes_written = 0;
        if WriteProcessMemory(handle, shellcode_addr, shellcode.as_ptr() as *const c_void, shellcode.len(), Some(&mut bytes_written)).is_err() {
            let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
            if let Some((addr, _)) = param_addr {
                let _ = VirtualFreeEx(handle, addr as *mut c_void, 0, MEM_RELEASE);
            }
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::WriteFailed("Failed to write shellcode".to_string()));
        }

        // Execute
        let thread_result = CreateRemoteThread(handle, None, 0, Some(std::mem::transmute(shellcode_addr)), None, 0, None);

        match thread_result {
            Ok(thread) => {
                WaitForSingleObject(thread, u32::MAX);

                let mut exit_code: u32 = 0;
                let _ = GetExitCodeThread(thread, &mut exit_code);

                // Cleanup
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                if let Some((addr, _)) = param_addr {
                    let _ = VirtualFreeEx(handle, addr as *mut c_void, 0, MEM_RELEASE);
                }
                let _ = CloseHandle(thread);
                let _ = CloseHandle(handle);

                Ok(exit_code)
            }
            Err(e) => {
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                if let Some((addr, _)) = param_addr {
                    let _ = VirtualFreeEx(handle, addr as *mut c_void, 0, MEM_RELEASE);
                }
                let _ = CloseHandle(handle);
                Err(DllInjectorError::CreateThreadFailed(e.to_string()))
            }
        }
    }
}

/// Call a remote function with NO parameters (optimized version)
///
/// This is an optimized version for calling functions that take no parameters.
/// It generates simpler and more efficient shellcode than passing `None` to
/// [`call_function_with_raw_bytes`].
///
/// # Arguments
/// - `pid`: Target process ID
/// - `function_address`: Virtual address of the function to call
///
/// # Returns
/// - `Ok(exit_code)` - The thread exit code (function return value)
/// - `Err(DllInjectorError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::dll_injector::{get_exported_function_address, call_function_no_params};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let func_addr = get_exported_function_address(12345, "game_mod.dll", "reset_stats")?;
///     let result = call_function_no_params(12345, func_addr)?;
///     println!("Reset stats returned: {}", result);
///     Ok(())
/// }
/// ```

/// # When to Use
/// Use this function when the target function signature is `void function()` or
/// returns a value but takes no parameters. For functions with parameters, use
/// [`call_function_with_raw_bytes`] instead.
///
/// # Performance Advantage
/// - Smaller shellcode (~40 bytes vs ~80 bytes)
/// - Faster execution (no parameter allocation/cleanup)
/// - Cleaner intent (explicitly shows no parameters)
///
/// Typical execution time: 3-15ms per call
pub fn call_function_no_params(
    pid: u32,
    function_address: usize,
) -> Result<u32, DllInjectorError> {
    unsafe {
        let handle = open_process_full(pid)?;

        // Generate optimized shellcode
        #[cfg(target_arch = "x86_64")]
        let shellcode = shellcode::call_function::generate_call_function_no_params_shellcode_x64(function_address);

        #[cfg(target_arch = "x86")]
        let shellcode = shellcode::call_function::generate_call_function_no_params_shellcode_x86(function_address);

        // Allocate and write shellcode
        let shellcode_addr = VirtualAllocEx(handle, None, shellcode.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE);
        if shellcode_addr.is_null() {
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        let mut bytes_written = 0;
        if WriteProcessMemory(handle, shellcode_addr, shellcode.as_ptr() as *const c_void, shellcode.len(), Some(&mut bytes_written)).is_err() {
            let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::WriteFailed("Failed to write shellcode".to_string()));
        }

        // Execute
        let thread_result = CreateRemoteThread(handle, None, 0, Some(std::mem::transmute(shellcode_addr)), None, 0, None);

        match thread_result {
            Ok(thread) => {
                WaitForSingleObject(thread, u32::MAX);

                let mut exit_code: u32 = 0;
                let _ = GetExitCodeThread(thread, &mut exit_code);

                // Cleanup
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(thread);
                let _ = CloseHandle(handle);

                Ok(exit_code)
            }
            Err(e) => {
                let _ = VirtualFreeEx(handle, shellcode_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(handle);
                Err(DllInjectorError::CreateThreadFailed(e.to_string()))
            }
        }
    }
}
