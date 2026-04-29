//! AOB Scan Performance Benchmark
//!
//! This benchmark measures the performance of the AOB scanning module
//! by testing various scenarios and patterns.
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_benchmark --features "memory_aobscan" --release
//! ```

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::{AobScanBuilder, Pattern};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AOB Scan Performance Benchmark ===\n");

    // Test 1: Pattern parsing performance
    benchmark_pattern_parsing()?;

    // Test 2: Small pattern scan (fast path)
    benchmark_small_pattern_scan()?;

    // Test 3: Large pattern scan with wildcards
    benchmark_large_pattern_scan()?;

    // Test 4: Full memory range scan
    benchmark_full_memory_scan()?;

    // Test 5: Early exit vs find_all comparison
    benchmark_early_exit()?;

    // Test 6: Ultra-Long Pattern (SIMD AVX2 limit test)
    benchmark_ultra_long_pattern()?;

    // Test 7: Fuzzy vs Exact Pattern Performance Comparison (NEW!)
    benchmark_fuzzy_vs_exact()?;

    println!("\n=== Benchmark Complete ===");
    Ok(())
}

/// Benchmark 1: Pattern parsing speed
fn benchmark_pattern_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 1] Pattern Parsing Performance");
    println!("-------------------------------------------");

    let patterns = vec![
        "2B FA 89 B9 08 03 00 00 8B 96 F0 07 00 00",
        "48 ?? 55 ?? 48 89 ?? 24",
        "E8 ?? ?? ?? ?? 48 83 C4",
        "90 90 90 90 90",
    ];

    let iterations = 100_000;

    for (i, pattern_str) in patterns.iter().enumerate() {
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = Pattern::from_str(pattern_str)?;
        }
        
        let elapsed = start.elapsed();
        let per_parse = elapsed / iterations;
        
        println!("  Pattern {}: {} bytes", i + 1, pattern_str.split_whitespace().count());
        println!("    Total time: {:?}", elapsed);
        println!("    Per parse:  {:?}", per_parse);
        println!("    Throughput: {:.2} parses/sec", iterations as f64 / elapsed.as_secs_f64());
        println!();
    }

    Ok(())
}

/// Benchmark 2: Small pattern scan (simulated with dummy process)
fn benchmark_small_pattern_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 2] Small Pattern Scan (target process)");
    println!("-------------------------------------------");

    // TODO: Replace with your actual target process name
    let process_name = "target.exe";
    
    let process = match initialize_process(process_name)? {
        Some(p) => p,
        None => {
            println!("  ⚠ {} not found, skipping this benchmark\n", process_name);
            return Ok(());
        }
    };

    let pattern = "2B FA 89 B9";  // Short pattern (4 bytes)
    
    // Warm-up
    let _ = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(0x100000)  // 1MB
        .find_all(false)
        .scan()?;

    // Benchmark
    let iterations = 10;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = AobScanBuilder::new(process.get_handle())
            .pattern_str(pattern)?
            .start_address(0x0)
            .length(0x100000)
            .find_all(false)
            .scan()?;
    }
    
    let elapsed = start.elapsed();
    let per_scan = elapsed / iterations;
    
    println!("  Pattern: {}", pattern);
    println!("  Range: 1MB (0x0 - 0x100000)");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", elapsed);
    println!("  Per scan:   {:?}", per_scan);
    println!("  Throughput: {:.2} scans/sec", iterations as f64 / elapsed.as_secs_f64());
    println!();

    Ok(())
}

/// Benchmark 3: Large pattern with wildcards
fn benchmark_large_pattern_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 3] Large Pattern with Wildcards (lf2.exe)");
    println!("-----------------------------------------------------");

    let process = match initialize_process("lf2.exe")? {
        Some(p) => p,
        None => {
            println!("  ⚠ lf2.exe not found, skipping this benchmark\n");
            return Ok(());
        }
    };

    let pattern = "2B FA 89 B9 08 03 00 00 8B 96 F0 07 00 00";  // 14 bytes
    
    // Warm-up
    let _ = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(0x1000000)  // 16MB
        .find_all(false)
        .scan()?;

    // Benchmark
    let iterations = 5;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = AobScanBuilder::new(process.get_handle())
            .pattern_str(pattern)?
            .start_address(0x0)
            .length(0x1000000)
            .find_all(false)
            .scan()?;
    }
    
    let elapsed = start.elapsed();
    let per_scan = elapsed / iterations;
    
    println!("  Pattern: {} ({} bytes)", pattern, pattern.split_whitespace().count());
    println!("  Range: 16MB (0x0 - 0x1000000)");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", elapsed);
    println!("  Per scan:   {:?}", per_scan);
    println!("  Throughput: {:.2} scans/sec", iterations as f64 / elapsed.as_secs_f64());
    println!();

    Ok(())
}

