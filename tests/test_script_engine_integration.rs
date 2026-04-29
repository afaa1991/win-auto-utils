//! Comprehensive script engine integration tests
//!
//! This test suite covers all major aspects of the script engine:
//! - Control flow (loop, break, continue, nested loops)
//! - Timing instructions (sleep, time blocks)
//! - Keyboard and mouse operations
//! - Interrupt mechanism
//! - Error handling
//!
//! # Running these tests
//! ```bash
//! # Run all comprehensive tests
//! cargo test --test test_script_engine_integration --features "script_engine,scripts_builtin" -- --nocapture
//!
//! # Run specific test category
//! cargo test test_control_flow --test test_script_engine_integration --features "script_engine,scripts_builtin" -- --nocapture
//! ```

use win_auto_utils::script_engine::{ScriptEngine, InterruptController};
use std::thread;
use std::time::Duration;

/// Helper function to create an engine with all builtin instructions
fn create_engine() -> ScriptEngine {
    ScriptEngine::with_builtin()
}

// ===== Control Flow Tests =====

#[test]
fn test_basic_loop() {
    let engine = create_engine();
    let script = r#"loop 3
    sleep 10
end"#;
    
    println!("\n=== Test: Basic Loop ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    assert!(elapsed < std::time::Duration::from_millis(100),
            "Basic loop too slow: {:?}", elapsed);
    println!("✅ Basic loop works correctly\n");
}

