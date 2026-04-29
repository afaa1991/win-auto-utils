//! AOB Scan Extreme Performance Benchmark for 64-bit Process with 32-bit Comparison
//!
//! This benchmark tests the absolute limits of AOB scanning on a 64-bit process
//! and compares it with 32-bit performance.
//!
//! ## Target Processes
//! - **64-bit**: PUBPETS.exe (Address Range: 0x7FF000000000 - 0x7FFFFFFFFFFF, ~16TB)
//! - **32-bit**: lf2.exe (Address Range: 0x0 - 0xFFFFFFFF, 4GB max)
//!
//! ## Test Patterns
//! The test uses complex multi-instruction bytecode patterns from user request:
//! - `48 81 C2 00010000` - ADD RDX, 0x100
//! - `49 81 E8 00010000` - SUB R8, 0x100
//! - `49 81 F8 00010000` - CMP R8, 0x100
//! - And more complex combined patterns...
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_extreme_64bit --features "memory_aobscan" --release
//! ```
//!
//! # Output
//! - Console output with real-time progress
//! - Detailed report saved to: `aobscan_performance_report.txt`
//! - Comparison between 64-bit and 32-bit performance
//!
//! # Requirements
//! 1. PUBPETS.exe must be running (64-bit target)
//! 2. lf2.exe should be running for 32-bit comparison (optional)
//! 3. Sufficient permissions to read process memory
//!
//! # Test Scenarios
//! 1. **Individual Patterns**: Tests each instruction separately
//! 2. **Combined Pattern**: Tests longer, more specific pattern
//! 3. **Early Exit**: Measures find_first performance across full range
//! 4. **Find All (EXTREME)**: Full address space scan stress test
//! 5. **Range Comparison**: Performance at different scale levels
//!
//! # Key Insights
//! - 64-bit processes have vastly larger virtual address spaces
//! - Actual performance depends on COMMITTED memory, not theoretical range
//! - Scanner automatically skips uncommitted regions efficiently
//! - Pattern complexity and anchor selection significantly impact speed
//!
//! Target: PUBPETS.exe (64-bit) vs typical 32-bit process
//! Address Range: 0x7FF000000000 - 0x7FFFFFFFFFFF (~16TB for 64-bit)
//! Pattern: Complex multi-instruction bytecode from user request
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_extreme_64bit --features "memory_aobscan" --release
//! ```

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;
use std::time::Instant;
use std::fs::File;
use std::io::Write;

struct BenchmarkResult {
    test_name: String,
    #[allow(dead_code)]
    pattern: String,
    range_size: String,
    elapsed: std::time::Duration,
    matches_found: usize,
    throughput: f64, // scans/sec
}

impl BenchmarkResult {
    fn new(test_name: &str, pattern: &str, range_size: &str, 
           elapsed: std::time::Duration, matches: usize, throughput: f64) -> Self {
        Self {
            test_name: test_name.to_string(),
            pattern: pattern.to_string(),
            range_size: range_size.to_string(),
            elapsed,
            matches_found: matches,
            throughput,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AOB Scan EXTREME 64-bit Performance Benchmark ===\n");
    println!("Target: PUBPETS.exe (64-bit)");
    println!("Address Range: 0x7FF000000000 - 0x7FFFFFFFFFFF (~16TB)");
    println!("Comparison: Will compare with 32-bit process performance\n");

    let mut results_64bit = Vec::new();

    // Initialize 64-bit process
    let process_64 = match initialize_process("PUBPETS.exe")? {
        Some(p) => p,
        None => {
            eprintln!("❌ ERROR: PUBPETS.exe not found or cannot be accessed!");
            eprintln!("Please ensure:");
            eprintln!("  1. PUBPETS.exe is running");
            eprintln!("  2. You have sufficient permissions");
            eprintln!("  3. The process is 64-bit\n");
            return Ok(());
        }
    };

    println!("✓ Connected to PUBPETS.exe (PID: {})\n", process_64.get_pid());

    // Test 1: Individual instruction patterns
    let test1_results = benchmark_individual_patterns(&process_64)?;
    results_64bit.extend(test1_results);

    // Test 2: Combined complex pattern
    let test2_results = benchmark_combined_pattern(&process_64)?;
    results_64bit.extend(test2_results);

    // Test 3: Full range scan with early exit
    let test3_results = benchmark_full_range_early_exit(&process_64)?;
    results_64bit.extend(test3_results);

    // Test 4: Find all matches in full range (EXTREME TEST!)
    let test4_results = benchmark_full_range_find_all(&process_64)?;
    results_64bit.extend(test4_results);

    // Test 5: Performance comparison with different ranges
    let test5_results = benchmark_range_comparison(&process_64)?;
    results_64bit.extend(test5_results);

    // Now test 32-bit process for comparison
    println!("\n\n=== Starting 32-bit Comparison Test ===\n");
    let mut results_32bit = Vec::new();

    if let Some(process_32) = initialize_process("lf2.exe")? {
        println!("✓ Connected to lf2.exe (PID: {}) for 32-bit comparison\n", process_32.get_pid());
        
        let test1_32 = benchmark_individual_patterns_32bit(&process_32)?;
        results_32bit.extend(test1_32);
        
        let test2_32 = benchmark_combined_pattern_32bit(&process_32)?;
        results_32bit.extend(test2_32);
    } else {
        println!("⚠ lf2.exe not found, skipping 32-bit comparison\n");
    }

    // Generate comparison report
    generate_comparison_report(&results_64bit, &results_32bit)?;

    println!("\n=== Extreme Benchmark Complete ===");
    println!("All tests completed successfully!");
    
    Ok(())
}

/// Helper: Initialize process connection
fn initialize_process(name: &str) -> Result<Option<Process>, Box<dyn std::error::Error>> {
    let process = Process::builder(name).build();
    
    match process.init() {
        Ok(()) => Ok(Some(process)),
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", name, e);
            Ok(None)
        }
    }
}

/// Test 1: Benchmark individual instruction patterns (64-bit)
fn benchmark_individual_patterns(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 1] Individual Instruction Patterns (64-bit)");
    println!("--------------------------------------------------");

