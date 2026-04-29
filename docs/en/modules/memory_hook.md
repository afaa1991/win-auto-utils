# Memory Hook Module

[中文文档](../../zh/modules/memory_hook.md) | [Back to Overview](overview.md)

The `memory_hook` module provides advanced function interception capabilities through inline hooks and trampoline hooks, allowing you to modify or monitor program execution at runtime.

## Features

- **Inline Hooks**: Replace target instructions with custom shellcode
- **Trampoline Hooks**: Preserve original functionality while intercepting calls
- **Register Extraction**: Automatically capture CPU register values at hook points
- **Memory Locking**: Protect memory regions from modification
- **Automatic Memory Management**: RAII-based cleanup prevents memory leaks

## Quick Start

### Basic Inline Hook

```rust
use win_auto_utils::memory_hook::TrampolineHook;

// Target: Intercept a function at address 0x12345678
let handle = /* process handle */;
let target_addr = 0x12345678;

// Shellcode: add edx, edx (example instruction)
let shellcode = vec![0x01, 0xD2];

// Create and install hook (x86 mode)
let mut hook = TrampolineHook::x86(handle, target_addr, shellcode);
hook.install()?;

// ... trigger the hook by executing the target code ...

// Uninstall when done (automatically frees memory)
hook.uninstall()?;
```

### x64 Hook

```rust
use win_auto_utils::memory_hook::TrampolineHook;

let mut hook = TrampolineHook::x64(handle, target_addr, shellcode);
hook.install()?;
// ... use hook ...
hook.uninstall()?;
```

### Register Extractor (High-Level API)

Capture register values automatically when a hook triggers:

```rust
use win_auto_utils::memory_hook::register_extractor::{RegisterExtractor, Register};

// Extract ECX (player base) and EDX (MP value) from game instruction
let mut extractor = RegisterExtractor::builder()
    .handle(handle)
    .target_address(0x0041FAF2)
    .bytes_to_overwrite(6)
    .extract_register(Register::ECX)  // Player base address
    .extract_register(Register::EDX)  // MP delta value
    .x86()
    .build()?;

extractor.install()?;

// After hook triggers, read captured values
let player_base: u32 = extractor.read_register::<u32>(Register::ECX)?;
let mp_delta: u32 = extractor.read_register::<u32>(Register::EDX)?;

println!("Player base: 0x{:08X}", player_base);
println!("MP being added: {}", mp_delta as i32);

extractor.uninstall()?;
```

## Core Concepts

### Trampoline Hook Workflow

```
Original Code Flow:
  [Target Address] → [Next Instruction] → [Continue...]

After Hook Installation:
  [Target Address] → JMP to Detour → Execute Custom Code
                                      ↓
                              JMP to Trampoline → Execute Original Instructions
                                                   ↓
                                            JMP back to Original Flow
```

### Key Components

1. **Detour Code**: Your custom shellcode that executes instead of original code
2. **Trampoline**: Backup of overwritten instructions + jump back to original flow
3. **JMP Instructions**: Redirect execution between original, detour, and trampoline

### Memory Layout

```
Target Process Memory:
┌─────────────────────┐
│  Target Address      │ ← Overwritten with JMP to Detour
│  (original instr.)   │
└─────────────────────┘

Allocated Memory:
┌─────────────────────┐
│  Detour Region       │ ← Your shellcode + JMP to Trampoline
│  (RWX permissions)   │
└─────────────────────┘
┌─────────────────────┐
│  Trampoline Region   │ ← Original instructions + JMP back
│  (RX permissions)    │
└─────────────────────┘
```

## Advanced Usage

### Builder Pattern for Fine Control

```rust
use win_auto_utils::memory_hook::TrampolineHookBuilder;

let mut hook = TrampolineHookBuilder::new()
    .handle(handle)
    .target_address(0x12345678)
    .detour_code(vec![0x01, 0xD2])
    .bytes_to_overwrite(5)
    .x86()
    .build()?;

hook.install()?;
```

### Skip Trampoline Mode (Advanced)

For scenarios where you don't need to execute original instructions:

```rust
let mut hook = TrampolineHookBuilder::new()
    .handle(handle)
    .target_address(0x12345678)
    .detour_code(your_complete_shellcode)
    .bytes_to_overwrite(5)
    .skip_trampoline(true)  // No trampoline, no return to original
    .x86()
    .build()?;
```

**Warning**: In this mode, your shellcode must handle all logic including returning to caller if needed.

### Memory Locking

Protect memory regions from external modifications:

```rust
use win_auto_utils::memory_hook::MemoryLock;

let lock = MemoryLock::new(handle, address, size)?;
lock.lock()?;  // Prevent writes to this region

// ... perform operations ...

lock.unlock()?;  // Allow writes again
```

## Use Cases

### 1. Game Function Interception

Hook a damage calculation function to implement "god mode":

