# Memory Operations

[中文文档](../../zh/modules/memory.md) | [Back to Overview](overview.md)

The `memory` module provides type-safe functions for reading and writing process memory. It supports primitive types (i32, f32, u64, etc.) and raw byte arrays with proper error handling.

## Feature Flag

Enable this module in your `Cargo.toml`:

```toml
[dependencies]
win-auto-utils = { version = "0.2.3", features = ["memory"] }
```

## Quick Start

### Read a Value from Memory

```rust
use win_auto_utils::memory::read_memory_i32;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let address = 0x7FF6A1B2C3D4;
    
    // Get process handle
    let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // Read i32 value
    let value = read_memory_i32(handle, address)?;
    println!("Read value: {}", value);
    
    Ok(())
}
```

### Write Bytes to Memory

```rust
use win_auto_utils::memory::write_memory_bytes;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let address = 0x7FF6A1B2C3D4;
    let data = vec![0x90, 0x90, 0x90]; // NOP instructions
    
    let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    write_memory_bytes(handle, address, &data)?;
    println!("Wrote {} bytes", data.len());
    
    Ok(())
}
```

## Key Features

- **Type-Safe Reads**: Generic `read_memory_t<T>()` for any Copy type
- **Convenience Functions**: Specialized functions for common types (i32, f32, u64, etc.)
- **Byte Array Support**: Read/write raw byte vectors
- **Error Handling**: Returns `Result` with descriptive `MemoryError` types
- **Windows API Integration**: Uses `ReadProcessMemory` and `WriteProcessMemory`

## Usage Examples

### Example 1: Reading Different Types

```rust
use win_auto_utils::memory::{
    read_memory_t, read_memory_i32, read_memory_f32, read_memory_u64
};

// Generic approach - works for any Copy type
let health: f32 = read_memory_t::<f32>(handle, health_addr)?;
let ammo: i32 = read_memory_t::<i32>(handle, ammo_addr)?;
let player_ptr: u64 = read_memory_t::<u64>(handle, ptr_addr)?;

// Convenience functions - more readable
let health = read_memory_f32(handle, health_addr)?;
let ammo = read_memory_i32(handle, ammo_addr)?;
let player_ptr = read_memory_u64(handle, ptr_addr)?;
```

### Example 2: Writing Values

```rust
use win_auto_utils::memory::{write_memory_t, write_memory_bytes};

// Write a float value (set health to 999.0)
write_memory_t::<f32>(handle, health_addr, 999.0)?;

// Write bytes (NOP out an instruction)
let nop_bytes = vec![0x90, 0x90, 0x90, 0x90, 0x90];
write_memory_bytes(handle, target_addr, &nop_bytes)?;

// Write multiple values
write_memory_t::<i32>(handle, ammo_addr, 999)?;
write_memory_t::<u64>(handle, ptr_addr, 0x123456789ABCDEF)?;
```

### Example 3: Reading Byte Arrays

```rust
use win_auto_utils::memory::read_memory_bytes;

// Read 16 bytes for signature verification
let signature = read_memory_bytes(handle, base_addr, 16)?;
println!("Signature: {:02X?}", signature);

// Verify against expected pattern
if signature == vec![0x48, 0x89, 0x5C, 0x24, 0x10, 0x48, 0x89, 0x74] {
    println!("Signature matches!");
}
```

### Example 4: Error Handling

```rust
use win_auto_utils::memory::{read_memory_i32, MemoryError};

match read_memory_i32(handle, suspicious_addr) {
    Ok(value) => println!("Value: {}", value),
    Err(MemoryError::ReadFailed(e)) => eprintln!("Read failed: {}", e),
    Err(MemoryError::InvalidAddress(msg)) => eprintln!("Invalid address: {}", msg),
    Err(e) => eprintln!("Unknown error: {:?}", e),
}
```

## API Reference

### Main Types

- **`MemoryError`**: Error enum for memory operations
  - `ReadFailed(String)` - ReadProcessMemory failed
  - `WriteFailed(String)` - WriteProcessMemory failed
  - `InvalidAddress(String)` - Null or invalid address

