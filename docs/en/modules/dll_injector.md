# DLL Injector

[中文文档](../../zh/modules/dll_injector.md) | [Back to Overview](overview.md)

The `dll_injector` module provides functionality to inject and unload DLLs into remote processes, and call exported functions from injected DLLs. It supports cross-architecture injection (x64 injector → x86/x64 target) through WOW64 compatibility layer.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["dll_injector"] }
```

**Platform**: Windows only

## Quick Start

### Basic DLL Injection

```rust
use win_auto_utils::dll_injector::inject_dll;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inject a DLL into a process by PID
    let pid = 12345;
    inject_dll(pid, "C:\\path\\to\\library.dll")?;
    println!("DLL injected successfully!");
    Ok(())
}
```

### Call Exported Functions

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_with_raw_bytes
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // Get function address
    let func_addr = get_exported_function_address(
        pid, 
        "game_mod.dll", 
        "set_health"
    )?;

    // Call with i32 parameter (999 health)
    let health: i32 = 999;
    let param_bytes = health.to_ne_bytes();
    let result = call_function_with_raw_bytes(
        pid, 
        func_addr, 
        Some(&param_bytes)
    )?;

    println!("Function returned: {}", result);
    Ok(())
}
```

## Key Features

- **Cross-Architecture Support**: x64 injector can inject into x86 (WOW64) targets
- **Function Calling**: Call arbitrary exported functions with parameters
- **Unload Support**: Dynamically unload injected DLLs
- **Diagnostic Tools**: Pre-injection feasibility checks
- **Shellcode Generation**: Architecture-specific shellcode (x86/x64)
- **Type Safety**: Parameter serialization via `.to_ne_bytes()`

## Usage Examples

### Example 1: DLL Injection with Error Handling

```rust
use win_auto_utils::dll_injector::{inject_dll, DllInjectorError};

fn inject_with_retry(pid: u32, dll_path: &str) -> Result<(), DllInjectorError> {
    match inject_dll(pid, dll_path) {
        Ok(_) => {
            println!("DLL injected successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Injection failed: {}", e);
            
            // Retry once after delay
            std::thread::sleep(std::time::Duration::from_secs(1));
            inject_dll(pid, dll_path)
        }
    }
}
```

### Example 2: Unload DLL

```rust
use win_auto_utils::dll_injector::unload_dll;

let pid = 12345;

// Unload previously injected DLL
unload_dll(pid, "game_mod.dll")?;
println!("DLL unloaded successfully");

// Note: May not work for system proxy DLLs due to reference counting
```

### Example 3: Call Function with Multiple Parameters

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_with_raw_bytes
};

let pid = 12345;

// Get function that takes multiple parameters
let func_addr = get_exported_function_address(
    pid,
    "my_lib.dll",
    "update_player_stats"
)?;

// Serialize struct as bytes
#[repr(C)]
struct PlayerStats {
    health: i32,
    mana: i32,
    level: u32,
}

let stats = PlayerStats {
    health: 999,
    mana: 500,
    level: 99,
};

// Convert to bytes (native endian)
let param_bytes = unsafe {
    std::slice::from_raw_parts(
        &stats as *const PlayerStats as *const u8,
        std::mem::size_of::<PlayerStats>(),
    )
};

let result = call_function_with_raw_bytes(
    pid,
    func_addr,
    Some(param_bytes)
)?;

println!("Update result: {}", result);
```

### Example 4: Call Parameterless Function

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_no_params
};

let pid = 12345;

// Get parameterless function
let func_addr = get_exported_function_address(
    pid,
    "my_lib.dll",
    "reset_game_state"
)?;

// Call without parameters (more efficient)
let result = call_function_no_params(pid, func_addr)?;
println!("Reset result: {}", result);
```

### Example 5: Diagnostic Before Injection

```rust
use win_auto_utils::dll_injector::diagnose_injection;

let pid = 12345;

// Check if injection is feasible
match diagnose_injection(pid) {
    Ok(report) => {
        println!("Process accessible: {}", report.process_accessible);
        println!("Memory allocatable: {}", report.memory_allocatable);
        println!("Write permissions: {}", report.write_permissions);
        
        if report.can_inject() {
            inject_dll(pid, "mod.dll")?;
        } else {
            eprintln!("Cannot inject: {:?}", report);
        }
    }
    Err(e) => eprintln!("Diagnostic failed: {}", e),
}
```

### Example 6: Cross-Architecture Injection

```rust
use win_auto_utils::dll_injector::inject_dll;

// x64 injector targeting x86 process (WOW64)
let x86_pid = 12345; // 32-bit process
inject_dll(x86_pid, "C:\\mods\\library_x86.dll")?;

// x64 injector targeting x64 process
let x64_pid = 67890; // 64-bit process
inject_dll(x64_pid, "C:\\mods\\library_x64.dll")?;
```

## API Reference

### Main Functions

#### Core Injection

- **`inject_dll(pid: u32, dll_path: &str) -> Result<(), DllInjectorError>`**
  - Inject DLL using LoadLibraryW + CreateRemoteThread
  - Supports x64→x64, x86→x86, x64→x86 (WOW64)
  - Returns error if injection fails

- **`unload_dll(pid: u32, module_name: &str) -> Result<(), DllInjectorError>`**
  - Unload DLL using FreeLibrary + CreateRemoteThread
  - May fail for system proxy DLLs (reference counting)
  - Prefer configuration switches for proxy DLLs

#### Remote Function Calling

- **`get_exported_function_address(pid: u32, module_name: &str, function_name: &str) -> Result<usize, DllInjectorError>`**
  - Resolve exported function address in remote process
  - Uses GetProcAddress via shellcode
  - Returns virtual address of function