    let handle = process.get_handle();
    
    let patterns = vec![
        ("Pattern 1: 48 81 C2 00010000", "48 81 C2 00 01 00 00"),
        ("Pattern 2: 49 81 E8 00010000", "49 81 E8 00 01 00 00"),
        ("Pattern 3: 49 81 F8 00010000", "49 81 F8 00 01 00 00"),
        ("Pattern 4: 0F83 78FFFFFF", "0F 83 78 FF FF FF"),
        ("Pattern 5: 4D 8D 48 1F", "4D 8D 48 1F"),
        ("Pattern 6: 49 83 E1 E0", "49 83 E1 E0"),
        ("Pattern 7: 4D 8B D9", "4D 8B D9"),
        ("Pattern 8: 49 C1 EB 05", "49 C1 EB 05"),
    ];

    let iterations = 5;
    let start_addr = 0x7FF000000000usize;
    let length = 0x100000000usize; // 4GB chunks
    
    let mut results = Vec::new();

    for (name, pattern_str) in &patterns {
        println!("\n  Testing: {}", name);
        println!("  Pattern bytes: {}", pattern_str.split_whitespace().count());
        
        // Warm-up
        let _ = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(false)
            .scan()?;

        // Benchmark
        let start = Instant::now();
        let mut total_matches = 0;
        
        for _iter in 0..iterations {
            let scan_results = AobScanBuilder::new(handle)
                .pattern_str(pattern_str)?
                .start_address(start_addr)
                .length(length)
                .find_all(true)
                .scan()?;
            
            total_matches += scan_results.len();
            if !scan_results.is_empty() && _iter == 0 {
                println!("    First match at: 0x{:X}", scan_results[0]);
            }
        }
        
        let elapsed = start.elapsed();
        let per_scan = elapsed / iterations as u32;
        let throughput = iterations as f64 / elapsed.as_secs_f64();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Per scan:   {:?}", per_scan);
        println!("    Throughput: {:.2} scans/sec", throughput);
        println!("    Avg matches per scan: {:.1}", total_matches as f64 / iterations as f64);
        
        results.push(BenchmarkResult::new(
            name,
            pattern_str,
            "4GB (64-bit range)",
            elapsed,
            total_matches / iterations,
            throughput,
        ));
    }

    println!("\n");
    Ok(results)
}