### Core Functions

#### Reading Memory

- **`read_memory_t<T: Copy>(handle: HANDLE, address: usize) -> Result<T, MemoryError>`**
  - Generic read for any Copy type
  - Most flexible option
  
- **`read_memory_i32(handle, address)`** - Read 32-bit signed integer
- **`read_memory_f32(handle, address)`** - Read 32-bit float
- **`read_memory_u64(handle, address)`** - Read 64-bit unsigned integer
- **`read_memory_bytes(handle, address, size)`** - Read byte array

#### Writing Memory

- **`write_memory_t<T: Copy>(handle: HANDLE, address: usize, value: T) -> Result<(), MemoryError>`**
  - Generic write for any Copy type
  
- **`write_memory_bytes(handle, address, bytes: &[u8])`** - Write byte array

### Helper Functions

- **`is_valid_address(address: usize) -> bool`** - Check if address is non-null
- **`bytes_to_hex(bytes: &[u8]) -> String`** - Convert bytes to hex string

## Best Practices

1. **Use Appropriate Access Rights**
   ```rust
   // For reading only
   let access = PROCESS_VM_READ;
   
   // For reading and writing
   let access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
   ```

2. **Validate Addresses Before Operations**
   ```rust
   if !is_valid_address(addr) {
       eprintln!("Null address detected");
       return;
   }
   ```

3. **Close Handles Properly**
   ```rust
   use windows::Win32::Foundation::CloseHandle;
   
   unsafe { CloseHandle(handle); }
   // Or use RAII wrappers
   ```

4. **Handle Errors Gracefully**
   ```rust
   match read_memory_i32(handle, addr) {
       Ok(val) => process_value(val),
       Err(e) => log_error(e),
   }
   ```

5. **Use Type-Specific Functions for Clarity**
   ```rust
   // Better than generic for readability
   let health = read_memory_f32(handle, addr)?;
   
   // vs
   let health: f32 = read_memory_t(handle, addr)?;
   ```

## Common Pitfalls

### ❌ Using Invalid Handle

```rust
// Wrong: Handle without proper permissions
let handle = open_process_handle(pid, PROCESS_QUERY_INFORMATION)?;
read_memory_i32(handle, addr)?; // Will fail!

// Correct: Include VM_READ
let handle = open_process_handle(pid, PROCESS_VM_READ)?;
read_memory_i32(handle, addr)?; // Works!
```

### ❌ Not Checking Return Values

```rust
// Wrong: Ignoring errors
let value = read_memory_i32(handle, addr).unwrap(); // May panic!

// Correct: Handle errors
let value = read_memory_i32(handle, addr)?; // Propagate error
```

### ❌ Reading Uninitialized Memory

```rust
// Wrong: Address might not be allocated
let value = read_memory_i32(handle, 0x0)?; // Null pointer!

// Correct: Validate first
if is_valid_address(addr) {
    let value = read_memory_i32(handle, addr)?;
}
```

## Performance Considerations

- **Single Read/Write**: ~1-5μs per operation (depends on target process state)
- **Batch Operations**: Group multiple reads/writes to reduce syscall overhead
- **Large Arrays**: Use `read_memory_bytes()` instead of multiple typed reads
- **Frequent Access**: Consider caching values if they don't change often

### Benchmark Example

```rust
use std::time::Instant;

let start = Instant::now();
for _ in 0..1000 {
    let _ = read_memory_i32(handle, addr)?;
}
let elapsed = start.elapsed();
println!("1000 reads in {:?}", elapsed);
// Typical: 1000 reads in 2-5ms
```

## Related Modules

- [`memory_resolver`](memory_resolver.md): Resolve symbolic addresses like "game.exe+0x123"
- [`memory_hook`](memory_hook.md): Intercept and modify function execution
- [`memory_aobscan`](memory_aobscan.md): Scan for byte patterns in memory
- [`handle`](../process_window.md): Open and manage process handles

---

**Language**: [English](memory.md) | [中文](../../zh/modules/memory.md)