```rust
// Original: sub [ecx+0x10], eax  (reduce health)
// Hook: NOP out the subtraction
let nop_shellcode = vec![0x90, 0x90, 0x90, 0x90, 0x90];
let mut hook = TrampolineHook::x86(handle, damage_func_addr, nop_shellcode);
hook.install()?;
// Player now takes no damage!
```

### 2. Resource Monitoring

Extract resource values (health, mana, stamina) in real-time:

```rust
let mut extractor = RegisterExtractor::builder()
    .handle(handle)
    .target_address(resource_update_addr)
    .bytes_to_overwrite(6)
    .extract_register(Register::ECX)  // Player object
    .x86()
    .build()?;

extractor.install()?;

// Periodically read current health
loop {
    let player_ptr: u32 = extractor.read_register::<u32>(Register::ECX)?;
    if player_ptr != 0 {
        let health: f32 = read_memory_t(handle, (player_ptr + 0x20) as usize)?;
        println!("Current health: {:.1}", health);
    }
    std::thread::sleep(Duration::from_millis(100));
}
```

### 3. Function Call Logging

Log every time a specific function is called:

```rust
// Shellcode that logs call (simplified example)
let logging_shellcode = vec![
    0x60,           // PUSHAD (save registers)
    0x9C,           // PUSHFD (save flags)
    // ... call your logging function here ...
    0x9D,           // POPFD
    0x61,           // POPAD
    // Original instruction would be here
];

let mut hook = TrampolineHook::x86(handle, func_addr, logging_shellcode);
hook.install()?;
```

## Best Practices

### 1. Choose Correct Architecture

```rust
// For 32-bit processes
TrampolineHook::x86(handle, addr, code)

// For 64-bit processes
TrampolineHook::x64(handle, addr, code)
```

### 2. Calculate Bytes to Overwrite Accurately

Use disassembly tools (IDA Pro, Ghidra, Cheat Engine) to determine exact instruction lengths:

```rust
// Example: "add [ecx+0x308],edx" is 6 bytes
.bytes_to_overwrite(6)

// Common x86 instruction sizes:
// - JMP rel32: 5 bytes
// - MOV reg, imm32: 5 bytes
// - CALL rel32: 5 bytes
// - PUSH reg: 1 byte
```

### 3. Always Uninstall Hooks

```rust
// Method 1: Explicit uninstall
hook.uninstall()?;

// Method 2: RAII (automatic on drop)
{
    let hook = TrampolineHook::x86(handle, addr, code);
    hook.install()?;
    // ... use hook ...
} // hook.uninstall() called automatically here
```

### 4. Handle Errors Gracefully

```rust
match hook.install() {
    Ok(()) => println!("Hook installed successfully"),
    Err(e) => eprintln!("Failed to install hook: {:?}", e),
}
```

## Performance Considerations

- **Hook Installation**: ~1-5ms (memory allocation + patching)
- **Hook Execution**: ~10-100ns per trigger (JMP overhead)
- **Register Extraction**: Minimal overhead (PUSHAD/POPAD ~50ns)
- **Memory Usage**: ~4KB per hook (detour + trampoline regions)

## Common Pitfalls

### ❌ Incorrect Byte Count

```rust
// Wrong: Instruction is 6 bytes but you specify 5
.bytes_to_overwrite(5)  // Will corrupt next instruction!

// Correct: Match actual instruction length
.bytes_to_overwrite(6)
```

### ❌ Forgetting to Restore

```rust
// Bad: Hook remains installed after panic
hook.install()?;
do_something_that_might_panic()?;  // If this panics, hook leaks!

// Good: Use RAII or explicit error handling
let result = (|| {
    hook.install()?;
    let res = do_something()?;
    hook.uninstall()?;
    Ok(res)
})();
```

### ❌ Wrong Register Names (x86 vs x64)

```rust
// x86: Use ECX, EDX, etc.
.extract_register(Register::ECX)  // ✓ Correct for 32-bit

// x64: Use RCX, RDX, etc.
.extract_register(Register::RCX)  // ✓ Correct for 64-bit
```

## Examples

See these example files for complete implementations:

- `_debug_lf2_register_extractor.rs`: x86 register extraction demo
- `memory_hook_reset.rs`: Hook installation and cleanup
- `memory_operations.rs`: Basic memory read/write with hooks

Run examples:

```bash
cargo run --example memory_hook_reset --features "memory_hook"
cargo run --example _debug_lf2_register_extractor --features "memory_register_extractor process"
```

## Related Modules

- [`memory`](memory.md): Basic memory read/write operations
- [`memory_resolver`](memory_resolver.md): Resolve symbolic addresses
- [`memory_aobscan`](memory_aobscan.md): Scan for byte patterns

---

**Language**: [English](memory_hook.md) | [中文](../../zh/modules/memory_hook.md)
