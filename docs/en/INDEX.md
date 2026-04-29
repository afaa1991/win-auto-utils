# Documentation Index

Welcome to the win-auto-utils documentation! This index helps you find the right documentation for your needs.

## 🚀 Quick Start

**Don't know where to start?** Check out our [Quick Start Guide](QUICK_START.md) - it will help you find exactly what you need based on your goals!

## 📚 Documentation Structure

### Main Documentation
- [English README](../../README.md) - Project overview and quick start
- [中文 README](../zh/README.md) - 项目概览和快速开始

### Module Documentation

#### English
- [Modules Overview](modules/overview.md) - High-level view of all modules
- [Memory Operations](modules/memory.md) - Basic read/write operations
- [Memory Hooking](modules/memory_hook.md) - Function interception and register extraction
- [Address Resolution](modules/memory_resolver.md) - Symbolic address parsing
- [AOB Scanning](modules/memory_aobscan.md) - Pattern scanning with SIMD
- [Script Engine](modules/script_engine.md) - Automation script interpreter
- [Input Control](modules/input.md) - Keyboard and mouse simulation
- [Process & Window](modules/process_window.md) - Process and window management
- [Screen Capture](modules/dxgi.md) - DXGI-based capture
- [Template Matching](modules/template_matcher.md) - Image-based UI detection
- [DLL Injection](modules/dll_injector.md) - Cross-architecture injection

#### Chinese (中文)
- [模块概览](modules/overview.md) - 所有模块的高层视图
- [内存操作](modules/memory.md) - 基础读写操作
- [内存钩子](modules/memory_hook.md) - 函数拦截和寄存器提取
- [地址解析](modules/memory_resolver.md) - 符号地址解析
- [字节扫描](modules/memory_aobscan.md) - SIMD 模式扫描
- [脚本引擎](modules/script_engine.md) - 自动化脚本解释器
- [输入控制](modules/input.md) - 键盘和鼠标模拟
- [进程与窗口](modules/process_window.md) - 进程和窗口管理
- [屏幕捕获](modules/dxgi.md) - 基于 DXGI 的捕获
- [模板匹配](modules/template_matcher.md) - 基于图像的 UI 检测
- [DLL注入](modules/dll_injector.md) - 跨架构注入

## 🎯 Quick Navigation by Use Case

### Game Automation
1. [Process Management](modules/process_window.md) - Find game process
2. [Memory Operations](modules/memory.md) - Read/write game state
3. [Memory Hooking](modules/memory_hook.md) - Intercept functions
4. [AOB Scanning](modules/memory_aobscan.md) - Find dynamic addresses
5. [Input Control](modules/input.md) - Simulate player actions
6. [Screen Capture](modules/dxgi.md) - Monitor game screen

### UI Testing
1. [Window Management](modules/process_window.md) - Control application windows
2. [Input Control](modules/input.md) - Interact with UI elements
3. [Template Matching](modules/template_matcher.md) - Verify UI appearance
4. [Script Engine](modules/script_engine.md) - Write test scripts

### Reverse Engineering
1. [Memory Operations](modules/memory.md) - Examine process memory
2. [AOB Scanning](modules/memory_aobscan.md) - Locate code patterns
3. [Address Resolution](modules/memory_resolver.md) - Resolve pointers
4. [Memory Hooking](modules/memory_hook.md) - Monitor function calls
5. [Register Extraction](modules/memory_hook.md) - Capture CPU state
6. [DLL Injection](modules/dll_injector.md) - Inject debugging tools

### Desktop Automation
1. [Script Engine](modules/script_engine.md) - Create automation scripts
2. [Input Control](modules/input.md) - Control keyboard/mouse
3. [Window Management](modules/process_window.md) - Manage applications
4. [Screen Capture](modules/dxgi.md) - Visual verification

## 📖 Learning Path

### Beginner
1. Start with [README](../../README.md) for project overview
2. Read [Modules Overview](modules/overview.md) to understand architecture
3. Try simple examples:
   - [memory_operations.rs](../../examples/memory_operations.rs)
   - [mouse_control.rs](../../examples/mouse_control.rs)
4. Explore individual module docs as needed

### Intermediate
1. Study advanced modules:
   - [Memory Hooking](modules/memory_hook.md)
   - [AOB Scanning](modules/memory_aobscan.md)
2. Review benchmark examples:
   - [aobscan_benchmark.rs](../../examples/aobscan_benchmark.rs)
   - [template_matching_benchmark.rs](../../examples/template_matching_benchmark.rs)
3. Learn about feature flags in [Cargo.toml](../../Cargo.toml)

### Advanced
1. Deep dive into implementation details in source code
2. Contribute improvements or new features
3. Optimize performance for specific use cases
4. Build custom extensions on top of the library

## 🔍 Finding Information

### By Feature Flag
- `process` → [Process & Window](modules/process_window.md)
- `keyboard`, `mouse` → [Input Control](modules/input.md)
- `memory` → [Memory Operations](modules/memory.md)
- `memory_hook` → [Memory Hooking](modules/memory_hook.md)
- `memory_aobscan` → [AOB Scanning](modules/memory_aobscan.md)
- `dxgi` → [Screen Capture](modules/dxgi.md)
- `template_matcher` → [Template Matching](modules/template_matcher.md)
- `script_engine` → [Script Engine](modules/script_engine.md)
- `dll_injector` → [DLL Injection](modules/dll_injector.md)

### By Example File
All examples are in the `examples/` directory:
- `_debug_*.rs` - Debug/testing examples
- `*_benchmark.rs` - Performance benchmarks
- Others - Feature demonstrations

## 💡 Tips

1. **Start Simple**: Begin with basic modules before advancing to hooks and scanning
2. **Use Feature Flags**: Only enable what you need to reduce compilation time
3. **Check Examples**: Most modules have working examples you can run
4. **Read Error Messages**: Rust's compiler provides helpful guidance
5. **Community**: Check GitHub issues for common problems and solutions

## 🤝 Contributing to Documentation

If you find errors or want to improve documentation:

1. Fork the repository
2. Edit the markdown files
3. Submit a pull request
4. Ensure both English and Chinese versions are updated

## 📞 Support

- **GitHub Issues**: Report bugs or request features
- **Examples**: Run example code to see features in action
- **Source Code**: Read inline documentation in Rust source files

---

**Language**: [English](INDEX.md) | [中文](../zh/INDEX.md)
