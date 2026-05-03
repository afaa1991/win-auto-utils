# Memory Manager Module

[中文文档](../../zh/modules/memory_manager.md) | [Back to Overview](overview.md)

The `memory_manager` module provides a unified interface for managing memory modifications (locks, hooks, etc.) with support for dynamic address resolution and process context binding.

## Features

- **Unified Management**: Centralized management of all memory modifiers (Lock, Hook, BytesSwitch)
- **Dynamic Address Resolution**: Supports both static addresses and AOB pattern scanning
- **Process Context Binding**: Automatically associates modifiers with target processes
- **Activation Control**: Enable or disable specific features on demand
- **RAII Cleanup**: Automatic resource release to prevent memory leaks
- **Batch Operations**: Activate or deactivate all modifiers at once

## Architecture

```
ModifierManager
├── ProcessContext (handle + pid)
├── Handler Registry (HashMap<String, ModifierHandler>)
│   ├── LockHandler - Memory locking
│   ├── BytesSwitchHandler - Bytecode switching
│   └── TrampolineHookHandler - Trampoline hooking
└── Lifecycle Management
    ├── activate() / deactivate()
    ├── activate_all() / deactivate_all()
    └── Drop (automatic cleanup)
```

## Use Cases

- **Game Trainers**: Manage multiple cheat features (infinite health, infinite mana, etc.)
- **Debugging Tools**: Dynamically enable/disable different monitoring points
- **Automated Testing**: Test feature modules sequentially
- **Hot Reloading**: Switch between different configuration schemes at runtime

## Quick Start

### Basic Usage

```rust
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_manager::builtin::{LockHandler, BytesSwitchHandler};
use win_auto_utils::process::ProcessManager;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize target process
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("target_app.exe")?;
    process_mgr.init("target_app.exe")?;
    
    let proc = process_mgr.get("target_app.exe").unwrap();
    let handle = proc.handle().expect("No handle");
    let pid = proc.pid().unwrap();
    
    // 2. Create manager and bind process context
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    
    // 3. Register memory lock feature
    let value_lock = LockHandler::new_lock(
        "value_lock",
        "target_app.exe+0x12345",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    
    // 4. Activate feature
    manager.activate("value_lock")?;
    
    // ... application running ...
    
    // 5. Deactivate feature
    manager.deactivate("value_lock")?;
    
    Ok(())
}
```

### Using AOB Pattern Scanning

```rust
use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;

// Dynamically find address via byte pattern (architecture auto-detected)
let shellcode = vec![0x90, 0x90]; // NOP instruction
let hook_handler = TrampolineHookHandler::new_hook_aob_with_offset(
    "function_hook",
    "48 8B 05 ?? ?? ?? ??",  // AOB pattern
    shellcode,
    2,                        // bytes_to_overwrite
    0x10,                     // offset from pattern match
)?;
manager.register("function_hook", hook_handler);
```

### Using Custom Memory Range for AOB Scanning

```rust
use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;

// Scan within specific memory range for better performance
let start_address = 0x10000000000usize;
let length = 0x20000000000usize;

let hook_handler = TrampolineHookHandler::new_hook_aob_with_range_and_offset(
    "optimized_hook",
    "48 8B 05 ?? ?? ?? ??",
    shellcode,
    2,
    start_address,
    length,
    0x10,
)?;
manager.register("optimized_hook", hook_handler);
```

### Batch Operations

```rust
// Activate all registered modifiers
manager.activate_all()?;

// Deactivate all modifiers
manager.deactivate_all()?;

// Check if a modifier is active
if manager.is_active("value_lock") {
    println!("Value lock is enabled");
}

// List all registered modifiers
for name in manager.list_handlers() {
    println!("Registered: {}", name);
}
```

### Complete Example: Multi-Feature Management

