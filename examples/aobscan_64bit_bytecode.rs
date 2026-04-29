//! AOB Scan Extreme Performance Benchmark for 64-bit Process
//!
//! This benchmark tests the performance of scanning complex 64-bit bytecode patterns
//! in large address spaces.
//!
//! ## Target Process
//! - **64-bit**: Replace with your target 64-bit process name
//! - **Address Range**: Configurable (default: 0x7FF000000000 - 0x7FFFFFFFFFFF, ~16TB)
//!
//! ## Test Pattern
//! Complex multi-instruction 64-bit bytecode sequence:
//! ```
//! 48 81 C2 00010000    - ADD RDX, 0x100
//! 49 81 E8 00010000    - SUB R8, 0x100
//! 49 81 F8 00010000    - CMP R8, 0x100
//! 0F83 78FFFFFF        - JAE rel32
//! 4D 8D 48 1F          - LEA R9, [R8+0x1F]
//! 49 83 E1 E0          - AND RCX, 0xFFFFFFFFFFFFFFE0
//! 4D 8B D9             - MOV R11, R9
//! 49 C1 EB 05          - SHR R11, 5
//! 47 8B 9C 9A D02FE201 - MOV EBX, [R10+R11*4+0x1E22FD0]
//! 4D 03 DA             - ADD R11, RDX
//! 41 FF E3             - JMP R11
//! C4A17E6F8C 0A 00 FFFFFF - VMOVDQU XMM1, [RDX+RCX-0x100]
//! C4A17E7F8C 09 00 FFFFFF - VMOVDQU [RCX+R9-0x100], XMM1
//! ... (more AVX instructions)
//! ```
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_64bit_bytecode --features "memory_aobscan" --release
//! ```
//!
//! # Output
//! - Console output with real-time progress
//! - Detailed report saved to: `aobscan_64bit_report.txt`

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;
use std::time::Instant;
use std::fs::File;
use std::io::Write;

struct BenchmarkResult {
    test_name: String,
    pattern_desc: String,
    range_size: String,
    elapsed: std::time::Duration,
    matches_found: usize,
    #[allow(dead_code)]
    throughput: f64, // scans/sec
}

