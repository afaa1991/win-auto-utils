# 输入控制 (Input Control - Keyboard & Mouse)

[English](../../en/modules/input.md) | [返回概览](overview.md)

`input` 模块通过两种方法提供全面的键盘和鼠标控制：**SendInput**（系统级，适用于所有应用）和 **PostMessage**（后台输入到特定窗口）。两者都支持高性能的原子操作，零运行时分配。

## Feature Flags

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["keyboard", "mouse"] }
```

## 快速开始

### 键盘输入 (SendInput)

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();
kb.click("a")?;  // 按下并释放 'A' 键
kb.press("ctrl")?;
kb.click("c")?;  // Ctrl+C 复制
kb.release("ctrl")?;
```

### 鼠标输入 (SendInput)

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();
mouse.click_left()?;
mouse.move_to(100, 200)?;
mouse.click_right()?;
```

## 核心功能

- **两种输入方法**: SendInput（全局）vs PostMessage（窗口特定）
- **高性能**: 零分配的原子操作
- **完全控制**: 点击、按下、释放、移动、滚动
- **字符串 API**: 易用的字符串按键（"a"、"ctrl"、"space"）
- **坐标支持**: 绝对和相对鼠标定位
- **后台输入**: PostMessage 无需窗口焦点即可工作

## 使用示例

### 示例 1: 键盘快捷键

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();

// Ctrl+C（复制）
kb.press("ctrl")?;
kb.click("c")?;
kb.release("ctrl")?;

// Alt+Tab（切换窗口）
kb.press("alt")?;
kb.click("tab")?;
kb.release("alt")?;

// Win+R（运行对话框）
kb.press("win")?;
kb.click("r")?;
kb.release("win")?;
```

### 示例 2: 输入文本

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();

// 输入消息
for ch in "Hello, World!".chars() {
    kb.click(&ch.to_string())?;
}

// 按回车
kb.click("enter")?;
```

### 示例 3: 鼠标移动和点击

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();

// 移动到屏幕坐标
mouse.move_to(500, 300)?;

// 左键点击
mouse.click_left()?;

// 在指定位置右键点击
mouse.click_right_at(600, 400)?;

// 双击
mouse.double_click_left()?;
```

### 示例 4: 后台窗口输入 (PostMessage)

```rust
use win_auto_utils::keyboard::PostMessageKeyboard;
use win_auto_utils::hwnd::find_window_by_title;

// 查找目标窗口
let hwnd = find_window_by_title("Notepad")?;

// 发送输入而无需聚焦窗口
let mut kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;  // 即使记事本最小化也能工作
```

### 示例 5: 高性能原子操作

```rust
use win_auto_utils::keyboard::send_input;

// 在解析时构建 INPUT 结构（零开销）
let inputs = send_input::build_key_click_inputs(0x41, false); // 'A' 键

// 以最小延迟执行
send_input::execute_inputs(&inputs)?;

// 非常适合紧密循环或实时输入
for _ in 0..1000 {
    send_input::execute_inputs(&inputs)?;
}
```

### 示例 6: 鼠标滚动

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();

// 向下滚动 3 格
for _ in 0..3 {
    mouse.scroll_down()?;
}

