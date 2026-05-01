# 内存管理器模块 (Memory Manager)

[English](../../en/modules/memory_manager.md) | [返回概览](overview.md)

`memory_manager` 模块提供了一个统一的接口来管理内存修改操作（锁定、钩子等），支持动态地址解析和进程上下文绑定。

## 功能特性

- **统一管理**: 集中管理所有内存修改器（Lock、Hook、BytesSwitch）
- **动态地址解析**: 支持静态地址和AOB模式扫描两种方式
- **进程上下文绑定**: 自动将修改器与目标进程关联
- **激活/停用控制**: 按需启用或禁用特定功能
- **RAII清理**: 自动释放资源，防止内存泄漏
- **批量操作**: 支持一次性激活或停用所有修改器

## 架构设计

```
ModifierManager
├── ProcessContext (handle + pid)
├── Handler Registry (HashMap<String, ModifierHandler>)
│   ├── LockHandler - 内存锁定
│   ├── BytesSwitchHandler - 字节切换
│   └── TrampolineHookHandler - 蹦床钩子
└── Lifecycle Management
    ├── activate() / deactivate()
    ├── activate_all() / deactivate_all()
    └── Drop (自动清理)
```

## 使用场景

- **游戏修改器**: 统一管理多个作弊功能（无限生命、无限魔法等）
- **调试工具**: 动态启用/禁用不同的监控点
- **自动化测试**: 按顺序测试各个功能模块
- **热重载**: 运行时切换不同的配置方案

## 快速开始

### 基本用法

``rust
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_manager::builtin::{LockHandler, BytesSwitchHandler};
use win_auto_utils::process::ProcessManager;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化目标进程
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("target_app.exe")?;
    process_mgr.init("target_app.exe")?;
    
    let proc = process_mgr.get("target_app.exe").unwrap();
    let handle = proc.handle().expect("No handle");
    let pid = proc.pid().unwrap();
    
    // 2. 创建管理器并绑定进程上下文
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    
    // 3. 注册内存锁定功能
    let value_lock = LockHandler::new_lock_x86_typed(
        "value_lock",
        "target_app.exe+0x12345",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    
    // 4. 激活功能
    manager.activate("value_lock")?;
    
    // ... 应用程序运行中 ...
    
    // 5. 停用功能
    manager.deactivate("value_lock")?;
    
    Ok(())
}
```

### 使用AOB模式扫描

``rust
use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
use win_auto_utils::memory_resolver::AddressSource;

// 通过字节模式动态查找地址
let shellcode = vec![0x90, 0x90]; // NOP指令
let hook_handler = TrampolineHookHandler::new_x86_skip_trampoline(
    "function_hook",
    AddressSource::from_pattern_x86("target_app.exe+1F9C9")?,
    shellcode,
    2,
);
manager.register("function_hook", hook_handler);
```

### 批量操作

``rust
// 激活所有已注册的修改器
manager.activate_all()?;

// 停用所有修改器
manager.deactivate_all()?;

// 检查某个修改器是否激活
if manager.is_active("value_lock") {
    println!("Value lock is enabled");
}

// 列出所有已注册的修改器
for name in manager.list_handlers() {
    println!("Registered: {}", name);
}
```

### 完整示例：多功能管理

``rust
use win_auto_utils::memory_manager::builtin::{
    BytesSwitchHandler, LockHandler, TrampolineHookHandler,
};
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_resolver::AddressSource;
use win_auto_utils::process::ProcessManager;
use std::io::{self, BufRead};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化目标进程
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("target_app.exe")?;
    process_mgr.init("target_app.exe")?;
    
    let proc = process_mgr.get("target_app.exe").unwrap();
    let handle = proc.handle().unwrap();
    let pid = proc.pid().unwrap();
    
    // 创建管理器
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    
    // 注册多个功能
    let value_lock = LockHandler::new_lock_x86_typed(
        "value_lock",
        "target_app.exe+0x1000",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    
    let nop_patch = BytesSwitchHandler::new_nop_switch_x86(
        "nop_patch",
        "target_app.exe+0x2000",
        2,
    )?;
    manager.register("nop_patch", nop_patch);
    
    let shellcode = vec![0x90, 0x90]; // NOP指令
    let func_hook = TrampolineHookHandler::new_x86_skip_trampoline(
        "func_hook",
        AddressSource::from_pattern_x86("target_app.exe+0x3000")?,
        shellcode,
        2,
    );
    manager.register("func_hook", func_hook);
    
    // 逐个测试功能
    let features = vec!["value_lock", "nop_patch", "func_hook"];
    
    for feature in features {
        println!("Testing: {}", feature);
        
        // 激活
        manager.activate(feature)?;
        println!("  ✓ Activated");
        
        // 等待用户测试
        println!("  → Press Enter to continue...");
        io::stdin().lock().read_line(&mut String::new())?;
        
        // 停用
        manager.deactivate(feature)?;
        println!("  ✓ Deactivated\n");
    }
    
    Ok(())
}
```

