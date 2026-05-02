use std::time::Instant;
use win_auto_utils::script_engine::ScriptEngine;

#[test]
fn test_mixed_instruction_breakdown() {
    let engine = ScriptEngine::with_builtin();

    println!("\n=== Mixed Instruction Time Breakdown ===\n");

    // Test 1: Only sleep (baseline)
    println!("Test 1: Sleep only (50 iterations × 1ms)");
    let script_sleep = r#"loop 50
    sleep 1
end"#;

    let compiled = engine.compile(script_sleep).unwrap();
    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_sleep = start.elapsed();
    println!("  Total time: {:?} (~{}ms expected)", elapsed_sleep, 50);
    println!("  Overhead: {:?}\n", elapsed_sleep.as_millis() as i64 - 50);

    // Test 2: Only key (no sleep)
    println!("Test 2: Key only (50 iterations)");
    let script_key = r#"loop 50
    key A
end"#;

    let compiled = engine.compile(script_key).unwrap();
    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_key = start.elapsed();
    println!("  Total time: {:?}", elapsed_key);
    println!("  Per instruction: {} μs\n", elapsed_key.as_micros() / 50);

    // Test 3: Only move (no sleep)
    println!("Test 3: Move only (50 iterations)");
    let script_move = r#"loop 50
    move 100 200
end"#;

    let compiled = engine.compile(script_move).unwrap();
    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_move = start.elapsed();
    println!("  Total time: {:?}", elapsed_move);
    println!("  Per instruction: {} μs\n", elapsed_move.as_micros() / 50);

    // Test 4: Only click (no sleep)
    println!("Test 4: Click only (50 iterations)");
    let script_click = r#"loop 50
    click 300 400
end"#;

    let compiled = engine.compile(script_click).unwrap();
    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_click = start.elapsed();
    println!("  Total time: {:?}", elapsed_click);
    println!("  Per instruction: {} μs\n", elapsed_click.as_micros() / 50);

    // Test 5: Mixed (all together)
    println!("Test 5: Mixed (key + move + click + sleep)");
    let script_mixed = r#"loop 50
    key A
    move 100 200
    click 300 400
    sleep 1
end"#;

    let compiled = engine.compile(script_mixed).unwrap();
    let start = Instant::now();
    engine.execute(&compiled).unwrap();
    let elapsed_mixed = start.elapsed();
    println!("  Total time: {:?}", elapsed_mixed);
    println!(
        "  Expected (sum): {:?} (key + move + click + sleep)",
        elapsed_key + elapsed_move + elapsed_click + elapsed_sleep
    );

    // Analysis
    println!("\n=== Analysis ===");
    let estimated_os_calls =
        elapsed_key.as_micros() + elapsed_move.as_micros() + elapsed_click.as_micros();
    let actual_total = elapsed_mixed.as_micros();
    let sleep_component = elapsed_sleep.as_micros();

    println!("OS calls overhead: {} μs", estimated_os_calls);
    println!("Sleep component: {} μs", sleep_component);
    println!("Actual total: {} μs", actual_total);
    println!(
        "Difference: {} μs (likely scheduling/context switching overhead)\n",
        actual_total as i64 - (estimated_os_calls as i64 + sleep_component as i64)
    );
}
