# 进程与窗口管理 (Process & Window Management)

[English](../../en/modules/process_window.md) | [返回概览](overview.md)

`process` 模块通过流畅的构建器模式提供全面的进程和窗口管理。它支持多种设备上下文（DC）模式以适应不同的屏幕捕获场景，延迟初始化以优化性能，以及自动资源清理。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["process"] }
```

## 快速开始

### 基础进程管理

```rust
use win_auto_utils::process::Process;

// 使用默认设置的简单用法
let mut process = Process::builder("notepad.exe").build();
process.init()?;

println!("PID: {}", process.get_pid());
println!("句柄: {:?}", process.get_handle());
```

### 全屏捕获的桌面模式

```rust
use win_auto_utils::process::{Process, DCMode};

// 配置为全屏捕获
let mut game = Process::builder("game.exe")
    .set_dc_mode(DCMode::Desktop)
    .build();

game.init()?;
// 现在已准备好进行桌面级屏幕捕获
```

## 核心功能

- **构建器模式**: 灵活配置的流畅 API
- **多种 DC 模式**: Standard、WindowClient、Desktop
- **延迟初始化**: 仅在需要时分配资源
- **自动清理**: 通过 Drop trait 实现 RAII
- **窗口过滤**: 按标题模式查找特定窗口
- **线程安全**: 细粒度锁定以支持并发访问

## 使用示例

### 示例 1: 带窗口过滤的进程

```rust
use win_auto_utils::process::Process;

// 按标题过滤窗口
let filters = vec![
    ("Document".to_string(), 1),  // 必须包含 "Document"
    ("Untitled".to_string(), 0),  // 不能包含 "Untitled"
];

let mut word = Process::builder("winword.exe")
    .hwnd_filter(filters)
    .build();

word.init()?;
println!("找到的窗口: {:?}", word.get_hwnd());
```

### 示例 2: 不同的 DC 模式

```rust
use win_auto_utils::process::{Process, DCMode};

// 模式 1: Standard - 捕获整个窗口（标题栏 + 边框）
let mut proc1 = Process::builder("app.exe")
    .set_dc_mode(DCMode::Standard)
    .build();

// 模式 2: WindowClient - 仅捕获客户区（内容）
let mut proc2 = Process::builder("app.exe")
    .set_dc_mode(DCMode::WindowClient)
    .build();

// 模式 3: Desktop - 捕获整个桌面（用于全屏游戏）
let mut proc3 = Process::builder("game.exe")
    .desktop_mode()  // set_dc_mode(DCMode::Desktop) 的简写
    .build();

proc1.init()?;
proc2.init()?;
proc3.init()?;
```

### 示例 3: 便捷方法

```rust
use win_auto_utils::process::Process;

// 对常见配置使用便捷方法
let game = Process::builder("fullscreen_game.exe")
    .desktop_mode()  // 等同于 .set_dc_mode(DCMode::Desktop)
    .build();

let app = Process::builder("windowed_app.exe")
    .window_client_mode()  // 等同于 .set_dc_mode(DCMode::WindowClient)
    .build();
```

### 示例 4: 错误处理

```rust
use win_auto_utils::process::{Process, ProcessError};

match Process::builder("nonexistent.exe").build().init() {
    Ok(_) => println!("进程已初始化"),
    Err(ProcessError::ProcessNotFound(name)) => {
        eprintln!("未找到进程 '{}'", name);
    }
    Err(ProcessError::HandleOpenFailed(pid)) => {
        eprintln!("无法打开 PID {} 的句柄", pid);
    }
    Err(e) => eprintln!("错误: {}", e),
}
```

### 示例 5: 访问进程信息

```rust
use win_auto_utils::process::Process;

let mut process = Process::builder("chrome.exe").build();
process.init()?;

// 获取进程信息
let pid = process.get_pid();
let handle = process.get_handle();
let hwnd = process.get_hwnd();
let dc = process.get_dc();