#[test]
fn test_loop_with_continue() {
    let engine = create_engine();
    let script = r#"loop 5
    sleep 5
    continue
    sleep 100
end"#;
    
    println!("\n=== Test: Loop with Continue ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    // Expected: ~25ms (5 × 5ms), not ~525ms (5 × 105ms)
    assert!(elapsed < std::time::Duration::from_millis(100),
            "Continue may not work. Time: {:?}", elapsed);
    println!("✅ Continue works correctly\n");
}

#[test]
fn test_loop_with_break() {
    let engine = create_engine();
    let script = r#"loop 10
    sleep 5
    break
    sleep 100
end"#;
    
    println!("\n=== Test: Loop with Break ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    // Expected: ~5ms (1 iteration only)
    assert!(elapsed < std::time::Duration::from_millis(50),
            "Break may not work. Time: {:?}", elapsed);
    println!("✅ Break works correctly\n");
}

#[test]
fn test_nested_loops() {
    let engine = create_engine();
    let script = r#"loop 2
    sleep 10
    loop 2
        sleep 10
    end
end"#;
    
    println!("\n=== Test: Nested Loops ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    // Expected: ~60ms (2 × [10 + 2×10])
    assert!(elapsed < std::time::Duration::from_millis(150),
            "Nested loops too slow: {:?}", elapsed);
    println!("✅ Nested loops work correctly\n");
}

#[test]
fn test_infinite_loop_with_break() {
    let engine = create_engine();
    let script = r#"loop
    sleep 5
    break
    sleep 100
end"#;
    
    println!("\n=== Test: Infinite Loop with Break ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    assert!(elapsed < std::time::Duration::from_millis(50),
            "Infinite loop break failed: {:?}", elapsed);
    println!("✅ Infinite loop with break works correctly\n");
}

// ===== Timing Tests =====

#[test]
fn test_timing_instructions() {
    let engine = create_engine();
    let script = r#"sleep 10
time 100
    sleep 20
end
sleep 5"#;
    
    println!("\n=== Test: Timing Instructions ===");
    let start = std::time::Instant::now();
    engine.compile_and_execute(script).unwrap();
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    // Expected: ~115ms (10 + 5×20 + 5)
    assert!(elapsed > std::time::Duration::from_millis(100),
            "Timing too fast: {:?}", elapsed);
    assert!(elapsed < std::time::Duration::from_millis(200),
            "Timing too slow: {:?}", elapsed);
    println!("✅ Timing instructions work correctly\n");
}

// ===== Keyboard & Mouse Tests =====

#[test]
fn test_keyboard_instructions() {
    let engine = create_engine();
    let script = r#"key A
key B 50
loop 3
    key C
end"#;
    
    println!("\n=== Test: Keyboard Instructions ===");
    let compile_time = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    let compile_elapsed = compile_time.elapsed();
    
    println!("Compilation time: {:?}", compile_elapsed);
    
    let exec_start = std::time::Instant::now();
    engine.execute(&compiled).unwrap();
    let exec_elapsed = exec_start.elapsed();
    
    println!("Execution time: {:?}", exec_elapsed);
    assert!(compile_elapsed < std::time::Duration::from_millis(10),
            "Keyboard compilation too slow: {:?}", compile_elapsed);
    println!("✅ Keyboard instructions compile efficiently\n");
}

#[test]
fn test_mouse_instructions() {
    let engine = create_engine();
    let script = r#"move 100 200
click 300 400
loop 2
    moverel 10 -10
end"#;
    
    println!("\n=== Test: Mouse Instructions ===");
    let compile_time = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    let compile_elapsed = compile_time.elapsed();
    
    println!("Compilation time: {:?}", compile_elapsed);
    
    let exec_start = std::time::Instant::now();
    engine.execute(&compiled).unwrap();
    let exec_elapsed = exec_start.elapsed();
    
    println!("Execution time: {:?}", exec_elapsed);
    assert!(compile_elapsed < std::time::Duration::from_millis(10),
            "Mouse compilation too slow: {:?}", compile_elapsed);
    println!("✅ Mouse instructions compile efficiently\n");
}

// ===== Interrupt Mechanism Tests =====

#[test]
fn test_interrupt_mechanism() {
    let engine = create_engine();
    let script = r#"loop 1000
    sleep 10
end"#;
    
    println!("\n=== Test: Interrupt Mechanism ===");
    
    let interrupt = InterruptController::new();
    let interrupt_clone = interrupt.clone();
    
    // Schedule interrupt after 100ms
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        interrupt_clone.request_interrupt();
    });
    
    let start = std::time::Instant::now();
    let _result = engine.compile_and_execute_with_interrupt(script, &interrupt);
    let elapsed = start.elapsed();
    
    println!("Execution time: {:?}", elapsed);
    println!("Interrupted: {}", interrupt.is_interrupted());
    
    // Should be interrupted around 100ms, not complete all 1000 iterations (~10s)
    assert!(elapsed < std::time::Duration::from_millis(500),
            "Interrupt not working. Time: {:?}", elapsed);
    assert!(interrupt.is_interrupted(),
            "Script should have been interrupted");
    
    println!("✅ Interrupt mechanism works correctly\n");
    
    // Reset for potential reuse
    interrupt.reset();
}

// ===== Error Handling Tests =====

#[test]
fn test_error_handling() {
    let engine = create_engine();
    
    println!("\n=== Test: Error Handling ===");
    
    // Test 1: Invalid syntax
    let invalid_script = "invalid_instruction";
    let result = engine.compile(invalid_script);
    assert!(result.is_err(), "Should fail on invalid instruction");
    println!("✓ Invalid instruction detected");
    
    // Test 2: Missing parameters
    let missing_params = "sleep";
    let result = engine.compile(missing_params);
    assert!(result.is_err(), "Should fail on missing parameters");
    println!("✓ Missing parameters detected");
    
    // Test 3: Invalid parameter type
    let invalid_param = "sleep abc";
    let result = engine.compile(invalid_param);
    assert!(result.is_err(), "Should fail on invalid parameter type");
    println!("✓ Invalid parameter type detected");
    
    println!("✅ Error handling works correctly\n");
}

// ===== Integration Test: Complex Script =====

#[test]
fn test_complex_script_integration() {
    let engine = create_engine();
    let script = r#"# Complex integration test
sleep 5
key A
loop 3
    sleep 5
    click 100 200
    loop 2
        moverel 10 10
        continue
    end
    break
    sleep 100
end
time 50
    sleep 10
end
sleep 5"#;
    
    println!("\n=== Test: Complex Script Integration ===");
    let compile_start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    let compile_elapsed = compile_start.elapsed();
    
    println!("Compilation time: {:?}", compile_elapsed);
    
    let exec_start = std::time::Instant::now();
    engine.execute(&compiled).unwrap();
    let exec_elapsed = exec_start.elapsed();
    
    println!("Execution time: {:?}", exec_elapsed);
    
    assert!(compile_elapsed < std::time::Duration::from_millis(20),
            "Complex script compilation too slow: {:?}", compile_elapsed);
    assert!(exec_elapsed < std::time::Duration::from_millis(300),
            "Complex script execution too slow: {:?}", exec_elapsed);
    
    println!("✅ Complex script integration test passed\n");
}
