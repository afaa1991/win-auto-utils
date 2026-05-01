# Memory Lock Module

[中文文档](../../zh/modules/memory_lock.md) | [Back to Overview](overview.md)

The `memory_lock` module provides continuous monitoring and restoration of memory values, useful for creating "freeze" effects in game trainers or maintaining constant values in target processes.

## Features

- **Continuous Monitoring**: Background thread periodically checks memory values
- **Automatic Restoration**: Immediately writes back locked value when changes detected
- **Dynamic Address Support**: Pointer chain resolution adapts to object recreation
- **Flexible Configuration**: Builder pattern supports static/dynamic addresses
- **RAII Management**: Automatic cleanup of background threads prevents resource leaks

## Quick Start

### Basic Usage

```rust
use win_auto_utils::memory_lock::MemoryLock;
use windows::Win32::Foundation::HANDLE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = HANDLE::default(); // Replace with actual handle
    let address = 0x7FF6A1B2C3D4;
    
    // Lock a value (e.g., health = 100)
    let mut lock = MemoryLock::builder()
        .handle(handle)
        .address(address)
        .value(100u32)
        .build()?;
    
    lock.lock_value(100u32)?; // Start locking immediately
    
    // The value will be continuously restored to 100
    // ... your code here ...
    
    // Automatically stops when `lock` goes out of scope
    // Or manually drop: drop(lock);
    
    Ok(())
}
```

### Dynamic Address Resolution

```rust
use win_auto_utils::memory_lock::MemoryLock;
use win_auto_utils::memory_resolver::MemoryAddress;
use windows::Win32::Foundation::HANDLE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let handle = HANDLE::default(); // Replace with actual handle
    
    // Use pointer chain that resolves dynamically
    let addr = MemoryAddress::new_x86("game.exe+58C94->308")?;
    
    let mut lock = MemoryLock::builder()
        .pid(pid)
        .handle(handle)
        .address_from_resolver(addr)
        .value(500u32)
        .build()?;
    
    lock.lock_value(500u32)?; // Start locking
    // Address will be re-resolved on each check cycle
    
    Ok(())
}
```

### Lock Raw Bytes

```rust
use win_auto_utils::memory_lock::MemoryLock;
use windows::Win32::Foundation::HANDLE;

let handle = HANDLE::default();
let address = 0x1000;

// Lock byte sequence directly
let bytes = vec![0x90, 0x90, 0x90]; // NOP instructions
let mut lock = MemoryLock::builder()
    .handle(handle)
    .address(address)
    .bytes(bytes.clone())
    .build()?;

lock.lock_bytes(&bytes)?;
```

## Core Concepts

### How It Works

```
Main Thread:              Background Monitor Thread:
lock.lock_value(100)  →   while !stop_flag {
                            sleep(scan_interval)
                            current = read_memory(address)
                            if current != locked_value {
                                write_memory(address, locked_value)
                            }
                          }
```

### Key Components

1. **Builder Pattern**: Configure lock parameters in stages
2. **Background Thread**: Independent monitoring loop, non-blocking
3. **Atomic Flag**: `Arc<AtomicBool>` for safe thread stopping
4. **Address Source Abstraction**: Supports static and dynamic resolution

### Scan Interval Tuning

```rust
let mut lock = MemoryLock::builder()
    .handle(handle)
    .address(0x1000)
    .value(100u32)
    .scan_interval_ms(5)  // Check every 5ms (default 10ms)
    .build()?;

lock.lock_value(100u32)?;
```

**Recommended Values**:
- **Game Values**: 5-20ms (balance performance and responsiveness)
- **Critical Data**: 1-5ms (tighter locking)
- **Low-Frequency Updates**: 50-100ms (reduce CPU usage)

## Advanced Usage

### Deferred Binding Mode

Pre-configure static parameters, bind dynamic parameters at runtime:

```rust
use win_auto_utils::memory_lock::MemoryLock;

// Step 1: Pre-configure static parameters
let builder = MemoryLock::builder()
    .address(0x7FF6A1B2C3D4)
    .scan_interval_ms(5);

// ... wait for process to start ...
let handle = open_process("game.exe")?;

// Step 2: Bind dynamic parameters and start
let mut lock = builder.clone()
    .handle(handle)
    .value(100u32)
    .build()?;

lock.lock_value(100u32)?;
```

### Fault Tolerance (Transient Error Handling)

For temporary unavailability during game loading or scene transitions:

```rust
// MemoryLock has built-in fault tolerance
// - Initial resolution failure doesn't block
// - Background thread retries until address is valid
// - Suitable for JIT compilation, heap allocation, etc.

let addr = MemoryAddress::new_x86("game.exe+ptr_chain")?;
let mut lock = MemoryLock::builder()
    .pid(pid)
    .handle(handle)
    .address_from_resolver(addr)
    .value(100u32)
    .build()?;

// Even if address is currently invalid, monitoring starts
// Background thread automatically works when address becomes available
lock.lock_value(100u32)?;
```

### Reset After Program Restart

```rust
// First usage
let mut lock = MemoryLock::builder()
    .handle(handle1)
    .address(addr1)
    .value(100u32)
    .build()?;
lock.lock_value(100u32)?;
lock.unlock()?;

// After program restart - reset state
lock.reset();

// Second usage (no side effects)
let mut lock = MemoryLock::builder()
    .handle(new_handle)
    .address(new_addr)
    .value(100u32)
    .build()?;
lock.lock_value(100u32)?;
```

