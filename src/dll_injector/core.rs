//! Core DLL Injection and Unloading Functions

use std::ffi::{c_void, OsStr};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::CloseHandle,
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE},
            Threading::{CreateRemoteThread, IsWow64Process, OpenProcess, WaitForSingleObject, PROCESS_QUERY_INFORMATION},
        },
    },
};

use super::helpers::open_process_full;
use super::DllInjectorError;

/// Detect target process architecture
/// Returns true if the process is x86 (WOW64), false if x64
fn get_process_architecture(pid: u32) -> Result<bool, DllInjectorError> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)
            .map_err(|e| DllInjectorError::OpenProcessFailed(e.to_string()))?;
        
        if handle.is_invalid() {
            return Err(DllInjectorError::OpenProcessFailed(
                format!("Invalid handle for process {}", pid)
            ));
        }
        
        // windows_core::BOOL is a newtype wrapper around i32
        // We can safely cast *mut i32 to *mut BOOL since they have the same layout
        let mut is_wow64: i32 = 0;
        IsWow64Process(handle, &mut is_wow64 as *mut i32 as *mut _)
            .map_err(|e| DllInjectorError::Other(format!("IsWow64Process failed: {}", e)))?;
        
        let _ = CloseHandle(handle);
        
        // is_wow64 != 0 → 32-bit process on 64-bit Windows
        Ok(is_wow64 != 0)
    }
}

/// Detect DLL architecture by parsing PE header
/// Returns true if x64, false if x86
fn get_dll_architecture(dll_path: &str) -> Result<bool, DllInjectorError> {
    let mut file = File::open(dll_path)
        .map_err(|e| DllInjectorError::Other(format!("Cannot open DLL: {}", e)))?;
    
    // Read DOS header (first 64 bytes)
    let mut dos_header = [0u8; 64];
    file.read_exact(&mut dos_header)
        .map_err(|e| DllInjectorError::Other(format!("Cannot read DOS header: {}", e)))?;
    
    // Check MZ signature
    if &dos_header[0..2] != b"MZ" {
        return Err(DllInjectorError::Other("Invalid PE file (missing MZ signature)".to_string()));
    }
    
    // Get PE header offset from DOS header (at offset 0x3C)
    let pe_offset = u32::from_le_bytes([
        dos_header[0x3C],
        dos_header[0x3D],
        dos_header[0x3E],
        dos_header[0x3F],
    ]) as u64;
    
    // Seek to PE header
    file.seek(SeekFrom::Start(pe_offset))
        .map_err(|e| DllInjectorError::Other(format!("Cannot seek to PE header: {}", e)))?;
    
    // Read PE signature and COFF header (24 bytes)
    let mut pe_header = [0u8; 24];
    file.read_exact(&mut pe_header)
        .map_err(|e| DllInjectorError::Other(format!("Cannot read PE header: {}", e)))?;
    
    // Check PE signature ("PE\0\0")
    if &pe_header[0..4] != b"PE\0\0" {
        return Err(DllInjectorError::Other("Invalid PE file (missing PE signature)".to_string()));
    }
    
    // Extract machine type from COFF header (offset 4-5 in PE header)
    let machine = u16::from_le_bytes([pe_header[4], pe_header[5]]);
    
    // Machine types: 0x14c = x86, 0x8664 = x64
    match machine {
        0x14c => Ok(false),  // x86
        0x8664 => Ok(true),  // x64
        _ => Err(DllInjectorError::Other(format!(
            "Unsupported DLL architecture (machine type: 0x{:04X})",
            machine
        ))),
    }
}

/// Inject a DLL into a target process
///
/// # Arguments
/// - `pid`: Target process ID
/// - `dll_path`: Absolute path to the DLL file
///
/// # Returns
/// - `Ok(())` on successful injection
/// - `Err(DllInjectorError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::dll_injector::inject_dll;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     inject_dll(12345, "C:\\mods\\game_mod.dll")?;
///     Ok(())
/// }
/// ```

