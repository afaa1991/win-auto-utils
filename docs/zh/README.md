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
win-auto-utils = { version = "0.2.3", features = ["standard"] }
```

如需包含模板匹配等完整功能：

```
[dependencies]
win-auto-utils = { version = "0.2.3", features = ["full"] }
```

## 🎯 快速开始

### 进程管理

**仅需三种类型即可完成全部功能！**

```rust
use win_auto_utils::process::{Process, ProcessConfig, ProcessManager};

// 方式1: 按名称一步初始化（最便捷）
let mut process = Process::init_by_name("notepad.exe")?;
println!("PID: {}", process.pid_or_default());

// 方式2: 按 PID 一步初始化（已知 PID 时使用）
let process = Process::init_by_pid(12345)?;
println!("HWND: {:?}", process.hwnd_or_default());

// 方式3: 使用 Builder 构建配置（无需枚举类型！）
let config = ProcessConfig::builder("target.exe")
    .set_window_client_mode()  // 易于记忆 - 无需 DCMode 枚举！
    .exclude_invisible()
    .include_by_title("Game Window")
    .build();
let mut game = Process::new(config);
game.init()?;

// 方式4: 实例方法 init_with_pid（用于区分同一进程的多个实例）
let mut app = Process::by_name("target.exe");
app.init()?;  // 初始化第一个实例
// 之后切换到特定 PID 的第二个实例
app.init_with_pid(20908)?;

// Manager API（简洁直观）：
let mut manager = ProcessManager::new();
manager.register("notepad.exe")?;  // 进程名作为 key
manager.register_alias("game", "target.exe")?;  // 自定义别名
manager.init("notepad.exe")?;
manager.init_with_pid("game", 12345)?;  // 用指定 PID 初始化

// 查询进程（只读，无需 mut）
if let Some(proc) = manager.get("notepad.exe") {
    println!("PID: {:?}", proc.pid());
}

// DC 模式选项（直观的方法名）：
// - .set_window_mode()        -> 标准窗口 DC (GetWindowDC)
// - .set_window_client_mode() -> 客户端区域 DC (GetDC) - 游戏最佳选择
// - .set_desktop_mode()       -> 桌面 DC 用于全屏捕获
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
use win_auto_utils::keyboard::{SendInputKeyboard, PostMessageKeyboard};
use win_auto_utils::mouse::{SendInputMouse, PostMessageMouse};

// SendInput 方式（系统级输入，适用于所有应用）
let mut kb = SendInputKeyboard::new();
kb.click("a")?;

let mut mouse = SendInputMouse::new();
mouse.move_to(100, 200)?;
mouse.click_left()?;

// PostMessage 方式（后台输入，不需要焦点，需要目标窗口句柄）
let kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;

let mouse = PostMessageMouse::new(hwnd);
mouse.move_to(100, 200)?;
mouse.click_left_at(100, 200)?;
```

### 屏幕捕获 (DXGI)

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;
let image = capture.capture_window(hwnd)?;
```

### 颜色查找

在屏幕区域或像素缓冲区中搜索颜色，支持 AVX2 自动优化：

```rust
use win_auto_utils::color_finder::{find_color, find_color_in_buffer};

// 方式1: 在屏幕区域中查找颜色（内部使用 DXGI 捕获）
match find_color(100, 100, 50, 50, (255, 0, 0)) {  // 在 50x50 区域中搜索红色
    Ok(result) => {
        if result.matched {
            println!("在屏幕坐标 ({}, {}) 找到", result.x, result.y);
        }
    }
    Err(e) => eprintln!("错误: {}", e),
}

// 方式2: 在像素缓冲区中查找颜色（纯算法，无需屏幕捕获）
use win_auto_utils::color_finder::algorithms::find_color_in_buffer;
let buffer: Vec<u8> = vec![0; 100 * 100 * 4];  // 100x100 BGRA 像素
let result = find_color_in_buffer(&buffer, 100, 100, (0, 255, 0));  // 搜索绿色
```

**特性:**
- AVX2 SIMD 加速（在支持的硬件上快 4-8 倍）
- 自动回退到标量实现
- 支持任何 BGRA 像素缓冲区来源

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

