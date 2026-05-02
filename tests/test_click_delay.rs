use win_auto_utils::script_engine::ScriptEngine;

#[test]
fn test_click_with_delay() {
    let engine = ScriptEngine::with_builtin();

    // Test click with delay parameter
    let script = r#"
click 100 200 50
click 200 300 100
"#;

    println!("\n=== Test: Click with Delay ===");

    let compile_start = std::time::Instant::now();
    let compiled = engine.compile(script).unwrap();
    let compile_time = compile_start.elapsed();
    println!("Compilation time: {:?}", compile_time);

    let exec_start = std::time::Instant::now();
    engine.execute(&compiled).unwrap();
    let exec_time = exec_start.elapsed();

    println!("Execution time: {:?}", exec_time);
    println!("Expected minimum time: ~150ms (50ms + 100ms delays)");

    // Should take at least 150ms due to delays
    assert!(
        exec_time.as_millis() >= 140,
        "Click with delays should take at least 140ms, but took {:?}",
        exec_time
    );

    println!("✅ Click delay test passed\n");
}