println!("PID: {}", pid);
println!("窗口句柄: {:?}", hwnd);
println!("设备上下文: {:?}", dc);
```

### 示例 6: 多个进程

```rust
use win_auto_utils::process::Process;

// 管理多个进程
let mut notepad = Process::builder("notepad.exe").build();
let mut calc = Process::builder("calc.exe").build();

notepad.init()?;
calc.init()?;

println!("记事本 PID: {}", notepad.get_pid());
println!("计算器 PID: {}", calc.get_pid());

// 丢弃时自动清理资源
```

## API 参考

### 主要类型

#### Process

管理进程和窗口的主要结构体。

**构造函数**:
- `Process::builder(name: &str) -> ProcessBuilder` - 创建配置构建器

**方法**:
- `init(&mut self) -> ProcessResult<()>` - 初始化进程（延迟）
- `get_pid(&self) -> u32` - 获取进程 ID
- `get_handle(&self) -> HANDLE` - 获取进程句柄
- `get_hwnd(&self) -> HWND` - 获取窗口句柄
- `get_dc(&self) -> HDC` - 获取设备上下文
- `get_dc_mode(&self) -> DCMode` - 获取当前 DC 模式

#### ProcessBuilder

用于配置 Process 实例的流畅构建器。

**构造函数**:
- `Process::builder(name: &str)` - 开始构建进程

**配置方法**:
- `set_dc_mode(mode: DCMode) -> Self` - 设置 DC 获取模式
- `set_dc_mode_num(value: u8) -> Self` - 按数字设置 DC 模式（1/2/3）
- `try_set_dc_mode_num(value: u8) -> Result<Self, Self>` - 可失败版本
- `desktop_mode() -> Self` - Desktop DC 模式的简写
- `window_client_mode() -> Self` - WindowClient DC 模式的简写
- `standard_mode() -> Self` - Standard DC 模式的简写
- `hwnd_filter(filters: HwndFilter) -> Self` - 设置窗口标题过滤器
- `build() -> Process` - 构建配置好的 Process 实例

#### DCMode

设备上下文获取模式枚举。

**变体**:
- `DCMode::Standard`（值: 1）- 完整窗口包括标题栏/边框
- `DCMode::WindowClient`（值: 2）- 仅客户区（内容）
- `DCMode::Desktop`（值: 3）- 全屏桌面捕获

**方法**:
- `as_u8(&self) -> u8` - 转换为数字值
- `from_u8(value: u8) -> Option<DCMode>` - 从数字值创建

#### ProcessError

进程操作的错误类型。

**变体**:
- `ProcessNotFound(String)` - 按名称未找到进程
- `HandleOpenFailed(u32)` - 无法打开进程句柄
- `WindowNotFound(u32)` - 未找到 PID 的窗口
- `DCNotFound(HWND)` - 无法获取设备上下文
- `InvalidDCMode(u8)` - 无效的 DC 模式值

### 类型别名

- `HwndFilter` - `Vec<(String, u8)>` - 窗口标题过滤器
  - `String`: 要匹配的模式
  - `u8`: 过滤器掩码（1 = 必须包含，0 = 不能包含）

- `ProcessResult<T>` - `Result<T, ProcessError>` - 操作的结果类型

## DC 模式对比

| 模式 | 捕获内容 | 用例 | 方法 |
|------|---------|------|------|
| **Standard** | 完整窗口（标题 + 边框 + 内容） | 一般窗口应用 | `.standard_mode()` |
| **WindowClient** | 仅客户区（内容） | 不带窗口装饰的应用内容 | `.window_client_mode()` |
| **Desktop** | 整个桌面 | 全屏游戏、覆盖层 | `.desktop_mode()` |

### 可视化对比

```
Standard 模式:
┌─────────────────────┐
│  标题栏              │  ← 包含
├─────────────────────┤
│                     │
│   内容区域           │  ← 包含
│                     │
└─────────────────────┘

