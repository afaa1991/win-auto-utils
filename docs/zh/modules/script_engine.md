# 脚本引擎 (Script Engine)

[English](../../en/modules/script_engine.md) | [返回概览](overview.md)

`script_engine` 模块提供一个轻量级、可扩展的脚本执行引擎，采用三阶段流水线：解析 → 编译 → 执行。它具有基于寄存器的虚拟机、标签解析和可选的生命周期钩子以进行编译时分析。核心完全由纯 Rust 编写，零外部依赖。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["script_engine"] }
```

**平台**: 跨平台（纯 Rust 实现）

## 快速开始

### 基础脚本执行

```rust
use win_auto_utils::script_engine::ScriptEngine;

// 创建内置指令的引擎
let engine = ScriptEngine::with_builtin();

// 执行简单脚本
engine.compile_and_execute(r#"
    sleep 10
"#)?;

println!("脚本执行成功！");
```

### 带内置指令的脚本

```rust
use win_auto_utils::script_engine::{InstructionRegistry, ScriptConfig, ScriptEngine};

// 创建自定义注册表并注册一些指令
let mut registry = InstructionRegistry::new();

// 使用辅助函数注册所有内置指令
win_auto_utils::scripts_builtin::register_all(&mut registry);

let engine = ScriptEngine::with_registry_and_config(registry, ScriptConfig::default());

// 带循环和变量的脚本
let script = r#"
    loop 3 {
        sleep 50
    }
"#;

engine.compile_and_execute(script)?;
println!("循环完成！");
```

## 核心功能

- **三阶段流水线**: 解析 → 编译 → 执行
- **基于寄存器的 VM**: 高效的状态管理
- **标签解析**: 支持跳转和循环
- **生命周期钩子**: 可选的编译时分析
- **纯 Rust**: 零外部依赖，跨平台
- **可扩展**: 通过 trait 实现自定义指令处理器
- **中断控制**: 暂停/恢复/停止执行
- **错误处理**: 带行号的详细错误消息

## 使用示例

### 示例 1: 条件逻辑

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// 带条件分支的脚本
let script = r#"
    loop 3
        sleep 100
    end
"#;

engine.compile_and_execute(script)?;
```

### 示例 2: 嵌套循环

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// 嵌套循环
let script = r#"
    loop 2
        loop 3
            sleep 10
        end
    end
"#;

engine.compile_and_execute(script)?;
```

### 示例 3: 中断控制

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// 在后台启动脚本
engine.compile_and_execute_async(r#"
    loop 100 {
        sleep 100
    }
"#)?;

// 2秒后暂停（需要保存 controller 引用）
std::thread::sleep(std::time::Duration::from_secs(2));
// controller.pause()?;

// 稍后恢复
// controller.resume()?;

// 或完全停止
// controller.stop()?;
```

### 示例 4: 自定义配置

```rust
use win_auto_utils::script_engine::{InstructionRegistry, ScriptConfig, ScriptEngine};

let config = ScriptConfig::default();

let mut registry = InstructionRegistry::new();
win_auto_utils::scripts_builtin::register_all(&mut registry);

let engine = ScriptEngine::with_registry_and_config(registry, config);
engine.compile_and_execute(your_script)?;
```

### 示例 5: 错误处理

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

match engine.compile_and_execute("sleep 10") {
    Ok(_) => println!("成功"),
    Err(e) => eprintln!("错误: {}", e),
}
```

### 示例 6: 简单循环

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// 使用循环进行重复操作
let script = r#"
    loop 5
        sleep 10
    end
"#;

engine.compile_and_execute(script)?;
```

## API 参考

### 主要类型

#### ScriptEngine

脚本执行的主要入口点。

**构造函数**:
- `ScriptEngine::with_builtin()` - 创建带有内置指令的引擎
- `ScriptEngine::with_registry_and_config(registry, config)` - 使用自定义注册表和配置创建

**方法**:
- `compile_and_execute(script: &str) -> Result<(), ScriptError>` - 同步编译并执行脚本
- `compile_and_execute_async(script: &str) -> Result<(), ScriptError>` - 在后台线程编译并执行
- `get_interrupt_controller() -> InterruptController` - 获取控制句柄
- `get_vm_context() -> VMContext` - 访问 VM 状态

#### ScriptConfig

脚本引擎行为的配置（使用默认配置通常足够）。

**字段**:
- `max_loop_iterations: u64` - 最大循环迭代次数（防止无限循环）
- `enable_debug_logging: bool` - 启用详细日志记录
- `timeout_ms: Option<u64>` - 执行超时（毫秒）
- `stack_size: usize` - VM 栈大小

#### InterruptController

控制正在运行的脚本执行。

**方法**:
- `pause() -> Result<(), ScriptError>` - 暂停执行
- `resume() -> Result<(), ScriptError>` - 从暂停恢复
- `stop() -> Result<(), ScriptError>` - 永久停止执行
- `is_running() -> bool` - 检查脚本是否正在运行

#### ScriptError

脚本操作的错误类型。

**变体**:
- `ParseError { line: usize, message: String }` - 解析期间的语法错误
- `CompileError { message: String }` - 编译/验证错误
- `RuntimeError { instruction: String, message: String }` - 运行时执行错误
- `TimeoutError` - 执行超过超时时间
- `Interrupted` - 脚本被手动中断

### 内置指令

内置指令是通过 `ScriptEngine::with_builtin()` 自动可用的简化指令集。

#### 控制流

| 指令 | 语法 | 描述 |
|------|------|------|
| `loop` | `loop N { ... }` 或 `loop N ... end` | 重复 N 次 |
| `time` | `time MS { ... }` 或 `time MS ... end` | 在时间限制内循环 |
| `sleep` | `sleep MS` | 等待毫秒数 |
| `continue` | `continue` | 跳过当前循环剩余迭代 |
| `break` | `break` | 退出当前循环 |

#### 指令组合示例

```rust
// 简单循环
loop 3
    sleep 100
end

// 时间限制循环
time 500
    sleep 100
end

// 带 continue 和 break
loop 10
    sleep 50
    continue    // 跳过后面的代码
    sleep 100   // 这行不会执行
end
```

## 脚本语法

### 基础语法

脚本使用简单的指令序列：

```rust
sleep 100        // 等待 100 毫秒
loop 5           // 重复 5 次
    sleep 50
end
```

### 时间限制循环

`time` 指令创建一个在指定时间内运行的循环：

```rust
time 500         // 运行最多 500 毫秒
    sleep 100
end              // 约执行 5 次
```

### 循环控制

- `continue` - 跳过当前迭代的剩余指令
- `break` - 立即退出循环

## 架构细节

### 三阶段流水线

1. **解析阶段** (`parser` 模块)
   - 标记化脚本文本
   - 转换为指令序列
   - 验证基本语法

2. **编译阶段** (`compiler` 模块)
   - 解析标签和跳转目标
   - 验证控制流
   - 优化指令顺序
   - 通过生命周期钩子执行静态分析

3. **执行阶段** (`vm` 模块)
   - 运行编译后的指令
   - 管理寄存器状态
   - 处理中断
   - 报告运行时错误

### 生命周期钩子

指令可以实现可选的生命周期方法：

```rust
trait InstructionHandler {
    // 必需
    fn parse(&self, tokens: &[Token]) -> Result<InstructionData>;
    fn execute(&self, vm: &mut VM, data: &InstructionData);
    
    // 可选（提供默认实现）
    fn compile(&self, data: &mut CompiledInstruction) { }
    fn validate(&self, data: &InstructionData) -> Result<()> { Ok(()) }
}
```

### 基于寄存器的 VM

VM 使用命名寄存器而非栈：

```
寄存器:
  $x, $y, $z          - 通用
  $counter, $index    - 循环计数器
  $temp1, $temp2      - 临时存储
  $result             - 操作结果
```

## 最佳实践

1. **使用 `with_builtin()` 创建引擎**
   ```rust
   // 推荐：使用内置指令
   let engine = ScriptEngine::with_builtin();

   // 或自定义注册表
   let mut registry = InstructionRegistry::new();
   win_auto_utils::scripts_builtin::register_all(&mut registry);
   let engine = ScriptEngine::with_registry_and_config(registry, ScriptConfig::default());
   ```

2. **使用 `compile_and_execute()` 执行脚本**
   ```rust
   // 好：编译并执行
   engine.compile_and_execute("sleep 100\n    continue\n    sleep 100\nend")?;

   // 旧 API（已弃用）
   // engine.execute_script(...);
   ```

3. **设置合理的循环限制**
   ```rust
   // 好：有限循环
   loop 100 {
       sleep 10
   }
   ```

4. **使用 time 指令进行超时控制**
   ```rust
   // 好：带超时
   time 5000
       sleep 100
   end
   ```

5. **使用 continue 和 break 控制循环**
   ```rust
   loop 100
       sleep 50
       continue    // 跳到下次迭代
       sleep 100   // 不会执行
   end
   ```

## 常见陷阱

### ❌ 使用花括号语法（新版不支持）

```rust
// 错误：使用花括号
loop 3 {
    sleep 100
}

// 正确：使用 end 关键字
loop 3
    sleep 100
end
```

### ❌ 使用旧版 API

```rust
// 错误：旧版 API
let engine = ScriptEngine::new();
engine.execute_script("...");

// 正确：新版 API
let engine = ScriptEngine::with_builtin();
engine.compile_and_execute("...")?;
```

### ❌ 忘记 end 关键字

```rust
// 错误：缺少 end
loop 3
    sleep 100
// 缺少 end

// 正确：闭合代码块
loop 3
    sleep 100
end
```

## 扩展引擎

### 注册自定义指令

```rust
use win_auto_utils::script_engine::{Instruction, InstructionHandler, InstructionRegistry, VM};

struct MyInstruction;

impl InstructionHandler for MyInstruction {
    fn parse(&self, tokens: &[&str]) -> Result<Instruction, String> {
        Ok(Instruction::new("my_instruction"))
    }

    fn execute(&self, _vm: &mut VM, _instruction: &Instruction) -> Result<(), String> {
        println!("执行自定义指令");
        Ok(())
    }
}

// 注册到引擎
let mut registry = InstructionRegistry::new();
registry.register("my_instruction", Box::new(MyInstruction));
```

## 性能特征

| 操作 | 典型时间 |
|------|---------|
| 脚本解析 | 1-5ms |
| 编译 | 2-10ms |
| 简单指令执行 | <1μs |
| 循环迭代 | 1-5μs |
| 上下文切换（异步） | 10-50μs |

## 相关模块

- [`keyboard`](input.md): 键盘输入指令
- [`mouse`](input.md): 鼠标控制指令
- [`memory_resolver`](memory_resolver.md): 脚本中的动态地址解析
- [`process_window`](process_window.md): 脚本的进程上下文

---

**语言**: [English](../../en/modules/script_engine.md) | [中文](script_engine.md)