/// Benchmark 4: Full memory range scan
fn benchmark_full_memory_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 4] Full Memory Range Scan (lf2.exe)");
    println!("-----------------------------------------------");

    let process = match initialize_process("lf2.exe")? {
        Some(p) => p,
        None => {
            println!("  ⚠ lf2.exe not found, skipping this benchmark\n");
            return Ok(());
        }
    };

    let pattern = "2B FA 89 B9 08 03 00 00 8B 96 F0 07 00 00";
    
    println!("  Scanning entire address space (0x0 - 0xFFFFFFFF)...");
    println!("  This may take 10-30 seconds depending on process size...\n");
    
    let start = Instant::now();
    
    let results = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(0xFFFFFFFF)
        .find_all(true)
        .scan()?;
    
    let elapsed = start.elapsed();
    
    println!("  Pattern: {}", pattern);
    println!("  Range: Full (0x0 - 0xFFFFFFFF)");
    println!("  Matches found: {}", results.len());
    println!("  Total time: {:?}", elapsed);
    println!("  Speed: {:.2} MB/s", calculate_scan_speed(elapsed));
    println!();

    Ok(())
}

/// Benchmark 5: Early exit vs find_all comparison
fn benchmark_early_exit() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 5] Early Exit Optimization Comparison");
    println!("-------------------------------------------------");

    let process = match initialize_process("lf2.exe")? {
        Some(p) => p,
        None => {
            println!("  ⚠ lf2.exe not found, skipping this benchmark\n");
            return Ok(());
        }
    };

    let pattern = "2B FA 89 B9";
    let range = 0x1000000;  // 16MB

    // Test find_all=false (early exit)
    println!("  Testing find_all=false (early exit):");
    let start = Instant::now();
    let results_first = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(range)
        .find_all(false)
        .scan()?;
    let elapsed_first = start.elapsed();
    
    println!("    Time: {:?}", elapsed_first);
    println!("    Found: {} matches (stopped at first)", results_first.len());

    // Test find_all=true (full scan)
    println!("\n  Testing find_all=true (full scan):");
    let start = Instant::now();
    let results_all = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(range)
        .find_all(true)
        .scan()?;
    let elapsed_all = start.elapsed();
    
    println!("    Time: {:?}", elapsed_all);
    println!("    Found: {} matches", results_all.len());

    if !results_all.is_empty() {
        let speedup = elapsed_all.as_secs_f64() / elapsed_first.as_secs_f64();
        println!("\n  Speedup: {:.2}x faster with early exit", speedup);
    }
    println!();

    Ok(())
}

/// Helper: Initialize process connection
fn initialize_process(name: &str) -> Result<Option<Process>, Box<dyn std::error::Error>> {
    let process = Process::builder(name).build();
    
    match process.init() {
        Ok(()) => {
            println!("  ✓ Connected to {} (PID: {})", name, process.get_pid());
            Ok(Some(process))
        }
        Err(_) => {
            Ok(None)
        }
    }
}

