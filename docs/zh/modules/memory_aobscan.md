# 内存 AOB 扫描器 (Memory AOB Scanner - Array of Bytes)

[English](../../en/modules/memory_aobscan.md) | [返回概览](overview.md)

`memory_aobscan` 模块使用 SIMD 加速、基于 memchr 的启发式搜索和并行处理，在远程进程内存中提供高性能的模式扫描。它非常适合查找动态地址、函数签名和代码模式，而无需硬编码偏移量。

## Feature Flag

```
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["memory_aobscan"] }
```

## 快速开始

### 基础模式扫描

```
use win_auto_utils::memory_aobscan::AobScanBuilder;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // 获取进程句柄
    let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // 扫描带通配符的模式
    let results = AobScanBuilder::new(handle)
        .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
        .find_all(false)  // 仅查找第一个匹配（更快）
        .scan()?;
    
    if let Some(addr) = results.first() {
        println!("找到于: 0x{:X}", addr);
    }
    
    Ok(())
}
```

### 带范围限制的高级用法

```
use win_auto_utils::memory_aobscan::AobScanBuilder;

// 扫描特定内存范围
let results = AobScanBuilder::new(handle)
    .pattern_str("E8 ?? ?? ?? ??")?  // 调用指令模式
    .start_address(0x10000000)
    .length(0x1000000)  // 扫描 16MB
    .find_all(true)     // 查找所有匹配
    .scan()?;

println!("找到 {} 个匹配", results.len());
```

## 核心功能

- **双重 SIMD 加速**: AVX-512（64字节/周期）和 AVX2（32字节/周期）自动选择
- **memchr 优化**: 快速锚点字节搜索
- **并行扫描**: 通过 Rayon 进行多线程区域处理
- **智能过滤**: 仅扫描已提交、可读的内存区域
- **提前退出**: 找到第一个匹配时立即停止
- **通配符支持**: 使用 `??` 或 `?` 表示未知字节
- **区域缓存**: 缓存内存布局以进行重复扫描
- **多字节锚点**: 智能锚点序列选择

## 使用示例

### 示例 1: 查找函数签名

```
use win_auto_utils::memory_aobscan::AobScanBuilder;

// 通过机器码签名查找特定函数
let pattern = "55 48 8B EC 48 83 EC 30";  // 典型函数序言
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .find_all(false)
    .scan()?;

if let Some(func_addr) = results.first() {
    println!("函数找到于: 0x{:X}", func_addr);
}
```

### 示例 2: 扫描多个通配符

```
use win_auto_utils::memory_aobscan::AobScanBuilder;

// 带多个未知字节的模式（例如相对地址）
let pattern = "48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B D8";
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .find_all(true)
    .scan()?;

println!("找到 {} 个调用序列", results.len());
for addr in &results {
    println!("  - 0x{:X}", addr);
}
```

### 示例 3: 限制范围以提高性能

```
use win_auto_utils::memory_aobscan::AobScanBuilder;

// 仅扫描游戏模块（比完整内存快得多）
let module_base = 0x140000000;
let module_size = 0x5000000;  // 80MB

let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ??")?
    .start_address(module_base)
    .length(module_size)
    .find_all(true)
    .scan()?;

println!("在游戏模块中找到 {} 个匹配", results.len());
```

### 示例 4: 使用缓存的重复扫描

```
use win_auto_utils::memory_aobscan::{AobScanBuilder, clear_region_cache};

let mut scanner = AobScanBuilder::new(handle)
    .pattern_str("48 8B 05 ?? ?? ?? ??")?
    .find_all(false);

// 第一次扫描（构建缓存）
let result1 = scanner.clone().scan()?;

// 第二次扫描（使用缓存的区域 - 更快）
let result2 = scanner.clone().scan()?;

// 如果进程内存布局改变则清除缓存
clear_region_cache(pid);
```

### 示例 5: 查找数据模式

```
use win_auto_utils::memory_aobscan::AobScanBuilder;

// 搜索特定的数据值（例如生命值 = 100.0f32）
let health_bytes = 100.0f32.to_le_bytes();
let pattern_hex = format!(
    "{:02X} {:02X} {:02X} {:02X}",
    health_bytes[0], health_bytes[1],
    health_bytes[2], health_bytes[3]
);

let results = AobScanBuilder::new(handle)
    .pattern_str(&pattern_hex)?
    .find_all(true)
    .scan()?;

println!("找到 {} 个 100.0f32 实例", results.len());
```

