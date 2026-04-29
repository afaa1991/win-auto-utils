# Process & Window Management

[中文文档](../../zh/modules/process_window.md) | [Back to Overview](overview.md)

The `process` module provides comprehensive process and window management with a fluent builder pattern. It supports multiple device context (DC) modes for different screen capture scenarios, lazy initialization for optimal performance, and automatic resource cleanup.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["process"] }
```

## Quick Start

### Basic Process Management

```rust
use win_auto_utils::process::Process;

// Simple usage with default settings
let mut process = Process::builder("notepad.exe").build();
process.init()?;

println!("PID: {}", process.get_pid());
println!("Handle: {:?}", process.get_handle());
```

### Desktop Mode for Full-Screen Capture

```rust
use win_auto_utils::process::{Process, DCMode};

// Configure for full-screen capture
let mut game = Process::builder("game.exe")
    .set_dc_mode(DCMode::Desktop)
    .build();

game.init()?;
// Now ready for desktop-level screen capture
```

## Key Features

- **Builder Pattern**: Fluent API for flexible configuration
- **Multiple DC Modes**: Standard, WindowClient, Desktop
- **Lazy Initialization**: Resources allocated only when needed
- **Automatic Cleanup**: RAII via Drop trait
- **Window Filtering**: Find specific windows by title patterns
- **Thread-Safe**: Fine-grained locking for concurrent access

## Usage Examples

### Example 1: Process with Window Filtering

```rust
use win_auto_utils::process::Process;

// Filter windows by title
let filters = vec![
    ("Document".to_string(), 1),  // Must contain "Document"
    ("Untitled".to_string(), 0),  // Must NOT contain "Untitled"
];

let mut word = Process::builder("winword.exe")
    .hwnd_filter(filters)
    .build();

word.init()?;
println!("Found window: {:?}", word.get_hwnd());
```

### Example 2: Different DC Modes

```rust
use win_auto_utils::process::{Process, DCMode};

// Mode 1: Standard - captures entire window (title bar + borders)
let mut proc1 = Process::builder("app.exe")
    .set_dc_mode(DCMode::Standard)
    .build();

// Mode 2: WindowClient - captures only client area (content)
let mut proc2 = Process::builder("app.exe")
    .set_dc_mode(DCMode::WindowClient)
    .build();

// Mode 3: Desktop - captures full desktop (for fullscreen games)
let mut proc3 = Process::builder("game.exe")
    .desktop_mode()  // Shorthand for set_dc_mode(DCMode::Desktop)
    .build();

proc1.init()?;
proc2.init()?;
proc3.init()?;
```

### Example 3: Convenience Methods

```rust
use win_auto_utils::process::Process;

// Use convenience methods for common configurations
let game = Process::builder("fullscreen_game.exe")
    .desktop_mode()  // Same as .set_dc_mode(DCMode::Desktop)
    .build();

let app = Process::builder("windowed_app.exe")
    .window_client_mode()  // Same as .set_dc_mode(DCMode::WindowClient)
    .build();
```

### Example 4: Error Handling

```rust
use win_auto_utils::process::{Process, ProcessError};

