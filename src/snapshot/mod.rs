//! Process snapshot module
//!
//! Provides process enumeration, module enumeration, and PID lookup functionality using ToolHelp32 API.
//!
//! # Features
//! - Process enumeration and PID lookup
//! - Module (DLL/EXE) base address retrieval
//! - Module listing for loaded libraries

use windows::{
    Win32::Foundation::CloseHandle,
    Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next,
        MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
    },
};

use crate::utils::char_array_to_string;

/// Find process ID by executable name
///
/// # Arguments
/// * `process_name` - The name of the executable (e.g., "notepad.exe")
///
/// # Returns
/// * `Some(u32)` - The process ID if found
/// * `None` - If the process is not running or cannot be found
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid;
///
/// if let Some(pid) = find_pid("notepad.exe") {
///     println!("Found notepad with PID: {}", pid);
/// } else {
///     println!("Notepad is not running");
/// }
/// ```
pub fn get_process_pid(process_name: &str) -> Option<u32> {
    unsafe {
        // Create a snapshot of all processes
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(handle) => handle,
            Err(_) => return None,
        };

        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        // Get the first process
        if Process32First(snapshot, &mut entry).is_err() {
            let _ = CloseHandle(snapshot);
            return None;
        }

        // Iterate through all processes
        loop {
            let name = char_array_to_string(&entry.szExeFile);

            // Case-insensitive comparison
            if name.eq_ignore_ascii_case(process_name) {
                let pid = entry.th32ProcessID;
                let _ = CloseHandle(snapshot);
                return Some(pid);
            }

            // Clear the buffer before next iteration to prevent name overlap
            entry.szExeFile.fill(0);

            // Try to get the next process
            if Process32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snapshot);
        None
    }
}

/// Find all process IDs by executable name
///
/// This function returns ALL processes matching the given name, which is useful
/// for applications that can have multiple instances (e.g., notepad.exe).
///
/// # Arguments
/// * `process_name` - The name of the executable (e.g., "notepad.exe")
///
/// # Returns
/// A vector of process IDs. Empty if no matching processes found.
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pids_by_name;
///
/// let pids = find_pids_by_name("notepad.exe");
/// println!("Found {} Notepad processes", pids.len());
/// for pid in pids {
///     println!("  PID: {}", pid);
/// }
/// ```
pub fn find_pids_by_name(process_name: &str) -> Vec<u32> {
    let mut pids = Vec::new();

    unsafe {
        // Create a snapshot of all processes
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(handle) => handle,
            Err(_) => return pids,
        };

        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        // Get the first process
        if Process32First(snapshot, &mut entry).is_err() {
            let _ = CloseHandle(snapshot);
            return pids;
        }

        // Iterate through all processes
        loop {
            let name = char_array_to_string(&entry.szExeFile);

            // Case-insensitive comparison
            if name.eq_ignore_ascii_case(process_name) {
                pids.push(entry.th32ProcessID);
            }

            // Clear the buffer before next iteration to prevent name overlap
            entry.szExeFile.fill(0);

            // Try to get the next process
            if Process32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snapshot);
    }

    pids
}

/// List all running processes with their PIDs and names
///
/// # Returns
/// A vector of tuples containing (PID, process name)
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::list_processes;
///
/// for (pid, name) in list_processes() {
///     println!("PID: {}, Name: {}", pid, name);
/// }
/// ```
pub fn list_processes() -> Vec<(u32, String)> {
    let mut processes = Vec::new();

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(handle) => handle,
            Err(_) => return processes,
        };

        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                let name = char_array_to_string(&entry.szExeFile);
                processes.push((entry.th32ProcessID, name));

                // Clear the buffer before next iteration to prevent name overlap
                entry.szExeFile.fill(0);

                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    processes
}

/// Get the base address of a module (DLL or EXE) in a process
///
/// # Arguments
/// * `pid` - The process ID
/// * `module_name` - The name of the module (e.g., "kernel32.dll" or "game.exe")
///
/// # Returns
/// * `Some(usize)` - The base address of the module if found
/// * `None` - If the module is not loaded in the process
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::{find_pid, get_module_base_address};
///
/// if let Some(pid) = find_pid("notepad.exe") {
///     if let Some(addr) = get_module_base_address(pid, "notepad.exe") {
///         println!("Notepad base address: 0x{:X}", addr);
///     }
///     
///     if let Some(addr) = get_module_base_address(pid, "kernel32.dll") {
///         println!("Kernel32.dll base address: 0x{:X}", addr);
///     }
/// }
/// ```
pub fn get_module_base_address(pid: u32, module_name: &str) -> Option<usize> {
    get_module_info(pid, module_name).map(|info| info.addr)
}

/// List all modules loaded in a process with their base addresses
///
/// # Arguments
/// * `pid` - The process ID
///
/// # Returns
/// A vector of tuples containing (module name, base address as usize)
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::{find_pid, list_modules};
///
/// if let Some(pid) = find_pid("notepad.exe") {
///     for (name, addr) in list_modules(pid) {
///         println!("Module: {}, Address: 0x{:X}", name, addr);
///     }
/// }
/// ```
pub fn list_modules(pid: u32) -> Vec<(String, usize)> {
    list_modules_info(pid).into_iter().map(|info| (info.name, info.addr)).collect()
}

/// Module information struct
pub struct ModuleInfo {
    /// Module name
    pub name: String,
    /// Module base address
    pub addr: usize,
    /// Size of the module
    pub size: usize,
}