### 示例 6: 错误处理

```
use win_auto_utils::memory_aobscan::{AobScanBuilder, AobScanError};

match AobScanBuilder::new(handle)
    .pattern_str("INVALID PATTERN")?
    .scan()
{
    Ok(results) => println!("找到 {} 个匹配", results.len()),
    Err(AobScanError::InvalidPattern(msg)) => {
        eprintln!("无效模式: {}", msg);
    }
    Err(AobScanError::MemoryReadFailed(addr)) => {
        eprintln!("在 0x{:X} 读取内存失败", addr);
    }
    Err(e) => eprintln!("扫描错误: {}", e),
}
```

## API 参考

### 主要类型

#### AobScanBuilder

用于配置和执行 AOB 扫描的构建器。

**构造函数**:
- `AobScanBuilder::new(handle: HANDLE)` - 创建新扫描器

**配置方法**:
- `pattern_str(pattern: &str) -> Result<Self, AobScanError>` - 从字符串设置模式
- `pattern(pattern: Pattern) -> Self` - 设置预解析的模式
- `start_address(addr: usize) -> Self` - 设置扫描起始地址
- `length(len: usize) -> Self` - 设置扫描长度（字节）
- `find_all(find_all: bool) -> Self` - 查找所有匹配或仅第一个
- `scan(self) -> Result<Vec<usize>, AobScanError>` - 执行扫描

#### Pattern

表示带通配符的已解析 AOB 模式。

**构造函数**:
- `Pattern::from_str(pattern: &str) -> Result<Self, AobScanError>` - 解析模式字符串

**方法**:
- `len(&self) -> usize` - 获取模式长度
- `has_wildcards(&self) -> bool` - 检查模式是否包含通配符

#### AobScanError

AOB 扫描操作的错误类型。

**变体**:
- `InvalidPattern(String)` - 模式语法错误
- `MemoryReadFailed(usize)` - 在地址处读取内存失败
- `NoReadableRegions` - 未找到有效的内存区域
- `ParseError(String)` - 模式解析失败

## 模式语法

### 格式

模式是空格分隔的十六进制字节，带有可选的通配符。

| 语法 | 示例 | 描述 |
|------|------|------|
| 十六进制字节 | `48` | 确切的字节值 |
| 通配符 | `??` 或 `?` | 任意字节值 |
| 混合 | `48 ?? 89` | 特定 + 通配符 |

### 示例

```
// 精确模式（无通配符）
"48 89 5C 24 10"

// 带通配符（用于可变地址/偏移）
"48 89 5C 24 ?? 48 89 6C 24 ??"

// 带相对偏移的调用指令
"E8 ?? ?? ?? ??"

// 混合特异性
"48 8B ?? ?? ?? ?? ?? 48 85 C0"
```

### 大小写不敏感

```
// 全部等价
"48 89 5C"
"48 89 5c"
"48 89 5C"
```

## 性能特征

### 不同配置的扫描速度

| 配置 | 典型速度 | 说明 |
|------|---------|------|
| 仅第一个匹配 | 10-50ms | 提前退出优化 |
| 所有匹配（小范围） | 50-200ms | < 100MB 扫描 |
| 所有匹配（完整内存） | 500ms-2s | 取决于 RAM 大小 |
| 缓存的第二次扫描 | 5-20ms | 区域缓存命中 |

### 优化技术

1. **锚点选择**: 选择最稀有的字节序列进行初始 memchr 搜索
2. **SIMD 验证**: 自动选择 AVX-512（64字节）或 AVX2（32字节）进行并行比较
3. **并行区域**: 将内存分成块进行多线程扫描
4. **预取**: 使用软件预取提示隐藏内存延迟
5. **区域过滤**: 跳过未提交/不可读的内存页

### 基准测试示例

```
use std::time::Instant;
use win_auto_utils::memory_aobscan::AobScanBuilder;

let start = Instant::now();
let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ??")?
    .find_all(true)
    .scan()?;
let elapsed = start.elapsed();

println!("扫描耗时 {:?}", elapsed);
println!("找到 {} 个匹配", results.len());
// 典型结果: 现代 CPU 上 1GB 范围需 50-200ms
```