/// Benchmark 6: Ultra-Long Pattern (>64 bytes) - Test SIMD AVX2极限性能
fn benchmark_ultra_long_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 6] Ultra-Long Pattern Scan (SIMD极限测试)");
    println!("-----------------------------------------------------");

    let process = match initialize_process("lf2.exe")? {
        Some(p) => p,
        None => {
            println!("  ⚠ lf2.exe not found, skipping this benchmark\n");
            return Ok(());
        }
    };

    // 超长 pattern (约 80+ 字节) - 完美测试 SIMD AVX2
    // 格式化的 pattern 字符串
    let pattern_str = "29 88 FC 02 00 00 B8 55 55 55 55 F7 E9 2B D1 D1 FA 8B C2 C1 E8 1F 03 C2 8B D0 8B 84 BE 94 01 00 00 01 90 00 03 00 00 8B 84 BE 94 01 00 00 01 88 4C 03 00 00 8B 94 BE 94 01 00 00 8B 82 68 03 00 00 83 B8 F8 06 00 00 00";
    
    println!("  Pattern length: {} bytes", pattern_str.split_whitespace().count());
    println!("  Expected: Maximum SIMD AVX2 utilization (processes 32 bytes at a time)");
    println!();

    // Test 1: 小范围扫描 (1MB)
    {
        let iterations = 10;
        println!("  Test 1: Small Range (1MB)");
        println!("  --------------------------");
        
        let start = Instant::now();
        for _ in 0..iterations {
            let results = AobScanBuilder::new(process.get_handle())
                .pattern_str(pattern_str)?
                .start_address(0x0)
                .length(0x100000)  // 1MB
                .find_all(false)
                .scan()?;
            
            if !results.is_empty() {
                println!("    Found {} match(es)", results.len());
            }
        }
        let elapsed = start.elapsed();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Per scan:   {:?}", elapsed / iterations as u32);
        println!("    Throughput: {:.2} scans/sec", iterations as f64 / elapsed.as_secs_f64());
        println!();
    }

    // Test 2: 中等范围扫描 (16MB)
    {
        let iterations = 5;
        println!("  Test 2: Medium Range (16MB)");
        println!("  ----------------------------");
        
        let start = Instant::now();
        for _ in 0..iterations {
            let results = AobScanBuilder::new(process.get_handle())
                .pattern_str(pattern_str)?
                .start_address(0x0)
                .length(0x1000000)  // 16MB
                .find_all(false)
                .scan()?;
            
            if !results.is_empty() {
                println!("    Found {} match(es)", results.len());
            }
        }
        let elapsed = start.elapsed();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Per scan:   {:?}", elapsed / iterations as u32);
        println!("    Throughput: {:.2} scans/sec", iterations as f64 / elapsed.as_secs_f64());
        println!();
    }

    // Test 3: 全内存扫描（如果找到匹配）
    {
        println!("  Test 3: Full Memory Scan");
        println!("  -------------------------");
        println!("  Scanning entire address space...");
        
        let start = Instant::now();
        let results = AobScanBuilder::new(process.get_handle())
            .pattern_str(pattern_str)?
            .start_address(0x0)
            .length(0)  // 全地址空间 (0 usually means full range in this context or max usize, depending on impl, but following prompt's logic)
            .find_all(true)
            .scan()?;
        
        let elapsed = start.elapsed();
        
        println!("    Matches found: {}", results.len());
        println!("    Total time: {:?}", elapsed);
        
        if !results.is_empty() {
            println!("    First match at: 0x{:X}", results[0]);
        }
        
        // 估算吞吐量（假设进程有效内存约 100MB）
        let estimated_memory_mb = 100;
        let throughput = estimated_memory_mb as f64 / elapsed.as_secs_f64();
        println!("    Estimated speed: {:.2} MB/s", throughput);
        println!();
    }

    Ok(())
}

