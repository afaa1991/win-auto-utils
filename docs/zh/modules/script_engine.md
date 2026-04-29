# 脚本引擎 (Script Engine)

[English](../../en/modules/script_engine.md) | [返回概览](overview.md)

`script_engine` 模块提供一个轻量级、可扩展的脚本执行引擎，采用三阶段流水线：解析 → 编译 → 执行。它具有基于寄存器的虚拟机、标签解析和可选的生命周期钩子以进行编译时分析。核心完全由纯 Rust 编写，零外部依赖。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["script_engine"] }
```

**平台**: 跨平台（纯 Rust 实现）

## 快速开始

### 基础脚本执行

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// 执行简单脚本
engine.execute_script(r#"
    click "a"
    sleep 100
    click "b"
"#)?;

println!("脚本执行成功！");
```

### 带变量和循环的脚本

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// 带循环和变量的脚本
let script = r#"
    set $counter 5
    loop $counter {
        click "x"
        sleep 50
        dec $counter
    }
"#;

engine.execute_script(script)?;
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

let mut engine = ScriptEngine::new();

// 带条件分支的脚本
let script = r#"
    set $health 100
    if $health > 50 {
        click "potion"
    } else {
        click "retreat"
    }
"#;

engine.execute_script(script)?;
```

### 示例 2: 嵌套循环

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// 网格模式的嵌套循环
let script = r#"
    set $row 3
    set $col 4
    
    loop $row {
        loop $col {
            click "cell"
            sleep 10
        }
        move_down
    }
"#;

engine.execute_script(script)?;
```

### 示例 3: 中断控制

```rust
use win_auto_utils::script_engine::{ScriptEngine, InterruptController};

let mut engine = ScriptEngine::new();
let controller = engine.get_interrupt_controller();

// 在后台启动脚本
engine.execute_script_async(r#"
    loop 100 {
        click "a"
        sleep 100
    }
"#)?;

// 2秒后暂停
std::thread::sleep(std::time::Duration::from_secs(2));
controller.pause()?;

// 稍后恢复
controller.resume()?;

// 或完全停止
controller.stop()?;
```

### 示例 4: 自定义配置

```rust
use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};

let config = ScriptConfig {
    max_loop_iterations: 10000,
    enable_debug_logging: true,
    timeout_ms: Some(30000), // 30秒超时
};

let mut engine = ScriptEngine::with_config(config);
engine.execute_script(your_script)?;
```

### 示例 5: 错误处理

```rust
use win_auto_utils::script_engine::{ScriptEngine, ScriptError};

let mut engine = ScriptEngine::new();

match engine.execute_script("invalid syntax here") {
    Ok(_) => println!("成功"),
    Err(ScriptError::ParseError { line, message }) => {
        eprintln!("第 {} 行解析错误: {}", line, message);
    }
    Err(ScriptError::RuntimeError { instruction, message }) => {
        eprintln!("指令 '{}' 运行时错误: {}", instruction, message);
    }
    Err(e) => eprintln!("错误: {}", e),
}
```

### 示例 6: 寄存器操作

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// 使用寄存器进行计算
let script = r#"
    set $x 10
    set $y 20
    add $x $y      # $x = 30
    mul $x 2       # $x = 60
    click_at $x $y # 点击位置 (60, 20)
"#;

