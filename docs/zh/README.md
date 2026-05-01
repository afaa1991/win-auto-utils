# Win Auto Utils

[English Documentation](../README.md) | [英文文档](../README.md)

一个通用的 Windows 自动化工具库，提供原子化模块用于内存操作、窗口管理、输入模拟和颜色处理。使用 Rust 构建，兼顾性能与安全性。

## 🚀 功能特性

### 核心能力

- **进程管理**: 进程枚举、句柄聚合、模块快照 (ToolHelp32)
- **窗口操作**: 窗口句柄查询、枚举、操控、DC 管理
- **输入模拟**: 键盘输入 (按下/释放/敲击)、鼠标点击与移动
- **屏幕捕获**: 基于 DXGI 的高性能捕获、GDI 颜色拾取
- **颜色处理**: 纯 Rust 实现的像素颜色查找算法（零依赖）
- **内存操作**: 读写进程内存、地址解析、字节扫描
- **钩子系统**: 内联钩子、蹦床钩子、寄存器提取
- **脚本引擎**: 纯 Rust 解释器，支持控制流、定时、键鼠指令
- **DLL 注入**: 跨架构注入 (x64→x86/x64，兼容 WOW64)
- **模板匹配**: 基于图像的 UI 元素检测，支持并行处理

### 核心优势

✅ **原子化设计**: 通过 feature flags 按需启用功能  
✅ **最小依赖**: 核心功能仅依赖 `windows` crate  
✅ **高性能**: Release 模式开启 LTO、体积优化、符号剥离  
✅ **安全性**: Rust 所有权系统防止常见内存错误  
✅ **跨平台脚本引擎**: 纯 Rust 实现，无外部依赖  

## 📦 安装

将以下内容添加到你的 `Cargo.toml`：

```
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["standard"] }
```

如需包含模板匹配等完整功能：

```
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["full"] }
```

## 🎯 快速开始

### 进程管理

```rust
use win_auto_utils::process::Process;

let process = Process::builder("notepad.exe").build();
process.init()?;
println!("PID: {}", process.get_pid());
```

### 内存操作

```rust
use win_auto_utils::memory::{read_memory_t, write_memory_t};

// 读取 32 位整数
let value: i32 = read_memory_t(handle, address)?;

// 写入浮点值
write_memory_t::<f32>(handle, address, 999.0)?;
```

### 输入模拟

```rust
use win_auto_utils::keyboard::key_press;
use win_auto_utils::mouse::{move_to, left_click};

// 按下 'A' 键
key_press(handle, 0x41)?;

// 移动鼠标并点击
move_to(handle, 100, 200)?;
left_click(handle)?;
```

### 屏幕捕获 (DXGI)

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;
let image = capture.capture_window(hwnd)?;
```

### 内存钩子（推荐：使用内存管理器）

```rust
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
use win_auto_utils::process::ProcessManager;

// 初始化进程
let mut process_mgr = ProcessManager::new();
process_mgr.register("target_app.exe")?;
process_mgr.init("target_app.exe")?;

let proc = process_mgr.get("target_app.exe").unwrap();
let handle = proc.handle().unwrap();
let pid = proc.pid().unwrap();

// 创建管理器并绑定上下文
let mut manager = ModifierManager::new();
manager.set_context(handle, pid);

// 注册钩子和shellcode
let shellcode = vec![0x90, 0x90]; // NOP指令
let hook_handler = TrampolineHookHandler::new_x86_skip_trampoline(
    "func_hook",
    AddressSource::from_static_x86("target_app.exe+0x1000")?,
    shellcode,
    2,
);
manager.register("func_hook", hook_handler);

// 激活钩子
manager.activate("func_hook")?;

// ... 触发钩子 ...

// 停用（自动释放内存）
manager.deactivate("func_hook")?;
```

**传统直接API**（仍然支持但不推荐）：

```
use win_auto_utils::memory_hook::TrampolineHook;

