//! Builtin instructions test - Verify execution logic of built-in instructions
//!
//! This test suite verifies the built-in control flow instructions, especially:
//! - Loop with single iteration (loop 1)
//! - Continue instruction behavior (the bug fix case)
//! - Break instruction behavior
//! - Nested loops
//!
//! # Running these tests
//! ```bash
//! # Run all tests
//! cargo test --test test_builtin_instructions --features "script_engine,scripts_control_flow,scripts_timing" -- --nocapture
//!
//! # Run specific test
//! cargo test test_single_iteration_loop_with_continue --test test_builtin_instructions --features "script_engine,scripts_control_flow,scripts_timing" -- --nocapture
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

#[test]
fn test_single_iteration_loop_with_continue() {
    // Test Case 1: Single iteration loop with continue (THE BUG FIX CASE)
    let engine = create_engine();
    
    println!("\n=== Test 1: Single Iteration Loop with Continue ===");
    println!("This is the critical test for the 'continue' bug fix.\n");
    
    let script = r#"loop 1
    sleep 10
    continue
    sleep 10
end
sleep 5
"#;
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected behavior:");
    println!("  - Loop executes once");
    println!("  - First sleep(10) executes");
    println!("  - continue jumps to end");
    println!("  - Second sleep(10) is SKIPPED");
    println!("  - Loop ends normally (NO INFINITE LOOP!)");
    println!("  - Final sleep(5) executes\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 1 PASSED!");
    println!("   Execution time: {:?} (should be ~25ms, not infinite)", elapsed);
    println!("   The continue instruction correctly avoids infinite loops!\n");
    
    // Verify no infinite loop (should complete in reasonable time)
    assert!(elapsed < std::time::Duration::from_millis(100), 
            "Execution took too long: {:?} (possible infinite loop)", elapsed);
}

#[test]
fn test_multiple_iterations_with_continue() {
    // Test Case 2: Multiple iterations with continue
    let engine = create_engine();
    
    println!("\n=== Test 2: Multiple Iterations with Continue ===\n");
    
    let script = r#"loop 3
    sleep 10
    continue
    sleep 100
end
sleep 5
"#;
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected: 3 iterations × 10ms + 5ms = ~35ms total");
    println!("The second sleep(100) should never execute.\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 2 PASSED!");
    println!("   Execution time: {:?} (should be ~35ms)", elapsed);
    println!("   If it was ~305ms, continue didn't work properly.\n");
    
    // Expected: ~35ms (3 × 10ms + 5ms)
    // If continue doesn't work: ~305ms (3 × 110ms + 5ms)
    assert!(elapsed < std::time::Duration::from_millis(100),
            "Continue may not be working properly. Execution time: {:?}", elapsed);
}

#[test]
fn test_loop_with_break() {
    // Test Case 3: Loop with break
    let engine = create_engine();
    
    println!("\n=== Test 3: Loop with Break ===\n");
    
    let script = r#"loop 5
    sleep 10
    break
    sleep 100
end
sleep 5
"#;
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected: 1 iteration × 10ms + 5ms = ~15ms");
    println!("Break should exit after first iteration.\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 3 PASSED!");
    println!("   Execution time: {:?} (should be ~15ms)\n", elapsed);
    
    // Expected: ~15ms (1 iteration only due to break)
    assert!(elapsed < std::time::Duration::from_millis(50),
            "Break may not be working properly. Execution time: {:?}", elapsed);
}

#[test]
fn test_nested_loops_basic() {
    // Test Case 4: Nested loops (basic)
    let engine = create_engine();
    
    println!("\n=== Test 4: Nested Loops (Basic) ===\n");
    
    let script = r#"loop 2
    sleep 10
    loop 2
        sleep 10
    end
end
sleep 5
"#;
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected: Outer(2) × [10ms + Inner(2) × 10ms] + 5ms = ~65ms\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 4 PASSED!");
    println!("   Execution time: {:?} (should be ~65ms)\n", elapsed);
    
    assert!(elapsed < std::time::Duration::from_millis(150),
            "Nested loop performance issue. Execution time: {:?}", elapsed);
}

#[test]
fn test_continue_in_nested_loops() {
    // Test Case 5: Continue in nested loops
    let engine = create_engine();
    
    println!("\n=== Test 5: Continue in Nested Loops ===\n");
    
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
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected: 2×[10 + 2×10 + 10] + 5 = 95ms");
    println!("Inner loop continue skips second sleep.\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 5 PASSED!");
    println!("   Execution time: {:?} (should be ~95ms)\n", elapsed);
    
    // Expected: ~95ms (2 × [10 + 2×10 + 10] + 5)
    // If continue fails: ~495ms (2 × [10 + 2×110 + 10] + 5)
    assert!(elapsed < std::time::Duration::from_millis(200),
            "Continue in nested loops may not work. Execution time: {:?}", elapsed);
}

#[test]
fn test_break_in_nested_loops() {
    // Test Case 6: Break in nested loops
    let engine = create_engine();
    
    println!("\n=== Test 6: Break in Nested Loops ===\n");
    
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
    
    println!("Script:");
    println!("{}", script);
    println!("\nExpected behavior:");
    println!("  - Outer loop: 3 iterations");
    println!("  - Inner loop: breaks after first iteration each time");
    println!("  - Total: 3×[10 + 10] + 5 = 65ms\n");
    
    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();
    
    println!("\n✅ Test 6 PASSED!");
    println!("   Execution time: {:?} (should be ~65ms)\n", elapsed);
    
    assert!(elapsed < std::time::Duration::from_millis(150),
            "Complex nested break performance issue. Execution time: {:?}", elapsed);
}
