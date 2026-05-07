# Script Engine

[中文文档](../../zh/modules/script_engine.md) | [Back to Overview](overview.md)

The `script_engine` module provides a lightweight, extensible script execution engine with a three-stage pipeline: Parse → Compile → Execute. It features a register-based virtual machine, label resolution, and optional lifecycle hooks for compile-time analysis. The core is written in pure Rust with zero external dependencies.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["script_engine"] }
```

**Platform**: Cross-platform (pure Rust implementation)

## Quick Start

### Basic Script Execution

```rust
use win_auto_utils::script_engine::ScriptEngine;

// Create engine with built-in instructions
let engine = ScriptEngine::with_builtin();

// Execute simple script
engine.compile_and_execute(r#"
    sleep 10
"#)?;

println!("Script executed successfully!");
```

### Script with Built-in Instructions

```rust
use win_auto_utils::script_engine::{InstructionRegistry, ScriptConfig, ScriptEngine};

// Create a custom registry and register some instructions
let mut registry = InstructionRegistry::new();

// Use helper function to register all built-in instructions
win_auto_utils::scripts_builtin::register_all(&mut registry);

let engine = ScriptEngine::with_registry_and_config(registry, ScriptConfig::default());

// Script with loop and variables
let script = r#"
    loop 3 {
        sleep 50
    }
"#;

engine.compile_and_execute(script)?;
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

let engine = ScriptEngine::with_builtin();

// Script with conditional branching
let script = r#"
    loop 3
        sleep 100
    end
"#;

engine.compile_and_execute(script)?;
```

### Example 2: Nested Loops

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// Nested loops
let script = r#"
    loop 2
        loop 3
            sleep 10
        end
    end
"#;

engine.compile_and_execute(script)?;
```

### Example 3: Interrupt Control

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// Start script in background
engine.compile_and_execute_async(r#"
    loop 100 {
        sleep 100
    }
"#)?;

// Pause after 2 seconds (need to save controller reference)
std::thread::sleep(std::time::Duration::from_secs(2));
// controller.pause()?;

// Resume later
// controller.resume()?;

// Or stop completely
// controller.stop()?;
```

### Example 4: Custom Configuration

```rust
use win_auto_utils::script_engine::{InstructionRegistry, ScriptConfig, ScriptEngine};

let config = ScriptConfig::default();

let mut registry = InstructionRegistry::new();
win_auto_utils::scripts_builtin::register_all(&mut registry);

let engine = ScriptEngine::with_registry_and_config(registry, config);
engine.compile_and_execute(your_script)?;
```

### Example 5: Error Handling

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

match engine.compile_and_execute("sleep 10") {
    Ok(_) => println!("Success"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Example 6: Simple Loop

```rust
use win_auto_utils::script_engine::ScriptEngine;

let engine = ScriptEngine::with_builtin();

// Use loops for repetitive operations
let script = r#"
    loop 5
        sleep 10
    end
"#;

engine.compile_and_execute(script)?;
```

## API Reference

### Main Types

#### ScriptEngine

Main entry point for script execution.

**Constructors**:
- `ScriptEngine::with_builtin()` - Create engine with built-in instructions
- `ScriptEngine::with_registry_and_config(registry, config)` - Create with custom registry and config

**Methods**:
- `compile_and_execute(script: &str) -> Result<(), ScriptError>` - Synchronously compile and execute script
- `compile_and_execute_async(script: &str) -> Result<(), ScriptError>` - Compile and execute in background thread
- `get_interrupt_controller() -> InterruptController` - Get control handle
- `get_vm_context() -> VMContext` - Access VM state

#### ScriptConfig

Configuration for script engine behavior (default config is usually sufficient).

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

Built-in instructions are automatically available via `ScriptEngine::with_builtin()`.

#### Control Flow

| Instruction | Syntax | Description |
|-------------|--------|-------------|
| `loop` | `loop N { ... }` or `loop N ... end` | Repeat N times |
| `time` | `time MS { ... }` or `time MS ... end` | Loop within time limit |
| `sleep` | `sleep MS` | Wait milliseconds |
| `continue` | `continue` | Skip remaining iterations |
| `break` | `break` | Exit current loop |

#### Instruction Combination Examples

```rust
// Simple loop
loop 3
    sleep 100
end

// Time-limited loop
time 500
    sleep 100
end

// With continue and break
loop 10
    sleep 50
    continue    // Skip remaining code
    sleep 100   // This won't execute
end
```

## Script Syntax

### Basic Syntax

Scripts use simple instruction sequences:

```rust
sleep 100        // Wait 100 milliseconds
loop 5           // Repeat 5 times
    sleep 50
end
```

### Time-Limited Loops

The `time` instruction creates a loop that runs for a specified duration:

```rust
time 500         // Run for at most 500 milliseconds
    sleep 100
end              // Executes approximately 5 times
```

### Loop Control

- `continue` - Skip remaining instructions in current iteration
- `break` - Exit loop immediately

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

1. **Use `with_builtin()` to Create Engine**
   ```rust
   // Recommended: Use built-in instructions
   let engine = ScriptEngine::with_builtin();

   // Or custom registry
   let mut registry = InstructionRegistry::new();
   win_auto_utils::scripts_builtin::register_all(&mut registry);
   let engine = ScriptEngine::with_registry_and_config(registry, ScriptConfig::default());
   ```

2. **Use `compile_and_execute()` to Run Scripts**
   ```rust
   // Good: Compile and execute
   engine.compile_and_execute("sleep 100\n    continue\n    sleep 100\nend")?;

   // Old API (deprecated)
   // engine.execute_script(...);
   ```

3. **Set Reasonable Loop Limits**
   ```rust
   // Good: Finite loop
   loop 100 {
       sleep 10
   }
   ```

4. **Use `time` Instruction for Timeout Control**
   ```rust
   // Good: With timeout
   time 5000
       sleep 100
   end
   ```

5. **Use `continue` and `break` for Loop Control**
   ```rust
   loop 100
       sleep 50
       continue    // Jump to next iteration
       sleep 100   // Won't execute
   end
   ```

## Common Pitfalls

### ❌ Using Curly Brace Syntax (Not Supported in New Version)

```rust
// Wrong: Using curly braces
loop 3 {
    sleep 100
}

// Correct: Use end keyword
loop 3
    sleep 100
end
```

### ❌ Using Old API

```rust
// Wrong: Old API
let engine = ScriptEngine::new();
engine.execute_script("...");

// Correct: New API
let engine = ScriptEngine::with_builtin();
engine.compile_and_execute("...")?;
```

### ❌ Forgetting end Keyword

```rust
// Wrong: Missing end
loop 3
    sleep 100
// Missing end

// Correct: Close the block
loop 3
    sleep 100
end
```

## Extending the Engine

### Register Custom Instructions

```rust
use win_auto_utils::script_engine::{Instruction, InstructionHandler, InstructionRegistry, VM};

struct MyInstruction;

impl InstructionHandler for MyInstruction {
    fn parse(&self, tokens: &[&str]) -> Result<Instruction, String> {
        Ok(Instruction::new("my_instruction"))
    }

    fn execute(&self, _vm: &mut VM, _instruction: &Instruction) -> Result<(), String> {
        println!("Executing custom instruction");
        Ok(())
    }
}

// Register with engine
let mut registry = InstructionRegistry::new();
registry.register("my_instruction", Box::new(MyInstruction));
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