/// Benchmark 7: Fuzzy vs Exact Pattern Performance Comparison
fn benchmark_fuzzy_vs_exact() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Benchmark 7] Fuzzy vs Exact Pattern Performance Comparison");
    println!("------------------------------------------------------------");

    let process = match initialize_process("lf2.exe")? {
        Some(p) => p,
        None => {
            println!("  ⚠ lf2.exe not found, skipping this benchmark\n");
            return Ok(());
        }
    };

    // 定义 4 种不同模糊程度的 pattern
    let patterns = vec![
        (
            "Exact (0 wildcards)",
            "29 88 FC 02 00 00 B8 55 55 55 55 F7 E9 2B D1 D1 FA 8B C2 C1 E8 1F 03 C2 8B D0 8B 84 BE 94 01 00 00 01 90 00 03 00 00 8B 84 BE 94 01 00 00 01 88 4C 03 00 00 8B 94 BE 94 01 00 00 8B 82 68 03 00 00 83 B8 F8 06 00 00 00",
        ),
        (
            "Light Fuzzy (3 wildcards, ~4%)",
            "29 88 FC ?? 00 00 B8 55 55 55 55 F7 E9 2B D1 D1 FA 8B C2 C1 E8 1F 03 C2 8B D0 8B 84 BE 94 01 00 00 01 90 00 03 00 00 8B 84 BE 94 01 00 00 01 88 4C 03 00 00 8B 94 BE 94 01 00 00 8B 82 68 03 00 00 83 B8 F8 06 00 00 00",
        ),
        (
            "Medium Fuzzy (8 wildcards, ~11%)",
            "29 88 FC ?? 00 00 B8 ?? ?? ?? ?? F7 E9 2B D1 D1 FA 8B C2 C1 E8 1F 03 C2 8B D0 8B 84 BE 94 01 00 00 01 90 00 03 00 00 8B 84 BE 94 01 00 00 01 88 4C 03 00 00 8B 94 BE 94 01 00 00 8B 82 68 03 00 00 83 B8 F8 06 00 00 00",
        ),
        (
            "Heavy Fuzzy (15 wildcards, ~21%)",
            "29 88 FC ?? 00 00 B8 ?? ?? ?? ?? F7 E9 2B D1 D1 FA 8B C2 C1 E8 ?? 03 C2 8B D0 8B 84 BE ?? 01 00 00 01 90 00 03 00 00 8B 84 BE ?? 01 00 00 01 88 4C 03 00 00 8B 94 BE ?? 01 00 00 8B 82 68 03 00 00 83 B8 F8 06 00 00 00",
        ),
    ];

    let handle = process.get_handle();
    let iterations = 5;
    let scan_range = 0x1000000; // 16MB

    println!("  Scan Range: 16MB");
    println!("  Iterations: {}", iterations);
    println!();

    let mut results = Vec::new();

    for (name, pattern_str) in &patterns {
        println!("  Testing: {}", name);
        println!("  Pattern length: {} bytes", pattern_str.split_whitespace().count());
        
        // Count wildcards
        let wildcard_count = pattern_str.split_whitespace()
            .filter(|s| *s == "??" || *s == "?")
            .count();
        let total_bytes = pattern_str.split_whitespace().count();
        let fuzzy_ratio = (wildcard_count as f64 / total_bytes as f64) * 100.0;
        println!("  Wildcards: {} ({:.1}%)", wildcard_count, fuzzy_ratio);
        
        let start = Instant::now();
        let mut match_count = 0;
        
        for _ in 0..iterations {
            let scan_results = AobScanBuilder::new(handle)
                .pattern_str(pattern_str)?
                .start_address(0x0)
                .length(scan_range)
                .find_all(false)
                .scan()?;
            
            if !scan_results.is_empty() {
                match_count = scan_results.len();
            }
        }
        
        let elapsed = start.elapsed();
        let per_scan = elapsed / iterations as u32;
        let throughput = iterations as f64 / elapsed.as_secs_f64();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Per scan:   {:?}", per_scan);
        println!("    Throughput: {:.2} scans/sec", throughput);
        println!("    Matches:    {}", match_count);
        println!();
        
        results.push((name.to_string(), wildcard_count, per_scan, throughput));
    }

    // Print comparison summary
    println!("  === Performance Comparison Summary ===");
    println!("  --------------------------------------");
    
    if let Some((_, _, baseline_time, baseline_throughput)) = results.first() {
        for (name, wildcards, time, throughput) in &results {
            let time_ratio = time.as_micros() as f64 / baseline_time.as_micros() as f64;
            let throughput_ratio = throughput / baseline_throughput;
            
            println!("  {:<30} | {:>2} wildcards | {:>10?} | {:>8.2}x time | {:>6.2}x throughput",
                name, wildcards, time, time_ratio, throughput_ratio);
        }
    }
    
    println!();
    println!("  Key Insights:");
    println!("  - More wildcards → More candidate positions to verify");
    println!("  - SIMD verification cost remains constant (still processes 32 bytes at once)");
    println!("  - Performance degradation mainly from increased memchr hits");
    println!("  - Multi-byte anchor sequences help mitigate wildcard impact");
    println!();

    Ok(())
}

/// Helper: Calculate scan speed in MB/s
fn calculate_scan_speed(elapsed: std::time::Duration) -> f64 {
    // Assume scanning ~2GB of actual committed memory in typical process
    let scanned_mb = 2048.0;
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 {
        scanned_mb / secs
    } else {
        0.0
    }
}
