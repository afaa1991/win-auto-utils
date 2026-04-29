# Script Engine

[中文文档](../../zh/modules/script_engine.md) | [Back to Overview](overview.md)

The `script_engine` module provides a lightweight, extensible script execution engine with a three-stage pipeline: Parse → Compile → Execute. It features a register-based virtual machine, label resolution, and optional lifecycle hooks for compile-time analysis. The core is written in pure Rust with zero external dependencies.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["script_engine"] }
```

**Platform**: Cross-platform (pure Rust implementation)

## Quick Start

### Basic Script Execution

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// Execute simple script
engine.execute_script(r#"
    click "a"
    sleep 100
    click "b"
"#)?;

println!("Script executed successfully!");
```

### Script with Variables and Loops

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// Script with loop and variables
let script = r#"
    set $counter 5
    loop $counter {
        click "x"
        sleep 50
        dec $counter
    }
"#;

engine.execute_script(script)?;
println!("Loop completed!");
```

## Key Features

- **Three-Stage Pipeline**: Parse → Compile → Execute
- **Register-Based VM**: Efficient state management
- **Label Resolution**: Support for jumps and loops
- **Lifecycle Hooks**: Optional compile-time analysis
- **Pure Rust**: Zero external dependencies, cross-platform
- **Extensible**: Custom instruction handlers via trait implementation
- **Interrupt Control**: Pause/resume/stop execution
- **Error Handling**: Detailed error messages with line numbers

## Usage Examples

### Example 1: Conditional Logic

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// Script with conditional branching
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

### Example 2: Nested Loops

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// Nested loops for grid pattern
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

### Example 3: Interrupt Control

```rust
use win_auto_utils::script_engine::{ScriptEngine, InterruptController};

let mut engine = ScriptEngine::new();
let controller = engine.get_interrupt_controller();

