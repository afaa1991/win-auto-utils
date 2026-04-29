# Core Modules Overview

[中文文档](../../zh/modules/overview.md) | [Back to README](../../../README.md)

This document provides a high-level overview of all modules in win-auto-utils, organized by functionality layers.

## Module Categories

### 1. Process & Window Layer

Modules for managing Windows processes and windows.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`process`](process_window.md) | `process` | Process management, handle aggregation, PID lookup |
| [`hwnd`](process_window.md) | `hwnd` | Window handle queries by class name, title, PID |
| [`window`](process_window.md) | `window` | Window manipulation (show/hide, move, resize), enumeration |
| [`snapshot`](process_window.md) | `snapshot` | ToolHelp32 API for process/module enumeration |
| [`handle`](process_window.md) | `handle` | Process handle management (open/close with access rights) |
| [`hdc`](process_window.md) | `hdc` | Device Context management (get/release DCs) |

**Use Cases:**
- Find process by name and get its PID
- Enumerate all child windows of a parent window
- Show/hide/minimize windows programmatically
- Manage process handles with proper cleanup

---

### 2. Input Layer

Keyboard and mouse input simulation.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`keyboard`](input.md) | `keyboard` | Keyboard input via SendInput or PostMessage |
| [`mouse`](input.md) | `mouse` | Mouse control (click, move, scroll) via SendInput or PostMessage |

**Supported Methods:**
- **SendInput**: Foreground window input (simulates real hardware)
- **PostMessage**: Background window input (no focus required)

**Use Cases:**
- Automate UI interactions
- Game bot development (background mode supported)
- Accessibility tools

---

### 3. Graphics Layer

Screen capture and color processing.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`dxgi`](dxgi.md) | `dxgi` | High-performance screen capture using DXGI API |
| [`color_picker`](dxgi.md) | `color_picker` | GDI-based color picking from screen coordinates |
| [`color_finder`](dxgi.md) | `color_finder` | Pure Rust pixel color search algorithms |
| [`template_matcher`](template_matcher.md) | `template_matcher` | Image-based UI element detection with parallel processing |

**Use Cases:**
- Capture game screens at 60+ FPS
- Find specific colors on screen
- Detect UI elements by template image matching
- Build visual automation bots

---

### 4. Memory Layer

Process memory operations and advanced manipulation.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`memory`](memory.md) | `memory` | Basic memory read/write operations |
| [`memory_resolver`](memory_resolver.md) | `memory_resolver` | Symbolic address resolution (e.g., "game.exe+0x123->456") |
| [`memory_aobscan`](memory_aobscan.md) | `memory_aobscan` | Array-of-Bytes pattern scanning with SIMD acceleration |
| [`memory_hook`](memory_hook.md) | `memory_hook` | Inline hooks and trampoline hooks for function interception |
| [`memory_register_extractor`](memory_hook.md) | `memory_register_extractor` | Automatic CPU register capture at hook points |

**Use Cases:**
- Read/write game variables (health, ammo, coordinates)
- Resolve dynamic addresses from base pointers
- Scan memory for unknown values (AOB scanning)
- Hook functions to intercept calls or modify behavior
- Extract register values for reverse engineering

---

### 5. Advanced Features

Specialized tools for complex scenarios.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`dll_injector`](dll_injector.md) | `dll_injector` | DLL injection/unloading with cross-architecture support |
| [`clipboard`](memory.md) | `clipboard` | System clipboard read/write operations |

**Use Cases:**
- Inject custom DLLs into target processes
- Modify process behavior at runtime
- Exchange data via system clipboard

---

### 6. Script Engine

Pure Rust interpreter for automation scripts.

| Module | Feature Flag | Description |
|--------|-------------|-------------|
| [`script_engine`](script_engine.md) | `script_engine` | Core interpreter with parsing, compilation, and execution |
| [`scripts_builtin`](script_engine.md) | `scripts_builtin` | Built-in instruction sets (control flow, keyboard, mouse, timing) |

**Built-in Instructions:**
- **Control Flow**: `loop`, `break`, `continue`, conditional jumps
- **Keyboard**: `key`, `key_down`, `key_up`
- **Mouse**: `click`, `move`, `moverel`, `scroll`
- **Timing**: `sleep` (millisecond precision)
- **Window**: `active` (activate window by HWND)
- **Mode**: `script_mode` (dynamic configuration switching)

**Use Cases:**
- Write automation scripts without recompiling
- Build macro systems for games or applications
- Create configurable automation workflows

---

## Architecture Diagram

```
┌─────────────────────────────────────────────┐
│         Application Layer                    │
│  (Your code using win-auto-utils)            │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│         Script Engine Layer                  │
│  - Parser → Compiler → VM                   │
│  - Built-in instructions                    │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│       Functional Module Layer                │
│                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Process  │ │  Input   │ │ Graphics │    │
│  └──────────┘ └──────────┘ └──────────┘    │
│  ┌──────────┐ ┌──────────┐                 │
│  │  Memory  │ │ Advanced │                 │
│  └──────────┘ └──────────┘                 │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│      Windows API Layer (windows crate)       │
│  - Win32 APIs                               │
│  - DXGI, Direct3D                           │
└─────────────────────────────────────────────┘
```

## Feature Dependency Graph

```
core (default)
├── process
│   ├── hwnd
│   ├── hdc
│   ├── snapshot
│   └── handle
├── keyboard
├── mouse
├── memory
│   ├── memory_resolver
│   ├── memory_aobscan
│   └── memory_hook
│       └── memory_register_extractor
├── dxgi
│   └── color_finder
├── color_picker
├── dll_injector
├── script_engine
│   └── scripts_builtin
└── utils

full
└── core
    └── template_matcher (heavy: image + imageproc + rayon)
```

## Choosing the Right Modules

### For Game Automation
```toml
features = [
    "process",        # Find game process
    "memory",         # Read/write game state
    "memory_hook",    # Intercept game functions
    "keyboard",       # Send key presses
    "mouse",          # Control mouse
    "dxgi",           # Capture game screen
]
```

### For UI Testing
```toml
features = [
    "window",         # Manipulate application windows
    "keyboard",       # Type text
    "mouse",          # Click buttons
    "template_matcher", # Verify UI elements
    "script_engine",  # Write test scripts
]
```

### For Reverse Engineering
```toml
features = [
    "memory",              # Read process memory
    "memory_aobscan",      # Scan for patterns
    "memory_resolver",     # Resolve addresses
    "memory_hook",         # Hook functions
    "memory_register_extractor", # Capture registers
    "dll_injector",        # Inject debugging DLLs
]
```

### For Simple Macros
```toml
features = [
    "script_engine",       # Run scripts
    "scripts_builtin",     # Use built-in commands
    "keyboard",            # Key input
    "mouse",               # Mouse control
]
```

## Next Steps

- Explore individual module documentation for detailed usage
- Check out examples in the `examples/` directory
- Review feature flags in [Cargo.toml](../../../Cargo.toml)

---

**Language**: [English](overview.md) | [中文](../../zh/modules/overview.md)