match Process::builder("nonexistent.exe").build().init() {
    Ok(_) => println!("Process initialized"),
    Err(ProcessError::ProcessNotFound(name)) => {
        eprintln!("Process '{}' not found", name);
    }
    Err(ProcessError::HandleOpenFailed(pid)) => {
        eprintln!("Failed to open handle for PID {}", pid);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Example 5: Accessing Process Information

```rust
use win_auto_utils::process::Process;

let mut process = Process::builder("chrome.exe").build();
process.init()?;

// Get process information
let pid = process.get_pid();
let handle = process.get_handle();
let hwnd = process.get_hwnd();
let dc = process.get_dc();

println!("PID: {}", pid);
println!("Window Handle: {:?}", hwnd);
println!("Device Context: {:?}", dc);
```

### Example 6: Multiple Processes

```rust
use win_auto_utils::process::Process;

// Manage multiple processes
let mut notepad = Process::builder("notepad.exe").build();
let mut calc = Process::builder("calc.exe").build();

notepad.init()?;
calc.init()?;

println!("Notepad PID: {}", notepad.get_pid());
println!("Calculator PID: {}", calc.get_pid());

// Resources automatically cleaned up when dropped
```

## API Reference

### Main Types

#### Process

The main struct for managing processes and windows.

**Constructor**:
- `Process::builder(name: &str) -> ProcessBuilder` - Create builder for configuration

**Methods**:
- `init(&mut self) -> ProcessResult<()>` - Initialize process (lazy)
- `get_pid(&self) -> u32` - Get process ID
- `get_handle(&self) -> HANDLE` - Get process handle
- `get_hwnd(&self) -> HWND` - Get window handle
- `get_dc(&self) -> HDC` - Get device context
- `get_dc_mode(&self) -> DCMode` - Get current DC mode

#### ProcessBuilder

Fluent builder for configuring Process instances.

**Constructor**:
- `Process::builder(name: &str)` - Start building a process

**Configuration Methods**:
- `set_dc_mode(mode: DCMode) -> Self` - Set DC acquisition mode
- `set_dc_mode_num(value: u8) -> Self` - Set DC mode by number (1/2/3)
- `try_set_dc_mode_num(value: u8) -> Result<Self, Self>` - Fallible version
- `desktop_mode() -> Self` - Shorthand for Desktop DC mode
- `window_client_mode() -> Self` - Shorthand for WindowClient DC mode
- `standard_mode() -> Self` - Shorthand for Standard DC mode
- `hwnd_filter(filters: HwndFilter) -> Self` - Set window title filters
- `build() -> Process` - Build configured Process instance

#### DCMode

Device Context acquisition mode enum.

**Variants**:
- `DCMode::Standard` (value: 1) - Full window including title bar/borders
- `DCMode::WindowClient` (value: 2) - Client area only (content)
- `DCMode::Desktop` (value: 3) - Full desktop capture

**Methods**:
- `as_u8(&self) -> u8` - Convert to numeric value
- `from_u8(value: u8) -> Option<DCMode>` - Create from numeric value

#### ProcessError

Error types for process operations.

**Variants**:
- `ProcessNotFound(String)` - Process not found by name
- `HandleOpenFailed(u32)` - Failed to open process handle
- `WindowNotFound(u32)` - No window found for PID
- `DCNotFound(HWND)` - Failed to get device context
- `InvalidDCMode(u8)` - Invalid DC mode value

### Type Aliases

- `HwndFilter` - `Vec<(String, u8)>` - Window title filter
  - `String`: Pattern to match
  - `u8`: Filter mask (1 = must contain, 0 = must NOT contain)

- `ProcessResult<T>` - `Result<T, ProcessError>` - Result type for operations

## DC Mode Comparison

| Mode | Captures | Use Case | Method |
|------|----------|----------|--------|
| **Standard** | Full window (title + borders + content) | General windowed apps | `.standard_mode()` |
| **WindowClient** | Client area only (content) | App content without chrome | `.window_client_mode()` |
| **Desktop** | Entire desktop | Fullscreen games, overlays | `.desktop_mode()` |

### Visual Comparison

```
Standard Mode:
┌─────────────────────┐
│  Title Bar          │  ← Included
├─────────────────────┤
│                     │
│   Content Area      │  ← Included
│                     │
└─────────────────────┘

WindowClient Mode:
┌─────────────────────┐
│  Title Bar          │  ← Excluded
├─────────────────────┤
│                     │
│   Content Area      │  ← Captured
│                     │
└─────────────────────┘

Desktop Mode:
┌───────────────────────────┐
│  Entire Desktop Screen    │  ← Captured
│  (all windows combined)   │
└───────────────────────────┘
```

## Best Practices

1. **Use Builder Pattern for Clarity**
   ```rust
   // Clear and explicit
   let process = Process::builder("game.exe")
       .desktop_mode()
       .build();
   
   // vs manual configuration
   let mut process = Process::new("game.exe");
   process.set_dc_mode(DCMode::Desktop);
   ```

2. **Initialize Only When Needed**
   ```rust
   let mut process = Process::builder("app.exe").build();
   
   // Configuration happens here (no resources allocated)
   
   process.init()?;  // Resources allocated now
   
   // Use process...
   ```

3. **Choose Correct DC Mode**
   ```rust
   // For windowed applications
   let app = Process::builder("notepad.exe")
       .window_client_mode()
       .build();
   
   // For fullscreen games
   let game = Process::builder("game.exe")
       .desktop_mode()
       .build();
   ```

4. **Handle Errors Gracefully**
   ```rust
   match process.init() {
       Ok(_) => use_process(&process),
       Err(e) => log_error(e),
   }
   ```

5. **Let RAII Handle Cleanup**
   ```rust
   {
       let mut process = Process::builder("app.exe").build();
       process.init()?;
       // Use process...
   }  // Automatically cleaned up here
   ```

## Common Pitfalls

### ❌ Forgetting to Call init()

```rust
// Wrong: Process not initialized
let process = Process::builder("app.exe").build();
let pid = process.get_pid();  // Returns 0 or invalid!

// Correct: Always initialize first
let mut process = Process::builder("app.exe").build();
process.init()?;
let pid = process.get_pid();  // Valid PID
```

### ❌ Using Wrong DC Mode

```rust
// Wrong: Standard mode for fullscreen game
let game = Process::builder("game.exe")
    .standard_mode()  // Won't capture properly!
    .build();

// Correct: Desktop mode for fullscreen
let game = Process::builder("game.exe")
    .desktop_mode()
    .build();
```

### ❌ Not Checking Window Filters

```rust
// Wrong: Assuming window will be found
let filters = vec![("Specific Title".to_string(), 1)];
let process = Process::builder("app.exe")
    .hwnd_filter(filters)
    .build();
process.init()?;  // May fail if no matching window

// Correct: Check result
match process.init() {
    Ok(_) => println!("Window found"),
    Err(ProcessError::WindowNotFound(_)) => {
        eprintln!("No window matching filter");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Considerations

- **Lazy Initialization**: Zero overhead until `init()` called
- **Resource Caching**: DC and handles cached after first access
- **Batch Operations**: Initialize multiple processes together when possible
- **DC Mode Impact**: Desktop mode slightly slower than window-specific modes

### Initialization Time

| Operation | Typical Time |
|-----------|--------------|
| Builder creation | < 1μs |
| Process lookup | 1-5ms |
| Handle opening | 1-2ms |
| DC acquisition | 1-3ms |
| Total init() | 3-10ms |

## Related Modules

- [`hwnd`](process_window.md): Window handle utilities
- [`snapshot`](process_window.md): Process/module enumeration
- [`dxgi`](dxgi.md): Advanced screen capture
- [`memory`](memory.md): Read/write process memory

---

**Language**: [English](process_window.md) | [中文](../../zh/modules/process_window.md)