// Start script in background
engine.execute_script_async(r#"
    loop 100 {
        click "a"
        sleep 100
    }
"#)?;

// Pause after 2 seconds
std::thread::sleep(std::time::Duration::from_secs(2));
controller.pause()?;

// Resume later
controller.resume()?;

// Or stop completely
controller.stop()?;
```

### Example 4: Custom Configuration

```rust
use win_auto_utils::script_engine::{ScriptEngine, ScriptConfig};

let config = ScriptConfig {
    max_loop_iterations: 10000,
    enable_debug_logging: true,
    timeout_ms: Some(30000), // 30 second timeout
};

let mut engine = ScriptEngine::with_config(config);
engine.execute_script(your_script)?;
```

### Example 5: Error Handling

```rust
use win_auto_utils::script_engine::{ScriptEngine, ScriptError};

let mut engine = ScriptEngine::new();

match engine.execute_script("invalid syntax here") {
    Ok(_) => println!("Success"),
    Err(ScriptError::ParseError { line, message }) => {
        eprintln!("Parse error at line {}: {}", line, message);
    }
    Err(ScriptError::RuntimeError { instruction, message }) => {
        eprintln!("Runtime error in '{}': {}", instruction, message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Example 6: Register Manipulation

```rust
use win_auto_utils::script_engine::ScriptEngine;

let mut engine = ScriptEngine::new();

// Use registers for calculations
let script = r#"
    set $x 10
    set $y 20
    add $x $y      # $x = 30
    mul $x 2       # $x = 60
    click_at $x $y # Click at position (60, 20)
"#;

engine.execute_script(script)?;
```

## API Reference

### Main Types

#### ScriptEngine

Main entry point for script execution.

**Constructors**:
- `ScriptEngine::new()` - Create with default configuration
- `ScriptEngine::with_config(config: ScriptConfig)` - Create with custom config

**Methods**:
- `execute_script(script: &str) -> Result<(), ScriptError>` - Execute script synchronously
- `execute_script_async(script: &str) -> Result<(), ScriptError>` - Execute in background thread
- `get_interrupt_controller() -> InterruptController` - Get control handle
- `get_vm_context() -> VMContext` - Access VM state

#### ScriptConfig

Configuration for script engine behavior.

**Fields**:
- `max_loop_iterations: u64` - Maximum loop iterations (prevent infinite loops)
- `enable_debug_logging: bool` - Enable detailed logging
- `timeout_ms: Option<u64>` - Execution timeout in milliseconds
- `stack_size: usize` - VM stack size

#### InterruptController

Control running script execution.

**Methods**:
- `pause() -> Result<(), ScriptError>` - Pause execution
- `resume() -> Result<(), ScriptError>` - Resume from pause
- `stop() -> Result<(), ScriptError>` - Stop execution permanently
- `is_running() -> bool` - Check if script is running

#### ScriptError

Error types for script operations.

**Variants**:
- `ParseError { line: usize, message: String }` - Syntax error during parsing
- `CompileError { message: String }` - Compilation/validation error
- `RuntimeError { instruction: String, message: String }` - Runtime execution error
- `TimeoutError` - Execution exceeded timeout
- `Interrupted` - Script was manually interrupted

### Built-in Instructions

#### Control Flow

| Instruction | Syntax | Description |
|-------------|--------|-------------|
| `set` | `set $var value` | Set variable/register |
| `if` | `if $var > 10 { ... }` | Conditional branch |
| `loop` | `loop $count { ... }` | Repeat N times |
| `while` | `while $var > 0 { ... }` | Loop while condition |
| `jump` | `jump label_name` | Unconditional jump |
| `label` | `label my_label:` | Define jump target |

#### Arithmetic

| Instruction | Syntax | Description |
|-------------|--------|-------------|
| `add` | `add $a $b` | Add ($a += $b) |
| `sub` | `sub $a $b` | Subtract ($a -= $b) |
| `mul` | `mul $a $b` | Multiply ($a *= $b) |
| `div` | `div $a $b` | Divide ($a /= $b) |
| `inc` | `inc $var` | Increment by 1 |
| `dec` | `dec $var` | Decrement by 1 |

#### Keyboard/Mouse

| Instruction | Syntax | Description |
|-------------|--------|-------------|
| `click` | `click "key"` | Press and release key |
| `press` | `press "key"` | Press key (hold) |
| `release` | `release "key"` | Release key |
| `move_to` | `move_to x y` | Move mouse to coordinates |
| `click_at` | `click_at x y` | Click at position |
| `sleep` | `sleep ms` | Wait milliseconds |

## Script Syntax

### Variables and Registers

Variables are prefixed with `$` and stored in VM registers.

```rust
set $health 100
set $name "player"
set $position_x 500
```

### Comments

Single-line comments start with `#`.

```rust
set $x 10  # This is a comment
# Full line comment
```

### Blocks

Code blocks use curly braces `{ }`.

```rust
if $health > 50 {
    click "heal"
    sleep 100
}
```

### Operators

Supported comparison operators: `>`, `<`, `>=`, `<=`, `==`, `!=`

```rust
if $health >= 100 {
    # Full health
} else if $health > 50 {
    # Medium health
} else {
    # Low health
}
```

## Architecture Details

### Three-Stage Pipeline

1. **Parse Stage** (`parser` module)
   - Tokenizes script text
   - Converts to instruction sequences
   - Validates basic syntax

2. **Compile Stage** (`compiler` module)
   - Resolves labels and jump targets
   - Validates control flow
   - Optimizes instruction order
   - Performs static analysis via lifecycle hooks

3. **Execute Stage** (`vm` module)
   - Runs compiled instructions
   - Manages register state
   - Handles interrupts
   - Reports runtime errors

### Lifecycle Hooks

Instructions can implement optional lifecycle methods:

```rust
trait InstructionHandler {
    // Required
    fn parse(&self, tokens: &[Token]) -> Result<InstructionData>;
    fn execute(&self, vm: &mut VM, data: &InstructionData);
    
    // Optional (default implementations provided)
    fn compile(&self, data: &mut CompiledInstruction) { }
    fn validate(&self, data: &InstructionData) -> Result<()> { Ok(()) }
}
```

### Register-Based VM

The VM uses named registers instead of a stack:

```
Registers:
  $x, $y, $z          - General purpose
  $counter, $index    - Loop counters
  $temp1, $temp2      - Temporary storage
  $result             - Operation results
```

## Best Practices

1. **Use Meaningful Variable Names**
   ```rust
   # Good
   set $player_health 100
   set $enemy_count 5
   
   # Bad
   set $a 100
   set $b 5
   ```

2. **Add Comments for Complex Logic**
   ```rust
   # Heal when health drops below 30%
   if $health < 30 {
       click "health_potion"
       sleep 500  # Wait for animation
   }
   ```

3. **Set Reasonable Loop Limits**
   ```rust
   # Good: Finite loop
   loop 100 {
       click "farm"
   }
   
   # Bad: Potential infinite loop
   while $true {
       click "action"
   }
   ```

4. **Handle Errors Gracefully**
   ```rust
   match engine.execute_script(script) {
       Ok(_) => log_success(),
       Err(ScriptError::ParseError { line, .. }) => {
           eprintln!("Fix syntax at line {}", line);
       }
       Err(e) => log_error(e),
   }
   ```

5. **Use Interrupts for Long Scripts**
   ```rust
   let controller = engine.get_interrupt_controller();
   
   // Allow user to stop
   if user_pressed_stop() {
       controller.stop()?;
   }
   ```

## Common Pitfalls

### ❌ Forgetting Register Prefix

```rust
# Wrong
set health 100
if health > 50

# Correct
set $health 100
if $health > 50
```

### ❌ Unclosed Blocks

```rust
# Wrong: Missing closing brace
if $health > 50 {
    click "heal"

# Correct
if $health > 50 {
    click "heal"
}
```

### ❌ Infinite Loops

```rust
# Wrong: No termination condition
loop 999999999 {
    click "spam"
}

# Correct: Reasonable limit
loop 100 {
    click "action"
}
```

## Extending the Engine

### Custom Instruction Handler

```rust
use win_auto_utils::script_engine::{
    InstructionHandler, InstructionData, VM
};

struct MyCustomInstruction;

impl InstructionHandler for MyCustomInstruction {
    fn parse(&self, tokens: &[Token]) -> Result<InstructionData> {
        // Parse custom syntax
        Ok(InstructionData::new("my_instruction"))
    }
    
    fn execute(&self, vm: &mut VM, data: &InstructionData) {
        // Custom execution logic
        println!("Executing custom instruction");
    }
}

// Register with engine
let mut registry = InstructionRegistry::new();
registry.register("my_instruction", Box::new(MyCustomInstruction));
```

## Performance Characteristics

| Operation | Typical Time |
|-----------|--------------|
| Script parsing | 1-5ms |
| Compilation | 2-10ms |
| Simple instruction execution | <1μs |
| Loop iteration | 1-5μs |
| Context switch (async) | 10-50μs |

## Related Modules

- [`keyboard`](input.md): Keyboard input instructions
- [`mouse`](input.md): Mouse control instructions
- [`memory_resolver`](memory_resolver.md): Dynamic address resolution in scripts
- [`process_window`](process_window.md): Process context for scripts

---

**Language**: [English](script_engine.md) | [中文](../../zh/modules/script_engine.md)