impl BenchmarkResult {
    fn new(test_name: &str, pattern_desc: &str, range_size: &str, 
           elapsed: std::time::Duration, matches: usize, throughput: f64) -> Self {
        Self {
            test_name: test_name.to_string(),
            pattern_desc: pattern_desc.to_string(),
            range_size: range_size.to_string(),
            elapsed,
            matches_found: matches,
            throughput,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 64-bit Bytecode AOB Scan Performance Benchmark ===\n");
    
    // TODO: Replace with your actual 64-bit process name
    let target_process = "target_64bit.exe";
    
    println!("Target: {} (64-bit)", target_process);
    println!("Address Range: 0x7FF000000000 - 0x7FFFFFFFFFFF (~16TB)");
    println!("Pattern: Complex 64-bit instruction sequence (70+ bytes)\n");

    let mut results = Vec::new();

    // Initialize 64-bit process
    let process = match initialize_process(target_process)? {
        Some(p) => p,
        None => {
            eprintln!("❌ ERROR: {} not found or cannot be accessed!", target_process);
            eprintln!("Please ensure:");
            eprintln!("  1. {} is running", target_process);
            eprintln!("  2. You have sufficient permissions");
            eprintln!("  3. The process is 64-bit\n");
            return Ok(());
        }
    };

    println!("✓ Connected to {} (PID: {})\n", target_process, process.get_pid());

    // Test 1: Full complex pattern scan
    let test1_results = benchmark_full_pattern(&process)?;
    results.extend(test1_results);

    // Test 2: Pattern segments
    let test2_results = benchmark_pattern_segments(&process)?;
    results.extend(test2_results);

    // Test 3: Different range sizes
    let test3_results = benchmark_range_sizes(&process)?;
    results.extend(test3_results);

    // Test 4: Early exit vs find all
    let test4_results = benchmark_early_exit_vs_find_all(&process)?;
    results.extend(test4_results);

    // Generate report
    generate_report(&results)?;

    println!("\n=== Benchmark Complete ===");
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

/// Test 1: Full complex pattern scan with detailed results
fn benchmark_full_pattern(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 1] Full Complex Pattern Scan - Detailed Results");
    println!("====================================================");

    let handle = process.get_handle();
    
    // The complete bytecode pattern from user request
    let pattern_str = "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00 49 81 F8 00 01 00 00 0F 83 78 FF FF FF 4D 8D 48 1F 49 83 E1 E0 4D 8B D9 49 C1 EB 05 47 8B 9C 9A D0 2F E2 01 4D 03 DA 41 FF E3 C4 A1 7E 6F 8C 0A 00 FF FF FF C4 A1 7E 7F 8C 09 00 FF FF FF C4 A1 7E 6F 8C 0A 20 FF FF FF C4 A1 7E 7F 8C 09 20 FF FF FF C4 A1 7E 6F 8C 0A 40 FF FF FF";
    
    println!("  Pattern length: {} bytes", pattern_str.split_whitespace().count());
    println!("  Pattern preview: {}...", &pattern_str[..60]);
    println!();

    let start_addr = 0x7FF000000000usize;
    let length = 0xFFFFFFFFFFFFusize; // Full 64-bit range

    println!("  Scanning full 64-bit address space...");
    println!("  This may take a few seconds...\n");
    
    // Execute scan
    let start = Instant::now();
    let scan_results = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(true)
        .scan()?;
    
    let elapsed = start.elapsed();
    
    println!("  ✓ Scan completed!");
    println!("  Total time: {:?}", elapsed);
    println!("  Total matches found: {}\n", scan_results.len());

    if !scan_results.is_empty() {
        println!("  📍 All Match Addresses:");
        println!("  {}", "-".repeat(80));
        
        for (i, addr) in scan_results.iter().enumerate() {
            println!("  [{:3}] 0x{:016X}", i + 1, addr);
        }
        
        println!("  {}", "-".repeat(80));
        println!();

        // Verify each match by reading and comparing bytes
        println!("  🔍 Verification Results:");
        println!("  {}", "-".repeat(80));
        
        let pattern_bytes: Vec<u8> = pattern_str.split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect();
        
        let mut verified_count = 0;
        let mut failed_count = 0;
        
        for (i, addr) in scan_results.iter().enumerate() {
            match verify_match(handle, *addr, &pattern_bytes) {
                Ok(true) => {
                    println!("  [{:3}] 0x{:016X} ✅ VERIFIED", i + 1, addr);
                    verified_count += 1;
                },
                Ok(false) => {
                    println!("  [{:3}] 0x{:016X} ❌ FAILED (bytes mismatch)", i + 1, addr);
                    failed_count += 1;
                },
                Err(e) => {
                    println!("  [{:3}] 0x{:016X} ⚠️  ERROR: {}", i + 1, addr, e);
                    failed_count += 1;
                }
            }
        }
        
        println!("  {}", "-".repeat(80));
        println!("\n  Verification Summary:");
        println!("    ✅ Verified: {}", verified_count);
        println!("    ❌ Failed:   {}", failed_count);
        println!("    Total:       {}", scan_results.len());
        
        if failed_count == 0 {
            println!("\n  🎉 All matches are ACCURATE!");
        } else {
            println!("\n  ⚠️  Warning: Some matches failed verification!");
        }
    } else {
        println!("  No matches found in the entire address space.");
    }
    
    println!();

    let throughput = 1.0 / elapsed.as_secs_f64();
    
    Ok(vec![BenchmarkResult::new(
        "Full Pattern (Full Range)",
        &format!("{} bytes", pattern_str.split_whitespace().count()),
        "~16TB",
        elapsed,
        scan_results.len(),
        throughput,
    )])
}

/// Helper function to verify a match by reading memory and comparing bytes
fn verify_match(handle: windows::Win32::Foundation::HANDLE, address: usize, pattern: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    
    let mut buffer = vec![0u8; pattern.len()];
    let mut bytes_read = 0;
    
    unsafe {
        let result = ReadProcessMemory(
            handle,
            address as *const std::ffi::c_void,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            pattern.len(),
            Some(&mut bytes_read),
        );
        
        if result.is_ok() && bytes_read == pattern.len() {
            // Compare bytes
            Ok(buffer == pattern)
        } else {
            Err("Failed to read memory".into())
        }
    }
}

/// Test 2: Pattern segments performance
fn benchmark_pattern_segments(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 2] Pattern Segments Performance");
    println!("======================================");

    let handle = process.get_handle();
    
    // Break down the pattern into segments
    let segments = vec![
        ("Segment 1: First 3 instructions", "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00 49 81 F8 00 01 00 00"),
        ("Segment 2: Branch + LEA", "0F 83 78 FF FF FF 4D 8D 48 1F"),
        ("Segment 3: Bit operations", "49 83 E1 E0 4D 8B D9 49 C1 EB 05"),
        ("Segment 4: Memory access", "47 8B 9C 9A D0 2F E2 01 4D 03 DA 41 FF E3"),
        ("Segment 5: AVX instructions", "C4 A1 7E 6F 8C 0A 00 FF FF FF C4 A1 7E 7F 8C 09 00 FF FF FF"),
    ];

    let iterations = 5;
    let start_addr = 0x7FF000000000usize;
    let length = 0x100000000usize; // 4GB
    
    let mut results = Vec::new();

    for (name, pattern_str) in &segments {
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
        }
        
        let elapsed = start.elapsed();
        let per_scan = elapsed / iterations as u32;
        let throughput = iterations as f64 / elapsed.as_secs_f64();
        
        println!("    Total time: {:?}", elapsed);
        println!("    Per scan:   {:?}", per_scan);
        println!("    Throughput: {:.2} scans/sec", throughput);
        println!("    Avg matches: {:.1}", total_matches as f64 / iterations as f64);
        
        results.push(BenchmarkResult::new(
            name,
            &format!("{} bytes", pattern_str.split_whitespace().count()),
            "4GB",
            elapsed,
            total_matches / iterations,
            throughput,
        ));
    }

