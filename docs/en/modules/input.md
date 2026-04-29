# Input Control (Keyboard & Mouse)

[中文文档](../../zh/modules/input.md) | [Back to Overview](overview.md)

The `input` module provides comprehensive keyboard and mouse control through two methods: **SendInput** (system-level, works with all applications) and **PostMessage** (background input to specific windows). Both support high-performance atomic operations with zero runtime allocation.

## Feature Flags

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["keyboard", "mouse"] }
```

## Quick Start

### Keyboard Input (SendInput)

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();
kb.click("a")?;  // Press and release 'A' key
kb.press("ctrl")?;
kb.click("c")?;  // Ctrl+C copy
kb.release("ctrl")?;
```

### Mouse Input (SendInput)

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();
mouse.click_left()?;
mouse.move_to(100, 200)?;
mouse.click_right()?;
```

## Key Features

- **Two Input Methods**: SendInput (global) vs PostMessage (window-specific)
- **High Performance**: Zero-allocation atomic operations
- **Full Control**: Click, press, release, move, scroll
- **String-based API**: Easy-to-use string keys ("a", "ctrl", "space")
- **Coordinate Support**: Absolute and relative mouse positioning
- **Background Input**: PostMessage works without window focus

## Usage Examples

### Example 1: Keyboard Shortcuts

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();

// Ctrl+C (Copy)
kb.press("ctrl")?;
kb.click("c")?;
kb.release("ctrl")?;

// Alt+Tab (Switch windows)
kb.press("alt")?;
kb.click("tab")?;
kb.release("alt")?;

// Win+R (Run dialog)
kb.press("win")?;
kb.click("r")?;
kb.release("win")?;
```

### Example 2: Typing Text

```rust
use win_auto_utils::keyboard::SendInputKeyboard;

let mut kb = SendInputKeyboard::new();

// Type a message
for ch in "Hello, World!".chars() {
    kb.click(&ch.to_string())?;
}

// Press Enter
kb.click("enter")?;
```

### Example 3: Mouse Movement and Clicking

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();

// Move to screen coordinates
mouse.move_to(500, 300)?;

// Left click
mouse.click_left()?;

// Right click at specific position
mouse.click_right_at(600, 400)?;

// Double-click
mouse.double_click_left()?;
```

### Example 4: Background Window Input (PostMessage)

```rust
use win_auto_utils::keyboard::PostMessageKeyboard;
use win_auto_utils::hwnd::find_window_by_title;

// Find target window
let hwnd = find_window_by_title("Notepad")?;

// Send input without focusing the window
let mut kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;  // Works even if Notepad is minimized
```

### Example 5: High-Performance Atomic Operations

```rust
use win_auto_utils::keyboard::send_input;

// Build INPUT structures at parse time (zero overhead)
let inputs = send_input::build_key_click_inputs(0x41, false); // 'A' key

// Execute with minimal latency
send_input::execute_inputs(&inputs)?;

// Perfect for tight loops or real-time input
for _ in 0..1000 {
    send_input::execute_inputs(&inputs)?;
}
```

### Example 6: Mouse Scrolling

```rust
use win_auto_utils::mouse::SendInputMouse;

let mut mouse = SendInputMouse::new();

// Scroll down 3 notches
for _ in 0..3 {
    mouse.scroll_down()?;
}

