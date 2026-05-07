# Process & Window Management

[中文文档](../../zh/modules/process_window.md) | [Back to Overview](overview.md)

The `process` module provides comprehensive process and window management with a clean, intuitive API. It supports multiple device context (DC) modes for different screen capture scenarios and automatic resource cleanup.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["process"] }
```

## Quick Start

### Method 1: One-Step Initialization (Simplest)

```rust
use win_auto_utils::process::Process;

// Initialize by process name - finds first matching process
let mut process = Process::init_by_name("notepad.exe")?;
println!("PID: {}", process.pid_or_default());
println!("HWND: {:?}", process.hwnd_or_default());
```

### Method 2: Using Builder (Flexible Configuration)

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// Build configuration with intuitive methods
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()  // Best for games
    .exclude_invisible()       // Skip hidden windows
    .include_by_title("Game Window")  // Filter by title
    .build();

let mut process = Process::new(config);
process.init()?;
```

### Method 3: Initialize by PID (Multi-Instance Support)

```rust
use win_auto_utils::process::Process;

// When you know the specific PID
let mut process = Process::init_by_pid(12345)?;
println!("Connected to PID: {}", process.pid_or_default());
```

### Method 4: Re-initialize Existing Process

```rust
use win_auto_utils::process::Process;

// Create without initializing
let mut process = Process::by_name("app.exe");

// Initialize later
process.init()?;

// Switch to different instance
process.init_with_pid(67890)?;
```

## DC Mode Options

Choose the right DC mode for your use case:

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// Option 1: Standard - Full window (title bar + borders)
let config = ProcessConfig::builder("app.exe")
    .set_window_mode()
    .build();

// Option 2: WindowClient - Client area only (recommended for games)
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()
    .build();

// Option 3: Desktop - Full desktop capture
let config = ProcessConfig::builder("fullscreen_game.exe")
    .set_desktop_mode()
    .build();
```

## Window Filtering

Filter windows by title or visibility:

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// Example 1: Exclude invisible windows
let config = ProcessConfig::builder("app.exe")
    .exclude_invisible()
    .build();

// Example 2: Filter by title pattern (case-insensitive)
let config = ProcessConfig::builder("chrome.exe")
    .include_by_title("YouTube")
    .build();

// Example 3: Exact title match (case-sensitive)
let config = ProcessConfig::builder("notepad.exe")
    .include_by_exact_title("document.txt - Notepad")
    .build();

// Example 4: Combined filters
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()
    .exclude_invisible()
    .include_by_title("Main Window")
    .build();

let mut process = Process::new(config);
process.init()?;
```

## Custom Initialization Flags (InitFlags)

Use `InitFlags` to control which system resources (HWND, HANDLE, HDC) are initialized during process connection, allowing you to choose the right initialization strategy for different use cases:

```rust
use win_auto_utils::process::{Process, ProcessConfig, InitFlags};

// Strategy 1: Minimal initialization (PID only, lowest resource usage)
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::minimal())
    .build();

// Strategy 2: Memory-only operations (PID + HANDLE, for memory read/write)
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::memory_only())
    .build();

// Strategy 3: GUI-only operations (PID + HWND + HDC, for screen capture)
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::gui_only())
    .build();

// Strategy 4: Custom configuration (fine-grained control over each resource)
let custom_flags = InitFlags::new()
    .with_pid(true)
    .with_hwnd(true)
    .with_handle(false)  // Skip process handle
    .with_dc(false);     // Skip device context

let config = ProcessConfig::builder("app.exe")
    .init_flags(custom_flags)
    .build();

let mut process = Process::new(config);
process.init()?;
```

### InitFlags Preset Strategies

| Strategy | PID | HWND | HANDLE | HDC | Use Case |
|----------|-----|------|--------|-----|----------|
| `InitFlags::new()` | ✓ | ✓ | ✓ | ✓ | Full access (default) |
| `InitFlags::minimal()` | ✓ | ✗ | ✗ | ✗ | Process identification only |
| `InitFlags::memory_only()` | ✓ | ✗ | ✓ | ✗ | Memory read/write only |
| `InitFlags::gui_only()` | ✓ | ✓ | ✗ | ✓ | Screen capture/GUI automation |

