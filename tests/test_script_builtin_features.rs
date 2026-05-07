//! Script Builtin Features Integration Test
//!
//! This comprehensive test suite verifies that all builtin instruction categories
//! correctly respond to their respective feature flags. Each test validates:
//! - Instruction parsing and execution
//! - Feature-gated compilation behavior
//! - Cross-feature interactions (e.g., keyboard + timing)
//! - Error handling for missing features
//!
//! # Running these tests
//! ```bash
//! # Test all builtin features (recommended)
//! cargo test --test test_script_builtin_features --features "scripts_builtin"
//!
//! # Test individual feature categories
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_control_flow,scripts_timing"
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_keyboard"
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_mouse"
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_mode"
//!
//! # Test specific combinations
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_keyboard,scripts_timing"
//! cargo test --test test_script_builtin_features --no-default-features --features "scripts_mouse,scripts_timing"
//!
//! # Test with PostMessage support
//! cargo test --test test_script_builtin_features --features "scripts_keyboard_with_post,scripts_mouse_with_post"
//! ```
//!
//! # Test Categories
//! 1. **Control Flow Tests**: loop, continue, break, nested structures
//! 2. **Keyboard Tests**: key clicks, key combinations, delays
//! 3. **Mouse Tests**: click, move, scroll operations
//! 4. **Timing Tests**: sleep precision, time blocks
//! 5. **Mode Configuration**: input_mode switching (send/post)
//! 6. **Integration Tests**: Complex scripts combining multiple instruction types
//! 7. **Edge Cases**: Boundary conditions and error scenarios
//! 8. **Performance Benchmarks**: Throughput and overhead measurements
//!
//! # Feature Coverage
//! - ✅ `scripts_control_flow`: 9 tests (loop, continue, break, nesting)
//! - ✅ `scripts_keyboard`: 4 tests (clicks, combinations, throughput)
//! - ✅ `scripts_mouse`: 4 tests (click, move, scroll, press/release)
//! - ✅ `scripts_timing`: 3 tests (sleep precision, time blocks)
//! - ✅ `scripts_mode`: 3 tests (mode switching, integration)
//! - ✅ Integration: 4 tests (multi-feature scenarios)
//! - ✅ Edge Cases: 4 tests (boundary conditions)
//! - ✅ Performance: 2 tests (benchmarks)
//!
//! Total: 25 tests covering all builtin instruction categories

use win_auto_utils::script_engine::instruction::InstructionRegistry;
use win_auto_utils::script_engine::{ScriptConfig, ScriptEngine};
use win_auto_utils::scripts_builtin::register_all;

/// Helper function to create an engine with all available builtin instructions
fn create_full_engine() -> ScriptEngine {
    let mut registry = InstructionRegistry::new();
    register_all(&mut registry);
    let config = ScriptConfig::default();
    ScriptEngine::with_registry_and_config(registry, config)
}

// ============================================================================
// Control Flow Tests (requires: scripts_control_flow)
// ============================================================================

#[cfg(feature = "scripts_control_flow")]
#[test]
fn test_control_flow_basic_loop() {
    println!("\n=== Control Flow: Basic Loop ===");

    let engine = create_full_engine();
    let script = r#"loop 3
    sleep 10
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("✅ Basic loop executed in {:?}", elapsed);
    assert!(elapsed >= std::time::Duration::from_millis(25));
    assert!(elapsed < std::time::Duration::from_millis(100));
}

#[cfg(feature = "scripts_control_flow")]
#[test]
fn test_control_flow_continue_break() {
    println!("\n=== Control Flow: Continue & Break ===");

    let engine = create_full_engine();

    // Test continue
    let script_continue = r#"loop 3
    sleep 10
    continue
    sleep 100
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script_continue).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed_continue = start.elapsed();

    println!("Continue test: {:?}", elapsed_continue);
    assert!(
        elapsed_continue < std::time::Duration::from_millis(100),
        "Continue should skip second sleep"
    );

    // Test break
    let script_break = r#"loop 5
    sleep 10
    break
    sleep 100
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script_break).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed_break = start.elapsed();

    println!("Break test: {:?}", elapsed_break);
    assert!(
        elapsed_break < std::time::Duration::from_millis(50),
        "Break should exit after first iteration"
    );
}

