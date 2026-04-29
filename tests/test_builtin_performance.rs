//! Performance tests for builtin instructions
//!
//! This test suite measures the execution time of each phase:
//! - Compilation: Parsing script text and converting to compiled instructions
//! - Execution: Running compiled instructions in VM
//!
//! # Purpose
//! - Verify correctness of control flow instructions (loop, continue, break)
//! - Measure performance characteristics of compilation and execution phases
//! - Detect regressions (e.g., infinite loops from incorrect continue implementation)
//! - Serve as reusable template for testing new builtin instructions
//!
//! # Running these tests
//! ```bash
//! # Run all tests with output
//! cargo test --test test_builtin_performance --features "script_engine,scripts_control_flow,scripts_timing" -- --nocapture
//!
//! # Run specific test
//! cargo test test_single_loop_continue_performance --test test_builtin_performance --features "script_engine,scripts_control_flow,scripts_timing" -- --nocapture
//! ```
//!
//! # Adding New Tests
//! To test a new builtin instruction:
//! 1. Copy an existing test function as template
//! 2. Modify the script to use your new instruction
//! 3. Add appropriate assertions for expected behavior
//! 4. Include timing measurements using helper functions
//!
//! # Example Test Pattern
//! ```ignore
//! #[test]
//! fn test_your_new_instruction() {
//!     let engine = create_engine();
//!     let script = r#"your_script_here"#;
//!     
//!     let compile_time = measure_compile(&engine, script);
//!     let exec_time = measure_execution(&engine, script);
//!     
//!     println!("Compilation: {:?}", compile_time);
//!     println!("Execution: {:?}", exec_time);
//!     
//!     // Add assertions to verify correctness
//!     assert!(exec_time < expected_max_time);
//! }
//! ```

use win_auto_utils::script_engine::{ScriptConfig, ScriptEngine};
use win_auto_utils::scripts_builtin::register_all;
use win_auto_utils::script_engine::instruction::InstructionRegistry;

/// Helper function to create an engine with all builtin instructions
fn create_engine() -> ScriptEngine {
    let mut registry = InstructionRegistry::new();
    register_all(&mut registry);
    let config = ScriptConfig::default();
    ScriptEngine::with_registry_and_config(registry, config)
}

/// Measure compilation time (parsing + code generation)
fn measure_compile(engine: &ScriptEngine, script: &str) -> std::time::Duration {
    let start = std::time::Instant::now();
    let _ = engine.compile(script).unwrap();
    start.elapsed()
}

/// Measure execution time of a compiled script
fn measure_execution(engine: &ScriptEngine, script: &str) -> std::time::Duration {
    let compiled = engine.compile(script).unwrap();
    let start = std::time::Instant::now();
    engine.execute(&compiled).unwrap();
    start.elapsed()
}

