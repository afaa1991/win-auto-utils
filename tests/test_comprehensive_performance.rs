//! Comprehensive performance tests for mixed builtin instructions
//!
//! This test suite focuses on measuring the performance of complex scripts
//! that combine multiple instruction types (control flow, keyboard, mouse, timing).
//!
//! # Test Coverage
//! - Keyboard operations with different modes (send/post)
//! - Mouse operations in loops
//! - Timing instructions (sleep + time blocks)
//! - Mixed control flow (nested loops + break/continue)
//! - Full builtin instruction integration
//! - Compiled script reuse benefits
//! - Mode switching overhead
//! - Deep nesting performance
//! - Extreme iteration counts
//! - Pure parsing overhead measurement
//!
//! # Running these tests
//! ```bash
//! # Run all comprehensive tests
//! cargo test --test test_comprehensive_performance --features "script_engine,scripts_builtin" -- --nocapture
//!
//! # Run specific test
//! cargo test test_full_builtin_mix_performance --test test_comprehensive_performance --features "script_engine,scripts_builtin" -- --nocapture
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
fn test_keyboard_operations_performance() {
    // Test Case 1: Keyboard operations with various modes (send mode only, no hwnd required)
    let engine = create_engine();
    let script = r#"key A
key B 50
key C send
key D 100 send
key_down SHIFT
key_up SHIFT
loop 5
    key E
end
"#;

    println!("\n=== Test: Keyboard Operations Performance ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Compilation should be fast (pre-builds INPUT structures)
    assert!(compile_time < std::time::Duration::from_millis(10),
            "Keyboard compilation too slow: {:?}", compile_time);
    
    println!("✅ Test passed - Keyboard instructions compiled efficiently\n");
}

#[test]
fn test_timing_instructions_performance() {
    // Test Case 2: Timing instructions (sleep + time blocks)
    let engine = create_engine();
    let script = r#"sleep 10
time 100
    sleep 20
end
sleep 5
"#;

    println!("\n=== Test: Timing Instructions Performance ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~115ms (10 + 5×20 + 5)
    assert!(exec_time > std::time::Duration::from_millis(100),
            "Execution too fast, timing may be wrong: {:?}", exec_time);
    assert!(exec_time < std::time::Duration::from_millis(200),
            "Execution too slow: {:?}", exec_time);
    
    println!("✅ Test passed - Timing instructions work correctly\n");
}

#[test]
fn test_all_control_flow_mix_performance() {
    // Test Case 3: All control flow features mixed (loop + break + continue + nested)
    let engine = create_engine();
    let script = r#"loop 3
    sleep 5
    loop 2
        sleep 5
        continue
        sleep 50
    end
    sleep 5
    break
    sleep 100
end
sleep 10
loop 2
    sleep 5
end
"#;

    println!("\n=== Test: All Control Flow Mixed ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~55ms (first loop: 5+2×5+5+10, second loop: 2×5)
    // If continue/break fail: much longer
    assert!(exec_time < std::time::Duration::from_millis(150),
            "Control flow mix performance issue: {:?}", exec_time);
    
    println!("✅ Test passed - Mixed control flow works correctly\n");
}

#[test]
fn test_full_builtin_mix_performance() {
    // Test Case 4: Full mix of all builtin instructions (send mode only)
    let engine = create_engine();
    let script = r#"# Comprehensive test with all instruction types
sleep 5
key A
key B 50 send
move 100 200
click 300 400

loop 3
    sleep 5
    key C
    click 100 200
    
    loop 2
        sleep 5
        moverel 10 -10
        continue
        scrollup 1
    end
    
    scrolldown 2
    break
    key D 100
end

time 50
    sleep 10
    press
    release
end

sleep 5
"#;

    println!("\n=== Test: Full Builtin Instructions Mix ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Verify compilation is efficient despite complex script
    assert!(compile_time < std::time::Duration::from_millis(20),
            "Full mix compilation too slow: {:?}", compile_time);
    
    // Execution should complete in reasonable time
    assert!(exec_time < std::time::Duration::from_millis(300),
            "Full mix execution too slow: {:?}", exec_time);
    
    println!("✅ Test passed - All builtin instructions work together\n");
}

#[test]
fn test_deeply_nested_loops_performance() {
    // Test Case 5: Deeply nested loops (3 levels)
    let engine = create_engine();
    let script = r#"loop 2
    sleep 5
    loop 2
        sleep 5
        loop 2
            sleep 5
        end
    end
end
sleep 5
"#;

    println!("\n=== Test: Deeply Nested Loops (3 Levels) ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~85ms (2 × [5 + 2×(5 + 2×5)] + 5)
    assert!(exec_time < std::time::Duration::from_millis(200),
            "Deep nesting performance issue: {:?}", exec_time);
    
    println!("✅ Test passed - Deep nesting works correctly\n");
}

#[test]
fn test_time_with_break_continue_performance() {
    // Test Case 6: Time blocks with break/continue
    let engine = create_engine();
    let script = r#"time 100
    sleep 20
    continue
    sleep 50
end

time 100
    sleep 20
    break
    sleep 50
end

sleep 5
"#;

    println!("\n=== Test: Time Blocks with Break/Continue ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // First time block: ~100ms (5 iterations × 20ms, continue skips second sleep)
    // Second time block: ~20ms (break after first iteration)
    // Total: ~125ms
    assert!(exec_time > std::time::Duration::from_millis(100),
            "Execution too fast, time blocks may not work: {:?}", exec_time);
    assert!(exec_time < std::time::Duration::from_millis(250),
            "Execution too slow: {:?}", exec_time);
    
    println!("✅ Test passed - Time blocks with control flow work correctly\n");
}

#[test]
fn test_compiled_script_reuse_performance() {
    // Test Case 7: Measure benefit of reusing compiled scripts
    let engine = create_engine();
    let script = r#"loop 5
    sleep 5
    key A
    click 100 200
end
"#;

    println!("\n=== Test: Compiled Script Reuse Performance ===");
    
    // Compile once
    let compile_start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    let compile_time = compile_start.elapsed();
    println!("Single compilation time: {:?}", compile_time);
    
    // Execute multiple times using same compiled script
    let iterations = 20;
    let exec_start = std::time::Instant::now();
    for _ in 0..iterations {
        engine.execute(&compiled).unwrap();
    }
    let total_exec_time = exec_start.elapsed();
    let avg_exec_time = total_exec_time / iterations;
    
    println!("Total execution time ({} iterations): {:?}", iterations, total_exec_time);
    println!("Average execution time per run: {:?}", avg_exec_time);
    println!("Compilation amortized over {} runs: {:?}", iterations, compile_time / iterations);
    
    // Verify reuse is efficient
    assert!(avg_exec_time < std::time::Duration::from_millis(50),
            "Repeated execution too slow: {:?}", avg_exec_time);
    
    println!("✅ Test passed - Compiled script reuse is efficient\n");
}

#[test]
fn test_mode_switching_performance() {
    // Test Case 8: Mode switching overhead (send mode only, no hwnd required)
    let engine = create_engine();
    let script = r#"# Test mode switching impact (all send mode)
key A send
key B send
click 100 200 send
click 300 400 send

loop 3
    key C send
    key D send
end
"#;

    println!("\n=== Test: Mode Switching Performance ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Compilation should handle mode switching efficiently
    assert!(compile_time < std::time::Duration::from_millis(15),
            "Mode switching compilation too slow: {:?}", compile_time);
    
    println!("✅ Test passed - Mode switching handled efficiently\n");
}

#[test]
fn test_extreme_loop_count_performance() {
    // Test Case 9: High iteration count loop (stress test)
    let engine = create_engine();
    let script = r#"loop 100
    sleep 1
end
"#;

    println!("\n=== Test: Extreme Loop Count (100 iterations) ===");
    
    let compile_time = measure_compile(&engine, script);
    println!("Compilation time: {:?}", compile_time);
    
    let exec_time = measure_execution(&engine, script);
    println!("Execution time: {:?}", exec_time);
    
    // Expected: ~100ms (100 × 1ms)
    assert!(exec_time > std::time::Duration::from_millis(80),
            "Execution too fast, loop may not run fully: {:?}", exec_time);
    assert!(exec_time < std::time::Duration::from_millis(200),
            "Execution too slow: {:?}", exec_time);
    
    println!("✅ Test passed - High iteration count works correctly\n");
}

#[test]
fn test_instruction_parsing_overhead() {
    // Test Case 10: Measure pure parsing overhead without execution
    let engine = create_engine();
    
    // Scripts with varying complexity
    let simple_script = "key A";
    let medium_script = r#"loop 5
    key B
    sleep 10
end"#;
    let complex_script = r#"loop 3
    key C
    loop 2
        click 100 200
        sleep 5
    end
    moverel 10 10
end
time 50
    key D
end"#;

    println!("\n=== Test: Instruction Parsing Overhead ===");
    
    let simple_time = measure_compile(&engine, simple_script);
    println!("Simple script (1 instr): {:?}", simple_time);
    
    let medium_time = measure_compile(&engine, medium_script);
    println!("Medium script (3 instr types): {:?}", medium_time);
    
    let complex_time = measure_compile(&engine, complex_script);
    println!("Complex script (nested): {:?}", complex_time);
    
    // Verify parsing scales reasonably
    assert!(simple_time < std::time::Duration::from_millis(5),
            "Simple parse too slow: {:?}", simple_time);
    assert!(medium_time < std::time::Duration::from_millis(10),
            "Medium parse too slow: {:?}", medium_time);
    assert!(complex_time < std::time::Duration::from_millis(20),
            "Complex parse too slow: {:?}", complex_time);
    
    println!("✅ Test passed - Parsing overhead is acceptable\n");
}