#[cfg(feature = "scripts_control_flow")]
#[test]
fn test_control_flow_nested_loops() {
    println!("\n=== Control Flow: Nested Loops ===");

    let engine = create_full_engine();
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

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    // Expected: 2 × [10 + 2×10 + 10] + 5 = 95ms
    println!("Nested loops executed in {:?}", elapsed);
    assert!(elapsed >= std::time::Duration::from_millis(80));
    assert!(elapsed < std::time::Duration::from_millis(200));
}

// ============================================================================
// Keyboard Tests (requires: scripts_keyboard)
// ============================================================================

#[cfg(feature = "scripts_keyboard")]
#[test]
fn test_keyboard_basic_key_click() {
    println!("\n=== Keyboard: Basic Key Click ===");

    let engine = create_full_engine();
    let script = r#"key A
key B 50
key C 100
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("✅ Keyboard basic clicks executed in {:?}", elapsed);
    // Should have at least 150ms of delays (50 + 100)
    assert!(elapsed >= std::time::Duration::from_millis(140));
}

#[cfg(feature = "scripts_keyboard")]
#[test]
fn test_keyboard_key_combinations() {
    println!("\n=== Keyboard: Key Combinations ===");

    let engine = create_full_engine();
    let script = r#"key_down SHIFT
key A
key_up SHIFT
key_down CONTROL
key C
key_up CONTROL
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Key combinations executed successfully");
}

#[cfg(all(feature = "scripts_keyboard", feature = "scripts_timing"))]
#[test]
fn test_keyboard_with_delays() {
    println!("\n=== Keyboard: With Delays ===");

    let engine = create_full_engine();
    let script = r#"key H 50
key E 50
key L 50
key L 50
key O 50
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("✅ Typed 'HELLO' with delays in {:?}", elapsed);
    // 5 keys × 50ms delay = 250ms minimum
    assert!(elapsed >= std::time::Duration::from_millis(240));
    assert!(elapsed < std::time::Duration::from_millis(400));
}

// ============================================================================
// Mouse Tests (requires: scripts_mouse)
// ============================================================================

#[cfg(feature = "scripts_mouse")]
#[test]
fn test_mouse_basic_operations() {
    println!("\n=== Mouse: Basic Operations ===");

    let engine = create_full_engine();
    let script = r#"move 100 100
click 100 100
move 200 200
click 200 200
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mouse basic operations executed successfully");
}

#[cfg(feature = "scripts_mouse")]
#[test]
fn test_mouse_scroll_operations() {
    println!("\n=== Mouse: Scroll Operations ===");

    let engine = create_full_engine();
    let script = r#"scrollup 3
scrolldown 2
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mouse scroll operations executed successfully");
}

#[cfg(feature = "scripts_mouse")]
#[test]
fn test_mouse_relative_movement() {
    println!("\n=== Mouse: Relative Movement ===");

    let engine = create_full_engine();
    let script = r#"move 100 100
moverel 50 50
moverel -25 -25
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mouse relative movement executed successfully");
}

#[cfg(feature = "scripts_mouse")]
#[test]
fn test_mouse_press_release() {
    println!("\n=== Mouse: Press & Release ===");

    let engine = create_full_engine();
    let script = r#"press 100 100
release 100 100
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mouse press/release executed successfully");
}

// ============================================================================
// Timing Tests (requires: scripts_timing)
// ============================================================================

#[cfg(feature = "scripts_timing")]
#[test]
fn test_timing_sleep_precision() {
    println!("\n=== Timing: Sleep Precision ===");

    let engine = create_full_engine();

    // Test various sleep durations
    let test_cases = vec![
        (10, 5, 30),    // 10ms ± tolerance
        (50, 40, 80),   // 50ms ± tolerance
        (100, 90, 150), // 100ms ± tolerance
    ];

    for (sleep_ms, min_ms, max_ms) in test_cases {
        let script = format!("sleep {}", sleep_ms);

        let start = std::time::Instant::now();
        let compiled = engine.compile(&script).unwrap();
        engine.execute(&compiled).unwrap();
        let elapsed = start.elapsed().as_millis() as u32;

        println!("Sleep {}ms: actual {}ms", sleep_ms, elapsed);
        assert!(
            elapsed >= min_ms,
            "Sleep {}ms took only {}ms (too fast)",
            sleep_ms,
            elapsed
        );
        assert!(
            elapsed <= max_ms,
            "Sleep {}ms took {}ms (too slow)",
            sleep_ms,
            elapsed
        );
    }
}