- **`call_function_with_raw_bytes(pid: u32, function_address: usize, param_data: Option<&[u8]>) -> Result<i32, DllInjectorError>`**
  - Call remote function with raw byte parameters
  - Most flexible API - supports any parameter type
  - Parameters serialized via `.to_ne_bytes()`

- **`call_function_no_params(pid: u32, function_address: usize) -> Result<i32, DllInjectorError>`**
  - Call parameterless remote function
  - More efficient than generic version
  - Use for `void function()` signatures

#### Diagnostic

- **`diagnose_injection(pid: u32) -> Result<InjectionReport, DllInjectorError>`**
  - Perform feasibility checks before injection
  - Checks process accessibility, memory allocation, write permissions
  - Returns detailed diagnostic report

### Error Types

#### DllInjectorError

Error types for DLL injection operations.

**Variants**:
- `ProcessOpenFailed(u32)` - Failed to open process handle
- `MemoryAllocationFailed` - VirtualAllocEx failed
- `MemoryWriteFailed` - WriteProcessMemory failed
- `ThreadCreationFailed` - CreateRemoteThread failed
- `ModuleNotFound(String)` - Target module not found
- `FunctionNotFound(String)` - Exported function not found
- `ArchitectureMismatch` - Injector/target architecture incompatible
- `DiagnosticFailed(String)` - Pre-injection check failed

#### InjectionReport

Diagnostic report structure.

**Fields**:
- `process_accessible: bool` - Can open process handle
- `memory_allocatable: bool` - Can allocate memory in target
- `write_permissions: bool` - Has write permissions
- `is_wow64: bool` - Target is WOW64 process
- `target_architecture: String` - "x86" or "x64"

**Methods**:
- `can_inject(&self) -> bool` - Check if injection is feasible

## Architecture Compatibility Matrix

| Injector Arch | Target Process | DLL Architecture | Status | Notes |
|---------------|----------------|------------------|--------|-------|
| x64 | x64 | x64 | ✅ Supported | Standard case |
| x86 | x86 | x86 | ✅ Supported | Standard case |
| x64 | x86 (WOW64) | x86 | ✅ Supported | Via WOW64 layer |
| x86 | x64 | x64 | ❌ Not possible | Cannot access 64-bit from 32-bit |

## Best Practices

1. **Always Diagnose Before Injection**
   ```rust
   // Good: Check feasibility first
   let report = diagnose_injection(pid)?;
   if report.can_inject() {
       inject_dll(pid, "mod.dll")?;
   }
   
   // Bad: Blind injection
   inject_dll(pid, "mod.dll")?;  // May fail unexpectedly
   ```

2. **Use Correct DLL Architecture**
   ```rust
   // For x86 target process
   inject_dll(x86_pid, "mod_x86.dll")?;
   
   // For x64 target process
   inject_dll(x64_pid, "mod_x64.dll")?;
   ```

3. **Handle Proxy DLLs Carefully**
   ```rust
   // System proxy DLLs may not unload cleanly
   unload_dll(pid, "version.dll")?;  // May fail
   
   // Better: Use configuration instead of unloading
   // Configure proxy DLL to disable features
   ```

4. **Serialize Parameters Correctly**
   ```rust
   // Good: Native endian
   let value: i32 = 42;
   let bytes = value.to_ne_bytes();
   
   // Avoid: Manual byte ordering (error-prone)
   let bytes = [42, 0, 0, 0];  // Assumes little-endian!
   ```

5. **Clean Up After Injection**
   ```rust
   // Inject, use, then unload
   inject_dll(pid, "temp_mod.dll")?;
   use_mod_features(pid)?;
   unload_dll(pid, "temp_mod.dll")?;  // Clean up
   ```

## Common Pitfalls

### ❌ Architecture Mismatch

```rust
// Wrong: x64 DLL into x86 process
inject_dll(x86_pid, "mod_x64.dll")?;  // Will fail!

// Correct: Match architectures
inject_dll(x86_pid, "mod_x86.dll")?;
```

### ❌ Not Checking Return Values

```rust
// Wrong: Ignore function call result
call_function_with_raw_bytes(pid, addr, Some(&bytes))?;

// Correct: Check result
let result = call_function_with_raw_bytes(pid, addr, Some(&bytes))?;
if result != 0 {
    eprintln!("Function returned error code: {}", result);
}
```

### ❌ Leaving DLLs Loaded

```rust
// Wrong: Never unload temporary DLLs
inject_dll(pid, "debug_mod.dll")?;
// ... use it ...
// Forgot to unload - memory leak!

// Correct: Always clean up
inject_dll(pid, "debug_mod.dll")?;
// ... use it ...
unload_dll(pid, "debug_mod.dll")?;
```

## Security Considerations

- ⚠️ **Requires Administrative Privileges**: May need elevated permissions for some processes
- ⚠️ **Anti-Cheat Detection**: Game anti-cheat systems may detect injection
- ⚠️ **Process Stability**: Malicious DLLs can crash target processes
- ⚠️ **Legal Compliance**: Ensure you have permission to modify target processes

## Performance Characteristics

| Operation | Typical Time |
|-----------|--------------|
| Process open | 1-2ms |
| Memory allocation | 1-3ms |
| DLL injection | 10-50ms |
| Function address resolution | 5-15ms |
| Remote function call | 2-5ms |
| DLL unload | 10-30ms |

## Related Modules

- [`process_window`](process_window.md): Find target process by name
- [`memory_hook`](memory_hook.md): Alternative to DLL injection for code modification
- [`snapshot`](process_window.md): Enumerate loaded modules in target process

---

**Language**: [English](dll_injector.md) | [中文](../../zh/modules/dll_injector.md)
