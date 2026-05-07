# Memory Address Resolver

[中文文档](../../zh/modules/memory_resolver.md) | [Back to Overview](overview.md)

The `memory_resolver` module resolves symbolic memory addresses like `"game.exe+123->456->789"` to actual runtime addresses. It automatically parses module base addresses, offsets, and pointer chains with a clean, minimal syntax that defaults to hexadecimal (no `0x` prefix needed).

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["memory_resolver"] }
```

## Quick Start

### Resolve a Simple Address

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// Parse x86 (32-bit) address - hex by default, no 0x needed!
let addr_str = "game.exe+1234";
let memory_addr = MemoryAddress::new_x86(addr_str)?;

// Resolve to actual address
let handle = /* process handle */;
let pid = /* process ID */;
let resolved = memory_addr.resolve_address(handle, pid)?;

println!("Resolved address: 0x{:X}", resolved);
```

### Resolve Pointer Chain

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// Clean syntax: no 0x prefix needed for hex values
let addr_str = "game.exe+5000->10->20->30";
let memory_addr = MemoryAddress::new_x64(addr_str)?;

let resolved = memory_addr.resolve_address(handle, pid)?;
println!("Final address: 0x{:X}", resolved);
```

## Key Features

- **Clean Hex Syntax**: Default to hexadecimal without `0x` prefix (`game.exe+123` = 0x123)
- **Decimal Support**: Use `#` prefix for decimal numbers (`#1000` = decimal 1000)
- **Multi-Level Pointers**: Resolve chains like `base->offset1->offset2`
- **Architecture Support**: Works with both x86 (32-bit) and x64 (64-bit)
- **Readable Numbers**: Underscore separators for clarity (`1_0000` = 0x10000)
- **Error Handling**: Clear error messages for invalid formats
- **Builder Pattern**: Optional builder for complex configurations

## Usage Examples

### Example 1: Basic Resolution (Hex Default)

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // Get process handle
    let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // Hex by default - no 0x prefix needed!
    let addr = MemoryAddress::new_x86("client.dll+ABCD")?;
    let resolved = addr.resolve_address(handle, pid)?;
    
    // Read value at resolved address
    use win_auto_utils::memory::read_memory_i32;
    let value = read_memory_i32(handle, resolved)?;
    println!("Value: {}", value);
    
    Ok(())
}
```

### Example 2: Multi-Level Pointer Chain

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::memory::read_memory_f32;

// Player health: [[[game.exe+1000]+20]+30]
// All hex by default, no 0x needed
let health_addr = MemoryAddress::new_x64("game.exe+1000->20->30")?;
let health_ptr = health_addr.resolve_address(handle, pid)?;

let health: f32 = read_memory_f32(handle, health_ptr)?;
println!("Player health: {:.1}", health);
```

### Example 3: Decimal vs Hexadecimal

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// Hexadecimal (default) - no prefix needed
let addr1 = MemoryAddress::parse("game.exe+1000")?;  // = 0x1000 = 4096 decimal

// Decimal with # prefix
let addr2 = MemoryAddress::parse("#1000")?;          // = 1000 decimal = 0x3E8

// Mixed usage
let addr3 = MemoryAddress::parse("target.exe+100->#50")?;  // hex 100, then decimal 50
```

### Example 4: Dynamic Address Updates

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::memory::read_memory_i32;

// Addresses may change between game restarts
let addr = MemoryAddress::new_x86("game.exe+5000->10")?;

// Re-resolve each time you need the address
loop {
    let current_addr = addr.resolve_address(handle, pid)?;
    let value: i32 = read_memory_i32(handle, current_addr)?;
    println!("Current value: {}", value);
    
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```

### Example 5: Builder Pattern

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// Use builder for complex configurations
let addr = MemoryAddress::builder()
    .address("lf2.exe+58C94->308")
    .x86()
    .build()?;

let resolved = addr.resolve_address(handle, pid)?;
```

### Example 6: Readable Number Formatting

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// Underscores for readability (ignored during parsing)
let addr1 = MemoryAddress::parse("game.exe+1_0000->2FC")?;  // = 0x10000->0x2FC
let addr2 = MemoryAddress::parse("#10_000")?;                // = decimal 10000
```

## API Reference

### Main Types

- **`MemoryAddress`**: Represents a symbolic or resolved address
  - Contains parsed operations (module base, offsets, pointer jumps)
  - Architecture-aware (x86 vs x64 pointer sizes)
  
- **`ParseError`**: Address parsing errors
  - `EmptyInput` - Empty string provided
  - `InvalidHex(String)` - Malformed hexadecimal
  - `InvalidDecimal(String)` - Malformed decimal
  - `InvalidPointerSyntax(String)` - Missing '->' operator
  - `MultipleModules` - More than one module name
  
- **`ResolveError`**: Address resolution errors
  - `ModuleNotFound(String)` - Module not found in process
  - `PointerReadFailed(usize, MemoryError)` - Failed to dereference pointer

### Constructors

- **`MemoryAddress::new_x86(address_str: &str) -> Result<Self, ParseError>`**
  - Parse x86 (32-bit) address string
  - Uses 4-byte pointers for dereferencing
  
- **`MemoryAddress::new_x64(address_str: &str) -> Result<Self, ParseError>`**
  - Parse x64 (64-bit) address string
  - Uses 8-byte pointers for dereferencing
  
- **`MemoryAddress::parse(address_str: &str) -> Result<Self, ParseError>`**
  - Parse with default architecture (matches compiled binary)

### Builder