## API参考

### ModifierManager

#### 核心方法

- `new()` - 创建新的管理器实例
- `set_context(handle, pid)` - 绑定进程上下文
- `register(name, handler)` - 注册修改器
- `unregister(name)` - 注销并停用修改器
- `activate(name)` - 激活指定修改器
- `deactivate(name)` - 停用指定修改器
- `activate_all()` - 激活所有修改器
- `deactivate_all()` - 停用所有修改器
- `is_active(name)` - 检查修改器是否激活
- `list_handlers()` - 列出所有已注册的修改器名称

### 内置处理器 (Built-in Handlers)

#### LockHandler
用于持续监控和恢复内存值（冻结效果）。

**构造函数：**
- `new_lock_x86_typed(name, address, value, interval)` - x86静态地址
- `new_lock_x64_typed(name, address, value, interval)` - x64静态地址
- `new_lock_aob_typed(name, pattern, value, interval)` - AOB模式扫描

**示例：**
```rust
let handler = LockHandler::new_lock_x86_typed(
    "value_lock",
    "target_app.exe+0x1000",
    100i32,
    Duration::from_millis(100),
)?;
```

#### BytesSwitchHandler
用于字节码切换（NOP补丁等）。

**构造函数：**
- `new_nop_switch_x86(name, address, length)` - x86 NOP切换
- `new_nop_switch_x64(name, address, length)` - x64 NOP切换
- `new_custom_switch_x86(name, address, original_bytes, modified_bytes)` - 自定义切换

**示例：**
```rust
let handler = BytesSwitchHandler::new_nop_switch_x86(
    "nop_patch",
    "target_app.exe+0x2000",
    2,
)?;
```

#### TrampolineHookHandler
用于函数钩子，支持在拦截的同时保留原始功能。

**构造函数：**
- `new_x86_skip_trampoline(name, address_source, shellcode, bytes_to_overwrite)` - x86钩子
- `new_x64_skip_trampoline(name, address_source, shellcode, bytes_to_overwrite)` - x64钩子

**示例：**
```rust
let shellcode = vec![0x90, 0x90]; // NOP指令
let handler = TrampolineHookHandler::new_x86_skip_trampoline(
    "func_hook",
    AddressSource::from_static_x86("target_app.exe+0x3000")?,
    shellcode,
    2,
);
```

## 最佳实践

### 1. 进程生命周期管理

确保在进程关闭前停用所有修改器：

``rust
// 推荐：使用Drop自动清理
{
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);
    // ... 注册和激活 ...
} // manager.drop()会自动调用deactivate_all()
```

### 2. 错误处理

始终检查激活/停用操作的返回值：

``rust
match manager.activate("feature_name") {
    Ok(_) => println!("Feature activated"),
    Err(e) => eprintln!("Failed to activate: {}", e),
}
```

### 3. 命名规范

使用描述性的修改器名称：

``rust
// ✅ 好的命名
manager.register("value_monitor", monitor_handler);
manager.register("func_interceptor", interceptor_handler);

// ❌ 避免模糊命名
manager.register("hook1", handler1);
manager.register("mod2", handler2);
```

### 4. 隔离测试

按照用户偏好，逐个测试功能以避免状态干扰：

```rust
for feature in &features {
    manager.activate(feature)?;
    // 测试该功能
    manager.deactivate(feature)?;
}
```

## 常见问题

### Q: 如何处理进程重启？

A: 重新初始化进程后，需要重新设置上下文：

```rust
process_mgr.reinit("target_app.exe")?;
let proc = process_mgr.get("target_app.exe").unwrap();
manager.set_context(proc.handle().unwrap(), proc.pid().unwrap());
// 重新激活需要的功能
manager.activate_all()?;
```

### Q: 可以动态添加新的修改器吗？

A: 是的，随时可以注册新的修改器：

```rust
let new_handler = LockHandler::new_lock_x86_typed(...)?;
manager.register("new_feature", new_handler);
manager.activate("new_feature")?;
```

### Q: 如何验证修改器是否生效？

A: 使用`is_active()`方法检查状态：

```rust
if manager.is_active("hp_lock") {
    println!("HP lock is currently active");
}
```

## 性能考虑

- **轻量级**: 管理器本身开销极小，仅维护HashMap
- **按需激活**: 未激活的修改器不消耗CPU资源
- **后台线程**: LockHandler使用独立线程，不影响主线程性能
- **批量操作**: `activate_all()`会串行执行，适合初始化阶段

## 相关模块

- [`memory_hook`](memory_hook.md) - 底层钩子实现
- [`memory_lock`](memory_lock.md) - 内存锁定功能
- [`memory_resolver`](memory_resolver.md) - 地址解析
- [`process`](process_window.md) - 进程管理

---

**语言**: [English](../../en/modules/memory_manager.md) | [中文](memory_manager.md)
