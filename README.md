# Win Auto Utils

[中文文档](docs/zh/README.md) | [English Documentation](docs/en/README.md)

A universal Windows automation utility library providing atomic modules for memory operations, window management, input simulation, and color processing. Built with Rust for performance and safety.

## 🚀 Features

### Core Capabilities

- **Process Management**: Process enumeration, handle aggregation, module snapshot (ToolHelp32)
- **Window Operations**: Window handle queries, enumeration, manipulation, DC management
- **Input Simulation**: Keyboard input (Key Down/Up/Press), mouse click & movement
- **Screen Capture**: High-performance DXGI-based capture, GDI color picking
- **Color Processing**: Pure Rust pixel color finding algorithms (zero dependencies)
- **Memory Operations**: Read/write process memory, address resolution, AOB scanning
- **Hooking System**: Inline hooks, trampoline hooks, register extraction
- **Script Engine**: Pure Rust interpreter with control flow, timing, keyboard/mouse instructions
- **DLL Injection**: Cross-architecture injection (x64→x86/x64, WOW64 compatible)
- **Template Matching**: Image-based UI element detection with parallel processing

### Key Advantages

✅ **Atomic Design**: Enable only what you need via feature flags
✅ **Minimal Dependencies**: Core features depend only on `windows` crate
✅ **Performance**: Release mode with LTO, size optimization, symbol stripping
✅ **Safety**: Rust's ownership system prevents common memory errors
✅ **Cross-platform Script Engine**: Pure Rust, no external dependencies

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
win-auto-utils = { version = "0.2.3", features = ["standard"] }
```

For full functionality including template matching:

```toml
[dependencies]
win-auto-utils = { version = "0.2.3", features = ["full"] }
```

## 🎯 Quick Start

### Process Management

**Only three types needed for full functionality!**

```rust
use win_auto_utils::process::{Process, ProcessConfig, ProcessManager};

// Method 1: One-step initialization by name (most convenient)
let mut process = Process::init_by_name("notepad.exe")?;
println!("PID: {}", process.pid_or_default());

// Method 2: One-step initialization by PID (when you know the PID)
let process = Process::init_by_pid(12345)?;
println!("HWND: {:?}", process.hwnd_or_default());

// Method 3: Using Builder with intuitive methods (no enums needed!)
let config = ProcessConfig::builder("target.exe")
    .set_window_client_mode()  // Easy to remember - no DCMode enum!
    .exclude_invisible()
    .include_by_title("Game Window")
    .build();
let mut game = Process::new(config);
game.init()?;

// Method 4: Instance method init_with_pid (for existing Process objects)
// Useful for distinguishing multiple instances of the same process
let mut app = Process::by_name("target.exe");
app.init()?;  // Initialize first instance
// Later, switch to specific PID for second instance
app.init_with_pid(20908)?;

// Manager API (simple and straightforward):
let mut manager = ProcessManager::new();
manager.register("notepad.exe")?;  // process name becomes the key
manager.register_alias("game", "target.exe")?;  // custom alias
manager.init("notepad.exe")?;
manager.init_with_pid("game", 12345)?;  // initialize with specific PID

// Query processes (read-only, no mut needed)
if let Some(proc) = manager.get("notepad.exe") {
    println!("PID: {:?}", proc.pid());
}

// DC Mode Options (intuitive method names):
// - .set_window_mode()        -> Standard window DC (GetWindowDC)
// - .set_window_client_mode() -> Client area DC (GetDC) - best for games
// - .set_desktop_mode()       -> Desktop DC for full-screen capture
```

### Memory Operations

```rust
use win_auto_utils::memory::{read_memory_t, write_memory_t};

// Read a 32-bit integer
let value: i32 = read_memory_t(handle, address)?;

// Write a float value
write_memory_t::<f32>(handle, address, 999.0)?;
```

### Input Simulation

```rust
use win_auto_utils::keyboard::{SendInputKeyboard, PostMessageKeyboard};
use win_auto_utils::mouse::{SendInputMouse, PostMessageMouse};

// SendInput method (system-level input, works with all apps)
let mut kb = SendInputKeyboard::new();
kb.click("a")?;

let mut mouse = SendInputMouse::new();
mouse.move_to(100, 200)?;
mouse.click_left()?;

// PostMessage method (background input, no focus required, needs HWND)
let kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;