// Scroll up quickly
for _ in 0..10 {
    mouse.scroll_up()?;
}
```

## API Reference

### Keyboard Module

#### SendInputKeyboard (System-level Input)

**Constructor**:
- `SendInputKeyboard::new()` - Create new instance

**Methods**:
- `click(key: &str)` - Press and release a key
- `press(key: &str)` - Press key down (hold)
- `release(key: &str)` - Release key
- `type_text(text: &str)` - Type multiple characters

#### PostMessageKeyboard (Window-specific Input)

**Constructor**:
- `PostMessageKeyboard::new(hwnd: HWND)` - Target specific window

**Methods**: Same as SendInputKeyboard, but sends to specific window

#### Low-level Functions (`send_input` module)

- `execute_inputs(inputs: &[INPUT])` - Execute pre-built INPUT array
- `execute_single_input(input: &INPUT)` - Execute single INPUT
- `build_keybd_input(vk_code, extended, keyup)` - Build keyboard INPUT
- `build_key_click_inputs(vk_code, extended)` - Build click pair

### Mouse Module

#### SendInputMouse (System-level Input)

**Constructor**:
- `SendInputMouse::new()` - Create new instance

**Methods**:
- `click_left()` / `click_right()` / `click_middle()` - Single clicks
- `double_click_left()` - Double-click
- `press_left()` / `release_left()` - Button hold/release
- `move_to(x, y)` - Absolute positioning
- `move_relative(dx, dy)` - Relative movement
- `scroll_up()` / `scroll_down()` - Mouse wheel

#### PostMessageMouse (Window-specific Input)

**Constructor**:
- `PostMessageMouse::new(hwnd: HWND)` - Target specific window

**Methods**: Same as SendInputMouse, plus coordinate-aware variants:
- `click_left_at(x, y)` - Click at window coordinates
- `move_to(x, y)` - Move within window client area

#### Low-level Functions (`mouse::send_input` module)

- `execute_inputs(inputs: &[INPUT])` - Execute mouse INPUT array
- `build_mouse_input(flags, data)` - Build generic mouse INPUT
- `build_click_left()` - Build left click INPUT pair
- `build_move(x, y)` - Build absolute move INPUT
- `build_scroll(delta)` - Build scroll INPUT

## Supported Keys

### Common Keys (String Format)

| Key String | Description | Virtual Key Code |
|------------|-------------|------------------|
| `"a"` - `"z"` | Letters | VK_A - VK_Z |
| `"0"` - `"9"` | Numbers | VK_0 - VK_9 |
| `"f1"` - `"f12"` | Function keys | VK_F1 - VK_F12 |
| `"enter"` | Enter/Return | VK_RETURN |
| `"space"` | Spacebar | VK_SPACE |
| `"tab"` | Tab | VK_TAB |
| `"esc"` | Escape | VK_ESCAPE |
| `"backspace"` | Backspace | VK_BACK |
| `"delete"` | Delete | VK_DELETE |
| `"insert"` | Insert | VK_INSERT |
| `"home"` / `"end"` | Home/End | VK_HOME / VK_END |
| `"up"` / `"down"` / `"left"` / `"right"` | Arrow keys | VK_UP, etc. |
| `"ctrl"` / `"alt"` / `"shift"` | Modifiers | VK_CONTROL, etc. |
| `"win"` | Windows key | VK_LWIN |
| `"caps"` | Caps Lock | VK_CAPITAL |

## Best Practices

1. **Choose the Right Method**
   ```rust
   // For global input (works everywhere)
   let kb = SendInputKeyboard::new();
   
   // For background input to specific window
   let kb = PostMessageKeyboard::new(hwnd);
   ```

2. **Use Atomic Operations for Performance**
   ```rust
   // Slow: Creates INPUT structures each call
   kb.click("a")?;
   
   // Fast: Pre-build once, reuse
   let inputs = send_input::build_key_click_inputs(0x41, false);
   for _ in 0..1000 {
       send_input::execute_inputs(&inputs)?;
   }
   ```

3. **Handle Modifier Keys Properly**
   ```rust
   // Correct: Release modifiers after use
   kb.press("ctrl")?;
   kb.click("c")?;
   kb.release("ctrl")?;  // Don't forget!
   
   // Wrong: Leaving modifiers pressed
   kb.press("ctrl")?;
   kb.click("c")?;
   // Ctrl stays pressed, breaks future input!
   ```

4. **Add Delays for Reliability**
   ```rust
   use std::time::Duration;
   
   kb.click("a")?;
   std::thread::sleep(Duration::from_millis(50));  // Wait for processing
   kb.click("b")?;
   ```

5. **Validate Window Handles**
   ```rust
   use win_auto_utils::hwnd::is_window_valid;
   
   if is_window_valid(hwnd) {
       let kb = PostMessageKeyboard::new(hwnd);
       kb.click("a")?;
   } else {
       eprintln!("Window no longer exists");
   }
   ```

## Common Pitfalls

### ❌ Forgetting to Release Keys

```rust
// Wrong: Modifier stays pressed
kb.press("shift")?;
kb.click("a")?;
// Shift is still held down!

// Correct: Always release
kb.press("shift")?;
kb.click("a")?;
kb.release("shift")?;
```

### ❌ Using PostMessage Without Valid HWND

```rust
// Wrong: Invalid window handle
let hwnd = HWND(0);
let kb = PostMessageKeyboard::new(hwnd);
kb.click("a")?;  // Fails silently

// Correct: Validate first
if is_window_valid(hwnd) {
    let kb = PostMessageKeyboard::new(hwnd);
    kb.click("a")?;
}
```

### ❌ Coordinate System Confusion

```rust
// PostMessage uses CLIENT coordinates (relative to window)
mouse.click_left_at(10, 10)?;  // Top-left of client area

// SendInput uses SCREEN coordinates (absolute)
mouse.move_to(10, 10)?;  // Top-left of screen
```

## Performance Comparison

| Method | Latency | Use Case |
|--------|---------|----------|
| SendInput (high-level) | ~1-5ms | General automation |
| SendInput (atomic) | ~0.1-0.5ms | Real-time input, gaming |
| PostMessage | ~0.5-2ms | Background automation |

### Benchmark Example

```rust
use std::time::Instant;
use win_auto_utils::keyboard::send_input;

let inputs = send_input::build_key_click_inputs(0x41, false);

let start = Instant::now();
for _ in 0..1000 {
    send_input::execute_inputs(&inputs)?;
}
let elapsed = start.elapsed();
println!("1000 clicks in {:?}", elapsed);
// Typical: 100-500ms depending on system load
```

## Related Modules

- [`script_engine`](script_engine.md): Automate input with scripts
- [`process_window`](process_window.md): Find and manage windows
- [`hwnd`](process_window.md): Window handle utilities
- [`memory_hook`](memory_hook.md): Intercept input at low level

---

**Language**: [English](input.md) | [中文](../../zh/modules/input.md)