/// Test 1: Benchmark individual instruction patterns (32-bit)
fn benchmark_individual_patterns_32bit(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 1] Individual Instruction Patterns (32-bit)");
    println!("--------------------------------------------------");

    let handle = process.get_handle();
    
    let patterns = vec![
        ("Pattern 1: 48 81 C2 00010000", "48 81 C2 00 01 00 00"),
        ("Pattern 2: 49 81 E8 00010000", "49 81 E8 00 01 00 00"),
        ("Pattern 3: 49 81 F8 00010000", "49 81 F8 00 01 00 00"),
    ];

    let iterations = 5;
    let start_addr = 0x0usize;
    let length = 0x10000000usize; // 256MB for 32-bit
    
    let mut results = Vec::new();

    for (name, pattern_str) in &patterns {
        println!("\n  Testing: {}", name);
        println!("  Pattern bytes: {}", pattern_str.split_whitespace().count());
        
        // Warm-up
        let _ = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(false)
            .scan()?;

        // Benchmark
        let start = Instant::now();
        let mut total_matches = 0;
        
        for _ in 0..iterations {
            let scan_results = AobScanBuilder::new(handle)
                .pattern_str(pattern_str)?
                .start_address(start_addr)
                .length(length)
                .find_all(true)
                .scan()?;
            
            total_matches += scan_results.len();
        }
        
        let elapsed = start.elapsed();
        let throughput = iterations as f64 / elapsed.as_secs_f64();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Throughput: {:.2} scans/sec", throughput);
        
        results.push(BenchmarkResult::new(
            &format!("{} (32-bit)", name),
            pattern_str,
            "256MB (32-bit range)",
            elapsed,
            total_matches / iterations,
            throughput,
        ));
    }

    println!("\n");
    Ok(results)
}

/// Test 2: Benchmark combined complex pattern (64-bit)
fn benchmark_combined_pattern(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 2] Combined Complex Pattern (64-bit)");
    println!("-------------------------------------------");

    let handle = process.get_handle();
    
    let pattern_str = "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00 49 81 F8 00 01 00 00";
    
    println!("  Pattern: {}", pattern_str);
    println!("  Pattern bytes: {}", pattern_str.split_whitespace().count());
    println!("  Expected: Better specificity, fewer false positives\n");

    let iterations = 5;
    let start_addr = 0x7FF000000000usize;
    let length = 0x100000000usize; // 4GB

    // Warm-up
    let _ = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(false)
        .scan()?;

    // Benchmark
    let start = Instant::now();
    let mut total_matches = 0;
    
    for iter in 0..iterations {
        let scan_results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(true)
            .scan()?;
        
        total_matches += scan_results.len();
        if !scan_results.is_empty() && iter == 0 {
            println!("  First match at: 0x{:X}", scan_results[0]);
        }
    }
    
    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();
    
    println!("  Total time: {:?}", elapsed);
    println!("  Throughput: {:.2} scans/sec", throughput);
    println!("  Avg matches per scan: {:.1}", total_matches as f64 / iterations as f64);
    println!();

    Ok(vec![BenchmarkResult::new(
        "Combined Pattern (64-bit)",
        pattern_str,
        "4GB (64-bit range)",
        elapsed,
        total_matches / iterations,
        throughput,
    )])
}

/// Test 2: Benchmark combined complex pattern (32-bit)
fn benchmark_combined_pattern_32bit(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 2] Combined Complex Pattern (32-bit)");
    println!("-------------------------------------------");

    let handle = process.get_handle();
    
    let pattern_str = "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00 49 81 F8 00 01 00 00";
    
    println!("  Pattern: {}", pattern_str);
    println!("  Pattern bytes: {}\n", pattern_str.split_whitespace().count());

    let iterations = 5;
    let start_addr = 0x0usize;
    let length = 0x10000000usize; // 256MB

    // Warm-up
    let _ = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(false)
        .scan()?;

    // Benchmark
    let start = Instant::now();
    let mut total_matches = 0;
    
    for _ in 0..iterations {
        let scan_results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(true)
            .scan()?;
        
        total_matches += scan_results.len();
    }
    
    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();
    
    println!("  Total time: {:?}", elapsed);
    println!("  Throughput: {:.2} scans/sec", throughput);
    println!();

    Ok(vec![BenchmarkResult::new(
        "Combined Pattern (32-bit)",
        pattern_str,
        "256MB (32-bit range)",
        elapsed,
        total_matches / iterations,
        throughput,
    )])
}