// 快速向上滚动
for _ in 0..10 {
    mouse.scroll_up()?;
}
```

## API 参考

### 键盘模块

#### SendInputKeyboard（系统级输入）

**构造函数**:
- `SendInputKeyboard::new()` - 创建新实例

**方法**:
- `click(key: &str)` - 按下并释放一个键
- `press(key: &str)` - 按下键（保持）
- `release(key: &str)` - 释放键
- `type_text(text: &str)` - 输入多个字符

#### PostMessageKeyboard（窗口特定输入）

**构造函数**:
- `PostMessageKeyboard::new(hwnd: HWND)` -  targeting 特定窗口

**方法**: 与 SendInputKeyboard 相同，但发送到特定窗口

#### 底层函数 (`send_input` 模块)

- `execute_inputs(inputs: &[INPUT])` - 执行预构建的 INPUT 数组
- `execute_single_input(input: &INPUT)` - 执行单个 INPUT
- `build_keybd_input(vk_code, extended, keyup)` - 构建键盘 INPUT
- `build_key_click_inputs(vk_code, extended)` - 构建点击对

### 鼠标模块

#### SendInputMouse（系统级输入）

**构造函数**:
- `SendInputMouse::new()` - 创建新实例

**方法**:
- `click_left()` / `click_right()` / `click_middle()` - 单击
- `double_click_left()` - 双击
- `press_left()` / `release_left()` - 按钮保持/释放
- `move_to(x, y)` - 绝对定位
- `move_relative(dx, dy)` - 相对移动
- `scroll_up()` / `scroll_down()` - 鼠标滚轮

#### PostMessageMouse（窗口特定输入）

**构造函数**:
- `PostMessageMouse::new(hwnd: HWND)` - targeting 特定窗口

**方法**: 与 SendInputMouse 相同，加上坐标感知变体：
- `click_left_at(x, y)` - 在窗口坐标处点击
- `move_to(x, y)` - 在窗口客户区内移动

#### 底层函数 (`mouse::send_input` 模块)

- `execute_inputs(inputs: &[INPUT])` - 执行鼠标 INPUT 数组
- `build_mouse_input(flags, data)` - 构建通用鼠标 INPUT
- `build_click_left()` - 构建左键点击 INPUT 对
- `build_move(x, y)` - 构建绝对移动 INPUT
- `build_scroll(delta)` - 构建滚动 INPUT

## 支持的按键

### 常用按键（字符串格式）

| 按键字符串 | 描述 | 虚拟键码 |
|-----------|------|---------|
| `"a"` - `"z"` | 字母 | VK_A - VK_Z |
| `"0"` - `"9"` | 数字 | VK_0 - VK_9 |
| `"f1"` - `"f12"` | 功能键 | VK_F1 - VK_F12 |
| `"enter"` | 回车/返回 | VK_RETURN |
| `"space"` | 空格键 | VK_SPACE |
| `"tab"` | Tab | VK_TAB |
| `"esc"` | Escape | VK_ESCAPE |
| `"backspace"` | 退格 | VK_BACK |
| `"delete"` | 删除 | VK_DELETE |
| `"insert"` | 插入 | VK_INSERT |
| `"home"` / `"end"` | Home/End | VK_HOME / VK_END |
| `"up"` / `"down"` / `"left"` / `"right"` | 方向键 | VK_UP 等 |
| `"ctrl"` / `"alt"` / `"shift"` | 修饰键 | VK_CONTROL 等 |
| `"win"` | Windows 键 | VK_LWIN |
| `"caps"` | 大写锁定 | VK_CAPITAL |

## 脚本指令 (Script Instructions)

`scripts_builtin` 模块提供了用于脚本引擎的高级鼠标和键盘指令。

### Mouse 指令

#### `click` - 鼠标点击

在指定位置或当前位置执行鼠标点击。

**语法**: `click [x] [y] [delay_ms]`

**参数**:
- `x` (可选): X 坐标，如果省略则在当前位置点击
- `y` (可选): Y 坐标，提供 x 时必须提供
- `delay_ms` (可选): 按下和释放之间的延迟（毫秒），默认为 0

**示例**:
```text
click                      # 在当前位置点击
click 100 200              # 在屏幕坐标 (100, 200) 点击
click 50 50 100            # 在 (50, 50) 点击，延迟 100ms
```

**坐标系统**:
- 无 hwnd 时：使用绝对屏幕坐标
- 设置 hwnd 后：使用窗口相对坐标，自动转换为屏幕坐标

#### `dbclick` - 鼠标双击

执行鼠标双击操作。

**语法**: `dbclick [x] [y] [delay_ms]`

**参数**:
- `x` (可选): X 坐标
- `y` (可选): Y 坐标
- `delay_ms` (可选): 第二次点击后的延迟

**示例**:
```text
dbclick                    # 在当前位置双击
dbclick 100 200            # 在 (100, 200) 双击
```

#### `move` - 移动鼠标到绝对位置

**语法**: `move <x> <y>`

**示例**:
```text
move 500 300               # 移动到屏幕坐标 (500, 300)
```

#### `moverel` - 相对移动

**语法**: `moverel <dx> <dy>`

**示例**:
```text
moverel 10 -5              # 向右移动 10px，向上移动 5px
```

#### `scrollup` / `scrolldown` - 滚轮滚动

**语法**: `scrollup [x] [y] [times]`

**示例**:
```text
scrollup                   # 在当前位置向上滚动 1 格
scrollup 3                 # 向上滚动 3 格
scrollup 100 200           # 移动到 (100, 200) 后向上滚动
```

#### `press` / `release` - 按下/释放鼠标按钮

**语法**: `press` / `release`

**示例**:
```text
press                      # 按下左键并保持
release                    # 释放左键
```

### Keyboard 指令

#### `key` - 按键点击

执行完整的按键操作（按下 + 释放）。

**语法**: `key <key_name> [delay_ms] [mode]`

**参数**:
- `key_name`: 按键名称（如 "A", "ENTER", "CTRL"）
- `delay_ms` (可选): 按下和释放之间的延迟（毫秒），默认为 0
- `mode` (可选): 执行模式，`send`（前台，默认）或 `post`（后台）

**示例**:
```text
key A                      # 前台点击 'A' 键（默认）
key ENTER 50               # 前台点击 ENTER，延迟 50ms
key A post                 # 后台点击 'A' 键
key A 50 post              # 后台点击 'A'，延迟 50ms
```

#### `key_down` - 按下并保持按键

**语法**: `key_down <key_name> [mode]`

**示例**:
```text
key_down CONTROL           # 前台按下 CONTROL
key_down SHIFT post        # 后台按下 SHIFT
```

#### `key_up` - 释放按键

**语法**: `key_up <key_name> [mode]`

**示例**:
```text
key_up CONTROL             # 前台释放 CONTROL
key_up SHIFT post          # 后台释放 SHIFT
```

#### 组合键示例

```text
# Ctrl+C（复制）
key_down CONTROL
key C
key_up CONTROL