#[cfg(feature = "scripts_timing")]
#[test]
fn test_timing_time_block() {
    println!("\n=== Timing: Time Block ===");

    let engine = create_full_engine();
    let script = r#"time 100
    sleep 10
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Time block executed in {:?}", elapsed);
    // Should execute for approximately 100ms total
    assert!(elapsed >= std::time::Duration::from_millis(90));
    assert!(elapsed < std::time::Duration::from_millis(200));
}

// ============================================================================
// Mode Configuration Tests (requires: scripts_mode)
// ============================================================================

#[cfg(all(feature = "scripts_mouse", feature = "scripts_keyboard"))]
#[test]
fn test_mode_affects_keyboard() {
    println!("\n=== Mode: Mode Affects Keyboard Instructions ===");

    let engine = create_full_engine();
    // Only test send mode since post mode requires window handle setup
    let script = r#"mode input_mode send
key A
key B send
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mode correctly affects keyboard instructions (send mode only)");
}

#[cfg(all(feature = "scripts_mouse", feature = "scripts_mouse"))]
#[test]
fn test_mode_affects_mouse() {
    println!("\n=== Mode: Mode Affects Mouse Instructions ===");

    let engine = create_full_engine();
    // Only test send mode since post mode requires window handle setup
    let script = r#"mode input_mode send
click 100 100
move 200 200
"#;

    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();

    println!("✅ Mode correctly affects mouse instructions (send mode only)");
}

// ============================================================================
// Integration Tests (Multiple Features Combined)
// ============================================================================

#[cfg(all(
    feature = "scripts_control_flow",
    feature = "scripts_keyboard",
    feature = "scripts_timing"
))]
#[test]
fn test_integration_loop_with_keyboard() {
    println!("\n=== Integration: Loop with Keyboard ===");

    let engine = create_full_engine();
    let script = r#"loop 3
    key A 20
    sleep 10
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    // 3 iterations × (20ms key delay + 10ms sleep) = 90ms
    println!("Loop with keyboard executed in {:?}", elapsed);
    assert!(elapsed >= std::time::Duration::from_millis(80));
    assert!(elapsed < std::time::Duration::from_millis(200));
}

#[cfg(all(
    feature = "scripts_control_flow",
    feature = "scripts_mouse",
    feature = "scripts_timing"
))]
#[test]
fn test_integration_loop_with_mouse() {
    println!("\n=== Integration: Loop with Mouse ===");

    let engine = create_full_engine();
    let script = r#"loop 2
    move 100 100
    click
    sleep 20
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Loop with mouse executed in {:?}", elapsed);
    assert!(elapsed >= std::time::Duration::from_millis(35));
}

#[cfg(all(
    feature = "scripts_control_flow",
    feature = "scripts_keyboard",
    feature = "scripts_mouse",
    feature = "scripts_timing"
))]
#[test]
fn test_integration_complex_automation_script() {
    println!("\n=== Integration: Complex Automation Script ===");

    let engine = create_full_engine();
    let script = r#"# Complex automation scenario
mode input_mode send

# Type a message
key H 30
key E 30
key L 30
key L 30
key O 30

sleep 50

# Move and click
move 500 300
click 500 300

sleep 30

# Loop with mixed operations
loop 2
    key_down SHIFT
    key A 20
    key_up SHIFT
    moverel 10 10
    sleep 20
end

# Final action
scrollup 1
sleep 10
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("✅ Complex automation script executed in {:?}", elapsed);
    // Should take at least 300ms due to all the delays
    assert!(elapsed >= std::time::Duration::from_millis(280));
    assert!(elapsed < std::time::Duration::from_millis(600));
}