/// Test 3: Full range scan with early exit (find_first)
fn benchmark_full_range_early_exit(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 3] Full Range Scan - Early Exit (64-bit)");
    println!("-----------------------------------------------");

    let handle = process.get_handle();
    
    let pattern_str = "48 81 C2 00 01 00 00";
    let start_addr = 0x7FF000000000usize;
    let length = 0xFFFFFFFFFFFFusize; // Nearly full 64-bit range
    
    println!("  Pattern: {}", pattern_str);
    println!("  Start: 0x{:X}", start_addr);
    println!("  Length: 0x{:X} (~{} TB)", length, length / 1024 / 1024 / 1024 / 1024);
    println!("  Mode: find_all=false (stops at first match)\n");

    let iterations = 3;
    let mut total_time = std::time::Duration::new(0, 0);
    let mut found_count = 0;

    for i in 0..iterations {
        println!("  Iteration {}/{}...", i + 1, iterations);
        let start = Instant::now();
        
        let scan_results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(false)
            .scan()?;
        
        let elapsed = start.elapsed();
        total_time += elapsed;
        
        if !scan_results.is_empty() {
            found_count += 1;
            println!("    ✓ Found match at: 0x{:X}", scan_results[0]);
        } else {
            println!("    ✗ No match found");
        }
        println!("    Time: {:?}", elapsed);
    }

    let avg_time = total_time / iterations as u32;
    let throughput = iterations as f64 / total_time.as_secs_f64();
    
    println!("\n  Average time: {:?}", avg_time);
    println!("  Success rate: {}/{}", found_count, iterations);
    println!("  Throughput: {:.2} scans/sec", throughput);
    println!();

    Ok(vec![BenchmarkResult::new(
        "Full Range Early Exit (64-bit)",
        pattern_str,
        "~16TB (full 64-bit range)",
        avg_time,
        found_count,
        throughput,
    )])
}

/// Test 4: Find ALL matches in full range (EXTREME STRESS TEST!)
fn benchmark_full_range_find_all(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 4] Full Range Scan - Find ALL (EXTREME STRESS TEST!)");
    println!("----------------------------------------------------------");
    println!("  ⚠️  WARNING: Scanning ENTIRE 64-bit address space!");
    println!("  ⚠️  This may take several minutes!\n");

    let handle = process.get_handle();
    
    let pattern_str = "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00";
    let start_addr = 0x7FF000000000usize;
    let length = 0xFFFFFFFFFFFFusize;
    
    println!("  Pattern: {}", pattern_str);
    println!("  Pattern bytes: {}", pattern_str.split_whitespace().count());
    println!("  Range: 0x{:X} - 0x{:X}", start_addr, start_addr + length);
    println!("  Mode: find_all=true (scans entire range)");
    println!();
    println!("  Starting extreme scan... (this may take a while)");
    
    let start = Instant::now();
    
    let scan_results = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(true)
        .scan()?;
    
    let elapsed = start.elapsed();
    
    println!("\n  ✓ Extreme scan completed!");
    println!("  Total matches found: {}", scan_results.len());
    println!("  Total time: {:?}", elapsed);
    
    if !scan_results.is_empty() {
        println!("  First 5 matches:");
        for (i, addr) in scan_results.iter().take(5).enumerate() {
            println!("    [{}] 0x{:X}", i + 1, addr);
        }
        if scan_results.len() > 5 {
            println!("    ... and {} more", scan_results.len() - 5);
        }
    }
    
    let throughput = 1.0 / elapsed.as_secs_f64();
    
    println!();
    println!("  Note: Actual scanned memory depends on committed regions");
    println!("  Most of 64-bit address space is uncommitted and skipped automatically");
    println!();

    Ok(vec![BenchmarkResult::new(
        "Full Range Find All (64-bit EXTREME)",
        pattern_str,
        "~16TB (full 64-bit range)",
        elapsed,
        scan_results.len(),
        throughput,
    )])
}

/// Test 5: Performance comparison with different range sizes
fn benchmark_range_comparison(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 5] Range Size Performance Comparison (64-bit)");
    println!("----------------------------------------------------");

    let handle = process.get_handle();
    let pattern_str = "48 81 C2 00 01 00 00";
    let start_addr = 0x7FF000000000usize;
    
    let ranges = vec![
        ("1 GB", 0x40000000usize),
        ("10 GB", 0x280000000usize),
        ("100 GB", 0x1900000000usize),
        ("1 TB", 0x10000000000usize),
        ("10 TB", 0xA0000000000usize),
    ];

    let iterations = 3;
    let mut results = Vec::new();

    println!("  Pattern: {}", pattern_str);
    println!("  Start Address: 0x{:X}", start_addr);
    println!("  Iterations per range: {}\n", iterations);

    for (range_name, length) in &ranges {
        println!("  Testing range: {} (0x{:X})", range_name, length);
        
        let start = Instant::now();
        let mut total_matches = 0;
        
        for _ in 0..iterations {
            let scan_results = AobScanBuilder::new(handle)
                .pattern_str(pattern_str)?
                .start_address(start_addr)
                .length(*length)
                .find_all(true)
                .scan()?;
            
            total_matches += scan_results.len();
        }
        
        let elapsed = start.elapsed();
        let throughput = iterations as f64 / elapsed.as_secs_f64();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Throughput: {:.2} scans/sec", throughput);
        println!("    Avg matches: {:.1}\n", total_matches as f64 / iterations as f64);
        
        results.push(BenchmarkResult::new(
            &format!("Range {} (64-bit)", range_name),
            pattern_str,
            range_name,
            elapsed,
            total_matches / iterations,
            throughput,
        ));
    }

    Ok(results)
}

