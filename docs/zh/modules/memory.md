# 内存操作 (Memory Operations)

[English](../../en/modules/memory.md) | [返回概览](overview.md)

`memory` 模块提供类型安全的进程内存读写函数。它支持原始类型（i32、f32、u64 等）和原始字节数组，并带有适当的错误处理。

## Feature Flag

在 `Cargo.toml` 中启用此模块：

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["memory"] }
```

## 快速开始

### 从内存读取值

```rust
use win_auto_utils::memory::read_memory_i32;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let address = 0x7FF6A1B2C3D4;
    
    // 获取进程句柄
    let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // 读取 i32 值
    let value = read_memory_i32(handle, address)?;
    println!("读取的值: {}", value);
    
    Ok(())
}
```

### 向内存写入字节

```rust
use win_auto_utils::memory::write_memory_bytes;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let address = 0x7FF6A1B2C3D4;
    let data = vec![0x90, 0x90, 0x90]; // NOP 指令
    
    let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    write_memory_bytes(handle, address, &data)?;
    println!("写入了 {} 字节", data.len());
    
    Ok(())
}
```

## 核心功能

- **类型安全读取**: 通用 `read_memory_t<T>()` 适用于任何 Copy 类型
- **便捷函数**: 针对常见类型的专用函数（i32、f32、u64 等）
- **字节数组支持**: 读写原始字节向量
- **错误处理**: 返回带有描述性 `MemoryError` 类型的 `Result`
- **Windows API 集成**: 使用 `ReadProcessMemory` 和 `WriteProcessMemory`

## 使用示例

### 示例 1: 读取不同类型

```rust
use win_auto_utils::memory::{
    read_memory_t, read_memory_i32, read_memory_f32, read_memory_u64
};

// 通用方法 - 适用于任何 Copy 类型
let health: f32 = read_memory_t::<f32>(handle, health_addr)?;
let ammo: i32 = read_memory_t::<i32>(handle, ammo_addr)?;
let player_ptr: u64 = read_memory_t::<u64>(handle, ptr_addr)?;

// 便捷函数 - 更易读
let health = read_memory_f32(handle, health_addr)?;
let ammo = read_memory_i32(handle, ammo_addr)?;
let player_ptr = read_memory_u64(handle, ptr_addr)?;
```

### 示例 2: 写入值

```rust
use win_auto_utils::memory::{write_memory_t, write_memory_bytes};

// 写入浮点值（将生命值设为 999.0）
write_memory_t::<f32>(handle, health_addr, 999.0)?;

// 写字节（NOP 掉一条指令）
let nop_bytes = vec![0x90, 0x90, 0x90, 0x90, 0x90];
write_memory_bytes(handle, target_addr, &nop_bytes)?;

// 写入多个值
write_memory_t::<i32>(handle, ammo_addr, 999)?;
write_memory_t::<u64>(handle, ptr_addr, 0x123456789ABCDEF)?;
```

### 示例 3: 读取字节数组

```rust
use win_auto_utils::memory::read_memory_bytes;

// 读取 16 字节用于签名验证
let signature = read_memory_bytes(handle, base_addr, 16)?;
println!("签名: {:02X?}", signature);

// 与预期模式验证
if signature == vec![0x48, 0x89, 0x5C, 0x24, 0x10, 0x48, 0x89, 0x74] {
    println!("签名匹配！");
}
```

### 示例 4: 错误处理

```rust
use win_auto_utils::memory::{read_memory_i32, MemoryError};