For more details, see [InitFlags Usage Guide](../process_init_flags.md).

## Process Manager (Multi-Process Management)

Manage multiple processes with a single manager:

```rust
use win_auto_utils::process::{Process, ProcessConfig, ProcessManager};

let mut manager = ProcessManager::new();

// Register processes
manager.register("notepad.exe")?;
manager.register_alias("game", "target.exe")?;

// Initialize with different strategies
manager.init("notepad.exe")?;
manager.init_with_pid("game", 12345)?;

// Query processes (read-only, no mut needed)
if let Some(proc) = manager.get("notepad.exe") {
    println!("PID: {:?}", proc.pid());
}

// List all managed processes
for name in manager.list_processes() {
    if let Some(proc) = manager.get(&name) {
        println!("{}: PID={:?}", name, proc.pid());
    }
}
```

## Error Handling

Handle common errors gracefully:

```rust
use win_auto_utils::process::{Process, ProcessError};

match Process::init_by_name("nonexistent.exe") {
    Ok(mut process) => {
        println!("Process initialized: PID={}", process.pid_or_default());
    }
    Err(ProcessError::ProcessNotFound(name)) => {
        eprintln!("Process '{}' not found", name);
    }
    Err(ProcessError::WindowNotFound(pid)) => {
        eprintln!("No window found for PID {}", pid);
    }
    Err(e) => {
        eprintln!("Initialization failed: {}", e);
    }
}
```

## Complete Examples

### Example 1: Game Automation Setup

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// Configure for game capture
let config = ProcessConfig::builder("target.exe")
    .set_window_client_mode()  // Client area only
    .exclude_invisible()       // Skip minimized windows
    .include_by_title("Game")  // Ensure correct window
    .build();

let mut game = Process::new(config);
game.init()?;

println!("Game PID: {}", game.pid_or_default());
println!("Game HWND: {:?}", game.hwnd_or_default());
```

### Example 2: Multi-Instance Application

```rust
use win_auto_utils::process::Process;
use win_auto_utils::snapshot::find_pids_by_name;

// Find all instances
let pids = find_pids_by_name("notepad.exe");
println!("Found {} Notepad instances", pids.len());

// Connect to each instance
for pid in pids {
    let mut process = Process::init_by_pid(pid)?;
    println!("  PID {}: HWND={:?}", pid, process.hwnd_or_default());
}
```

### Example 3: Dynamic Process Switching

```rust
use win_auto_utils::process::Process;

let mut app = Process::by_name("target.exe");

// Initialize first instance
app.init()?;
println!("First instance: PID={}", app.pid_or_default());

// Later, switch to another instance
app.init_with_pid(67890)?;
println!("Switched to: PID={}", app.pid_or_default());
```

## Key Features

✅ **Intuitive API** - No enums to remember, method names are self-explanatory  
✅ **Multiple Initialization Methods** - Choose what fits your use case  
✅ **Smart Window Filtering** - Find exact windows by title or visibility  
✅ **Multi-Process Support** - Manage multiple instances easily  
✅ **Automatic Resource Cleanup** - RAII via Drop trait  
✅ **Performance Optimized** - Lazy initialization, minimal overhead  

## Migration Guide (v0.1.x → v0.2.0)

### Old API (v0.1.x)
```rust
// ❌ Don't do this anymore
let mut process = Process::new("app.exe");
process.dc_mode = DCMode::WindowClient;
process.hwnd_filter = Some(filters);
process.init()?;
```

### New API (v0.2.0)
```rust
// ✅ Use this instead
let config = ProcessConfig::builder("app.exe")
    .set_window_client_mode()
    .exclude_invisible()
    .include_by_title("App Window")
    .build();

let mut process = Process::new(config);
process.init()?;
```

### Key Changes
1. **Configuration is immutable** - Use `ProcessConfig` builder
2. **No direct field access** - All configuration through builder
3. **Intuitive method names** - `.set_window_client_mode()` instead of `.dc_mode(DCMode::WindowClient)`
4. **Better error handling** - More specific error types
5. **Simplified imports** - Only need `Process`, `ProcessConfig`, `ProcessManager`
