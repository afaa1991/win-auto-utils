# 内存地址解析器 (Memory Address Resolver)

[English](../../en/modules/memory_resolver.md) | [返回概览](overview.md)

`memory_resolver` 模块将符号内存地址（如 `"game.exe+123->456->789"`）解析为实际运行时地址。它自动解析模块基址、偏移量和指针链，采用简洁的语法，默认使用十六进制（无需 `0x` 前缀）。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["memory_resolver"] }
```

## 快速开始

### 解析简单地址

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// 解析 x86（32位）地址 - 默认十六进制，不需要 0x！
let addr_str = "game.exe+1234";
let memory_addr = MemoryAddress::new_x86(addr_str)?;

// 解析为实际地址
let handle = /* 进程句柄 */;
let pid = /* 进程 ID */;
let resolved = memory_addr.resolve_address(handle, pid)?;

println!("解析后的地址: 0x{:X}", resolved);
```

### 解析指针链

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// 简洁语法：十六进制值不需要 0x 前缀
let addr_str = "game.exe+5000->10->20->30";
let memory_addr = MemoryAddress::new_x64(addr_str)?;

let resolved = memory_addr.resolve_address(handle, pid)?;
println!("最终地址: 0x{:X}", resolved);
```

## 核心功能

- **简洁十六进制语法**: 默认十六进制，无需 `0x` 前缀（`game.exe+123` = 0x123）
- **十进制支持**: 使用 `#` 前缀表示十进制（`#1000` = 十进制 1000）
- **多级指针**: 解析如 `base->offset1->offset2` 的链
- **架构支持**: 同时支持 x86（32位）和 x64（64位）
- **可读数字**: 下划线分隔符提高清晰度（`1_0000` = 0x10000）
- **错误处理**: 无效格式的清晰错误消息
- **构建器模式**: 复杂配置的可选构建器

## 使用示例

### 示例 1: 基础解析（默认十六进制）

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // 获取进程句柄
    let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // 默认十六进制 - 不需要 0x 前缀！
    let addr = MemoryAddress::new_x86("client.dll+ABCD")?;
    let resolved = addr.resolve_address(handle, pid)?;
    
    // 在解析的地址读取值
    use win_auto_utils::memory::read_memory_i32;
    let value = read_memory_i32(handle, resolved)?;
    println!("值: {}", value);
    
    Ok(())
}
```

### 示例 2: 多级指针链

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::memory::read_memory_f32;

// 玩家生命值: [[[game.exe+1000]+20]+30]
// 全部默认十六进制，不需要 0x
let health_addr = MemoryAddress::new_x64("game.exe+1000->20->30")?;
let health_ptr = health_addr.resolve_address(handle, pid)?;

let health: f32 = read_memory_f32(handle, health_ptr)?;
println!("玩家生命值: {:.1}", health);
```

### 示例 3: 十进制与十六进制对比

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// 十六进制（默认）- 不需要前缀
let addr1 = MemoryAddress::parse("game.exe+1000")?;  // = 0x1000 = 十进制 4096

// 带 # 前缀的十进制
let addr2 = MemoryAddress::parse("#1000")?;          // = 十进制 1000 = 0x3E8

// 混合使用
let addr3 = MemoryAddress::parse("target.exe+100->#50")?;  // 十六进制 100，然后十进制 50
```

### 示例 4: 动态地址更新

```rust
use win_auto_utils::memory_resolver::MemoryAddress;
use win_auto_utils::memory::read_memory_i32;

// 地址可能在游戏重启后改变
let addr = MemoryAddress::new_x86("game.exe+5000->10")?;