let mouse = PostMessageMouse::new(hwnd);
mouse.move_to(100, 200)?;
mouse.click_left_at(100, 200)?;
```

### Screen Capture (DXGI)

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;
let image = capture.capture_window(hwnd)?;
```

### Color Finding

Search for colors in screen regions or pixel buffers with automatic AVX2 optimization:

```rust
use win_auto_utils::color_finder::{find_color, find_color_in_buffer};

// Method 1: Find color in screen region (uses DXGI capture internally)
match find_color(100, 100, 50, 50, (255, 0, 0)) {  // Search red in 50x50 region
    Ok(result) => {
        if result.matched {
            println!("Found at screen coordinates ({}, {})", result.x, result.y);
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}

// Method 2: Find color in pixel buffer (pure algorithm, no screen capture)
use win_auto_utils::color_finder::algorithms::find_color_in_buffer;
let buffer: Vec<u8> = vec![0; 100 * 100 * 4];  // 100x100 BGRA pixels
let result = find_color_in_buffer(&buffer, 100, 100, (0, 255, 0));  // Search green
```

**Features:**
- AVX2 SIMD acceleration (~4-8x faster on supported hardware)
- Automatic fallback to scalar implementation
- Works with any BGRA pixel buffer source

### Memory Hooking (Recommended: Using Memory Manager)

```rust
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
use win_auto_utils::process::ProcessManager;

// Initialize process
let mut process_mgr = ProcessManager::new();
process_mgr.register("target_app.exe")?;
process_mgr.init("target_app.exe")?;

let proc = process_mgr.get("target_app.exe").unwrap();
let handle = proc.handle().unwrap();
let pid = proc.pid().unwrap();

// Create manager and bind context
let mut manager = ModifierManager::new();
manager.set_context(handle, pid);

// Register hook with shellcode (architecture auto-detected)
let shellcode = vec![0x90, 0x90]; // NOP instruction
let hook_handler = TrampolineHookHandler::new_hook_aob_with_offset(
    "func_hook",
    "48 8B 05 ?? ?? ?? ??",  // AOB pattern
    shellcode,
    2,                        // bytes_to_overwrite
    0x10,                     // offset
)?;
manager.register("func_hook", hook_handler);

// Activate hook
manager.activate("func_hook")?;

// ... trigger hook ...

// Deactivate (auto-frees memory)
manager.deactivate("func_hook")?;
```

**Legacy Direct API** (still supported but not recommended):

```rust
use win_auto_utils::memory_hook::TrampolineHook;

let shellcode = vec![0x01, 0xD2]; // add edx, edx
let mut hook = TrampolineHook::new_x86(handle, target_addr, shellcode);
hook.install()?;
// ... trigger hook ...
hook.uninstall()?; // Auto-frees memory
```

### Script Engine

```rust
use win_auto_utils::script_engine::ScriptEngine;

let script = r#"
    loop 10
        key VK_SPACE
        sleep 100
    end
"#;

let mut engine = ScriptEngine::new();
engine.execute(script)?;
```

## 📚 Documentation

Comprehensive documentation is available in both English and Chinese:

### Quick Links
- **[Documentation Index (EN)](docs/en/INDEX.md)** - Complete navigation guide
- **[文档索引 (中文)](docs/zh/INDEX.md)** - 完整导航指南

### Module Documentation

#### English
- [Modules Overview](docs/en/modules/overview.md) - High-level view of all modules
- [Memory Operations](docs/en/modules/memory.md)
- [Memory Manager](docs/en/modules/memory_manager.md) - Unified memory modification manager
- [Memory Hooking](docs/en/modules/memory_hook.md)
- [Address Resolution](docs/en/modules/memory_resolver.md)
- [AOB Scanning](docs/en/modules/memory_aobscan.md)
- [Script Engine](docs/en/modules/script_engine.md)
- [Input Control](docs/en/modules/input.md)
- [Process & Window](docs/en/modules/process_window.md)
- [Screen Capture](docs/en/modules/dxgi.md)
- [Template Matching](docs/en/modules/template_matcher.md)
- [DLL Injection](docs/en/modules/dll_injector.md)

