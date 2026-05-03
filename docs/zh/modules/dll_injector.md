# DLL 注入器 (DLL Injector)

[English](../../en/modules/dll_injector.md) | [返回概览](overview.md)

`dll_injector` 模块提供向远程进程注入和卸载 DLL 的功能，并调用已注入 DLL 的导出函数。它通过 WOW64 兼容层支持跨架构注入（x64 注入器 → x86/x64 目标）。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.3", features = ["dll_injector"] }
```

**平台**: 仅 Windows

## 快速开始

### 基础 DLL 注入

```rust
use win_auto_utils::dll_injector::inject_dll;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 通过 PID 将 DLL 注入进程
    let pid = 12345;
    inject_dll(pid, "C:\\path\\to\\library.dll")?;
    println!("DLL 注入成功！");
    Ok(())
}
```

### 调用导出函数

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_with_raw_bytes
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // 获取函数地址
    let func_addr = get_exported_function_address(
        pid, 
        "game_mod.dll", 
        "set_health"
    )?;

    // 使用 i32 参数调用（999 生命值）
    let health: i32 = 999;
    let param_bytes = health.to_ne_bytes();
    let result = call_function_with_raw_bytes(
        pid, 
        func_addr, 
        Some(&param_bytes)
    )?;

    println!("函数返回值: {}", result);
    Ok(())
}
```

## 核心功能

- **跨架构支持**: x64 注入器可以注入到 x86 (WOW64) 目标
- **函数调用**: 带参数调用任意导出函数
- **卸载支持**: 动态卸载已注入的 DLL
- **诊断工具**: 注入前可行性检查
- **Shellcode 生成**: 架构特定的 shellcode（x86/x64）
- **类型安全**: 通过 `.to_ne_bytes()` 进行参数序列化

## 使用示例

### 示例 1: 带错误处理的 DLL 注入

```rust
use win_auto_utils::dll_injector::{inject_dll, DllInjectorError};

fn inject_with_retry(pid: u32, dll_path: &str) -> Result<(), DllInjectorError> {
    match inject_dll(pid, dll_path) {
        Ok(_) => {
            println!("DLL 注入成功");
            Ok(())
        }
        Err(e) => {
            eprintln!("注入失败: {}", e);
            
            // 延迟后重试一次
            std::thread::sleep(std::time::Duration::from_secs(1));
            inject_dll(pid, dll_path)
        }
    }
}
```

### 示例 2: 卸载 DLL

```rust
use win_auto_utils::dll_injector::unload_dll;

let pid = 12345;

// 卸载之前注入的 DLL
unload_dll(pid, "game_mod.dll")?;
println!("DLL 卸载成功");

// 注意: 由于引用计数，系统代理 DLL 可能无法工作
```

### 示例 3: 调用带多个参数的函数

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_with_raw_bytes
};

let pid = 12345;

// 获取带多个参数的函数
let func_addr = get_exported_function_address(
    pid,
    "my_lib.dll",
    "update_player_stats"
)?;

// 将结构体序列化为字节
#[repr(C)]
struct PlayerStats {
    health: i32,
    mana: i32,
    level: u32,
}

let stats = PlayerStats {
    health: 999,
    mana: 500,
    level: 99,
};

// 转换为字节（本机字节序）
let param_bytes = unsafe {
    std::slice::from_raw_parts(
        &stats as *const PlayerStats as *const u8,
        std::mem::size_of::<PlayerStats>(),
    )
};

let result = call_function_with_raw_bytes(
    pid,
    func_addr,
    Some(param_bytes)
)?;

println!("更新结果: {}", result);
```

### 示例 4: 调用无参数函数

```rust
use win_auto_utils::dll_injector::{
    get_exported_function_address,
    call_function_no_params
};

let pid = 12345;

// 获取无参数函数
let func_addr = get_exported_function_address(
    pid,
    "my_lib.dll",
    "reset_game_state"
)?;

// 无参数调用（更高效）
let result = call_function_no_params(pid, func_addr)?;
println!("重置结果: {}", result);
```

### 示例 5: 注入前诊断

```rust
use win_auto_utils::dll_injector::diagnose_injection;