```rust
use win_auto_utils::memory_manager::builtin::{
    BytesSwitchHandler, LockHandler, TrampolineHookHandler,
};
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_resolver::AddressSource;
use win_auto_utils::process::ProcessManager;
use std::io::{self, BufRead};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize target process
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("target_app.exe")?;
    process_mgr.init("target_app.exe")?;
    
    let proc = process_mgr.get("target_app.exe").unwrap();
    let handle = proc.handle().unwrap();
    let pid = proc.pid().unwrap();
    
    // Create manager
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    
    // Register multiple features
    let value_lock = LockHandler::new_lock(
        "value_lock",
        "target_app.exe+0x1000",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);

    let nop_patch = BytesSwitchHandler::new_nop_switch(
        "nop_patch",
        "target_app.exe+0x2000",
        2,
    )?;
    manager.register("nop_patch", nop_patch);

    let shellcode = vec![0x90, 0x90]; // NOP instruction
    let func_hook = TrampolineHookHandler::new_hook_aob(
        "func_hook",
        "48 89 5C 24",  // AOB pattern for function prologue
        shellcode,
        2,
    )?;
    manager.register("func_hook", func_hook);
    
    // Test features one by one
    let features = vec!["value_lock", "nop_patch", "func_hook"];
    
    for feature in features {
        println!("Testing: {}", feature);
        
        // Activate
        manager.activate(feature)?;
        println!("  ✓ Activated");
        
        // Wait for user testing
        println!("  → Press Enter to continue...");
        io::stdin().lock().read_line(&mut String::new())?;
        
        // Deactivate
        manager.deactivate(feature)?;
        println!("  ✓ Deactivated\n");
    }
    
    Ok(())
}
```

## API Reference

### ModifierManager

#### Core Methods

- `new()` - Create a new manager instance
- `set_context(handle, pid)` - Bind process context
- `register(name, handler)` - Register a modifier
- `unregister(name)` - Unregister and deactivate a modifier
- `activate(name)` - Activate a specific modifier
- `deactivate(name)` - Deactivate a specific modifier
- `activate_all()` - Activate all modifiers
- `deactivate_all()` - Deactivate all modifiers
- `is_active(name)` - Check if a modifier is active
- `list_handlers()` - List all registered modifier names

### Built-in Handlers

#### LockHandler
Used for continuous monitoring and value restoration (freeze effect). Architecture is automatically detected at activation time.

**Constructors:**
- `new_lock(name, address_pattern, value, interval)` - Static address with auto-detection

**Example:**
```rust
use std::time::Duration;

let handler = LockHandler::new_lock(
    "value_lock",
    "target_app.exe+1000",  // Hex by default, no 0x prefix needed
    100i32,
    Duration::from_millis(100),
)?;
```

**Performance Notes:**
- **First Activation**: Includes address resolution and thread creation (~50-100ms)
- **Subsequent Activations**: Reuses existing thread instance (<1ms) when reactivating in the same process
- **Deactivation**: Stops monitoring thread but keeps instance (~30-50µs)
- **Process Switch**: Automatically recreates instance for new process context

---

#### BytesSwitchHandler
Used for bytecode switching (NOP patches, etc.). Architecture is automatically detected.

**Constructors:**
- `new_bytes_switch(name, address_pattern, byte_count, patch_bytes)` - Static address with custom bytes
- `new_nop_switch(name, address_pattern, length)` - NOP switching
- `new_bytes_switch_aob(name, pattern, byte_count, patch_bytes)` - AOB pattern scanning
- `new_nop_switch_aob(name, pattern, length)` - AOB NOP switching

**Example:**
```rust
let handler = BytesSwitchHandler::new_nop_switch(
    "nop_patch",
    "target_app.exe+0x2000",
    2,
)?;
```

**Performance Notes:**
- **First Activation**: Includes address resolution (~50-500ms for AOB)
- **Subsequent Activations**: Reuses existing instance (<1ms) when reactivating in the same process
- **Deactivation**: Restores original bytes but keeps instance (~30-50µs)
- **Process Switch**: Automatically recreates instance for new process context

---

#### TrampolineHookHandler
Used for function hooking while preserving original functionality. Architecture is automatically detected from the target process.