engine.execute_script(script)?;
```

## API 参考

### 主要类型

#### ScriptEngine

脚本执行的主要入口点。

**构造函数**:
- `ScriptEngine::new()` - 使用默认配置创建
- `ScriptEngine::with_config(config: ScriptConfig)` - 使用自定义配置创建

**方法**:
- `execute_script(script: &str) -> Result<(), ScriptError>` - 同步执行脚本
- `execute_script_async(script: &str) -> Result<(), ScriptError>` - 在后台线程执行
- `get_interrupt_controller() -> InterruptController` - 获取控制句柄
- `get_vm_context() -> VMContext` - 访问 VM 状态

#### ScriptConfig

脚本引擎行为的配置。

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

#### 控制流

| 指令 | 语法 | 描述 |
|------|------|------|
| `set` | `set $var value` | 设置变量/寄存器 |
| `if` | `if $var > 10 { ... }` | 条件分支 |
| `loop` | `loop $count { ... }` | 重复 N 次 |
| `while` | `while $var > 0 { ... }` | 当条件为真时循环 |
| `jump` | `jump label_name` | 无条件跳转 |
| `label` | `label my_label:` | 定义跳转目标 |

#### 算术运算

| 指令 | 语法 | 描述 |
|------|------|------|
| `add` | `add $a $b` | 加法 ($a += $b) |
| `sub` | `sub $a $b` | 减法 ($a -= $b) |
| `mul` | `mul $a $b` | 乘法 ($a *= $b) |
| `div` | `div $a $b` | 除法 ($a /= $b) |
| `inc` | `inc $var` | 递增 1 |
| `dec` | `dec $var` | 递减 1 |

#### 键盘/鼠标

| 指令 | 语法 | 描述 |
|------|------|------|
| `click` | `click "key"` | 按下并释放键 |
| `press` | `press "key"` | 按下键（保持） |
| `release` | `release "key"` | 释放键 |
| `move_to` | `move_to x y` | 移动鼠标到坐标 |
| `click_at` | `click_at x y` | 在位置点击 |
| `sleep` | `sleep ms` | 等待毫秒数 |

## 脚本语法

### 变量和寄存器

变量以 `$` 为前缀并存储在 VM 寄存器中。

```rust
set $health 100
set $name "player"
set $position_x 500
```

### 注释

单行注释以 `#` 开头。

```rust
set $x 10  # 这是注释
# 整行注释
```

### 代码块

代码块使用花括号 `{ }`。

```rust
if $health > 50 {
    click "heal"
    sleep 100
}
```

### 运算符

支持的比较运算符：`>`、`<`、`>=`、`<=`、`==`、`!=`

```rust
if $health >= 100 {
    # 满血
} else if $health > 50 {
    # 中等血量
} else {
    # 低血量
}
```

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

1. **使用有意义的变量名**
   ```rust
   # 好
   set $player_health 100
   set $enemy_count 5
   
   # 不好
   set $a 100
   set $b 5
   ```

2. **为复杂逻辑添加注释**
   ```rust
   # 当血量低于 30% 时治疗
   if $health < 30 {
       click "health_potion"
       sleep 500  # 等待动画
   }
   ```

3. **设置合理的循环限制**
   ```rust
   # 好: 有限循环
   loop 100 {
       click "farm"
   }
   
   # 不好: 潜在无限循环
   while $true {
       click "action"
   }
   ```

4. **优雅地处理错误**
   ```rust
   match engine.execute_script(script) {
       Ok(_) => log_success(),
       Err(ScriptError::ParseError { line, .. }) => {
           eprintln!("修复第 {} 行的语法", line);
       }
       Err(e) => log_error(e),
   }
   ```

5. **对长脚本使用中断**
   ```rust
   let controller = engine.get_interrupt_controller();
   
   // 允许用户停止
   if user_pressed_stop() {
       controller.stop()?;
   }
   ```

## 常见陷阱

### ❌ 忘记寄存器前缀

```rust
# 错误
set health 100
if health > 50

# 正确
set $health 100
if $health > 50
```

### ❌ 未闭合的代码块

```rust
# 错误: 缺少右花括号
if $health > 50 {
    click "heal"

# 正确
if $health > 50 {
    click "heal"
}
```

### ❌ 无限循环

```rust
# 错误: 无终止条件
loop 999999999 {
    click "spam"
}

# 正确: 合理限制
loop 100 {
    click "action"
}
```

## 扩展引擎

### 自定义指令处理器

```rust
use win_auto_utils::script_engine::{
    InstructionHandler, InstructionData, VM
};

struct MyCustomInstruction;

impl InstructionHandler for MyCustomInstruction {
    fn parse(&self, tokens: &[Token]) -> Result<InstructionData> {
        // 解析自定义语法
        Ok(InstructionData::new("my_instruction"))
    }
    
    fn execute(&self, vm: &mut VM, data: &InstructionData) {
        // 自定义执行逻辑
        println!("执行自定义指令");
    }
}

// 注册到引擎
let mut registry = InstructionRegistry::new();
registry.register("my_instruction", Box::new(MyCustomInstruction));
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