let pid = 12345;

// 检查注入是否可行
match diagnose_injection(pid) {
    Ok(report) => {
        println!("进程可访问: {}", report.process_accessible);
        println!("内存可分配: {}", report.memory_allocatable);
        println!("写入权限: {}", report.write_permissions);
        
        if report.can_inject() {
            inject_dll(pid, "mod.dll")?;
        } else {
            eprintln!("无法注入: {:?}", report);
        }
    }
    Err(e) => eprintln!("诊断失败: {}", e),
}
```

### 示例 6: 跨架构注入

```rust
use win_auto_utils::dll_injector::inject_dll;

// x64 注入器 targeting x86 进程（WOW64）
let x86_pid = 12345; // 32位进程
inject_dll(x86_pid, "C:\\mods\\library_x86.dll")?;

// x64 注入器 targeting x64 进程
let x64_pid = 67890; // 64位进程
inject_dll(x64_pid, "C:\\mods\\library_x64.dll")?;
```

## 示例代码

查看这些示例文件了解使用演示：

- `dll_injection.rs`

运行示例：

```bash
cargo run --example dll_injection --features "dll_injector"
```

## API 参考

### 主要函数

#### 核心注入

- **`inject_dll(pid: u32, dll_path: &str) -> Result<(), DllInjectorError>`**
  - 使用 LoadLibraryW + CreateRemoteThread 注入 DLL
  - 支持 x64→x64、x86→x86、x64→x86 (WOW64)
  - 如果注入失败则返回错误

- **`unload_dll(pid: u32, module_name: &str) -> Result<(), DllInjectorError>`**
  - 使用 FreeLibrary + CreateRemoteThread 卸载 DLL
  - 系统代理 DLL 可能失败（引用计数）
  - 代理 DLL 优先使用配置开关

#### 远程函数调用

- **`get_exported_function_address(pid: u32, module_name: &str, function_name: &str) -> Result<usize, DllInjectorError>`**
  - 解析远程进程中导出函数的地址
  - 通过 shellcode 使用 GetProcAddress
  - 返回函数的虚拟地址

- **`call_function_with_raw_bytes(pid: u32, function_address: usize, param_data: Option<&[u8]>) -> Result<i32, DllInjectorError>`**
  - 使用原始字节参数调用远程函数
  - 最灵活的 API - 支持任何参数类型
  - 参数通过 `.to_ne_bytes()` 序列化

- **`call_function_no_params(pid: u32, function_address: usize) -> Result<i32, DllInjectorError>`**
  - 调用无参数远程函数
  - 比通用版本更高效
  - 用于 `void function()` 签名

#### 诊断

- **`diagnose_injection(pid: u32) -> Result<InjectionReport, DllInjectorError>`**
  - 注入前执行可行性检查
  - 检查进程可访问性、内存分配、写入权限
  - 返回详细的诊断报告

### 错误类型

#### DllInjectorError

DLL 注入操作的错误类型。

**变体**:
- `ProcessOpenFailed(u32)` - 无法打开进程句柄
- `MemoryAllocationFailed` - VirtualAllocEx 失败
- `MemoryWriteFailed` - WriteProcessMemory 失败
- `ThreadCreationFailed` - CreateRemoteThread 失败
- `ModuleNotFound(String)` - 未找到目标模块
- `FunctionNotFound(String)` - 未找到导出函数
- `ArchitectureMismatch` - 注入器/目标架构不兼容
- `DiagnosticFailed(String)` - 注入前检查失败

#### InjectionReport

诊断报告结构体。

**字段**:
- `process_accessible: bool` - 可以打开进程句柄
- `memory_allocatable: bool` - 可以在目标中分配内存
- `write_permissions: bool` - 具有写入权限
- `is_wow64: bool` - 目标是 WOW64 进程
- `target_architecture: String` - "x86" 或 "x64"

**方法**:
- `can_inject(&self) -> bool` - 检查注入是否可行

## 架构兼容性矩阵

| 注入器架构 | 目标进程 | DLL 架构 | 状态 | 说明 |
|-----------|---------|---------|------|------|
| x64 | x64 | x64 | ✅ 支持 | 标准情况 |
| x86 | x86 | x86 | ✅ 支持 | 标准情况 |
| x64 | x86 (WOW64) | x86 | ✅ 支持 | 通过 WOW64 层 |
| x86 | x64 | x64 | ❌ 不可能 | 无法从 32 位访问 64 位 |

## 最佳实践

1. **始终在注入前诊断**
   ```rust
   // 好: 先检查可行性
   let report = diagnose_injection(pid)?;
   if report.can_inject() {
       inject_dll(pid, "mod.dll")?;
   }
   
   // 不好: 盲目注入
   inject_dll(pid, "mod.dll")?;  // 可能意外失败
   ```

2. **使用正确的 DLL 架构**
   ```rust
   // 对于 x86 目标进程
   inject_dll(x86_pid, "mod_x86.dll")?;
   
   // 对于 x64 目标进程
   inject_dll(x64_pid, "mod_x64.dll")?;
   ```

3. **小心处理代理 DLL**
   ```rust
   // 系统代理 DLL 可能无法干净地卸载
   unload_dll(pid, "version.dll")?;  // 可能失败
   
   // 更好: 使用配置而非卸载
   // 配置代理 DLL 以禁用功能
   ```

4. **正确序列化参数**
   ```rust
   // 好: 本机字节序
   let value: i32 = 42;
   let bytes = value.to_ne_bytes();
   
   // 避免: 手动字节排序（容易出错）
   let bytes = [42, 0, 0, 0];  // 假设小端序！
   ```

5. **注入后清理**
   ```rust
   // 注入、使用、然后卸载
   inject_dll(pid, "temp_mod.dll")?;
   use_mod_features(pid)?;
   unload_dll(pid, "temp_mod.dll")?;  // 清理
   ```

## 常见陷阱

### ❌ 架构不匹配

```rust
// 错误: x64 DLL 注入到 x86 进程
inject_dll(x86_pid, "mod_x64.dll")?;  // 会失败！

