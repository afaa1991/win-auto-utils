//! Example demonstrating the ScriptEngine::with_builtin() convenience method
//!
//! This example shows how to quickly create a script engine with all built-in
//! instructions registered, without manually setting up the registry.
//!
//! # Run this example:
//! ```bash
//! cargo run --example with_builtin_demo --features "script_engine,scripts_builtin"
//! ```

use win_auto_utils::script_engine::{ScriptConfig, ScriptEngine};

fn main() {
    println!("=== ScriptEngine::with_builtin() Demo ===\n");

    // Create engine with all built-in instructions (most convenient way)
    let config = ScriptConfig::default();
    let engine = ScriptEngine::with_config(config);

    println!("✓ Engine created with all built-in instructions\n");

    // Example 1: Simple loop with keyboard input
    println!("Example 1: Loop with key press");
    let script1 = r#"
loop 3
    key A
end
"#;

    match engine.compile_and_execute(script1) {
        Ok(()) => println!("✓ Script executed successfully\n"),
        Err(e) => println!("✗ Execution error: {}\n", e),
    }

    // Example 2: Nested loops
    println!("Example 2: Nested loops");
    let script2 = r#"
loop 2
    loop 2
        key B
    end
end
"#;

    match engine.compile_and_execute(script2) {
        Ok(()) => println!("✓ Nested loops executed successfully\n"),
        Err(e) => println!("✗ Execution error: {}\n", e),
    }

    // Example 3: Timing instruction
    println!("Example 3: Sleep instruction");
    let script3 = r#"
key C
sleep 100
key D
"#;

    match engine.compile_and_execute(script3) {
        Ok(()) => println!("✓ Sleep instruction executed successfully\n"),
        Err(e) => println!("✗ Execution error: {}\n", e),
    }

    println!("=== Demo Complete ===");
    println!("\nNote: This demo uses mock handlers that don't actually send");
    println!("keyboard input. In real usage, enable 'keyboard' feature for");
    println!("actual input simulation.");
}
