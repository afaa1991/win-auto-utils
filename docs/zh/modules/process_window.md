# 进程与窗口管理

[English](../../en/modules/process_window.md) | [返回概览](overview.md)

`process` 模块提供全面的进程和窗口管理，拥有简洁直观的 API。支持多种设备上下文（DC）模式以适应不同的屏幕捕获场景，并自动清理资源。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["process"] }
```

## 快速开始

### 方法 1：一步初始化（最简单）

```rust
use win_auto_utils::process::Process;

// 通过进程名初始化 - 查找第一个匹配的进程
let mut process = Process::init_by_name("notepad.exe")?;
println!("PID: {}", process.pid_or_default());
println!("HWND: {:?}", process.hwnd_or_default());
```

### 方法 2：使用 Builder（灵活配置）

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// 使用直观的方法构建配置
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()  // 最适合游戏
    .exclude_invisible()       // 跳过隐藏窗口
    .include_by_title("Game Window")  // 按标题过滤
    .build();

let mut process = Process::new(config);
process.init()?;
```

### 方法 3：通过 PID 初始化（多实例支持）

```rust
use win_auto_utils::process::Process;

// 当你知道具体的 PID 时
let mut process = Process::init_by_pid(12345)?;
println!("已连接到 PID: {}", process.pid_or_default());
```

### 方法 4：重新初始化现有进程

```rust
use win_auto_utils::process::Process;

// 创建但不初始化
let mut process = Process::by_name("app.exe");

// 稍后初始化
process.init()?;

// 切换到不同实例
process.init_with_pid(67890)?;
```

## DC 模式选项

根据你的需求选择合适的 DC 模式：

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// 选项 1：Standard - 完整窗口（标题栏 + 边框）
let config = ProcessConfig::builder("app.exe")
    .set_window_mode()
    .build();

// 选项 2：WindowClient - 仅客户区（推荐用于游戏）
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()
    .build();

// 选项 3：Desktop - 全屏桌面捕获
let config = ProcessConfig::builder("fullscreen_game.exe")
    .set_desktop_mode()
    .build();
```

## 窗口过滤

按标题或可见性过滤窗口：

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// 示例 1：排除不可见窗口
let config = ProcessConfig::builder("app.exe")
    .exclude_invisible()
    .build();

// 示例 2：按标题模式过滤（不区分大小写）
let config = ProcessConfig::builder("chrome.exe")
    .include_by_title("YouTube")
    .build();

// 示例 3：精确标题匹配（区分大小写）
let config = ProcessConfig::builder("notepad.exe")
    .include_by_exact_title("document.txt - Notepad")
    .build();

// 示例 4：组合过滤
let config = ProcessConfig::builder("game.exe")
    .set_window_client_mode()
    .exclude_invisible()
    .include_by_title("Main Window")
    .build();

let mut process = Process::new(config);
process.init()?;
```

## 自定义初始化标志（InitFlags）

通过 `InitFlags` 可以精细控制进程初始化时获取哪些系统资源(HWND、HANDLE、HDC),根据不同的使用场景选择合适的初始化策略:

```rust
use win_auto_utils::process::{Process, ProcessConfig, InitFlags};

// 策略 1：最简初始化（仅 PID，最低资源占用）
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::minimal())
    .build();

// 策略 2：仅内存操作（PID + HANDLE，用于内存读写）
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::memory_only())
    .build();

// 策略 3：仅 GUI 操作（PID + HWND + HDC，用于屏幕捕获）
let config = ProcessConfig::builder("app.exe")
    .init_flags(InitFlags::gui_only())
    .build();

// 策略 4：自定义配置（精细控制每个资源）
let custom_flags = InitFlags::new()
    .with_pid(true)
    .with_hwnd(true)
    .with_handle(false)  // 跳过进程句柄
    .with_dc(false);     // 跳过设备上下文

let config = ProcessConfig::builder("app.exe")
    .init_flags(custom_flags)
    .build();

let mut process = Process::new(config);
process.init()?;
```

### InitFlags 预设策略

| 策略 | PID | HWND | HANDLE | HDC | 适用场景 |
|------|-----|------|--------|-----|----------|
| `InitFlags::new()` | ✓ | ✓ | ✓ | ✓ | 完全访问（默认） |
| `InitFlags::minimal()` | ✓ | ✗ | ✗ | ✗ | 仅识别进程 |
| `InitFlags::memory_only()` | ✓ | ✗ | ✓ | ✗ | 仅内存读写 |
| `InitFlags::gui_only()` | ✓ | ✓ | ✗ | ✓ | 屏幕捕获/GUI 自动化 |