# Alt+Tab（切换窗口）
key_down ALT
key TAB
key_up ALT
```

#### 后台模式配置

使用 `post` 模式前，需要在 Rust 代码中设置目标窗口句柄：

```rust
use win_auto_utils::script_engine::ScriptEngine;
use windows::Win32::Foundation::HWND;

let engine = ScriptEngine::with_builtin();

// 设置目标窗口
engine.compile_and_execute_with_context("key A post", |ctx| {
    ctx.set_persistent_state("target_hwnd", HWND(window_handle));
}).unwrap();
```

## 最佳实践

1. **选择正确的方法**
   ```rust
   // 用于全局输入（随处可用）
   let kb = SendInputKeyboard::new();
   
   // 用于特定窗口的后台输入
   let kb = PostMessageKeyboard::new(hwnd);
   ```

2. **使用原子操作提高性能**
   ```rust
   // 慢: 每次调用创建 INPUT 结构
   kb.click("a")?;
   
   // 快: 预构建一次，重复使用
   let inputs = send_input::build_key_click_inputs(0x41, false);
   for _ in 0..1000 {
       send_input::execute_inputs(&inputs)?;
   }
   ```

3. **正确处理修饰键**
   ```rust
   // 正确: 使用后释放修饰键
   kb.press("ctrl")?;
   kb.click("c")?;
   kb.release("ctrl")?;  // 别忘了！
   
   // 错误: 让修饰键保持按下
   kb.press("ctrl")?;
   kb.click("c")?;
   // Ctrl 保持按下，破坏未来的输入！
   ```

4. **添加延迟以提高可靠性**
   ```rust
   use std::time::Duration;
   
   kb.click("a")?;
   std::thread::sleep(Duration::from_millis(50));  // 等待处理
   kb.click("b")?;
   ```

5. **验证窗口句柄**
   ```rust
   use win_auto_utils::hwnd::is_window_valid;
   
   if is_window_valid(hwnd) {
       let kb = PostMessageKeyboard::new(hwnd);
       kb.click("a")?;
   } else {
       eprintln!("窗口不再存在");
   }
   ```

## 常见陷阱

### ❌ 忘记释放按键

```rust
// 错误: 修饰键保持按下
kb.press("shift")?;
kb.click("a")?;
// Shift 仍然保持按下！

// 正确: 始终释放
kb.press("shift")?;
kb.click("a")?;
kb.release("shift")?;
```

### ❌ 使用无效的 HWND 进行 PostMessage

```rust
// 错误: 无效的窗口句柄
let hwnd = HWND(0);
let kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;  // 静默失败

// 正确: 先验证
if is_window_valid(hwnd) {
    let kb = PostMessageKeyboard::new(hwnd);
    kb.click("a")?;
}
```

### ❌ 坐标系混淆

```rust
// PostMessage 使用 CLIENT 坐标（相对于窗口）
mouse.click_left_at(10, 10)?;  // 客户区左上角

// SendInput 使用 SCREEN 坐标（绝对）
mouse.move_to(10, 10)?;  // 屏幕左上角
```

## 性能对比

| 方法 | 延迟 | 用例 |
|------|------|------|
| SendInput（高级） | ~1-5ms | 一般自动化 |
| SendInput（原子） | ~0.1-0.5ms | 实时输入、游戏 |
| PostMessage | ~0.5-2ms | 后台自动化 |

### 基准测试示例

```rust
use std::time::Instant;
use win_auto_utils::keyboard::send_input;

let inputs = send_input::build_key_click_inputs(0x41, false);

let start = Instant::now();
for _ in 0..1000 {
    send_input::execute_inputs(&inputs)?;
}
let elapsed = start.elapsed();
println!("1000 次点击耗时 {:?}", elapsed);
// 典型结果: 100-500ms，取决于系统负载
```

## 相关模块

- [`script_engine`](script_engine.md): 使用脚本自动化输入
- [`process_window`](process_window.md): 查找和管理窗口
- [`hwnd`](process_window.md): 窗口句柄工具
- [`memory_hook`](memory_hook.md): 在底层拦截输入

---

**语言**: [English](../../en/modules/input.md) | [中文](input.md)
