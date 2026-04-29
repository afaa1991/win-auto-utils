# 内存钩子模块 (Memory Hook)

[English](../../en/modules/memory_hook.md) | [返回概览](overview.md)

`memory_hook` 模块通过内联钩子和蹦床钩子提供高级函数拦截功能，允许你在运行时修改或监控程序执行。

## 功能特性

- **内联钩子**: 用自定义 shellcode 替换目标指令
- **蹦床钩子**: 在拦截调用的同时保留原始功能
- **寄存器提取**: 在钩子点自动捕获 CPU 寄存器值
- **内存锁定**: 保护内存区域免受修改
- **自动内存管理**: 基于 RAII 的清理机制防止内存泄漏

## 快速开始

### 基础内联钩子

```rust
use win_auto_utils::memory_hook::TrampolineHook;

// 目标: 拦截地址 0x12345678 处的函数
let handle = /* 进程句柄 */;
let target_addr = 0x12345678;

// Shellcode: add edx, edx (示例指令)
let shellcode = vec![0x01, 0xD2];

// 创建并安装钩子（x86 模式）
let mut hook = TrampolineHook::x86(handle, target_addr, shellcode);
hook.install()?;

// ... 通过执行目标代码触发钩子 ...

// 完成后卸载（自动释放内存）
hook.uninstall()?;
```

### x64 钩子

```rust
use win_auto_utils::memory_hook::TrampolineHook;

let mut hook = TrampolineHook::x64(handle, target_addr, shellcode);
hook.install()?;
// ... 使用钩子 ...
hook.uninstall()?;
```

### 寄存器提取器（高级 API）

在钩子触发时自动捕获寄存器值：

```rust
use win_auto_utils::memory_hook::register_extractor::{RegisterExtractor, Register};

// 从游戏指令中提取 ECX（玩家基址）和 EDX（MP 值）
let mut extractor = RegisterExtractor::builder()
    .handle(handle)
    .target_address(0x0041FAF2)
    .bytes_to_overwrite(6)
    .extract_register(Register::ECX)  // 玩家基址
    .extract_register(Register::EDX)  // MP 增量值
    .x86()
    .build()?;

extractor.install()?;

// 钩子触发后，读取捕获的值
let player_base: u32 = extractor.read_register::<u32>(Register::ECX)?;
let mp_delta: u32 = extractor.read_register::<u32>(Register::EDX)?;

println!("玩家基址: 0x{:08X}", player_base);
println!("正在添加的 MP: {}", mp_delta as i32);

extractor.uninstall()?;
```

## 核心概念

### 蹦床钩子工作流程

```
原始代码流程:
  [目标地址] → [下一条指令] → [继续执行...]

安装钩子后:
  [目标地址] → JMP 到 Detour → 执行自定义代码
                                      ↓
                              JMP 到 Trampoline → 执行原始指令
                                                   ↓
                                            JMP 回到原始流程
```

### 关键组件

1. **Detour 代码**: 替代原始代码执行的自定义 shellcode
2. **Trampoline**: 被覆盖指令的备份 + 跳回原始流程的跳转
3. **JMP 指令**: 在原始代码、detour 和 trampoline 之间重定向执行流

### 内存布局

```
目标进程内存:
┌─────────────────────┐
│  目标地址            │ ← 被覆盖为 JMP 到 Detour
│  (原始指令)          │
└─────────────────────┘

分配的内存:
┌─────────────────────┐
│  Detour 区域         │ ← 你的 shellcode + JMP 到 Trampoline
│  (RWX 权限)          │
└─────────────────────┘
┌─────────────────────┐
│  Trampoline 区域     │ ← 原始指令 + JMP 回去
│  (RX 权限)           │
└─────────────────────┘
```

## 高级用法

### 构建器模式进行精细控制

```rust
use win_auto_utils::memory_hook::TrampolineHookBuilder;

let mut hook = TrampolineHookBuilder::new()
    .handle(handle)
    .target_address(0x12345678)
    .detour_code(vec![0x01, 0xD2])
    .bytes_to_overwrite(5)
    .x86()
    .build()?;

hook.install()?;
```

### 跳过 Trampoline 模式（高级）

适用于不需要执行原始指令的场景：

```rust
let mut hook = TrampolineHookBuilder::new()
    .handle(handle)
    .target_address(0x12345678)
    .detour_code(your_complete_shellcode)
    .bytes_to_overwrite(5)
    .skip_trampoline(true)  // 无 trampoline，不返回原始代码
    .x86()
    .build()?;
```

**警告**: 在此模式下，你的 shellcode 必须处理所有逻辑，包括在需要时返回调用者。

### 内存锁定

保护内存区域免受外部修改：

```rust
use win_auto_utils::memory_hook::MemoryLock;

let lock = MemoryLock::new(handle, address, size)?;
lock.lock()?;  // 防止写入此区域

// ... 执行操作 ...

lock.unlock()?;  // 允许再次写入
```

## 使用场景

