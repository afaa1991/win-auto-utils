//! Performance analysis for realistic script usage patterns
//!
//! This test analyzes performance for typical use cases:
//! - Scripts with ~100 lines
//! - Sub-script support (multiple small scripts)
//! - Compilation vs execution overhead
//! - Memory and CPU impact

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

/// Simulate a realistic 100-line script with mixed operations
fn generate_realistic_script() -> String {
    r#"
// Configuration
mode keyboard send
mode mouse send

// Main automation loop
loop 5
    // Navigate to target
    move 100 200
    click
    
    sleep 100
    
    // Keyboard shortcuts (simplified - just individual keys)
    key A
    key B
    key C
    
    sleep 50
    
    // Move to next item
    moverel 0 50
    click
    
    sleep 100
end

// Final cleanup
move 500 500
click
"#
    .to_string()
}

/// Simulate sub-script pattern: multiple small scripts
fn generate_sub_scripts() -> Vec<String> {
    vec![
        // Sub-script 1: Navigation
        r#"
move 100 200
click
sleep 100
"#
        .to_string(),
        // Sub-script 2: Input (simplified)
        r#"
key A
key B
key C
sleep 50
"#
        .to_string(),
        // Sub-script 3: Scrolling
        r#"
scrollup 3
sleep 100
"#
        .to_string(),
        // Sub-script 4: Selection
        r#"
move 300 400
click
sleep 150
key A
key B
key C
sleep 50
"#
        .to_string(),
        // Sub-script 5: Next item
        r#"
moverel 0 50
click
sleep 100
"#
        .to_string(),
    ]
}

