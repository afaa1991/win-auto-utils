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
    let value_lock = LockHandler::new_lock_x86_typed(
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
use win_auto_utils::memory_resolver::AddressSource;

// Dynamically find address via byte pattern
let shellcode = vec![0x90, 0x90]; // NOP instruction
let hook_handler = TrampolineHookHandler::new_x86_skip_trampoline(
    "function_hook",
    AddressSource::from_pattern_x86("target_app.exe+1F9C9")?,
    shellcode,
    2,
);
manager.register("function_hook", hook_handler);
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
    let value_lock = LockHandler::new_lock_x86_typed(
        "value_lock",
        "target_app.exe+0x1000",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    
    let nop_patch = BytesSwitchHandler::new_nop_switch_x86(
        "nop_patch",
        "target_app.exe+0x2000",
        2,
    )?;
    manager.register("nop_patch", nop_patch);
    
    let shellcode = vec![0x90, 0x90]; // NOP instruction
    let func_hook = TrampolineHookHandler::new_x86_skip_trampoline(
        "func_hook",
        AddressSource::from_pattern_x86("target_app.exe+0x3000")?,
        shellcode,
        2,
    );
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
Used for continuous monitoring and value restoration (freeze effect).

**Constructors:**
- `new_lock_x86_typed(name, address, value, interval)` - x86 static address
- `new_lock_x64_typed(name, address, value, interval)` - x64 static address
- `new_lock_aob_typed(name, pattern, value, interval)` - AOB pattern scanning

**Example:**
```rust
let handler = LockHandler::new_lock_x86_typed(
    "value_lock",
    "target_app.exe+0x1000",
    100i32,
    Duration::from_millis(100),
)?;
```

#### BytesSwitchHandler
Used for bytecode switching (NOP patches, etc.).

**Constructors:**
- `new_nop_switch_x86(name, address, length)` - x86 NOP switching
- `new_nop_switch_x64(name, address, length)` - x64 NOP switching
- `new_custom_switch_x86(name, address, original_bytes, modified_bytes)` - Custom switching

**Example:**
```rust
let handler = BytesSwitchHandler::new_nop_switch_x86(
    "nop_patch",
    "target_app.exe+0x2000",
    2,
)?;
```

#### TrampolineHookHandler
Used for function hooking while preserving original functionality.

**Constructors:**
- `new_x86_skip_trampoline(name, address_source, shellcode, bytes_to_overwrite)` - x86 hook
- `new_x64_skip_trampoline(name, address_source, shellcode, bytes_to_overwrite)` - x64 hook

**Example:**
```rust
let shellcode = vec![0x90, 0x90]; // NOP instruction
let handler = TrampolineHookHandler::new_x86_skip_trampoline(
    "func_hook",
    AddressSource::from_static_x86("target_app.exe+0x3000")?,
    shellcode,
    2,
);
```

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

A: After reinitializing the process, reset the context:

```rust
process_mgr.reinit("target_app.exe")?;
let proc = process_mgr.get("target_app.exe").unwrap();
manager.set_context(proc.handle().unwrap(), proc.pid().unwrap());
// Reactivate needed features
manager.activate_all()?;
```

### Q: Can I dynamically add new modifiers?

A: Yes, you can register new modifiers at any time:

```rust
let new_handler = LockHandler::new_lock_x86_typed(...)?;
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

## Related Modules

- [`memory_hook`](memory_hook.md) - Low-level hook implementation
- [`memory_lock`](memory_lock.md) - Memory locking functionality
- [`memory_resolver`](memory_resolver.md) - Address resolution
- [`process`](process_window.md) - Process management

---

**Language**: [English](memory_manager.md) | [中文](../../zh/modules/memory_manager.md)