更多详情参见 [InitFlags 使用指南](../process_init_flags.md)。

## 进程管理器（多进程管理）

使用单个管理器管理多个进程：

```rust
use win_auto_utils::process::{Process, ProcessConfig, ProcessManager};

let mut manager = ProcessManager::new();

// 注册进程
manager.register("notepad.exe")?;
manager.register_alias("game", "target.exe")?;

// 使用不同策略初始化
manager.init("notepad.exe")?;
manager.init_with_pid("game", 12345)?;

// 查询进程（只读，不需要 mut）
if let Some(proc) = manager.get("notepad.exe") {
    println!("PID: {:?}", proc.pid());
}

// 列出所有管理的进程
for name in manager.list_processes() {
    if let Some(proc) = manager.get(&name) {
        println!("{}: PID={:?}", name, proc.pid());
    }
}
```

## 错误处理

优雅地处理常见错误：

```rust
use win_auto_utils::process::{Process, ProcessError};

match Process::init_by_name("nonexistent.exe") {
    Ok(mut process) => {
        println!("进程已初始化: PID={}", process.pid_or_default());
    }
    Err(ProcessError::ProcessNotFound(name)) => {
        eprintln!("未找到进程 '{}'", name);
    }
    Err(ProcessError::WindowNotFound(pid)) => {
        eprintln!("PID {} 未找到窗口", pid);
    }
    Err(e) => {
        eprintln!("初始化失败: {}", e);
    }
}
```

## 完整示例

### 示例 1：游戏自动化设置

```rust
use win_auto_utils::process::{Process, ProcessConfig};

// 为游戏捕获配置
let config = ProcessConfig::builder("target.exe")
    .set_window_client_mode()  // 仅客户区
    .exclude_invisible()       // 跳过最小化窗口
    .include_by_title("Game")  // 确保是正确的窗口
    .build();

let mut game = Process::new(config);
game.init()?;

println!("游戏 PID: {}", game.pid_or_default());
println!("游戏 HWND: {:?}", game.hwnd_or_default());
```

### 示例 2：多实例应用

```rust
use win_auto_utils::process::Process;
use win_auto_utils::snapshot::find_pids_by_name;

// 查找所有实例
let pids = find_pids_by_name("notepad.exe");
println!("找到 {} 个记事本实例", pids.len());

// 连接到每个实例
for pid in pids {
    let mut process = Process::init_by_pid(pid)?;
    println!("  PID {}: HWND={:?}", pid, process.hwnd_or_default());
}
```

### 示例 3：动态进程切换

```rust
use win_auto_utils::process::Process;

let mut app = Process::by_name("target.exe");

// 初始化第一个实例
app.init()?;
println!("第一个实例: PID={}", app.pid_or_default());

// 稍后切换到另一个实例
app.init_with_pid(67890)?;
println!("已切换到: PID={}", app.pid_or_default());
```

## 核心特性

✅ **直观的 API** - 无需记忆枚举，方法名自解释  
✅ **多种初始化方式** - 选择适合你的用例  
✅ **智能窗口过滤** - 按标题或可见性精确定位窗口  
✅ **多进程支持** - 轻松管理多个实例  
✅ **自动资源清理** - 通过 Drop trait 实现 RAII  
✅ **性能优化** - 延迟初始化，最小开销  

## 迁移指南（v0.1.x → v0.2.0）

### 旧 API（v0.1.x）
```rust
// ❌ 不再这样使用
let mut process = Process::new("app.exe");
process.dc_mode = DCMode::WindowClient;
process.hwnd_filter = Some(filters);
process.init()?;
```

### 新 API（v0.2.0）
```rust
// ✅ 改用这种方式
let config = ProcessConfig::builder("app.exe")
    .set_window_client_mode()
    .exclude_invisible()
    .include_by_title("App Window")
    .build();

let mut process = Process::new(config);
process.init()?;
```

### 主要变更
1. **配置不可变** - 使用 `ProcessConfig` builder
2. **无直接字段访问** - 所有配置通过 builder
3. **直观的方法名** - `.set_window_client_mode()` 替代 `.dc_mode(DCMode::WindowClient)`
4. **更好的错误处理** - 更具体的错误类型
5. **简化的导入** - 只需 `Process`、`ProcessConfig`、`ProcessManager`