// 正确: 匹配架构
inject_dll(x86_pid, "mod_x86.dll")?;
```

### ❌ 不检查返回值

```rust
// 错误: 忽略函数调用结果
call_function_with_raw_bytes(pid, addr, Some(&bytes))?;

// 正确: 检查结果
let result = call_function_with_raw_bytes(pid, addr, Some(&bytes))?;
if result != 0 {
    eprintln!("函数返回错误码: {}", result);
}
```

### ❌ 让 DLL 保持加载

```rust
// 错误: 从不卸载临时 DLL
inject_dll(pid, "debug_mod.dll")?;
// ... 使用它 ...
// 忘记卸载 - 内存泄漏！

// 正确: 始终清理
inject_dll(pid, "debug_mod.dll")?;
// ... 使用它 ...
unload_dll(pid, "debug_mod.dll")?;
```

## 安全考虑

- ⚠️ **需要管理员权限**: 某些进程可能需要提升的权限
- ⚠️ **反作弊检测**: 游戏反作弊系统可能检测到注入
- ⚠️ **进程稳定性**: 恶意 DLL 可能导致目标进程崩溃
- ⚠️ **法律合规**: 确保您有权修改目标进程

## 性能特征

| 操作 | 典型时间 |
|------|---------|
| 进程打开 | 1-2ms |
| 内存分配 | 1-3ms |
| DLL 注入 | 10-50ms |
| 函数地址解析 | 5-15ms |
| 远程函数调用 | 2-5ms |
| DLL 卸载 | 10-30ms |

## 相关模块

- [`process_window`](process_window.md): 按名称查找目标进程
- [`memory_hook`](memory_hook.md): 代码修改的 DLL 注入替代方案
- [`snapshot`](process_window.md): 枚举目标进程中加载的模块

---

**语言**: [English](../../en/modules/dll_injector.md) | [中文](dll_injector.md)