## Use Cases

### 1. Game Value Freezing

Keep player health, mana constant:

```rust
// Infinite health
let health_lock = MemoryLock::builder()
    .handle(handle)
    .address(player_health_addr)
    .value(9999u32)
    .build()?;
health_lock.lock_value(9999u32)?;

// Infinite mana
let mana_lock = MemoryLock::builder()
    .handle(handle)
    .address(player_mana_addr)
    .value(9999u32)
    .build()?;
mana_lock.lock_value(9999u32)?;
```

### 2. Time/Counter Locking

Freeze game timers or countdowns:

```rust
// Freeze countdown
let timer_lock = MemoryLock::builder()
    .handle(handle)
    .address(timer_addr)
    .value(300u32)  // Keep at 300 seconds
    .scan_interval_ms(1)  // Fast response
    .build()?;
timer_lock.lock_value(300u32)?;
```

### 3. Dynamic Object Property Protection

Protect heap-allocated object properties (e.g., player structures):

```rust
// Dynamically resolve player base address using pointer chain
let player_base_addr = MemoryAddress::new_x86("game.exe+PlayerPtr->Offset")?;

let mut lock = MemoryLock::builder()
    .pid(pid)
    .handle(handle)
    .address_from_resolver(player_base_addr)
    .value(100u32)
    .build()?;

lock.lock_value(100u32)?;
// Automatically tracks new address even if player object is recreated
```

## Best Practices

### 1. Choose Appropriate Scan Interval

```rust
// High-frequency updates (real-time combat values)
.scan_interval_ms(1)

// Medium-frequency updates (regular resources)
.scan_interval_ms(10)  // Default

// Low-frequency updates (static configuration)
.scan_interval_ms(50)
```

### 2. Prefer Dynamic Addresses

For heap-allocated objects, always use pointer chains instead of hardcoded addresses:

```rust
// ❌ Bad: Hardcoded address (fails after restart)
.address(0x12345678)

// ✅ Good: Dynamic resolution (adapts to restart)
.address_from_resolver(MemoryAddress::new_x86("game.exe+Ptr->Offset")?)
```

### 3. Handle Lifecycle Gracefully

```rust
// Method 1: RAII (recommended)
{
    let lock = MemoryLock::builder()
        .handle(handle)
        .address(addr)
        .value(100u32)
        .build()?;
    lock.lock_value(100u32)?;
    // ... use lock ...
} // Background thread automatically stops here

// Method 2: Explicit control
let mut lock = /* ... */;
lock.lock_value(100u32)?;
// ... use lock ...
drop(lock); // Explicitly stop
```

### 4. Batch Manage Multiple Locks

```rust
struct GameTrainer {
    health_lock: Option<MemoryLock>,
    mana_lock: Option<MemoryLock>,
    stamina_lock: Option<MemoryLock>,
}

impl GameTrainer {
    fn enable_all(&mut self, handle: HANDLE, pid: u32) -> Result<(), Box<dyn Error>> {
        self.health_lock = Some(/* create health lock */);
        self.mana_lock = Some(/* create mana lock */);
        self.stamina_lock = Some(/* create stamina lock */);
        Ok(())
    }
    
    fn disable_all(&mut self) {
        self.health_lock.take();  // drop automatically stops
        self.mana_lock.take();
        self.stamina_lock.take();
    }
}
```

## Performance Considerations

- **CPU Usage**: Each lock's background thread uses ~0.1-0.5% CPU (depends on scan interval)
- **Memory Overhead**: Each lock uses ~1-2 KB (thread stack + state data)
- **Latency**: Time from value modification to restoration ≈ scan interval
- **Concurrency**: Multiple locks run in parallel without interference

## Common Pitfalls

### ❌ Forgetting to Set PID (Required for Dynamic Addresses)

```rust
// Error: Using dynamic address without PID
let addr = MemoryAddress::new_x86("game.exe+Ptr")?;
let lock = MemoryLock::builder()
    .handle(handle)
    .address_from_resolver(addr)
    // .pid(pid)  ← Missing!
    .value(100u32)
    .build()?;  // Build fails

// Correct: PID required for dynamic addresses
.address_from_resolver(addr)
.pid(pid)  // ✓ Required
```

### ❌ Too Short Scan Interval Causes High CPU Usage

```rust
// Bad: 1ms interval may cause high CPU usage
.scan_interval_ms(1)

// Good: Choose based on actual needs
.scan_interval_ms(10)  // Sufficient for most scenarios
```

### ❌ Modifying Builder After Locking

```rust
// Error: Cannot modify after build()
let mut lock = builder.value(100u32).build()?;
// builder.value(200u32)  ← Invalid, need to rebuild

// Correct: Unlock and recreate
lock.unlock()?;
let new_lock = builder.value(200u32).build()?;
```

## Examples

See these example files for complete implementations:

- `memory_lock_reset.rs`: Lock installation and cleanup demo
- `_debug_lf2_register_extractor.rs`: Dynamic locking combined with register extraction

Run examples:

```bash
cargo run --example memory_lock_reset --features "memory_lock"
```

## Related Modules

- [`memory`](memory.md): Basic memory read/write operations
- [`memory_resolver`](memory_resolver.md): Resolve symbolic addresses
- [`memory_hook`](memory_hook.md): Inline hooks and function interception

---

**Language**: [English](memory_lock.md) | [中文](../../zh/modules/memory_lock.md)