/// Get the base address of a module (DLL or EXE) in a process
///
/// # Arguments
/// * `pid` - The process ID
/// * `module_name` - The name of the module (e.g., "kernel32.dll" or "game.exe")
///
/// # Returns
/// * `Some(ModuleInfo)` - The module information if found
/// * `None` - If the module is not loaded in the process
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::{find_pid, get_module_base_address};
///
/// if let Some(pid) = find_pid("notepad.exe") {
///     if let Some(info) = get_module_info(pid, "kernel32.dll") {
///         println!("Kernel32.dll name: {}", info.name);
///         println!("Kernel32.dll base address: 0x{:X}", info.addr);
///         println!("Kernel32.dll size: {}", info.size);
///     }
/// }
/// ```
pub fn get_module_info(pid: u32, module_name: &str) -> Option<ModuleInfo> {
    unsafe {
        // Create a snapshot of all modules in the process
        // TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32 ensures compatibility with both 32-bit and 64-bit
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
        {
            Ok(handle) => handle,
            Err(_) => return None,
        };

        let mut entry = MODULEENTRY32::default();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;

        // Get the first module
        if Module32First(snapshot, &mut entry).is_err() {
            let _ = CloseHandle(snapshot);
            return None;
        }

        // Iterate through all modules
        loop {
            let name = char_array_to_string(&entry.szModule);

            // Case-insensitive comparison
            if name.eq_ignore_ascii_case(module_name) {
                let addr = entry.modBaseAddr as usize;
                let _ = CloseHandle(snapshot);
                return Some(ModuleInfo {
                    name,
                    addr,
                    size: entry.modBaseSize as usize,
                });
            }

            // Clear the buffer before next iteration to prevent name overlap
            entry.szModule = [0i8; 256];
            entry.szExePath = [0i8; 260];

            // Try to get the next module
            if Module32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snapshot);
        None
    }
}

/// List all modules loaded in a process with their base addresses
///
/// # Arguments
/// * `pid` - The process ID
///
/// # Returns
/// A vector of tuples containing (module name, base address as usize)
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::{find_pid, list_modules};
///
/// if let Some(pid) = find_pid("notepad.exe") {
///     for info in list_modules_info(pid) {
///         println!("Module: {}, Address: 0x{:X}, Size: {}", info.name, info.addr, info.size);
///     }
/// }
/// ```
pub fn list_modules_info(pid: u32) -> Vec<ModuleInfo> {
    let mut modules = Vec::new();

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
        {
            Ok(handle) => handle,
            Err(_) => return modules,
        };

        let mut entry = MODULEENTRY32::default();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;

        if Module32First(snapshot, &mut entry).is_ok() {
            loop {
                let name = char_array_to_string(&entry.szModule);
                modules.push(ModuleInfo {
                    name,
                    addr: entry.modBaseAddr as usize,
                    size: entry.modBaseSize as usize,
                });

                // Clear the buffer before next iteration to prevent name overlap
                entry.szModule = [0i8; 256];
                entry.szExePath = [0i8; 260];

                if Module32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_nonexistent_process() {
        // Use a very unique name that's unlikely to exist
        let result = get_process_pid("definitely_not_running_process_12345_xyz.exe");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_processes_not_empty() {
        let processes = list_processes();
        println!("{:?}", processes);
        // There should always be at least one process running (System Idle Process)
        assert!(!processes.is_empty());

        // Check that we can find some common fields
        for (_pid, name) in &processes {
            assert!(!name.is_empty());
        }

        // Verify we have at least the System Idle Process (PID 0)
        assert!(processes.iter().any(|(pid, _)| *pid == 0));
    }

    #[test]
    fn test_char_array_conversion() {
        // Test normal string
        let chars = [
            b'h' as i8, b'e' as i8, b'l' as i8, b'l' as i8, b'o' as i8, 0,
        ];
        assert_eq!(char_array_to_string(&chars), "hello");

        // Test empty string
        let chars = [0];
        assert_eq!(char_array_to_string(&chars), "");

        // Test without null terminator (should handle gracefully)
        let chars = [b't' as i8, b'e' as i8, b's' as i8, b't' as i8];
        assert_eq!(char_array_to_string(&chars), "test");
    }

    #[test]
    fn test_list_modules_for_current_process() {
        // Get current process PID
        let current_pid = std::process::id();

        // List all modules in current process
        let modules = list_modules(current_pid);

        // Should have at least one module (the executable itself)
        assert!(!modules.is_empty());

        println!("Found {} modules in current process", modules.len());

        // Check that all module names are non-empty
        for (name, addr) in &modules {
            assert!(!name.is_empty());
            assert!(*addr > 0);
            println!("  Module: {}, Address: 0x{:X}", name, addr);
        }

        // The first module should be the executable
        if let Some((first_name, _)) = modules.first() {
            assert!(first_name.ends_with(".exe"));
        }
    }

    #[test]
    fn test_find_moudle_nonexistent() {
        // Try to get address of a module that doesn't exist
        let current_pid = std::process::id();
        let result = get_module_base_address(current_pid, "definitely_not_exist_12345.dll");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_moudle_kernel32() {
        // kernel32.dll should be loaded in almost every Windows process
        let current_pid = std::process::id();

        if let Some(addr) = get_module_base_address(current_pid, "kernel32.dll") {
            assert!(addr > 0);
            println!("kernel32.dll found at: 0x{:X}", addr);
        } else {
            // On some systems it might be kernel32.DLL (uppercase)
            if let Some(addr) = get_module_base_address(current_pid, "KERNEL32.DLL") {
                assert!(addr > 0);
                println!("KERNEL32.DLL found at: 0x{:X}", addr);
            }
        }
    }
}
