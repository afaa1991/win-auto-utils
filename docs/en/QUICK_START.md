# Quick Start Guide - Finding the Right Documentation

[中文版本](../zh/QUICK_START.md)

This guide helps you quickly find the documentation you need based on your goals.

## 🎯 I Want To...

### 📖 Learn About the Project
- **New to win-auto-utils?** → Start with [README](../../README.md)
- **Want to see all modules?** → Read [Modules Overview](modules/overview.md)
- **Need full navigation?** → Check [Documentation Index](INDEX.md)

### 🎮 Automate a Game
**Start here:**
1. [Process Management](modules/process_window.md) - Find the game process
2. [Memory Reading](modules/memory.md) - Read health, ammo, coordinates
3. [AOB Scanning](modules/memory_aobscan.md) - Find dynamic addresses
4. [Input Simulation](modules/input.md) - Send keystrokes and mouse clicks
5. [Screen Capture](modules/dxgi.md) - Monitor the game screen

**Advanced:**
- [Memory Hooking](modules/memory_hook.md) - Intercept game functions (god mode, etc.)
- [Register Extraction](modules/memory_hook.md) - Capture player state automatically

### 🧪 Test a Desktop Application
**Start here:**
1. [Window Management](modules/process_window.md) - Control app windows
2. [Input Control](modules/input.md) - Click buttons, type text
3. [Template Matching](modules/template_matcher.md) - Verify UI elements appear
4. [Script Engine](modules/script_engine.md) - Write reusable test scripts

### 🔬 Reverse Engineer a Program
**Start here:**
1. [Memory Operations](modules/memory.md) - Examine process memory
2. [AOB Scanning](modules/memory_aobscan.md) - Locate code patterns
3. [Address Resolution](modules/memory_resolver.md) - Follow pointer chains
4. [Memory Hooking](modules/memory_hook.md) - Monitor function calls
5. [Register Extraction](modules/memory_hook.md) - Capture CPU register values
6. [DLL Injection](modules/dll_injector.md) - Inject debugging tools

### 🤖 Build a Desktop Bot
**Start here:**
1. [Script Engine](modules/script_engine.md) - Create automation scripts
2. [Input Control](modules/input.md) - Control keyboard and mouse
3. [Window Management](modules/process_window.md) - Manage application windows
4. [Screen Capture](modules/dxgi.md) - Visual verification
5. [Template Matching](modules/template_matcher.md) - Detect UI elements

### 💻 Just Browse Examples
Check the `examples/` directory:
- **Basic operations**: `memory_operations.rs`, `mouse_control.rs`
- **Advanced features**: `aobscan_benchmark.rs`, `template_matching.rs`
- **Debug/testing**: Files starting with `_debug_`

Run any example:
```bash
cargo run --example example_name --features "required_feature"
```

## 🚀 Quick Code Examples

### Read Process Memory
```rust
use win_auto_utils::memory::read_memory_t;

let value: i32 = read_memory_t(handle, address)?;
```
📖 Learn more: [Memory Operations](modules/memory.md)

### Simulate Keyboard Input
```rust
use win_auto_utils::keyboard::key_press;

key_press(handle, 0x41)?; // Press 'A'
```
📖 Learn more: [Input Control](modules/input.md)

### Find a Window
```rust
use win_auto_utils::hwnd::get_hwnd_by_title;

let hwnd = get_hwnd_by_title("Notepad")?;
```
📖 Learn more: [Process & Window](modules/process_window.md)

### Scan for Byte Pattern
```rust
use win_auto_utils::memory_aobscan::AobScanner;

let scanner = AobScanner::new(handle)?;
let results = scanner.scan("48 89 5C 24 ??")?;
```
📖 Learn more: [AOB Scanning](modules/memory_aobscan.md)

### Hook a Function
```rust
use win_auto_utils::memory_hook::TrampolineHook;

let mut hook = TrampolineHook::x86(handle, addr, shellcode);
hook.install()?;
```
📖 Learn more: [Memory Hooking](modules/memory_hook.md)

### Run a Script
```rust
use win_auto_utils::script_engine::ScriptEngine;

let script = r#"
    loop 5
        key VK_SPACE
        sleep 100
    end
"#;

let mut engine = ScriptEngine::new();
engine.execute(script)?;
```
📖 Learn more: [Script Engine](modules/script_engine.md)

## 📊 Choose by Feature Flag

If you know which feature flag you need:

| Feature | Module Doc | Use Case |
|---------|-----------|----------|
| `process` | [Process & Window](modules/process_window.md) | Find processes |
| `keyboard` | [Input Control](modules/input.md) | Send keys |
| `mouse` | [Input Control](modules/input.md) | Control mouse |
| `memory` | [Memory Operations](modules/memory.md) | Read/write memory |
| `memory_hook` | [Memory Hooking](modules/memory_hook.md) | Intercept functions |
| `memory_aobscan` | [AOB Scanning](modules/memory_aobscan.md) | Pattern scanning |
| `dxgi` | [Screen Capture](modules/dxgi.md) | Capture screen |
| `template_matcher` | [Template Matching](modules/template_matcher.md) | Image matching |
| `script_engine` | [Script Engine](modules/script_engine.md) | Run scripts |
| `dll_injector` | [DLL Injection](modules/dll_injector.md) | Inject DLLs |

## 🎓 Learning Path

### Beginner (Day 1-3)
1. Read [README](../../README.md)
2. Run basic examples: `memory_operations.rs`, `mouse_control.rs`
3. Understand feature flags in [Cargo.toml](../../Cargo.toml)

### Intermediate (Week 1-2)
1. Study [Memory Hooking](modules/memory_hook.md)
2. Learn [AOB Scanning](modules/memory_aobscan.md)
3. Explore [Script Engine](modules/script_engine.md)
4. Run benchmark examples

### Advanced (Month 1+)
1. Read source code for implementation details
2. Contribute improvements or new features
3. Optimize for specific use cases
4. Build custom extensions

## ❓ Common Questions

**Q: Which module should I start with?**  
A: Depends on your goal. See "I Want To..." section above.

**Q: How do I enable features?**  
A: Add them to `Cargo.toml`: `features = ["memory", "keyboard"]`

**Q: Where are the examples?**  
A: In the `examples/` directory. Run with `cargo run --example name`

**Q: Is there Chinese documentation?**  
A: Yes! All docs have Chinese versions. Look for links at the top of each page.

**Q: How do I contribute?**  
A: Fork the repo, make changes, submit a PR. See [Documentation Guide](DOCUMENTATION_GUIDE.md).

## 🔗 Useful Links

- **Main README**: [English](../../README.md) | [中文](../zh/README.md)
- **Full Index**: [English](INDEX.md) | [中文](../zh/INDEX.md)
- **Module Overview**: [English](modules/overview.md) | [中文](../zh/modules/overview.md)
- **GitHub Repo**: [win-auto-utils](https://github.com/your-repo/win-auto-utils)
- **Examples Directory**: `examples/`

---

**Still lost?** Start with [Modules Overview](modules/overview.md) - it explains everything!

**Language**: [English](QUICK_START.md) | [中文](../zh/QUICK_START.md)