    println!();
    Ok(results)
}

/// Test 3: Different range sizes
fn benchmark_range_sizes(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 3] Range Size Impact on Performance");
    println!("==========================================");

    let handle = process.get_handle();
    
    // Use a medium-length segment for this test
    let pattern_str = "48 81 C2 00 01 00 00 49 81 E8 00 01 00 00 49 81 F8 00 01 00 00";
    let start_addr = 0x7FF000000000usize;
    
    let ranges = vec![
        ("1 GB", 0x40000000usize),
        ("4 GB", 0x100000000usize),
        ("10 GB", 0x280000000usize),
        ("100 GB", 0x1900000000usize),
        ("1 TB", 0x10000000000usize),
    ];

    let iterations = 3;
    let mut results = Vec::new();

    println!("  Pattern: {} bytes", pattern_str.split_whitespace().count());
    println!("  Start Address: 0x{:X}", start_addr);
    println!("  Iterations per range: {}\n", iterations);

    for (range_name, length) in &ranges {
        println!("  Testing range: {} (0x{:X})", range_name, length);
        
        let start = Instant::now();
        let mut total_matches = 0;
        
        for _iter in 0..iterations {
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
            &format!("Range {}", range_name),
            &format!("{} bytes", pattern_str.split_whitespace().count()),
            range_name,
            elapsed,
            total_matches / iterations,
            throughput,
        ));
    }

    Ok(results)
}