- **`MemoryAddress::builder() -> MemoryAddressBuilder`**
  - Create builder for complex configurations
  - Methods: `.address()`, `.x86()`, `.x64()`, `.build()`

### Methods

- **`resolve_address(handle: HANDLE, pid: u32) -> Result<usize, ResolveError>`**
  - Resolve symbolic address to actual runtime address
  - Reads module base from process
  - Follows pointer chain if present
  - Returns final absolute address
  
- **`get_operations(&self) -> &[AddressOp]`**
  - Get parsed address operations
  - Useful for debugging or custom resolution logic

## Address Syntax

| Syntax | Example | Description |
|--------|---------|-------------|
| Module + Offset | `game.exe+1000` | Module base + hex offset (default) |
| Absolute Address | `7FF6A1B2C3D4` | Direct hex address (no 0x needed) |
| Decimal | `#1000` | Decimal number (prefix with `#`) |
| Direct Offset | `+100` | Add hex offset without dereference |
| Pointer Jump | `->20` | Dereference then add hex offset |
| Underscore | `1_0000` | Readability separator (= 0x10000) |

### Comparison: Old vs New Syntax

```rust
// ❌ Old verbose style (not required anymore)
"game.exe+0x1000->0x20->0x30"

// ✅ New clean style (recommended)
"game.exe+1000->20->30"

// Both work identically!
```

### Complex Examples

```rust
// Module-relative with pointer chain (all hex)
"app.dll+1000->20->30"

// Absolute address with direct offset
"7FF6A1B2C3D4+100"

// Decimal numbers
"#1000+#200"

// Mixed syntax
"target.exe+1_0000->2FC"

// Real-world example (LF2 game)
"lf2.exe+58C94->308"
```

## Best Practices

1. **Use Clean Hex Syntax (No 0x Prefix)**
   ```rust
   // Recommended: Clean and concise
   let addr = MemoryAddress::new_x86("game.exe+1000->20")?;
   
   // Also works but more verbose
   let addr = MemoryAddress::new_x86("game.exe+0x1000->0x20")?;
   ```

2. **Cache Resolved Addresses When Possible**
   ```rust
   // Bad: Resolve every time
   for _ in 0..100 {
       let addr = memory_addr.resolve_address(handle, pid)?;
       let val = read_memory_i32(handle, addr)?;
   }
   
   // Good: Resolve once, reuse
   let addr = memory_addr.resolve_address(handle, pid)?;
   for _ in 0..100 {
       let val = read_memory_i32(handle, addr)?;
   }
   ```

3. **Handle Resolution Failures**
   ```rust
   match addr.resolve_address(handle, pid) {
       Ok(resolved) => use_address(resolved),
       Err(ResolveError::ModuleNotFound(name)) => {
           eprintln!("Module '{}' not found", name);
       }
       Err(ResolveError::PointerReadFailed(addr, err)) => {
           eprintln!("Failed to read pointer at 0x{:X}: {}", addr, err);
       }
   }
   ```

4. **Validate Module Names**
   ```rust
   // Ensure module exists before resolving
   use win_auto_utils::snapshot::enumerate_modules;
   
   let modules = enumerate_modules(pid)?;
   if !modules.iter().any(|m| m.name == "game.exe") {
       eprintln!("Module not found!");
       return;
   }
   ```

5. **Choose Correct Architecture**
   ```rust
   // For 32-bit processes (old games like LF2)
   let addr = MemoryAddress::new_x86("game.exe+1000")?;
   
   // For 64-bit processes (modern apps)
   let addr = MemoryAddress::new_x64("game.exe+1000")?;
   ```

## Common Pitfalls

### ❌ Using Decimal Without # Prefix

```rust
// Wrong: This is interpreted as hex 1000 (= 4096 decimal)
let addr = MemoryAddress::parse("#1000")?;  // If you meant decimal 1000

// Correct: Use # for decimal
let addr = MemoryAddress::parse("#1000")?;  // = 1000 decimal
let addr = MemoryAddress::parse("1000")?;   // = 0x1000 hex (= 4096 decimal)
```

### ❌ Forgetting Architecture

```rust
// Wrong: Using x86 parser for x64 process
let addr = MemoryAddress::new_x86("game.exe+1000")?;
// Pointer dereferences will read wrong size!

// Correct: Match process architecture
let addr = MemoryAddress::new_x64("game.exe+1000")?;
```

### ❌ Invalid Pointer Chain Syntax

```rust
// Wrong: Single '>' instead of '->'
let addr = MemoryAddress::parse("game.exe+1000>20")?; // Error!

// Correct: Use '->' for pointer jumps
let addr = MemoryAddress::parse("game.exe+1000->20")?;
```

## Performance Considerations

- **Single Resolution**: ~50-200μs (includes module enumeration)
- **Cached Resolution**: <1μs (if module base is cached externally)
- **Pointer Chains**: Each level adds ~5-10μs for memory reads
- **Recommendation**: Cache module bases for frequent access

### Optimization Tips

```rust
// If resolving same module repeatedly, cache the base address
use win_auto_utils::snapshot::get_module_base;

let module_base = get_module_base(pid, "game.exe")?;

// Then calculate manually for faster repeated access
let final_addr = module_base + 0x1000;
```

## Related Modules

- [`memory`](memory.md): Read/write values at resolved addresses
- [`memory_aobscan`](memory_aobscan.md): Find addresses dynamically via scanning
- [`snapshot`](../process_window.md): Enumerate process modules
- [`handle`](../process_window.md): Open and manage process handles

---

**Language**: [English](memory_resolver.md) | [中文](../../zh/modules/memory_resolver.md)