### 1. 游戏函数拦截

钩住伤害计算函数以实现"无敌模式"：

```rust
// 原始: sub [ecx+0x10], eax  (减少生命值)
// 钩子: NOP 掉减法指令
let nop_shellcode = vec![0x90, 0x90, 0x90, 0x90, 0x90];
let mut hook = TrampolineHook::x86(handle, damage_func_addr, nop_shellcode);
hook.install()?;
// 玩家现在不受伤害！
```

### 2. 资源监控

实时提取资源值（生命值、魔法值、体力值）：

```rust
let mut extractor = RegisterExtractor::builder()
    .handle(handle)
    .target_address(resource_update_addr)
    .bytes_to_overwrite(6)
    .extract_register(Register::ECX)  // 玩家对象
    .x86()
    .build()?;

extractor.install()?;

// 定期读取当前生命值
loop {
    let player_ptr: u32 = extractor.read_register::<u32>(Register::ECX)?;
    if player_ptr != 0 {
        let health: f32 = read_memory_t(handle, (player_ptr + 0x20) as usize)?;
        println!("当前生命值: {:.1}", health);
    }
    std::thread::sleep(Duration::from_millis(100));
}
```

### 3. 函数调用日志

记录特定函数的每次调用：

```rust
// 记录调用的 shellcode（简化示例）
let logging_shellcode = vec![
    0x60,           // PUSHAD (保存寄存器)
    0x9C,           // PUSHFD (保存标志)
    // ... 在这里调用你的日志函数 ...
    0x9D,           // POPFD
    0x61,           // POPAD
    // 原始指令会在这里
];

let mut hook = TrampolineHook::x86(handle, func_addr, logging_shellcode);
hook.install()?;
```

## 最佳实践

### 1. 选择正确的架构

```rust
// 对于 32 位进程
TrampolineHook::x86(handle, addr, code)

// 对于 64 位进程
TrampolineHook::x64(handle, addr, code)
```

### 2. 准确计算要覆盖的字节数

使用反汇编工具（IDA Pro、Ghidra、Cheat Engine）确定确切的指令长度：

```rust
// 示例: "add [ecx+0x308],edx" 是 6 字节
.bytes_to_overwrite(6)

// 常见 x86 指令大小:
// - JMP rel32: 5 字节
// - MOV reg, imm32: 5 字节
// - CALL rel32: 5 字节
// - PUSH reg: 1 字节
```

### 3. 始终卸载钩子

```rust
// 方法 1: 显式卸载
hook.uninstall()?;

// 方法 2: RAII（drop 时自动调用）
{
    let hook = TrampolineHook::x86(handle, addr, code);
    hook.install()?;
    // ... 使用钩子 ...
} // 这里自动调用 hook.uninstall()
```

### 4. 优雅地处理错误

```rust
match hook.install() {
    Ok(()) => println!("钩子安装成功"),
    Err(e) => eprintln!("安装钩子失败: {:?}", e),
}
```

## 性能考虑

- **钩子安装**: ~1-5ms（内存分配 + 修补）
- **钩子执行**: 每次触发 ~10-100ns（JMP 开销）
- **寄存器提取**: 最小开销（PUSHAD/POPAD ~50ns）
- **内存使用**: 每个钩子 ~4KB（detour + trampoline 区域）

## 常见陷阱

### ❌ 错误的字节计数

```rust
// 错误: 指令是 6 字节但你指定 5
.bytes_to_overwrite(5)  // 会破坏下一条指令！

// 正确: 匹配实际指令长度
.bytes_to_overwrite(6)
```

### ❌ 忘记恢复

```rust
// 不好: panic 后钩子仍然安装
hook.install()?;
do_something_that_might_panic()?;  // 如果 panic，钩子泄漏！

// 好: 使用 RAII 或显式错误处理
let result = (|| {
    hook.install()?;
    let res = do_something()?;
    hook.uninstall()?;
    Ok(res)
})();
```

### ❌ 错误的寄存器名称（x86 vs x64）

```rust
// x86: 使用 ECX, EDX 等
.extract_register(Register::ECX)  // ✓ 32 位正确

// x64: 使用 RCX, RDX 等
.extract_register(Register::RCX)  // ✓ 64 位正确
```

## 示例

查看这些示例文件获取完整实现：

- `_debug_lf2_register_extractor.rs`: x86 寄存器提取演示
- `memory_hook_reset.rs`: 钩子安装和清理
- `memory_operations.rs`: 带钩子的基础内存读写

运行示例：

```bash
cargo run --example memory_hook_reset --features "memory_hook"
cargo run --example _debug_lf2_register_extractor --features "memory_register_extractor process"
```

## 相关模块

- [`memory`](memory.md): 基础内存读写操作
- [`memory_resolver`](memory_resolver.md): 解析符号地址
- [`memory_aobscan`](memory_aobscan.md): 扫描字节模式

---

**语言**: [English](../../en/modules/memory_hook.md) | [中文](memory_hook.md)