// 使用 shellcode 注册钩子（架构自动检测）
let shellcode = vec![0x90, 0x90]; // NOP 指令
let hook_handler = TrampolineHookHandler::new_hook_aob_with_offset(
    "func_hook",
    "48 8B 05 ?? ?? ?? ??",  // AOB 模式
    shellcode,
    2,                        // 覆盖字节数
    0x10,                     // 偏移量
)?;
manager.register("func_hook", hook_handler);

// 激活钩子
manager.activate("func_hook")?;

// ... 触发钩子 ...

// 停用（自动释放内存）
manager.deactivate("func_hook")?;
```

**传统直接 API**（仍支持，但不推荐）：

```rust
use win_auto_utils::memory_hook::TrampolineHook;

let shellcode = vec![0x01, 0xD2]; // add edx, edx
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

提供中英文综合文档：

### 快速链接
- **[文档索引 (中文)](docs/zh/INDEX.md)** - 完整导航指南
- **[Documentation Index (EN)](docs/en/INDEX.md)** - Complete navigation guide

### 模块文档

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

## 🏗️ 架构

库采用模块化架构，特性门控：

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
│   ├── memory       - 基本读写
│   ├── memory_resolver - 符号地址
│   ├── memory_aobscan  - 模式扫描
│   └── memory_hook     - 内联/蹦床钩子
│
├── 高级功能
│   ├── dll_injector    - DLL 注入
│   └── template_matcher - 图像匹配
│
└── 脚本引擎
    ├── script_engine      - 核心解释器
    └── scripts_builtin    - 内置指令
```

## 🔧 特性标志

按需选择：

### 最小配置（约 15s 编译）
```bash
cargo build --no-default-features --features "keyboard,mouse"
```

### 核心功能（默认）
```bash
cargo build  # 包含除 template_matcher 外的所有稳定功能
```

### 完整功能（约 45-60s 编译）
```bash
cargo build --no-default-features --features "full"
```

### 可用特性

| 特性 | 描述 | 依赖 |
|---------|-------------|--------------|
| `process` | 进程管理 | windows, hwnd, hdc, snapshot, handle |
| `keyboard` | 键盘输入 | windows |
| `mouse` | 鼠标控制 | windows |
| `color_picker` | GDI 颜色拾取 | windows |
| `color_finder` | 像素颜色搜索 (AVX2/SIMD) | dxgi |
| `memory` | 内存读写 | windows |
| `memory_hook` | 钩子系统 | memory, windows |
| `memory_aobscan` | 模式扫描 | memory, memchr, rayon |
| `dxgi` | 屏幕捕获 | windows |
| `template_matcher` | 图像匹配 | image, imageproc, rayon |
| `script_engine` | 脚本解释器 | （无，纯 Rust） |
| `dll_injector` | DLL 注入 | snapshot, windows |

完整的特性列表参见 [Cargo.toml](Cargo.toml)。

## 📖 示例

参考 `examples/` 目录下的使用演示：

### 内存管理器示例

```bash
# 通用内存管理器用法
cargo run --example memory_manager_example --features "memory_manager"
```

### 进程与窗口管理

```bash
# 进程管理器示例
cargo run --example process_manager_example --features "process"

# 自定义初始化标志
cargo run --example custom_init_flags --features "process"

# 窗口激活指令
cargo run --example active_instruction --features "scripts_window"
```

### 屏幕捕获

```bash
# DXGI 捕获示例
cargo run --example dxgi_capture --features "dxgi"

# 性能对比
cargo run --example dxgi_performance_comparison --features "dxgi"
```

### 脚本引擎

```bash
# 带内置指令的脚本引擎
cargo run --example script_engine --features "script_engine,scripts_builtin"
```

### 颜色操作

```bash
# 颜色转换工具
cargo run --example color_conversion --features "color_picker"

# 颜色查找器重导出
cargo run --example color_reexports --features "color_finder"
```

### 剪贴板

```bash
# 剪贴板用法示例
cargo run --example clipboard_usage --features "clipboard"
```

### 字节扫描

```bash
# 64 位字节码 AOB 扫描
cargo run --example aobscan_64bit_bytecode --features "memory_aobscan"
```

### DLL 注入

```bash
# DLL 注入示例
cargo run --example dll_injection --features "dll_injector"
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