WindowClient 模式:
┌─────────────────────┐
│  标题栏              │  ← 排除
├─────────────────────┤
│                     │
│   内容区域           │  ← 捕获
│                     │
└─────────────────────┘

Desktop 模式:
┌───────────────────────────┐
│  整个桌面屏幕              │  ← 捕获
│  （所有窗口组合）          │
└───────────────────────────┘
```

## 最佳实践

1. **使用构建器模式提高清晰度**
   ```rust
   // 清晰明了
   let process = Process::builder("game.exe")
       .desktop_mode()
       .build();
   
   // vs 手动配置
   let mut process = Process::new("game.exe");
   process.set_dc_mode(DCMode::Desktop);
   ```

2. **仅在需要时初始化**
   ```rust
   let mut process = Process::builder("app.exe").build();
   
   // 配置发生在这里（不分配资源）
   
   process.init()?;  // 现在分配资源
   
   // 使用进程...
   ```

3. **选择正确的 DC 模式**
   ```rust
   // 对于窗口应用
   let app = Process::builder("notepad.exe")
       .window_client_mode()
       .build();
   
   // 对于全屏游戏
   let game = Process::builder("game.exe")
       .desktop_mode()
       .build();
   ```

4. **优雅地处理错误**
   ```rust
   match process.init() {
       Ok(_) => use_process(&process),
       Err(e) => log_error(e),
   }
   ```

5. **让 RAII 处理清理**
   ```rust
   {
       let mut process = Process::builder("app.exe").build();
       process.init()?;
       // 使用进程...
   }  // 在这里自动清理
   ```

## 常见陷阱

### ❌ 忘记调用 init()

```rust
// 错误: 进程未初始化
let process = Process::builder("app.exe").build();
let pid = process.get_pid();  // 返回 0 或无效！

// 正确: 始终先初始化
let mut process = Process::builder("app.exe").build();
process.init()?;
let pid = process.get_pid();  // 有效的 PID
```

### ❌ 使用错误的 DC 模式

```rust
// 错误: 全屏游戏使用 Standard 模式
let game = Process::builder("game.exe")
    .standard_mode()  // 无法正确捕获！
    .build();

// 正确: 全屏使用 Desktop 模式
let game = Process::builder("game.exe")
    .desktop_mode()
    .build();
```

### ❌ 不检查窗口过滤器

```rust
// 错误: 假设窗口会被找到
let filters = vec![("Specific Title".to_string(), 1)];
let process = Process::builder("app.exe")
    .hwnd_filter(filters)
    .build();
process.init()?;  // 如果没有匹配的窗口可能会失败

// 正确: 检查结果
match process.init() {
    Ok(_) => println!("找到窗口"),
    Err(ProcessError::WindowNotFound(_)) => {
        eprintln!("没有匹配过滤器的窗口");
    }
    Err(e) => eprintln!("错误: {}", e),
}
```

## 性能考虑

- **延迟初始化**: 调用 `init()` 之前零开销
- **资源缓存**: 首次访问后缓存 DC 和句柄
- **批量操作**: 尽可能一起初始化多个进程
- **DC 模式影响**: Desktop 模式比窗口特定模式稍慢

### 初始化时间

| 操作 | 典型时间 |
|------|---------|
| 构建器创建 | < 1μs |
| 进程查找 | 1-5ms |
| 句柄打开 | 1-2ms |
| DC 获取 | 1-3ms |
| 总 init() | 3-10ms |

## 相关模块

- [`hwnd`](process_window.md): 窗口句柄工具
- [`snapshot`](process_window.md): 进程/模块枚举
- [`dxgi`](dxgi.md): 高级屏幕捕获
- [`memory`](memory.md): 读/写进程内存

---

**语言**: [English](../../en/modules/process_window.md) | [中文](process_window.md)