/// Generate comprehensive comparison report
fn generate_comparison_report(results_64: &[BenchmarkResult], results_32: &[BenchmarkResult]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       AOB SCAN PERFORMANCE COMPARISON REPORT               ║");
    println!("║       64-bit (PUBPETS.exe) vs 32-bit (lf2.exe)             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Save to file
    let mut report_file = File::create("aobscan_performance_report.txt")?;

    writeln!(report_file, "AOB Scan Performance Comparison Report")?;
    writeln!(report_file, "Generated: {}\n", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())?;
    writeln!(report_file, "{:<50} | {:<15} | {:<15} | {:<15} | {:<10}", 
             "Test Name", "Range", "Time", "Throughput", "Matches")?;
    writeln!(report_file, "{}", "-".repeat(110))?;

    println!("{:<50} | {:<15} | {:<15} | {:<15} | {:<10}", 
             "Test Name", "Range", "Time", "Throughput", "Matches");
    println!("{}", "-".repeat(110));

    // Print 64-bit results
    println!("\n📊 64-BIT RESULTS (PUBPETS.exe):");
    writeln!(report_file, "\n64-BIT RESULTS (PUBPETS.exe):")?;
    for result in results_64 {
        let line = format!("{:<50} | {:<15} | {:<15?} | {:<15.2} | {:<10}", 
                          result.test_name, result.range_size, result.elapsed, 
                          result.throughput, result.matches_found);
        println!("{}", line);
        writeln!(report_file, "{}", line)?;
    }

    // Print 32-bit results
    if !results_32.is_empty() {
        println!("\n📊 32-BIT RESULTS (lf2.exe):");
        writeln!(report_file, "\n32-BIT RESULTS (lf2.exe):")?;
        for result in results_32 {
            let line = format!("{:<50} | {:<15} | {:<15?} | {:<15.2} | {:<10}", 
                              result.test_name, result.range_size, result.elapsed, 
                              result.throughput, result.matches_found);
            println!("{}", line);
            writeln!(report_file, "{}", line)?;
        }

        // Comparison analysis
        println!("\n📈 PERFORMANCE COMPARISON:");
        writeln!(report_file, "\nPERFORMANCE COMPARISON:")?;
        
        for (result_64, result_32) in results_64.iter().zip(results_32.iter()) {
            if result_64.test_name.contains(&result_32.test_name.replace(" (32-bit)", "")) {
                let speedup = result_32.elapsed.as_secs_f64() / result_64.elapsed.as_secs_f64();
                let comparison = if speedup > 1.0 {
                    format!("64-bit is {:.2}x FASTER", speedup)
                } else if speedup < 1.0 {
                    format!("32-bit is {:.2}x FASTER", 1.0 / speedup)
                } else {
                    "Similar performance".to_string()
                };
                
                let line = format!("  {:<40} → {}", result_64.test_name, comparison);
                println!("{}", line);
                writeln!(report_file, "{}", line)?;
            }
        }
    }

    // Key insights
    println!("\n🔑 KEY INSIGHTS:");
    writeln!(report_file, "\nKEY INSIGHTS:")?;
    
    let insights = vec![
        "1. 64-bit processes have vastly larger address spaces (theoretically 16EB vs 4GB)",
        "2. Actual scan performance depends on COMMITTED memory, not virtual address range",
        "3. AOB scanner automatically skips uncommitted regions, making 64-bit scans efficient",
        "4. Pattern complexity and anchor byte selection significantly impact performance",
        "5. Early exit (find_first) provides massive speedup when matches are common",
    ];
    
    for insight in insights {
        println!("  {}", insight);
        writeln!(report_file, "  {}", insight)?;
    }

    println!("\n✅ Report saved to: aobscan_performance_report.txt");
    
    Ok(())
}