///
/// # Architecture Notes
/// The DLL architecture MUST match the target process architecture:
/// - For 32-bit target: Use 32-bit DLL
/// - For 64-bit target: Use 64-bit DLL
pub fn inject_dll(pid: u32, dll_path: &str) -> Result<(), DllInjectorError> {
    // Validate inputs
    if pid == 0 {
        return Err(DllInjectorError::Other("Invalid PID: cannot be 0".to_string()));
    }

    if !Path::new(dll_path).exists() {
        return Err(DllInjectorError::Other(format!("DLL file not found: {}", dll_path)));
    }

    // ✅ Architecture validation
    let target_is_wow64 = get_process_architecture(pid)?;
    let dll_is_x64 = get_dll_architecture(dll_path)?;
    
    if target_is_wow64 && dll_is_x64 {
        return Err(DllInjectorError::ArchitectureMismatch(
            format!("Target process is x86 (WOW64) but DLL is x64. Please use an x86 DLL instead.")
        ));
    }
    
    if !target_is_wow64 && !dll_is_x64 {
        return Err(DllInjectorError::ArchitectureMismatch(
            format!("Target process is x64 but DLL is x86. Please use an x64 DLL instead.")
        ));
    }

    // ✅ Check if module is already loaded
    let module_name = Path::new(dll_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.dll");
    
    if crate::dll_injector::helpers::is_module_loaded(pid, module_name) {
        return Err(DllInjectorError::AlreadyLoaded(format!(
            "Module '{}' is already loaded in process {}. Use unload_dll() first if you want to reload.",
            module_name, pid
        )));
    }

    unsafe {
        // Open target process with required permissions
        let handle = open_process_full(pid)?;

        // Convert DLL path to UTF-16 (wide string)
        let dll_path_wide: Vec<u16> = OsStr::new(dll_path)
            .encode_wide()
            .chain(iter::once(0))
            .collect();

        let path_size = dll_path_wide.len() * std::mem::size_of::<u16>();

        // Allocate memory in target process
        let alloc_addr = VirtualAllocEx(
            handle,
            None,
            path_size,
            MEM_COMMIT,
            PAGE_READWRITE,
        );

        if alloc_addr.is_null() {
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::AllocationFailed);
        }

        // Write DLL path to allocated memory
        let mut bytes_written = 0;
        let write_result = WriteProcessMemory(
            handle,
            alloc_addr,
            dll_path_wide.as_ptr() as *const c_void,
            path_size,
            Some(&mut bytes_written),
        );

        if write_result.is_err() || bytes_written != path_size {
            let _ = VirtualFreeEx(handle, alloc_addr, 0, MEM_RELEASE);
            let _ = CloseHandle(handle);
            return Err(DllInjectorError::WriteFailed(
                "Failed to write DLL path to target process".to_string()
            ));
        }

        // Get LoadLibraryW address from kernel32.dll
        let h_kernel32 = GetModuleHandleA(PCSTR::from_raw("kernel32.dll\0".as_ptr()))
            .map_err(|e| DllInjectorError::GetProcAddressFailed(e.to_string()))?;

        let load_library_w = GetProcAddress(h_kernel32, PCSTR::from_raw("LoadLibraryW\0".as_ptr()));

        let load_library_w = match load_library_w {
            Some(addr) => addr,
            None => {
                let _ = VirtualFreeEx(handle, alloc_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(handle);
                return Err(DllInjectorError::GetProcAddressFailed(
                    "LoadLibraryW not found in kernel32.dll".to_string()
                ));
            }
        };

        // Create remote thread to call LoadLibraryW
        let thread_result = CreateRemoteThread(
            handle,
            None,
            0,
            Some(std::mem::transmute(load_library_w)),
            Some(alloc_addr),
            0,
            None,
        );

        match thread_result {
            Ok(thread) => {
                // Wait for thread completion
                WaitForSingleObject(thread, u32::MAX);

                // Cleanup
                let _ = VirtualFreeEx(handle, alloc_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(thread);
                let _ = CloseHandle(handle);

                Ok(())
            }
            Err(e) => {
                let _ = VirtualFreeEx(handle, alloc_addr, 0, MEM_RELEASE);
                let _ = CloseHandle(handle);
                Err(DllInjectorError::CreateThreadFailed(e.to_string()))
            }
        }
    }
}

/// Unload a DLL from a target process
///
/// # Arguments
/// - `pid`: Target process ID
/// - `module_name`: Name of the loaded module (e.g., "game_mod.dll")
///
/// # Returns
/// - `Ok(())` on successful unload
/// - `Err(DllInjectorError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::dll_injector::unload_dll;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     unload_dll(12345, "game_mod.dll")?;
///     Ok(())
/// }
/// ```

///
/// # Limitations
/// - May not work for system proxy DLLs (version.dll, winmm.dll, etc.) due to reference counting
/// - Recommended to keep DLL loaded until process exits
pub fn unload_dll(pid: u32, module_name: &str) -> Result<(), DllInjectorError> {
    unsafe {
        // Open target process
        let handle = open_process_full(pid)?;

        // Get module base address
        let module_addr = crate::dll_injector::helpers::get_module_base_address(pid, module_name)?;

        // Get FreeLibrary address from kernel32.dll
        let h_kernel32 = GetModuleHandleA(PCSTR::from_raw("kernel32.dll\0".as_ptr()))
            .map_err(|e| DllInjectorError::GetProcAddressFailed(e.to_string()))?;

        let free_library = GetProcAddress(h_kernel32, PCSTR::from_raw("FreeLibrary\0".as_ptr()));

        let free_library = match free_library {
            Some(addr) => addr,
            None => {
                let _ = CloseHandle(handle);
                return Err(DllInjectorError::GetProcAddressFailed(
                    "FreeLibrary not found in kernel32.dll".to_string()
                ));
            }
        };

        // Create remote thread to call FreeLibrary
        let thread_result = CreateRemoteThread(
            handle,
            None,
            0,
            Some(std::mem::transmute(free_library)),
            Some(module_addr as *mut c_void),
            0,
            None,
        );

        match thread_result {
            Ok(thread) => {
                // Wait for thread completion
                WaitForSingleObject(thread, u32::MAX);

                // Cleanup
                let _ = CloseHandle(thread);
                let _ = CloseHandle(handle);

                Ok(())
            }
            Err(e) => {
                let _ = CloseHandle(handle);
                Err(DllInjectorError::CreateThreadFailed(e.to_string()))
            }
        }
    }
}