// 每次需要地址时重新解析
loop {
    let current_addr = addr.resolve_address(handle, pid)?;
    let value: i32 = read_memory_i32(handle, current_addr)?;
    println!("当前值: {}", value);
    
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```

### 示例 5: 构建器模式

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// 对复杂配置使用构建器
let addr = MemoryAddress::builder()
    .address("lf2.exe+58C94->308")
    .x86()
    .build()?;

let resolved = addr.resolve_address(handle, pid)?;
```

### 示例 6: 可读数字格式

```rust
use win_auto_utils::memory_resolver::MemoryAddress;

// 下划线提高可读性（解析时忽略）
let addr1 = MemoryAddress::parse("game.exe+1_0000->2FC")?;  // = 0x10000->0x2FC
let addr2 = MemoryAddress::parse("#10_000")?;                // = 十进制 10000
```

## API 参考

### 主要类型

- **`MemoryAddress`**: 表示符号或已解析的地址
  - 包含解析的操作（模块基址、偏移量、指针跳转）
  - 架构感知（x86 vs x64 指针大小）
  
- **`ParseError`**: 地址解析错误
  - `EmptyInput` - 提供了空字符串
  - `InvalidHex(String)` - malformed 十六进制
  - `InvalidDecimal(String)` - malformed 十进制
  - `InvalidPointerSyntax(String)` - 缺少 '->' 运算符
  - `MultipleModules` - 超过一个模块名称
  
- **`ResolveError`**: 地址解析错误
  - `ModuleNotFound(String)` - 在进程中未找到模块
  - `PointerReadFailed(usize, MemoryError)` - 解引用指针失败

### 构造函数

- **`MemoryAddress::new_x86(address_str: &str) -> Result<Self, ParseError>`**
  - 解析 x86（32位）地址字符串
  - 使用 4 字节指针进行解引用
  
- **`MemoryAddress::new_x64(address_str: &str) -> Result<Self, ParseError>`**
  - 解析 x64（64位）地址字符串
  - 使用 8 字节指针进行解引用
  
- **`MemoryAddress::parse(address_str: &str) -> Result<Self, ParseError>`**
  - 使用默认架构解析（匹配编译的二进制文件）

### 构建器

- **`MemoryAddress::builder() -> MemoryAddressBuilder`**
  - 为复杂配置创建构建器
  - 方法：`.address()`, `.x86()`, `.x64()`, `.build()`

### 方法

- **`resolve_address(handle: HANDLE, pid: u32) -> Result<usize, ResolveError>`**
  - 将符号地址解析为实际运行时地址
  - 从进程读取模块基址
  - 如果存在则跟随指针链
  - 返回最终绝对地址
  
- **`get_operations(&self) -> &[AddressOp]`**
  - 获取解析的地址操作
  - 对调试或自定义解析逻辑有用

## 地址语法

| 语法 | 示例 | 描述 |
|------|------|------|
| 模块 + 偏移 | `game.exe+1000` | 模块基址 + 十六进制偏移（默认） |
| 绝对地址 | `7FF6A1B2C3D4` | 直接十六进制地址（不需要 0x） |
| 十进制 | `#1000` | 十进制数（前缀 `#`） |
| 直接偏移 | `+100` | 添加十六进制偏移而不解引用 |
| 指针跳转 | `->20` | 解引用然后添加十六进制偏移 |
| 下划线 | `1_0000` | 可读性分隔符（= 0x10000） |

### 对比：旧语法 vs 新语法

```rust
// ❌ 旧的冗长风格（不再需要）
"game.exe+0x1000->0x20->0x30"

// ✅ 新的简洁风格（推荐）
"game.exe+1000->20->30"

// 两者功能完全相同！
```

### 复杂示例

```rust
// 模块相对地址带指针链（全部十六进制）
"app.dll+1000->20->30"

// 绝对地址带直接偏移
"7FF6A1B2C3D4+100"

// 十进制数
"#1000+#200"

// 混合语法
"target.exe+1_0000->2FC"

// 真实示例（LF2 游戏）
"lf2.exe+58C94->308"
```

## 最佳实践

1. **使用简洁的十六进制语法（无 0x 前缀）**
   ```rust
   // 推荐：简洁明了
   let addr = MemoryAddress::new_x86("game.exe+1000->20")?;
   
   // 也可以但更冗长
   let addr = MemoryAddress::new_x86("game.exe+0x1000->0x20")?;
   ```

2. **尽可能缓存已解析的地址**
   ```rust
   // 不好: 每次都解析
   for _ in 0..100 {
       let addr = memory_addr.resolve_address(handle, pid)?;
       let val = read_memory_i32(handle, addr)?;
   }
   
   // 好: 解析一次，重复使用
   let addr = memory_addr.resolve_address(handle, pid)?;
   for _ in 0..100 {
       let val = read_memory_i32(handle, addr)?;
   }
   ```

3. **处理解析失败**
   ```rust
   match addr.resolve_address(handle, pid) {
       Ok(resolved) => use_address(resolved),
       Err(ResolveError::ModuleNotFound(name)) => {
           eprintln!("模块 '{}' 未找到", name);
       }
       Err(ResolveError::PointerReadFailed(addr, err)) => {
           eprintln!("在 0x{:X} 读取指针失败: {}", addr, err);
       }
   }
   ```

4. **验证模块名称**
   ```rust
   // 解析前确保模块存在
   use win_auto_utils::snapshot::enumerate_modules;
   
   let modules = enumerate_modules(pid)?;
   if !modules.iter().any(|m| m.name == "game.exe") {
       eprintln!("未找到模块！");
       return;
   }
   ```

5. **选择正确的架构**
   ```rust
   // 对于 32 位进程（如 LF2 等旧游戏）
   let addr = MemoryAddress::new_x86("game.exe+1000")?;
   
   // 对于 64 位进程（现代应用）
   let addr = MemoryAddress::new_x64("game.exe+1000")?;
   ```

## 常见陷阱

### ❌ 十进制未使用 # 前缀

```rust
// 错误：这被解释为十六进制 1000（= 十进制 4096）
let addr = MemoryAddress::parse("1000")?;  // 如果你想要十进制 1000

// 正确：使用 # 表示十进制
let addr = MemoryAddress::parse("#1000")?;  // = 十进制 1000
let addr = MemoryAddress::parse("1000")?;   // = 十六进制 0x1000（= 十进制 4096）
```

### ❌ 忘记指定架构

```rust
// 错误: 对 x64 进程使用 x86 解析器
let addr = MemoryAddress::new_x86("game.exe+1000")?;
// 指针解引用将读取错误的大小！

// 正确: 匹配进程架构
let addr = MemoryAddress::new_x64("game.exe+1000")?;
```

### ❌ 无效的指针链语法

```rust
// 错误: 单个 '>' 而不是 '->'
let addr = MemoryAddress::parse("game.exe+1000>20")?; // 错误！

// 正确: 使用 '->' 进行指针跳转
let addr = MemoryAddress::parse("game.exe+1000->20")?;
```

## 性能考虑

- **单次解析**: ~50-200μs（包括模块枚举）
- **缓存解析**: <1μs（如果模块基址外部缓存）
- **指针链**: 每层增加 ~5-10μs 用于内存读取
- **建议**: 频繁访问时缓存模块基址

### 优化提示

```rust
// 如果重复解析同一模块，缓存基址
use win_auto_utils::snapshot::get_module_base;

let module_base = get_module_base(pid, "game.exe")?;

// 然后手动计算以加快重复访问
let final_addr = module_base + 0x1000;
```

## 相关模块

- [`memory`](memory.md): 在解析的地址读/写值
- [`memory_aobscan`](memory_aobscan.md): 通过扫描动态查找地址
- [`snapshot`](../process_window.md): 枚举进程模块
- [`handle`](../process_window.md): 打开和管理进程句柄

---

**语言**: [English](../../en/modules/memory_resolver.md) | [中文](memory_resolver.md)