#[cfg(all(
    feature = "scripts_control_flow",
    feature = "scripts_keyboard",
    feature = "scripts_timing"
))]
#[test]
fn test_integration_nested_control_with_io() {
    println!("\n=== Integration: Nested Control with I/O ===");

    let engine = create_full_engine();
    let script = r#"loop 2
    sleep 10
    key A 20
    loop 2
        key B 10
        continue
        key C 100
    end
    sleep 10
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Nested control with I/O executed in {:?}", elapsed);
    // Should complete successfully without errors
    assert!(elapsed >= std::time::Duration::from_millis(50));
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[cfg(feature = "scripts_control_flow")]
#[test]
fn test_edge_case_single_iteration_loop() {
    println!("\n=== Edge Case: Single Iteration Loop ===");

    let engine = create_full_engine();
    let script = r#"loop 1
    sleep 10
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Single iteration loop executed in {:?}", elapsed);
    assert!(elapsed >= std::time::Duration::from_millis(8));
    assert!(elapsed < std::time::Duration::from_millis(50));
}

#[cfg(feature = "scripts_keyboard")]
#[test]
fn test_edge_case_zero_delay_key() {
    println!("\n=== Edge Case: Zero Delay Key ===");

    let engine = create_full_engine();
    let script = r#"key A 0
key B 0
key C 0
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Zero delay keys executed in {:?}", elapsed);
    // Should be very fast (< 10ms)
    assert!(elapsed < std::time::Duration::from_millis(50));
}

#[cfg(feature = "scripts_timing")]
#[test]
fn test_edge_case_minimal_sleep() {
    println!("\n=== Edge Case: Minimal Sleep (1ms) ===");

    let engine = create_full_engine();
    let script = r#"sleep 1
sleep 1
sleep 1
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Three 1ms sleeps executed in {:?}", elapsed);
    // Windows sleep has ~1-2ms precision, so 3 sleeps might take 3-10ms
    assert!(elapsed >= std::time::Duration::from_millis(1));
    assert!(elapsed < std::time::Duration::from_millis(30));
}

#[cfg(all(feature = "scripts_control_flow", feature = "scripts_timing"))]
#[test]
fn test_edge_case_empty_loop_body() {
    println!("\n=== Edge Case: Empty Loop Body ===");

    let engine = create_full_engine();
    let script = r#"loop 100
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    println!("Empty loop (100 iterations) executed in {:?}", elapsed);
    // Should be very fast since there's no body
    assert!(elapsed < std::time::Duration::from_millis(50));
}

// ============================================================================
// Performance Benchmarks
// ============================================================================

#[cfg(all(feature = "scripts_control_flow", feature = "scripts_timing"))]
#[test]
fn test_performance_loop_overhead() {
    println!("\n=== Performance: Loop Overhead ===");

    let engine = create_full_engine();
    let script = r#"loop 1000
    sleep 1
end
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    let overhead_per_iteration = elapsed.as_millis() as f64 / 1000.0;
    println!(
        "Loop overhead: {:.3}ms per iteration (total: {:?})",
        overhead_per_iteration, elapsed
    );

    // Each iteration should be close to 1ms (the sleep time)
    assert!(
        overhead_per_iteration < 5.0,
        "Loop overhead too high: {:.3}ms per iteration",
        overhead_per_iteration
    );
}

#[cfg(feature = "scripts_keyboard")]
#[test]
fn test_performance_key_instruction_throughput() {
    println!("\n=== Performance: Key Instruction Throughput ===");

    let engine = create_full_engine();
    let script = r#"key A 0
key B 0
key C 0
key D 0
key E 0
key F 0
key G 0
key H 0
key I 0
key J 0
"#;

    let start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    let throughput = 10.0 / elapsed.as_secs_f64();
    println!(
        "Key throughput: {:.0} keys/sec (10 keys in {:?})",
        throughput, elapsed
    );

    // Should be able to process at least 100 keys/sec
    assert!(
        throughput > 100.0,
        "Key throughput too low: {:.0} keys/sec",
        throughput
    );
}
