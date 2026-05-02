//! Performance test: Click with coordinates vs without coordinates
//!
//! This test verifies the performance difference between:
//! 1. `click x y` - Requires SetCursorPos + SendInput (2 system calls)
//! 2. `click` - Pure SendInput at current position (1 system call)
//!
//! Expected result: Without coordinates should be ~2x faster due to eliminating SetCursorPos overhead.

use std::time::Instant;
use win_auto_utils::script_engine::ScriptEngine;

fn create_engine() -> ScriptEngine {
    ScriptEngine::with_builtin()
}

fn calculate_ips(count: u64, duration: std::time::Duration) -> f64 {
    if duration.as_nanos() == 0 {
        f64::INFINITY
    } else {
        count as f64 / duration.as_secs_f64()
    }
}

#[test]
fn test_click_with_vs_without_coordinates() {
    println!("\n=== Click Performance: With Coordinates vs Without ===\n");

    let engine = create_engine();
    let iterations = 100;

    // Test 1: Click WITH coordinates (SetCursorPos + SendInput)
    println!("Test 1: click 100 200 (with coordinates)");
    println!("  Expected: SetCursorPos (~5μs) + SendInput (~50μs) = ~55μs per call");

    let script_with_coords = format!(
        r#"loop {}
    click 100 200
end"#,
        iterations
    );

    let compiled = engine.compile(&script_with_coords).unwrap();

    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_with_coords = start.elapsed();

    let ips_with_coords = calculate_ips(iterations, elapsed_with_coords);
    let per_call_with_coords = elapsed_with_coords.as_nanos() as f64 / iterations as f64;

    println!("  Total time: {:?}", elapsed_with_coords);
    println!("  Throughput: {:.0} IPS", ips_with_coords);
    println!(
        "  Per call: {:.2} ns ({:.3} μs)\n",
        per_call_with_coords,
        per_call_with_coords / 1000.0
    );

    // Test 2: Click WITHOUT coordinates (Pure SendInput)
    println!("Test 2: click (without coordinates)");
    println!("  Expected: Pure SendInput (~50μs) = ~50μs per call");

    let script_without_coords = format!(
        r#"loop {}
    click
end"#,
        iterations
    );

    let compiled = engine.compile(&script_without_coords).unwrap();

    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_without_coords = start.elapsed();

    let ips_without_coords = calculate_ips(iterations, elapsed_without_coords);
    let per_call_without_coords = elapsed_without_coords.as_nanos() as f64 / iterations as f64;

    println!("  Total time: {:?}", elapsed_without_coords);
    println!("  Throughput: {:.0} IPS", ips_without_coords);
    println!(
        "  Per call: {:.2} ns ({:.3} μs)\n",
        per_call_without_coords,
        per_call_without_coords / 1000.0
    );

    // Analysis
    println!("=== Performance Analysis ===");

    let speedup = per_call_with_coords / per_call_without_coords;
    let time_saved = per_call_with_coords - per_call_without_coords;
    let setcursorpos_overhead = time_saved;

    println!("Speedup factor: {:.2}x", speedup);
    println!(
        "Time saved per call: {:.2} ns ({:.3} μs)",
        time_saved,
        time_saved / 1000.0
    );
    println!(
        "Estimated SetCursorPos overhead: {:.2} ns ({:.3} μs)",
        setcursorpos_overhead,
        setcursorpos_overhead / 1000.0
    );

    // Verify expectations
    if speedup > 1.1 {
        println!("\n✅ CONFIRMED: Click without coordinates is significantly faster!");
        println!("   This proves that SetCursorPos adds measurable overhead.");
    } else {
        println!("\n⚠️  WARNING: Performance difference is minimal (<10%)");
        println!("   Possible reasons:");
        println!("   - SetCursorPos is faster than expected");
        println!("   - OS caching reduces SetCursorPos overhead");
        println!("   - Measurement noise");
    }

    // Additional verification: SetCursorPos should add at least some overhead
    assert!(
        per_call_with_coords >= per_call_without_coords * 0.9,
        "Click with coords should not be faster than without coords"
    );

    println!("\n✅ Test completed successfully\n");
}

#[test]
fn test_move_performance_baseline() {
    println!("\n=== Move Performance Baseline ===\n");

    let engine = create_engine();
    let iterations = 100;

    // Test move instruction (always requires coordinates)
    println!("Test: move 100 200");
    println!("  Includes: Window coordinate transformation + SetCursorPos");

    let script = format!(
        r#"loop {}
    move 100 200
end"#,
        iterations
    );

    let compiled = engine.compile(&script).unwrap();

    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed = start.elapsed();

    let ips = calculate_ips(iterations, elapsed);
    let per_call = elapsed.as_nanos() as f64 / iterations as f64;

    println!("  Total time: {:?}", elapsed);
    println!("  Throughput: {:.0} IPS", ips);
    println!(
        "  Per call: {:.2} ns ({:.3} μs)",
        per_call,
        per_call / 1000.0
    );

    // Compare with click without coords (pure SendInput baseline)
    let click_script = format!(
        r#"loop {}
    click
end"#,
        iterations
    );

    let compiled_click = engine.compile(&click_script).unwrap();

    let start = Instant::now();
    engine.execute(&compiled_click).unwrap();
    let elapsed_click = start.elapsed();

    let per_call_click = elapsed_click.as_nanos() as f64 / iterations as f64;

    println!("\nComparison with 'click' (no coords):");
    println!(
        "  Move overhead: {:.2} ns ({:.3} μs)",
        per_call - per_call_click,
        (per_call - per_call_click) / 1000.0
    );

    println!("\n✅ Test completed\n");
}