**Constructors:**
- `new_hook_aob(name, pattern, shellcode, bytes_to_overwrite)` - AOB pattern scanning
- `new_hook_aob_with_offset(name, pattern, shellcode, bytes_to_overwrite, offset)` - AOB with offset
- `new_hook_aob_with_range_and_offset(name, pattern, shellcode, bytes_to_overwrite, start_address, length, offset)` - AOB with custom range and offset
- `new_skip_trampoline_aob(name, pattern, shellcode, bytes_to_overwrite)` - Skip trampoline mode (no original function call)

**Example:**
```rust
let shellcode = vec![0x90, 0x90]; // NOP instruction
let handler = TrampolineHookHandler::new_hook_aob_with_offset(
    "func_hook",
    "48 8B 05 ?? ?? ?? ??",
    shellcode,
    2,
    0x10,
)?;
```

**Performance Notes:**
- **First Activation**: Includes AOB scanning (50-500ms depending on memory size)
- **Subsequent Activations**: Uses cached address (<1ms) when reactivating in the same process
- **Deactivation**: Fast operation (~30-50µs), preserves address cache for quick reactivation
- **Process Switch**: Automatically clears cache and rescans when switching to a different process

## Best Practices

### 1. Process Lifecycle Management

Ensure all modifiers are deactivated before the process closes:

```rust
// Recommended: Use Drop for automatic cleanup
{
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    // ... register and activate ...
} // manager.drop() automatically calls deactivate_all()
```

### 2. Error Handling

Always check return values of activation/deactivation operations:

```rust
match manager.activate("feature_name") {
    Ok(_) => println!("Feature activated"),
    Err(e) => eprintln!("Failed to activate: {}", e),
}
```

### 3. Naming Conventions

Use descriptive modifier names:

```rust
// ✅ Good naming
manager.register("value_monitor", monitor_handler);
manager.register("func_interceptor", interceptor_handler);

// ❌ Avoid vague naming
manager.register("hook1", handler1);
manager.register("mod2", handler2);
```

### 4. Isolated Testing

Following user preference, test features individually to avoid state interference:

```rust
for feature in &features {
    manager.activate(feature)?;
    // Test this feature
    manager.deactivate(feature)?;
}
```

## Common Questions

### Q: How to handle process restart?

A: After reinitializing the process, reset the context. The manager will automatically detect the change and clear caches:

```rust
process_mgr.reinit("target_app.exe")?;
let proc = process_mgr.get("target_app.exe").unwrap();
manager.set_context(proc.handle().unwrap(), proc.pid().unwrap());
// Reactivate needed features (will use cached addresses if same process)
manager.activate_all()?;
```

**Note**: `set_context()` automatically deactivates all handlers and clears AOB region caches to ensure safe switching between processes.

### Q: Can I dynamically add new modifiers?

A: Yes, you can register new modifiers at any time:

```rust
let new_handler = LockHandler::new_lock(...)?;
manager.register("new_feature", new_handler);
manager.activate("new_feature")?;
```

### Q: How to verify if a modifier is working?

A: Use the `is_active()` method to check status:

```rust
if manager.is_active("hp_lock") {
    println!("HP lock is currently active");
}
```

## Performance Considerations

- **Lightweight**: The manager itself has minimal overhead, only maintaining a HashMap
- **On-Demand Activation**: Inactive modifiers consume no CPU resources
- **Background Threads**: LockHandler uses independent threads without affecting main thread performance
- **Batch Operations**: `activate_all()` executes serially, suitable for initialization phase
- **Address Caching**: AOB scanning results are cached for fast reactivation in the same process
  - First activation: ~50-500ms (includes AOB scan)
  - Subsequent activations: <1ms (uses cached address)
  - Deactivation: ~30-50µs (preserves cache)
- **Instance Reuse**: Handlers reuse internal instances when reactivating in the same process, avoiding reconstruction overhead

## Related Modules

- [`memory_hook`](memory_hook.md) - Low-level hook implementation
- [`memory_lock`](memory_lock.md) - Memory locking functionality
- [`memory_resolver`](memory_resolver.md) - Address resolution
- [`process`](process_window.md) - Process management

---

**Language**: [English](memory_manager.md) | [中文](../../zh/modules/memory_manager.md)
