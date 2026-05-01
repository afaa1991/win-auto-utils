# 内存锁定模块 (Memory Lock)

[English](../../en/modules/memory_lock.md) | [返回概览](overview.md)

`memory_lock` 模块提供持续监控和恢复内存值的功能，适用于游戏修改器中的"冻结"效果或保持目标进程中的恒定值。

## 功能特性

- **持续监控**: 后台线程定期检查内存值
- **自动恢复**: 检测到变化时立即写回锁定值
- **动态地址支持**: 支持指针链解析，适应对象重建场景
- **灵活配置**: Builder 模式支持静态/动态地址
- **RAII 管理**: 自动清理后台线程，防止资源泄漏

## 快速开始

### 基础用法

```rust
use win_auto_utils::memory_lock::MemoryLock;
use windows::Win32::Foundation::HANDLE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = HANDLE::default(); // 替换为实际句柄
    let address = 0x7FF6A1B2C3D4;
    
    // 锁定一个值（例如：生命值 = 100）
    let mut lock = MemoryLock::builder()
        .handle(handle)
        .address(address)
        .value(100u32)
        .build()?;
    
    lock.lock_value(100u32)?; // 立即开始锁定
    
    // 值会持续恢复到 100
    // ... 你的代码在这里 ...
    
    // 当 `lock` 离开作用域时自动停止
    // 或者手动释放: drop(lock);
    
    Ok(())
}
```

### 动态地址解析

```rust
use win_auto_utils::memory_lock::MemoryLock;
use win_auto_utils::memory_resolver::MemoryAddress;
use windows::Win32::Foundation::HANDLE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    let handle = HANDLE::default(); // 替换为实际句柄
    
    // 使用动态解析的指针链
    let addr = MemoryAddress::new_x86("game.exe+58C94->308")?;
    
    let mut lock = MemoryLock::builder()
        .pid(pid)
        .handle(handle)
        .address_from_resolver(addr)
        .value(500u32)
        .build()?;
    
    lock.lock_value(500u32)?; // 开始锁定
    // 每次检查周期都会重新解析地址
    
    Ok(())
}
```

### 锁定原始字节

```rust
use win_auto_utils::memory_lock::MemoryLock;
use windows::Win32::Foundation::HANDLE;

let handle = HANDLE::default();
let address = 0x1000;

// 直接锁定字节序列
let bytes = vec![0x90, 0x90, 0x90]; // NOP 指令
let mut lock = MemoryLock::builder()
    .handle(handle)
    .address(address)
    .bytes(bytes.clone())
    .build()?;

lock.lock_bytes(&bytes)?;
```

## 核心概念

### 工作原理

```
主线程:                    后台监控线程:
lock.lock_value(100)  →   while !stop_flag {
                            sleep(scan_interval)
                            current = read_memory(address)
                            if current != locked_value {
                                write_memory(address, locked_value)
                            }
                          }
```

### 关键组件

1. **Builder 模式**: 分阶段配置锁定参数
2. **后台线程**: 独立的监控循环，不阻塞主线程
3. **原子标志**: `Arc<AtomicBool>` 用于安全停止线程
4. **地址源抽象**: 支持静态地址和动态解析

### 扫描间隔调优

```rust
let mut lock = MemoryLock::builder()
    .handle(handle)
    .address(0x1000)
    .value(100u32)
    .scan_interval_ms(5)  // 每 5ms 检查一次（默认 10ms）
    .build()?;

lock.lock_value(100u32)?;
```

**建议值**：
- **游戏数值**: 5-20ms（平衡性能和响应速度）
- **关键数据**: 1-5ms（更紧密的锁定）
- **低频更新**: 50-100ms（降低 CPU 占用）

## 高级用法

### 延迟绑定模式

预配置静态参数，运行时再绑定动态参数：

```rust
use win_auto_utils::memory_lock::MemoryLock;

// 步骤 1: 预配置静态参数
let builder = MemoryLock::builder()
    .address(0x7FF6A1B2C3D4)
    .scan_interval_ms(5);

// ... 等待进程启动 ...
let handle = open_process("game.exe")?;

// 步骤 2: 绑定动态参数并启动
let mut lock = builder.clone()
    .handle(handle)
    .value(100u32)
    .build()?;

lock.lock_value(100u32)?;
```

### 容错模式（瞬态错误处理）

对于游戏加载、场景切换等临时不可用的情况：

```rust
// MemoryLock 内部已实现容错机制
// - 初始解析失败不会阻塞
// - 后台线程会持续重试直到地址有效
// - 适合 JIT 编译、堆分配等动态环境

let addr = MemoryAddress::new_x86("game.exe+ptr_chain")?;
let mut lock = MemoryLock::builder()
    .pid(pid)
    .handle(handle)
    .address_from_resolver(addr)
    .value(100u32)
    .build()?;

// 即使当前地址无效，也会启动监控
// 后台线程会在地址可用时自动开始工作
lock.lock_value(100u32)?;
```

### 程序重启后的重置

```rust
// 第一次使用
let mut lock = MemoryLock::builder()
    .handle(handle1)
    .address(addr1)
    .value(100u32)
    .build()?;
lock.lock_value(100u32)?;
lock.unlock()?;

// 程序重启后 - 重置状态
lock.reset();

// 第二次使用（无副作用）
let mut lock = MemoryLock::builder()
    .handle(new_handle)
    .address(new_addr)
    .value(100u32)
    .build()?;
lock.lock_value(100u32)?;
```