/// Test 4: Early exit vs find all comparison
fn benchmark_early_exit_vs_find_all(process: &Process) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    println!("[Test 4] Early Exit vs Find All Comparison");
    println!("===========================================");

    let handle = process.get_handle();
    
    let pattern_str = "48 81 C2 00 01 00 00";
    let start_addr = 0x7FF000000000usize;
    let length = 0xFFFFFFFFFFFFusize; // Nearly full 64-bit range
    
    println!("  Pattern: {}", pattern_str);
    println!("  Range: ~16TB (full 64-bit address space)");
    println!();

    let mut results = Vec::new();

    // Test early exit (find_first)
    println!("  Test A: Early Exit (find_all=false)");
    println!("  ------------------------------------");
    
    let iterations = 5;
    let start = Instant::now();
    let mut found_count = 0;
    
    for _iter in 0..iterations {
        let scan_results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(false)
            .scan()?;
        
        if !scan_results.is_empty() {
            found_count += 1;
        }
    }
    
    let elapsed_early = start.elapsed();
    let throughput_early = iterations as f64 / elapsed_early.as_secs_f64();
    
    println!("    Total time: {:?}", elapsed_early);
    println!("    Per scan:   {:?}", elapsed_early / iterations as u32);
    println!("    Throughput: {:.2} scans/sec", throughput_early);
    println!("    Success rate: {}/{}\n", found_count, iterations);
    
    results.push(BenchmarkResult::new(
        "Early Exit (find_first)",
        pattern_str,
        "~16TB",
        elapsed_early,
        found_count,
        throughput_early,
    ));

    // Test find all
    println!("  Test B: Find All (find_all=true)");
    println!("  ---------------------------------");
    println!("  Warning: This will scan entire committed memory...\n");
    
    let start = Instant::now();
    let scan_results = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(true)
        .scan()?;
    
    let elapsed_all = start.elapsed();
    let throughput_all = 1.0 / elapsed_all.as_secs_f64();
    
    println!("    Total time: {:?}", elapsed_all);
    println!("    Matches found: {}", scan_results.len());
    println!("    Throughput: {:.2} scans/sec", throughput_all);
    
    if !scan_results.is_empty() {
        println!("    First 3 matches:");
        for (i, addr) in scan_results.iter().take(3).enumerate() {
            println!("      [{}] 0x{:X}", i + 1, addr);
        }
    }
    println!();
    
    results.push(BenchmarkResult::new(
        "Find All",
        pattern_str,
        "~16TB",
        elapsed_all,
        scan_results.len(),
        throughput_all,
    ));

    // Calculate speedup
    let speedup = elapsed_all.as_secs_f64() / elapsed_early.as_secs_f64();
    println!("  Speedup Analysis:");
    println!("    Early exit is {:.2}x faster than find all", speedup);
    println!();

    Ok(results)
}

/// Generate comprehensive report
fn generate_report(results: &[BenchmarkResult]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       64-BIT BYTECODE AOB SCAN PERFORMANCE REPORT          ║");
    println!("║       Target: Configurable 64-bit Process                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Save to file
    let mut report_file = File::create("aobscan_64bit_report.txt")?;

    writeln!(report_file, "64-bit Bytecode AOB Scan Performance Report")?;
    writeln!(report_file, "Target: Configurable 64-bit Process")?;
    writeln!(report_file, "Generated: {}\n", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())?;
    
    writeln!(report_file, "{:<45} | {:<12} | {:<12} | {:<15} | {:<10}", 
             "Test Name", "Pattern", "Range", "Time", "Matches")?;
    writeln!(report_file, "{}", "-".repeat(100))?;

    println!("{:<45} | {:<12} | {:<12} | {:<15} | {:<10}", 
             "Test Name", "Pattern", "Range", "Time", "Matches");
    println!("{}", "-".repeat(100));

    for result in results {
        let line = format!("{:<45} | {:<12} | {:<12} | {:<15?} | {:<10}", 
                          result.test_name, result.pattern_desc, result.range_size, 
                          result.elapsed, result.matches_found);
        println!("{}", line);
        writeln!(report_file, "{}", line)?;
    }

    // Key insights
    println!("\n🔑 KEY INSIGHTS:");
    writeln!(report_file, "\nKEY INSIGHTS:")?;
    
    let insights = vec![
        "1. Long patterns (>50 bytes) provide better specificity but require more processing",
        "2. 64-bit address space (~16TB) is efficiently handled by skipping uncommitted regions",
        "3. Early exit (find_first) provides massive speedup when matches exist",
        "4. Pattern segmentation helps identify which parts are most selective",
        "5. AVX instructions (C4 prefix) are rare and make excellent anchor points",
    ];
    
    for insight in insights {
        println!("  {}", insight);
        writeln!(report_file, "  {}", insight)?;
    }

    println!("\n✅ Report saved to: aobscan_64bit_report.txt");
    
    Ok(())
}