#[test]
fn test_single_loop_continue_performance() {
    // Test Case 1: Single iteration loop with continue (the bug fix case)
    let engine = create_engine();
    let script = r#"loop 1
    sleep 10
    continue
    sleep 10
end
sleep 5
"#;

    println!("\n=== Test: Single Loop with Continue ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Verify no infinite loop (should complete in reasonable time)
    assert!(exec_time < std::time::Duration::from_millis(100), 
            "Execution took too long: {:?} (possible infinite loop)", exec_time);
    
    println!("✅ Test passed - No infinite loop detected\n");
}

#[test]
fn test_multiple_iterations_continue_performance() {
    // Test Case 2: Multiple iterations with continue
    let engine = create_engine();
    let script = r#"loop 3
    sleep 10
    continue
    sleep 100
end
sleep 5
"#;

    println!("\n=== Test: Multiple Iterations with Continue ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~35ms (3 × 10ms + 5ms)
    // If continue doesn't work: ~305ms (3 × 110ms + 5ms)
    assert!(exec_time < std::time::Duration::from_millis(100),
            "Continue may not be working properly. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Continue works correctly\n");
}

#[test]
fn test_break_performance() {
    // Test Case 3: Loop with break
    let engine = create_engine();
    let script = r#"loop 5
    sleep 10
    break
    sleep 100
end
sleep 5
"#;

    println!("\n=== Test: Loop with Break ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~15ms (1 iteration only due to break)
    assert!(exec_time < std::time::Duration::from_millis(50),
            "Break may not be working properly. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Break works correctly\n");
}

#[test]
fn test_nested_loops_performance() {
    // Test Case 4: Nested loops
    let engine = create_engine();
    let script = r#"loop 2
    sleep 10
    loop 2
        sleep 10
    end
end
sleep 5
"#;

    println!("\n=== Test: Nested Loops ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~65ms (2 × [10 + 2×10] + 5)
    assert!(exec_time < std::time::Duration::from_millis(150),
            "Nested loops performance issue. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Nested loops work correctly\n");
}

#[test]
fn test_continue_in_nested_loops_performance() {
    // Test Case 5: Continue in nested loops
    let engine = create_engine();
    let script = r#"loop 2
    sleep 10
    loop 2
        sleep 10
        continue
        sleep 100
    end
    sleep 10
end
sleep 5
"#;

    println!("\n=== Test: Continue in Nested Loops ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~95ms (2 × [10 + 2×10 + 10] + 5)
    // If continue fails: ~495ms (2 × [10 + 2×110 + 10] + 5)
    assert!(exec_time < std::time::Duration::from_millis(200),
            "Continue in nested loops may not work. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Continue in nested loops works correctly\n");
}

#[test]
fn test_break_in_nested_loops_performance() {
    // Test Case 6: Break in nested loops
    let engine = create_engine();
    let script = r#"loop 3
    sleep 10
    loop 3
        sleep 10
        break
        sleep 100
    end
end
sleep 5
"#;

    println!("\n=== Test: Break in Nested Loops ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~65ms (3 × [10 + 10] + 5)
    assert!(exec_time < std::time::Duration::from_millis(150),
            "Break in nested loops performance issue. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Break in nested loops works correctly\n");
}

#[test]
fn test_complex_script_performance() {
    // Test Case 7: Complex script with multiple features
    let engine = create_engine();
    let script = r#"loop 2
    sleep 5
    loop 3
        sleep 5
        continue
        sleep 50
    end
    sleep 5
    break
    sleep 100
end
sleep 5
"#;

    println!("\n=== Test: Complex Script ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Should complete quickly due to break
    assert!(exec_time < std::time::Duration::from_millis(100),
            "Complex script performance issue. Execution time: {:?}", exec_time);
    
    println!("✅ Test passed - Complex script works correctly\n");
}

#[test]
fn test_repeated_execution_performance() {
    // Test Case 8: Measure overhead of repeated execution
    let engine = create_engine();
    let script = r#"loop 2
    sleep 5
end
"#;

    println!("\n=== Test: Repeated Execution Performance ===");
    
    // Compile once
    let compiled = engine.compile(script).unwrap();
    let compile_time = std::time::Duration::from_millis(0); // Already compiled
    
    // Execute multiple times
    let iterations = 10;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        engine.execute(&compiled).unwrap();
    }
    let total_exec_time = start.elapsed();
    let avg_exec_time = total_exec_time / iterations;
    
    println!("Compilation time: {:?}", compile_time);
    println!("Total execution time ({} iterations): {:?}", iterations, total_exec_time);
    println!("Average execution time per run: {:?}", avg_exec_time);
    
    // Each execution should be fast (~10ms for 2×5ms sleeps)
    assert!(avg_exec_time < std::time::Duration::from_millis(50),
            "Repeated execution too slow. Avg time: {:?}", avg_exec_time);
    
    println!("✅ Test passed - Repeated execution is efficient\n");
}

#[test]
fn test_mouse_instructions_parse_performance() {
    // Test Case 9: Mouse instruction parsing performance (pre-compilation)
    let engine = create_engine();
    let script = r#"move 100 200
click 300 400
moverel 50 -30
scrollup 1
scrolldown 2
press
release
"#;

    println!("\n=== Test: Mouse Instructions Parse Performance ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Compilation should be fast (all computation done at compile time)
    assert!(compile_time < std::time::Duration::from_millis(10),
            "Compilation too slow: {:?}", compile_time);
    
    // Execution should be very fast (no computation, just inline API calls)
    assert!(exec_time < std::time::Duration::from_millis(50),
            "Execution too slow: {:?}", exec_time);
    
    println!("✅ Test passed - Mouse instructions use pre-compilation\n");
}

#[test]
fn test_mouse_loop_performance() {
    // Test Case 10: Mouse operations in loop (verify inline optimization)
    let engine = create_engine();
    let script = r#"loop 10
    click 100 200
    move 300 400
end
"#;

    println!("\n=== Test: Mouse Operations in Loop ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // With inline optimization, the instruction dispatch overhead is minimal
    // Actual execution time depends on whether mouse feature is enabled and OS calls
    // The key point is that compilation pre-computes everything
    assert!(compile_time < std::time::Duration::from_millis(10),
            "Compilation too slow: {:?}", compile_time);
    
    println!("✅ Test passed - Pre-compilation working correctly\n");
}