## 最佳实践

1. **尽可能限制扫描范围**
   ```rust
   // 好: 仅扫描相关模块
   let results = AobScanBuilder::new(handle)
       .pattern_str(pattern)?
       .start_address(module_base)
       .length(module_size)
       .scan()?;
   
   // 不好: 扫描整个进程内存（慢）
   let results = AobScanBuilder::new(handle)
       .pattern_str(pattern)?
       .scan()?;
   ```

2. **单匹配时使用 find_all(false)**
   ```rust
   // 好: 第一个匹配后停止
   .find_all(false)
   
   // 不好: 不必要地继续扫描
   .find_all(true)  // 当你只需要一个结果时
   ```

3. **为重复扫描缓存区域布局**
   ```rust
   // 好: 重用相同句柄的扫描器
   let mut scanner = AobScanBuilder::new(handle).pattern_str(p1)?;
   let r1 = scanner.clone().scan()?;
   
   scanner = AobScanBuilder::new(handle).pattern_str(p2)?;
   let r2 = scanner.scan()?;  // 使用缓存的区域
   ```

4. **选择特定的模式**
   ```rust
   // 好: 更长、更具体的模式
   "48 89 5C 24 10 48 89 6C 24 18"
   
   // 不好: 短模式有很多误报
   "48 89"
   ```

5. **优雅地处理错误**
   ```rust
   match scanner.scan() {
       Ok(results) => use_results(results),
       Err(AobScanError::NoReadableRegions) => {
           eprintln!("进程可能已退出");
       }
       Err(e) => log_error(e),
   }
   ```

## 常见陷阱

### ❌ 使用过于通用的模式

```
// 错误: 太多匹配，扫描缓慢
let results = AobScanBuilder::new(handle)
    .pattern_str("90")?  // NOP - 出现数百万次
    .find_all(true)
    .scan()?;

// 正确: 更具体的模式
let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
    .find_all(true)
    .scan()?;
```

### ❌ 不限制扫描范围

```
// 错误: 扫描整个 64 位地址空间（非常慢）
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .scan()?;

// 正确: 限制到已知模块
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .start_address(0x140000000)
    .length(0x5000000)
    .scan()?;
```

### ❌ 忽略通配符影响

```
// 错误: 太多通配符降低性能
"?? ?? ?? ?? ?? ?? ?? ??"  // 几乎没有过滤

// 正确: 平衡特异性和灵活性
"48 89 ?? 24 ?? 48 89"  // 用于 memchr 的锚点字节
```

## 架构细节

### 三阶段扫描

1. **区域发现**
   - 使用 `VirtualQueryEx` 枚举内存区域
   - 过滤已提交、可读的页面
   - 缓存结果以进行重复扫描

2. **锚点搜索**
   - 从模式中选择最优字节序列
   - 使用 `memchr` 进行快速单字节搜索
   - 扩展到多字节验证

3. **模式验证**
   - 在候选位置验证完整模式
   - 使用 AVX-512（64字节）或 AVX2（32字节）进行并行比较
   - 对于短模式回退到带软件预取的标量实现
   - 软件预取隐藏内存延迟

### SIMD 加速

模块实现了三层验证策略：

- **AVX-512 指令**: 每周期处理 64 字节（最快，需要 AVX-512F 支持）
- **AVX2 指令**: 每周期处理 32 字节（快速，广泛支持）
- **标量回退**: 带软件预取的优化标量代码，适用于旧 CPU

**自动选择逻辑**：
1. 模式 ≥ 32 字节 + 检测到 AVX-512F → 使用 AVX-512
2. 模式 ≥ 16 字节 + 检测到 AVX2 → 使用 AVX2
3. 否则 → 使用优化的标量实现

这确保了在不同硬件代际上的最佳性能，同时保持兼容性。

## 相关模块

- [`memory_resolver`](memory_resolver.md): 将找到的地址解析为指针
- [`memory`](memory.md): 在找到的地址读/写值
- [`memory_hook`](memory_hook.md): Hook 找到的函数地址
- [`snapshot`](process_window.md): 枚举模块以获取扫描范围

---

**语言**: [English](../../en/modules/memory_aobscan.md) | [中文](memory_aobscan.md)
