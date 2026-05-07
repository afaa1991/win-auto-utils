# 核心模块概览

[English](../../en/modules/overview.md) | [返回 README](../../../docs/zh/README.md)

本文档提供 win-auto-utils 所有模块的高层概览，按功能层级组织。

## 模块分类

### 1. 进程与窗口层

用于管理 Windows 进程和窗口的模块。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`process`](process_window.md) | `process` | 进程管理、句柄聚合、PID 查找 |
| [`hwnd`](process_window.md) | `hwnd` | 通过类名、标题、PID 查询窗口句柄 |
| [`window`](process_window.md) | `window` | 窗口操控（显示/隐藏、移动、调整大小）、枚举 |
| [`snapshot`](process_window.md) | `snapshot` | ToolHelp32 API 用于进程/模块枚举 |
| [`handle`](process_window.md) | `handle` | 进程句柄管理（以适当权限打开/关闭） |
| [`hdc`](process_window.md) | `hdc` | 设备上下文管理（获取/释放 DC） |

**使用场景：**
- 按名称查找进程并获取其 PID
- 枚举父窗口的所有子窗口
- 以编程方式显示/隐藏/最小化窗口
- 管理进程句柄并正确清理

---

### 2. 输入层

键盘和鼠标输入模拟。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`keyboard`](input.md) | `keyboard` | 通过 SendInput 或 PostMessage 进行键盘输入 |
| [`mouse`](input.md) | `mouse` | 鼠标控制（点击、移动、滚动）通过 SendInput 或 PostMessage |

**支持的方法：**
- **SendInput**: 前台窗口输入（模拟真实硬件）
- **PostMessage**: 后台窗口输入（无需焦点）

**使用场景：**
- 自动化 UI 交互
- 游戏机器人开发（支持后台模式）
- 辅助工具

---

### 3. 图形层

屏幕捕获和颜色处理。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`dxgi`](dxgi.md) | `dxgi` | 使用 DXGI API 的高性能屏幕捕获 |
| [`color_picker`](dxgi.md) | `color_picker` | 基于 GDI 的屏幕坐标颜色拾取 |
| [`color_finder`](dxgi.md) | `color_finder` | 纯 Rust 实现的像素颜色搜索算法 |
| [`template_matcher`](template_matcher.md) | `template_matcher` | 基于图像的 UI 元素检测，支持并行处理 |

**使用场景：**
- 以 60+ FPS 捕获游戏画面
- 在屏幕上查找特定颜色
- 通过模板图像匹配检测 UI 元素
- 构建视觉自动化机器人

---

### 4. 内存层

进程内存操作和高级 manipulation。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`memory`](memory.md) | `memory` | 基础内存读写操作 |
| [`memory_resolver`](memory_resolver.md) | `memory_resolver` | 符号地址解析（例如 "game.exe+0x123->456"） |
| [`memory_aobscan`](memory_aobscan.md) | `memory_aobscan` | 字节数组模式扫描，支持 SIMD 加速 |
| [`memory_hook`](memory_hook.md) | `memory_hook` | 内联钩子和蹦床钩子，用于函数拦截 |
| [`memory_lock`](memory_lock.md) | `memory_lock` | 持续监控和恢复内存值（冻结效果） |
| [`memory_manager`](memory_manager.md) | `memory_manager` | 统一的内存修改管理器，支持动态地址解析 |
| [`memory_register_extractor`](memory_hook.md) | `memory_register_extractor` | 在钩子点自动捕获 CPU 寄存器值 |

**使用场景：**
- 读写游戏变量（生命值、弹药、坐标）
- 从基址指针解析动态地址
- 扫描未知值的内存（AOB 扫描）
- 钩住函数以拦截调用或修改行为
- 统一管理多个内存修改功能（推荐使用memory_manager）
- 提取寄存器值用于逆向工程

---

### 5. 高级功能

用于复杂场景的专用工具。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`dll_injector`](dll_injector.md) | `dll_injector` | 支持跨架构的 DLL 注入/卸载 |
| [`clipboard`](memory.md) | `clipboard` | 系统 clipboard 读写操作 |

**使用场景：**
- 向目标进程注入自定义 DLL
- 在运行时修改进程行为
- 通过系统剪贴板交换数据

---

### 6. 脚本引擎

纯 Rust 实现的自动化脚本解释器。

| 模块 | Feature Flag | 描述 |
|------|-------------|------|
| [`script_engine`](script_engine.md) | `script_engine` | 核心解释器，包含解析、编译和执行 |
| [`scripts_builtin`](script_engine.md) | `scripts_builtin` | 内置指令集（控制流、键盘、鼠标、定时） |

**内置指令：**
- **控制流**: `loop`, `break`, `continue`, 条件跳转
- **键盘**: `key`, `key_down`, `key_up`
- **鼠标**: `click`, `move`, `moverel`, `scroll`
- **定时**: `sleep`（毫秒精度）
- **窗口**: `active`（通过 HWND 激活窗口）
- **模式**: `script_mode`（动态配置切换）

**使用场景：**
- 编写自动化脚本而无需重新编译
- 为游戏或应用程序构建宏系统
- 创建可配置的自动化工作流

---

## 架构图

```
┌─────────────────────────────────────────────┐
│         应用层                                │
│  (使用 win-auto-utils 的代码)                 │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│         脚本引擎层                            │
│  - 解析器 → 编译器 → 虚拟机                   │
│  - 内置指令                                   │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│       功能模块层                              │
│                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ 进程管理  │ │ 输入控制  │ │ 图形处理  │    │
│  └──────────┘ └──────────┘ └──────────┘    │
│  ┌──────────┐ ┌──────────┐                 │
│  │ 内存操作  │ │ 高级功能  │                 │
│  └──────────┘ └──────────┘                 │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│      Windows API 层 (windows crate)          │
│  - Win32 APIs                               │
│  - DXGI, Direct3D                           │
└─────────────────────────────────────────────┘
```

## Feature 依赖图

```
core (默认)
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
    └── template_matcher (重量级: image + imageproc + rayon)
```

## 选择合适的模块

### 应用程序自动化
```toml
features = [
    "process",        # 查找目标进程
    "memory",         # 读写进程状态
    "memory_manager", # 统一管理内存修改
    "keyboard",       # 发送按键
    "mouse",          # 控制鼠标
    "dxgi",           # 捕获屏幕画面
]
```

### UI 测试
```toml
features = [
    "window",         # 操控应用程序窗口
    "keyboard",       # 输入文本
    "mouse",          # 点击按钮
    "template_matcher", # 验证 UI 元素
    "script_engine",  # 编写测试脚本
]
```

### 逆向工程
```toml
features = [
    "memory",              # 读取进程内存
    "memory_aobscan",      # 扫描模式
    "memory_resolver",     # 解析地址
    "memory_manager",      # 统一管理钩子和锁定
    "memory_hook",         # 钩住函数
    "memory_register_extractor", # 捕获寄存器
    "dll_injector",        # 注入调试 DLL
]
```

### 简单宏
```toml
features = [
    "script_engine",       # 运行脚本
    "scripts_builtin",     # 使用内置命令
    "keyboard",            # 键盘输入
    "mouse",               # 鼠标控制
]
```

## 下一步

- 探索各个模块的详细文档
- 查看 `examples/` 目录中的示例代码
- 查阅 [Cargo.toml](../../../Cargo.toml) 中的 feature flags

---

**语言**: [English](../../en/modules/overview.md) | [中文](overview.md)