#### Chinese (中文)
- [模块概览](docs/zh/modules/overview.md) - 所有模块的高层视图
- [内存操作](docs/zh/modules/memory.md)
- [内存管理器](docs/zh/modules/memory_manager.md) - 统一的内存修改管理器
- [内存钩子](docs/zh/modules/memory_hook.md)
- [地址解析](docs/zh/modules/memory_resolver.md)
- [字节扫描](docs/zh/modules/memory_aobscan.md)
- [脚本引擎](docs/zh/modules/script_engine.md)
- [输入控制](docs/zh/modules/input.md)
- [进程与窗口](docs/zh/modules/process_window.md)
- [屏幕捕获](docs/zh/modules/dxgi.md)
- [模板匹配](docs/zh/modules/template_matcher.md)
- [DLL注入](docs/zh/modules/dll_injector.md)

## 🏗️ Architecture

The library follows a modular architecture with feature-gated components:

```
win-auto-utils/
├── Process & Window Layer
│   ├── process      - Process management
│   ├── hwnd         - Handle queries
│   ├── window       - Window manipulation
│   └── snapshot     - ToolHelp32 enumeration
│
├── Input Layer
│   ├── keyboard     - Keyboard simulation
│   └── mouse        - Mouse control
│
├── Graphics Layer
│   ├── dxgi         - Screen capture
│   ├── color_picker - GDI color picking
│   └── color_finder - Pixel color search
│
├── Memory Layer
│   ├── memory       - Basic read/write
│   ├── memory_resolver - Symbolic addresses
│   ├── memory_aobscan  - Pattern scanning
│   └── memory_hook     - Inline/trampoline hooks
│
├── Advanced Features
│   ├── dll_injector    - DLL injection
│   └── template_matcher - Image matching
│
└── Script Engine
    ├── script_engine      - Core interpreter
    └── scripts_builtin    - Built-in instructions
```

## 🔧 Feature Flags

Choose only what you need:

### Minimal Setup (~15s compilation)
```bash
cargo build --no-default-features --features "keyboard,mouse"
```

### Core Features (default)
```bash
cargo build  # Includes all stable features except template_matcher
```

### Full Features (~45-60s compilation)
```bash
cargo build --no-default-features --features "full"
```

### Available Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `process` | Process management | windows, hwnd, hdc, snapshot, handle |
| `keyboard` | Keyboard input | windows |
| `mouse` | Mouse control | windows |
| `color_picker` | GDI color picking | windows |
| `color_finder` | Pixel color search (AVX2/SIMD) | dxgi |
| `memory` | Memory read/write | windows |
| `memory_hook` | Hooking system | memory, windows |
| `memory_aobscan` | Pattern scanning | memory, memchr, rayon |
| `dxgi` | Screen capture | windows |
| `template_matcher` | Image matching | image, imageproc, rayon |
| `script_engine` | Script interpreter | (none, pure Rust) |
| `dll_injector` | DLL injection | snapshot, windows |

See [Cargo.toml](Cargo.toml) for complete feature list.

## 📖 Examples

Explore the `examples/` directory for usage demonstrations:

### Memory Manager Examples

```bash
# General memory manager usage
cargo run --example memory_manager_example --features "memory_manager"
```

### Process & Window Management

```bash
# Process manager example
cargo run --example process_manager_example --features "process"

# Custom initialization flags
cargo run --example custom_init_flags --features "process"

# Window activation instruction
cargo run --example active_instruction --features "scripts_window"
```

### Screen Capture

```bash
# DXGI capture example
cargo run --example dxgi_capture --features "dxgi"

# Performance comparison
cargo run --example dxgi_performance_comparison --features "dxgi"
```

### Script Engine

```bash
# Script engine with built-in instructions
cargo run --example script_engine --features "script_engine,scripts_builtin"
```

### Color Operations

```bash
# Color conversion utilities
cargo run --example color_conversion --features "color_picker"

# Color finder re-exports
cargo run --example color_reexports --features "color_finder"
```

### Clipboard

```bash
# Clipboard usage example
cargo run --example clipboard_usage --features "clipboard"
```

### AOB Scanning

```bash
# 64-bit bytecode AOB scanning
cargo run --example aobscan_64bit_bytecode --features "memory_aobscan"
```

### DLL Injection

```bash
# DLL injection example
cargo run --example dll_injection --features "dll_injector"
```

## 🧪 Testing

Run tests for specific modules:

```bash
# Test script engine
cargo test --features "script_engine"

# Test built-in instructions
cargo test --features "scripts_builtin"

# Test memory operations
cargo test --features "memory"
```