#[test]
fn test_realistic_100_line_script_performance() {
    println!("\n=== Realistic 100-Line Script Performance ===\n");

    let engine = create_engine();
    let script_text = generate_realistic_script();

    println!("Script length: {} characters", script_text.len());
    println!("Estimated lines: ~{}", script_text.lines().count());

    // Measure compilation time
    let compile_start = Instant::now();
    let compiled = engine.compile(&script_text).unwrap();
    let compile_time = compile_start.elapsed();

    println!("\nCompilation:");
    println!("  Time: {:?}", compile_time);
    println!("  Instructions compiled: {}", compiled.instructions.len());

    // Measure first execution
    let exec1_start = Instant::now();
    engine.execute(&compiled).unwrap();
    let exec1_time = exec1_start.elapsed();

    println!("\nFirst Execution:");
    println!("  Time: {:?}", exec1_time);
    println!(
        "  Instructions executed: {}",
        compiled.instructions.len() * 5
    ); // loop 5 times

    // Measure cached execution (average of 9 runs)
    let runs = 9;
    let exec_cached_start = Instant::now();
    for _ in 0..runs {
        engine.execute(&compiled).unwrap();
    }
    let exec_cached_total = exec_cached_start.elapsed();
    let exec_cached_avg = exec_cached_total / runs;

    println!("\nCached Execution (avg of {} runs):", runs);
    println!("  Time: {:?}", exec_cached_avg);
    println!(
        "  Throughput: {:.0} IPS",
        calculate_ips((compiled.instructions.len() * 5) as u64, exec_cached_avg)
    );

    // Analysis
    println!("\n=== Performance Analysis ===");

    let compilation_overhead_percent = if exec_cached_avg.as_nanos() > 0 {
        (compile_time.as_nanos() as f64 / exec_cached_avg.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Compilation overhead: {:.1}% of execution time",
        compilation_overhead_percent
    );
    println!(
        "Memory footprint: ~{} KB (estimated)",
        compiled.instructions.len()
            * std::mem::size_of::<win_auto_utils::script_engine::instruction::InstructionData>()
            / 1024
    );

    if compilation_overhead_percent < 10.0 {
        println!("\n✅ Compilation overhead is negligible (<10%)");
        println!("   Recommendation: Compile once, execute multiple times");
    } else if compilation_overhead_percent < 50.0 {
        println!("\n⚠️  Compilation overhead is moderate (10-50%)");
        println!("   Recommendation: Cache compiled scripts for repeated use");
    } else {
        println!("\n❌ Compilation overhead is significant (>50%)");
        println!("   Recommendation: MUST cache compiled scripts");
    }

    println!("\n✅ Test completed\n");
}

#[test]
fn test_sub_script_pattern_performance() {
    println!("\n=== Sub-Script Pattern Performance ===\n");

    let engine = create_engine();
    let sub_scripts = generate_sub_scripts();

    println!("Number of sub-scripts: {}", sub_scripts.len());
    println!(
        "Average length: {} characters",
        sub_scripts.iter().map(|s| s.len()).sum::<usize>() / sub_scripts.len()
    );

    // Scenario 1: Compile all sub-scripts once, then execute sequentially
    println!("\nScenario 1: Compile Once, Execute Sequentially");

    let compile_all_start = Instant::now();
    let compiled_scripts: Vec<_> = sub_scripts
        .iter()
        .map(|s| engine.compile(s).unwrap())
        .collect();
    let compile_all_time = compile_all_start.elapsed();

    println!("  Total compilation time: {:?}", compile_all_time);
    println!(
        "  Average per script: {:?}",
        compile_all_time / sub_scripts.len() as u32
    );

    let exec_seq_start = Instant::now();
    for compiled in &compiled_scripts {
        engine.execute(compiled).unwrap();
    }
    let exec_seq_time = exec_seq_start.elapsed();

    println!("  Sequential execution time: {:?}", exec_seq_time);

    // Scenario 2: Compile and execute each sub-script on-demand (no caching)
    println!("\nScenario 2: Compile + Execute On-Demand (No Caching)");

    let total_start = Instant::now();
    for script in &sub_scripts {
        let compiled = engine.compile(script).unwrap();
        engine.execute(&compiled).unwrap();
    }
    let total_time = total_start.elapsed();

    println!("  Total time: {:?}", total_time);

    // Comparison
    println!("\n=== Performance Comparison ===");

    let scenario1_total = compile_all_time + exec_seq_time;
    let overhead = total_time.as_nanos() as f64 - scenario1_total.as_nanos() as f64;
    let overhead_percent = if scenario1_total.as_nanos() > 0 {
        (overhead / scenario1_total.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    println!("Scenario 1 (cached): {:?}", scenario1_total);
    println!("Scenario 2 (on-demand): {:?}", total_time);
    println!("Overhead of on-demand: {:.1}%", overhead_percent);

    if overhead_percent > 20.0 {
        println!("\n✅ Caching provides significant benefit!");
        println!("   Recommendation: Pre-compile all sub-scripts at startup");
    } else {
        println!("\n⚠️  Caching benefit is minimal");
        println!("   On-demand compilation may be acceptable for infrequent use");
    }

    println!("\n✅ Test completed\n");
}

#[test]
fn test_compilation_scaling_with_script_size() {
    println!("\n=== Compilation Scaling with Script Size ===\n");

    let engine = create_engine();

    // Test different script sizes
    let test_cases = vec![
        ("Tiny (10 lines)", 10),
        ("Small (50 lines)", 50),
        ("Medium (100 lines)", 100),
        ("Large (200 lines)", 200),
        ("Very Large (500 lines)", 500),
    ];

    println!(
        "{:<25} {:<15} {:<15} {:<15}",
        "Script Size", "Compile Time", "Instructions", "Time/Instr"
    );
    println!("{:-<70}", "");

    for (name, line_count) in test_cases {
        // Generate script with specified number of instructions
        let mut script = String::new();
        for _i in 0..line_count {
            script.push_str(&format!("key A\n"));
        }

        let compile_start = Instant::now();
        let compiled = engine.compile(&script).unwrap();
        let compile_time = compile_start.elapsed();

        let time_per_instr = if !compiled.instructions.is_empty() {
            compile_time.as_nanos() as f64 / compiled.instructions.len() as f64
        } else {
            0.0
        };

        println!(
            "{:<25} {:<15?} {:<15} {:<15.2} ns",
            name,
            compile_time,
            compiled.instructions.len(),
            time_per_instr
        );
    }

    println!("\n✅ Scaling test completed\n");
}

#[test]
fn test_memory_footprint_analysis() {
    println!("\n=== Memory Footprint Analysis ===\n");

    let engine = create_engine();

    // Test different script sizes
    let script_sizes = vec![10, 50, 100, 200, 500];

    println!(
        "{:<15} {:<20} {:<20} {:<15}",
        "Lines", "Instructions", "Memory (KB)", "Per Instr (B)"
    );
    println!("{:-<70}", "");

    for size in script_sizes {
        let mut script = String::new();
        for _ in 0..size {
            script.push_str("key A\n");
        }

        let compiled = engine.compile(&script).unwrap();

        let memory_bytes = compiled.instructions.len()
            * std::mem::size_of::<win_auto_utils::script_engine::instruction::InstructionData>();
        let memory_kb = memory_bytes as f64 / 1024.0;
        let per_instr_bytes = if !compiled.instructions.is_empty() {
            memory_bytes / compiled.instructions.len()
        } else {
            0
        };

        println!(
            "{:<15} {:<20} {:<20.2} {:<15}",
            size,
            compiled.instructions.len(),
            memory_kb,
            per_instr_bytes
        );
    }

    println!("\nNote: Memory footprint is linear with instruction count");
    println!("For 100-line script: typically <10 KB");
    println!("This is negligible for modern systems\n");

    println!("\n✅ Memory analysis completed\n");
}

#[test]
fn test_execution_vs_compilation_ratio() {
    println!("\n=== Execution vs Compilation Time Ratio ===\n");

    let engine = create_engine();

    // Create a realistic 100-line script
    let script_text = generate_realistic_script();
    let compiled = engine.compile(&script_text).unwrap();

    println!(
        "Script: ~100 lines, {} instructions",
        compiled.instructions.len()
    );

    // Measure compilation
    let compile_start = Instant::now();
    let _compiled = engine.compile(&script_text).unwrap();
    let compile_time = compile_start.elapsed();

    // Measure execution (single run)
    let exec_start = Instant::now();
    engine.execute(&compiled).unwrap();
    let exec_time = exec_start.elapsed();

    // Calculate ratio
    let ratio = if exec_time.as_nanos() > 0 {
        compile_time.as_nanos() as f64 / exec_time.as_nanos() as f64
    } else {
        f64::INFINITY
    };

    println!("\nResults:");
    println!("  Compilation time: {:?}", compile_time);
    println!("  Execution time:   {:?}", exec_time);
    println!("  Ratio (Compile/Exec): {:.2}x", ratio);

    if ratio < 0.1 {
        println!("\n✅ Compilation is <10% of execution time");
        println!("   For single execution: compile+execute together is fine");
        println!("   For repeated execution: definitely cache compilation");
    } else if ratio < 1.0 {
        println!("\n⚠️  Compilation is significant but less than execution");
        println!("   Caching recommended for 2+ executions");
    } else {
        println!("\n❌ Compilation takes longer than execution!");
        println!("   MUST cache compiled scripts");
    }

    println!("\n✅ Ratio analysis completed\n");
}