## 使用场景

### 1. 游戏数值冻结

保持玩家生命值、魔法值不变：

```rust
// 无限生命值
let health_lock = MemoryLock::builder()
    .handle(handle)
    .address(player_health_addr)
    .value(9999u32)
    .build()?;
health_lock.lock_value(9999u32)?;

// 无限魔法值
let mana_lock = MemoryLock::builder()
    .handle(handle)
    .address(player_mana_addr)
    .value(9999u32)
    .build()?;
mana_lock.lock_value(9999u32)?;
```

### 2. 时间/计数器锁定

冻结游戏计时器或倒计时：

```rust
// 冻结倒计时
let timer_lock = MemoryLock::builder()
    .handle(handle)
    .address(timer_addr)
    .value(300u32)  // 保持 300 秒
    .scan_interval_ms(1)  // 快速响应
    .build()?;
timer_lock.lock_value(300u32)?;
```

### 3. 动态对象属性保护

保护堆上分配的对象属性（如玩家结构体）：

```rust
// 使用指针链动态解析玩家基址
let player_base_addr = MemoryAddress::new_x86("game.exe+PlayerPtr->Offset")?;

let mut lock = MemoryLock::builder()
    .pid(pid)
    .handle(handle)
    .address_from_resolver(player_base_addr)
    .value(100u32)
    .build()?;

lock.lock_value(100u32)?;
// 即使玩家对象重建，也能自动跟踪新地址
```

## 最佳实践

### 1. 选择合适的扫描间隔

```rust
// 高频更新（实时战斗数值）
.scan_interval_ms(1)

// 中频更新（常规资源）
.scan_interval_ms(10)  // 默认值

// 低频更新（静态配置）
.scan_interval_ms(50)
```

### 2. 优先使用动态地址

对于堆分配的对象，始终使用指针链而非硬编码地址：

```rust
// ❌ 不好: 硬编码地址（重启后失效）
.address(0x12345678)

// ✅ 好: 动态解析（自动适应重启）
.address_from_resolver(MemoryAddress::new_x86("game.exe+Ptr->Offset")?)
```

### 3. 优雅地处理生命周期

```rust
// 方法 1: RAII（推荐）
{
    let lock = MemoryLock::builder()
        .handle(handle)
        .address(addr)
        .value(100u32)
        .build()?;
    lock.lock_value(100u32)?;
    // ... 使用锁定 ...
} // 这里自动停止后台线程

// 方法 2: 显式控制
let mut lock = /* ... */;
lock.lock_value(100u32)?;
// ... 使用锁定 ...
drop(lock); // 显式停止
```

### 4. 批量管理多个锁定

```rust
struct GameTrainer {
    health_lock: Option<MemoryLock>,
    mana_lock: Option<MemoryLock>,
    stamina_lock: Option<MemoryLock>,
}

impl GameTrainer {
    fn enable_all(&mut self, handle: HANDLE, pid: u32) -> Result<(), Box<dyn Error>> {
        self.health_lock = Some(/* 创建生命值锁定 */);
        self.mana_lock = Some(/* 创建魔法值锁定 */);
        self.stamina_lock = Some(/* 创建体力值锁定 */);
        Ok(())
    }
    
    fn disable_all(&mut self) {
        self.health_lock.take();  // drop 自动停止
        self.mana_lock.take();
        self.stamina_lock.take();
    }
}
```

## 性能考虑

- **CPU 占用**: 每个锁定的后台线程约 0.1-0.5% CPU（取决于扫描间隔）
- **内存开销**: 每个锁定约 1-2 KB（线程栈 + 状态数据）
- **延迟**: 从值被修改到恢复的时间 ≈ 扫描间隔
- **并发**: 多个锁定可并行运行，互不干扰

## 常见陷阱

### ❌ 忘记设置 PID（动态地址必需）

```rust
// 错误: 使用动态地址但未设置 PID
let addr = MemoryAddress::new_x86("game.exe+Ptr")?;
let lock = MemoryLock::builder()
    .handle(handle)
    .address_from_resolver(addr)
    // .pid(pid)  ← 缺失！
    .value(100u32)
    .build()?;  // 构建失败

// 正确: 动态地址必须提供 PID
.address_from_resolver(addr)
.pid(pid)  // ✓ 必需
```

### ❌ 扫描间隔过短导致 CPU 占用过高

```rust
// 不好: 1ms 间隔可能导致高 CPU 占用
.scan_interval_ms(1)

// 好: 根据实际需求选择
.scan_interval_ms(10)  // 大多数场景足够
```

### ❌ 在锁定时修改 Builder

```rust
// 错误: build() 后不能再修改
let mut lock = builder.value(100u32).build()?;
// builder.value(200u32)  ← 无效，需要重新 build

// 正确: 解锁后重新创建
lock.unlock()?;
let new_lock = builder.value(200u32).build()?;
```

## 示例

查看这些示例文件获取完整实现：

- `memory_lock_reset.rs`: 锁定安装和清理演示
- `_debug_lf2_register_extractor.rs`: 结合寄存器提取的动态锁定

运行示例：

```bash
cargo run --example memory_lock_reset --features "memory_lock"
```

## 相关模块

- [`memory`](memory.md): 基础内存读写操作
- [`memory_resolver`](memory_resolver.md): 解析符号地址
- [`memory_hook`](memory_hook.md): 内联钩子和函数拦截

---

**语言**: [English](../../en/modules/memory_lock.md) | [中文](memory_lock.md)