match read_memory_i32(handle, suspicious_addr) {
    Ok(value) => println!("值: {}", value),
    Err(MemoryError::ReadFailed(e)) => eprintln!("读取失败: {}", e),
    Err(MemoryError::InvalidAddress(msg)) => eprintln!("无效地址: {}", msg),
    Err(e) => eprintln!("未知错误: {:?}", e),
}
```

## API 参考

### 主要类型

- **`MemoryError`**: 内存操作的错误枚举
  - `ReadFailed(String)` - ReadProcessMemory 失败
  - `WriteFailed(String)` - WriteProcessMemory 失败
  - `InvalidAddress(String)` - 空指针或无效地址

### 核心函数

#### 读取内存

- **`read_memory_t<T: Copy>(handle: HANDLE, address: usize) -> Result<T, MemoryError>`**
  - 适用于任何 Copy 类型的通用读取
  - 最灵活的选项
  
- **`read_memory_i32(handle, address)`** - 读取 32 位有符号整数
- **`read_memory_f32(handle, address)`** - 读取 32 位浮点数
- **`read_memory_u64(handle, address)`** - 读取 64 位无符号整数
- **`read_memory_bytes(handle, address, size)`** - 读取字节数组

#### 写入内存

- **`write_memory_t<T: Copy>(handle: HANDLE, address: usize, value: T) -> Result<(), MemoryError>`**
  - 适用于任何 Copy 类型的通用写入
  
- **`write_memory_bytes(handle, address, bytes: &[u8])`** - 写入字节数组

### 辅助函数

- **`is_valid_address(address: usize) -> bool`** - 检查地址是否非空
- **`bytes_to_hex(bytes: &[u8]) -> String`** - 将字节转换为十六进制字符串

## 最佳实践

1. **使用适当的访问权限**
   ```rust
   // 仅读取
   let access = PROCESS_VM_READ;
   
   // 读取和写入
   let access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
   ```

2. **操作前验证地址**
   ```rust
   if !is_valid_address(addr) {
       eprintln!("检测到空地址");
       return;
   }
   ```

3. **正确关闭句柄**
   ```rust
   use windows::Win32::Foundation::CloseHandle;
   
   unsafe { CloseHandle(handle); }
   // 或使用 RAII 包装器
   ```

4. **优雅地处理错误**
   ```rust
   match read_memory_i32(handle, addr) {
       Ok(val) => process_value(val),
       Err(e) => log_error(e),
   }
   ```

5. **为清晰起见使用类型特定函数**
   ```rust
   // 比通用方法更易读
   let health = read_memory_f32(handle, addr)?;
   
   // vs
   let health: f32 = read_memory_t(handle, addr)?;
   ```

## 常见陷阱

### ❌ 使用无效句柄

```rust
// 错误: 没有适当权限的句柄
let handle = open_process_handle(pid, PROCESS_QUERY_INFORMATION)?;
read_memory_i32(handle, addr)?; // 会失败！

// 正确: 包含 VM_READ
let handle = open_process_handle(pid, PROCESS_VM_READ)?;
read_memory_i32(handle, addr)?; // 正常工作！
```

### ❌ 不检查返回值

```rust
// 错误: 忽略错误
let value = read_memory_i32(handle, addr).unwrap(); // 可能 panic！

// 正确: 处理错误
let value = read_memory_i32(handle, addr)?; // 传播错误
```

### ❌ 读取未初始化的内存

```rust
// 错误: 地址可能未分配
let value = read_memory_i32(handle, 0x0)?; // 空指针！

// 正确: 先验证
if is_valid_address(addr) {
    let value = read_memory_i32(handle, addr)?;
}
```

## 性能考虑

- **单次读/写**: 每次操作 ~1-5μs（取决于目标进程状态）
- **批量操作**: 分组多次读/写以减少系统调用开销
- **大数组**: 使用 `read_memory_bytes()` 而不是多个类型化读取
- **频繁访问**: 如果值不经常变化，考虑缓存值

### 基准测试示例

```rust
use std::time::Instant;

let start = Instant::now();
for _ in 0..1000 {
    let _ = read_memory_i32(handle, addr)?;
}
let elapsed = start.elapsed();
println!("1000 次读取耗时 {:?}", elapsed);
// 典型结果: 1000 次读取耗时 2-5ms
```

## 相关模块

- [`memory_resolver`](memory_resolver.md): 解析符号地址如 "game.exe+0x123"
- [`memory_hook`](memory_hook.md): 拦截和修改函数执行
- [`memory_aobscan`](memory_aobscan.md): 扫描内存中的字节模式
- [`handle`](../process_window.md): 打开和管理进程句柄

---

**语言**: [English](../../en/modules/memory.md) | [中文](memory.md)