let shellcode = vec![0x90, 0x90]; // NOP指令
let mut hook = TrampolineHook::new_x86(handle, target_addr, shellcode);
hook.install()?;
// ... 触发钩子 ...
hook.uninstall()?; // 自动释放内存
```

### 脚本引擎

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

## 📚 文档

提供完整的中英文文档：

### 快速链接
- **[Documentation Index (EN)](docs/en/INDEX.md)** - 完整导航指南
- **[文档索引 (中文)](docs/zh/INDEX.md)** - 完整导航指南

### 模块文档

#### 英文 (English)
- [Modules Overview](docs/en/modules/overview.md) - 所有模块的高层视图
- [Memory Operations](docs/en/modules/memory.md)
- [Memory Manager](docs/en/modules/memory_manager.md) - 统一的内存修改管理器
- [Memory Hooking](docs/en/modules/memory_hook.md)
- [Address Resolution](docs/en/modules/memory_resolver.md)
- [AOB Scanning](docs/en/modules/memory_aobscan.md)
- [Script Engine](docs/en/modules/script_engine.md)
- [Input Control](docs/en/modules/input.md)
- [Process & Window](docs/en/modules/process_window.md)
- [Screen Capture](docs/en/modules/dxgi.md)
- [Template Matching](docs/en/modules/template_matcher.md)
- [DLL Injection](docs/en/modules/dll_injector.md)

#### 中文
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

## 🏗️ 架构设计

库采用模块化架构，通过 feature-gated 组件实现解耦：

```
win-auto-utils/
├── 进程与窗口层
│   ├── process      - 进程管理
│   ├── hwnd         - 句柄查询
│   ├── window       - 窗口操控
│   └── snapshot     - ToolHelp32 枚举
│
├── 输入层
│   ├── keyboard     - 键盘模拟
│   └── mouse        - 鼠标控制
│
├── 图形层
│   ├── dxgi         - 屏幕捕获
│   ├── color_picker - GDI 颜色拾取
│   └── color_finder - 像素颜色搜索
│
├── 内存层
│   ├── memory       - 基础读写
│   ├── memory_resolver - 符号地址解析
│   ├── memory_aobscan  - 模式扫描
│   └── memory_hook     - 内联/蹦床钩子
│
├── 高级功能
│   ├── dll_injector    - DLL 注入
│   └── template_matcher - 图像匹配
│
└── 脚本引擎
    ├── script_engine      - 核心解释器
    └── scripts_builtin    - 内置指令集
```

## 🔧 Feature Flags

按需选择所需功能：

### 最小化配置 (~15秒编译)
```bash
cargo build --no-default-features --features "keyboard,mouse"
```

### 核心功能（默认）
```bash
cargo build  # 包含所有稳定功能（除 template_matcher）
```

### 完整功能 (~45-60秒编译)
```bash
cargo build --no-default-features --features "full"
```

### 可用 Features

| Feature | 描述 | 依赖 |
|---------|------|------|
| `process` | 进程管理 | windows, hwnd, hdc, snapshot, handle |
| `keyboard` | 键盘输入 | windows |
| `mouse` | 鼠标控制 | windows |
| `memory` | 内存读写 | windows |
| `memory_hook` | 钩子系统 | memory, windows |
| `memory_aobscan` | 模式扫描 | memory, memchr, rayon |
| `dxgi` | 屏幕捕获 | windows |
| `template_matcher` | 图像匹配 | image, imageproc, rayon |
| `script_engine` | 脚本解释器 | (无，纯 Rust) |
| `dll_injector` | DLL 注入 | snapshot, windows |

完整功能列表参见 [Cargo.toml](Cargo.toml)。

## 📖 示例代码

查看 `examples/` 目录了解使用演示：

```bash
# 运行脚本引擎示例
cargo run --example script_engine --features "script_engine"

# 运行 DXGI 捕获示例
cargo run --example dxgi_capture --features "dxgi"

# 运行内存钩子示例
cargo run --example memory_hook_reset --features "memory_hook"

# 运行 AOB 扫描基准测试
cargo run --example aobscan_benchmark --features "memory_aobscan"
```

## 🧪 测试

运行特定模块的测试：

```bash
# 测试脚本引擎
cargo test --features "script_engine"

# 测试内置指令
cargo test --features "scripts_builtin"

# 测试内存操作
cargo test --features "memory"
```

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📄 许可证

本项目采用 MIT 许可证 - 详见 LICENSE 文件。

## 🙏 致谢

- Windows API 绑定来自 [microsoft/windows-rs](https://github.com/microsoft/windows-rs)
- 模板匹配算法来自 [image-rs/imageproc](https://github.com/image-rs/imageproc)
- 社区反馈与测试支持

---

**语言**: [English](../README.md) | [中文](README.md)
