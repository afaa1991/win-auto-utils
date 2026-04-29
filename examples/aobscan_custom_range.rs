//! AOB Scan Performance Benchmark for Custom Address Range
//!
//! This benchmark tests AOB scanning performance with custom address range and bytecode pattern.
//!
//! ## Target Process
//! - **64-bit**: PUBPETS.exe
//!
//! ## Test Configuration
//! - **Address Range**: 0x20000000000 - 0x30000000000 (64GB range)
//! - **Pattern**: Custom 64-bit bytecode sequence (71 bytes)
//!   ```
//!   F3 0F11 68 30              - MOVSS [RAX+0x30], XMM5
//!   48 B8 E0A6FAE71E020000     - MOV RAX, 0x21EE7FAA6E0
//!   48 8B 00                   - MOV RAX, [RAX]
//!   48 8B 40 20                - MOV RAX, [RAX+0x20]
//!   48 89 45 F0                - MOV [RBP-0x10], RAX
//!   48 B8 E0A6FAE71E020000     - MOV RAX, 0x21EE7FAA6E0
//!   48 8B 08                   - MOV RCX, [RAX]
//!   48 85 C9                   - TEST RCX, RCX
//!   0F84 56010000              - JE rel32
//!   48 83 C1 30                - ADD RCX, 0x30
//!   48 8D 64 24 00             - LEA RSP, [RSP]
//!   49 BB 70224B9A1F020000     - MOV R11, 0x21F9A4B2270
//!   ```
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_custom_range --features "memory_aobscan" --release
//! ```

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;
use std::time::Instant;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Range AOB Scan Performance Benchmark ===\n");
    println!("Target: PUBPETS.exe (64-bit)");
    println!("Address Range: 0x20000000000 - 0x30000000000 (64GB)");
    println!("Pattern: Custom 64-bit bytecode sequence (71 bytes)\n");

    // Initialize process
    let process = match initialize_process("PUBPETS.exe")? {
        Some(p) => p,
        None => {
            eprintln!("❌ ERROR: PUBPETS.exe not found or cannot be accessed!");
            eprintln!("Please ensure:");
            eprintln!("  1. PUBPETS.exe is running");
            eprintln!("  2. You have sufficient permissions\n");
            return Ok(());
        }
    };

    println!("✓ Connected to PUBPETS.exe (PID: {})\n", process.get_pid());

    // The custom bytecode pattern
    let pattern_str = "F3 0F 11 68 30 48 B8 E0 A6 FA E7 1E 02 00 00 48 8B 00 48 8B 40 20 48 89 45 F0 48 B8 E0 A6 FA E7 1E 02 00 00 48 8B 08 48 85 C9 0F 84 56 01 00 00 48 83 C1 30 48 8D 64 24 00 49 BB 70 22 4B 9A 1F 02 00 00";
    
    println!("  Pattern length: {} bytes", pattern_str.split_whitespace().count());
    println!("  Pattern preview: {}...", &pattern_str[..60]);
    println!();

    // Execute scan - Test both modes
    println!("🔍 Testing BOTH scanning modes for comparison:\n");
    
    let handle = process.get_handle();
    let start_addr = 0x20000000000usize;
    let end_addr = 0x30000000000usize;
    let length = end_addr - start_addr; // 64GB
    
    println!("  Address Range: 0x{:016X} - 0x{:016X} ({} GB)", 
             start_addr, end_addr, length / 1024 / 1024 / 1024);
    println!("  Pattern: {} bytes\n", pattern_str.split_whitespace().count());

    // Test 1: Early Exit Mode (find_all=false)
    println!("  [Test 1] Early Exit Mode (find_all=false)");
    println!("  {}", "-".repeat(70));
    println!("  This mode stops immediately after finding the first match.\n");
    
    let start = Instant::now();
    let results_early = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(false)  // EARLY EXIT
        .scan()?;
    let elapsed_early = start.elapsed();
    
    println!("  ✓ Scan completed!");
    println!("  Time: {:?}", elapsed_early);
    println!("  Matches found: {}", results_early.len());
    if !results_early.is_empty() {
        println!("  First match: 0x{:016X}", results_early[0]);
    }
    println!();

    // Test 2: Find All Mode (find_all=true)
    println!("  [Test 2] Find All Mode (find_all=true)");
    println!("  {}", "-".repeat(70));
    println!("  This mode scans the entire range to find ALL matches.\n");
    
    let start = Instant::now();
    let results_all = AobScanBuilder::new(handle)
        .pattern_str(pattern_str)?
        .start_address(start_addr)
        .length(length)
        .find_all(true)  // FIND ALL
        .scan()?;
    let elapsed_all = start.elapsed();
    
    println!("  ✓ Scan completed!");
    println!("  Time: {:?}", elapsed_all);
    println!("  Matches found: {}", results_all.len());
    if !results_all.is_empty() {
        println!("  All matches:");
        for (i, addr) in results_all.iter().enumerate() {
            println!("    [{:3}] 0x{:016X}", i + 1, addr);
        }
    }
    println!();

    // Performance comparison
    println!("  📊 Performance Comparison:");
    println!("  {}", "=".repeat(70));
    let speedup = elapsed_all.as_secs_f64() / elapsed_early.as_secs_f64();
    println!("  Early Exit: {:?} ({:.2} scans/sec)", 
             elapsed_early, 1.0 / elapsed_early.as_secs_f64());
    println!("  Find All:   {:?} ({:.2} scans/sec)", 
             elapsed_all, 1.0 / elapsed_all.as_secs_f64());
    println!("  Speedup:    Early exit is {:.2}x FASTER!", speedup);
    println!();

    // Verify matches
    if !results_all.is_empty() {
        println!("  🔍 Verification Results:");
        println!("  {}", "-".repeat(70));
        
        let pattern_bytes: Vec<u8> = pattern_str.split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect();
        
        let mut verified_count = 0;
        let mut failed_count = 0;
        
        for (i, addr) in results_all.iter().enumerate() {
            match verify_match(handle, *addr, &pattern_bytes) {
                Ok(true) => {
                    println!("  [{:3}] 0x{:016X} ✅ VERIFIED", i + 1, addr);
                    verified_count += 1;
                },
                Ok(false) => {
                    println!("  [{:3}] 0x{:016X} ❌ FAILED", i + 1, addr);
                    failed_count += 1;
                },
                Err(e) => {
                    println!("  [{:3}] 0x{:016X} ⚠️  ERROR: {}", i + 1, addr, e);
                    failed_count += 1;
                }
            }
        }
        
        println!("  {}", "-".repeat(70));
        println!("  Verified: {}/{}", verified_count, results_all.len());
        if failed_count == 0 {
            println!("  🎉 All matches are ACCURATE!\n");
        }
    }

    // Performance metrics
    let throughput = 1.0 / elapsed_all.as_secs_f64();
    println!("  📊 Performance Metrics:");
    println!("    Total time:    {:?}", elapsed_all);
    println!("    Throughput:    {:.2} scans/sec", throughput);
    println!("    Matches found: {}", results_all.len());
    
    if !results_all.is_empty() {
        println!("    Avg per match: {:?}", elapsed_all / results_all.len() as u32);
    }
    println!();

    // Save report
    generate_report(pattern_str, start_addr, end_addr, elapsed_all, &results_all)?;

    println!("=== Benchmark Complete ===");
    
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
            Ok(buffer == pattern)
        } else {
            Err("Failed to read memory".into())
        }
    }
}

/// Generate report file
fn generate_report(
    pattern: &str,
    start_addr: usize,
    end_addr: usize,
    elapsed: std::time::Duration,
    matches: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut report_file = File::create("aobscan_custom_range_report.txt")?;

    writeln!(report_file, "Custom Range AOB Scan Performance Report")?;
    writeln!(report_file, "Target: PUBPETS.exe (64-bit)")?;
    writeln!(report_file, "Generated: {}\n", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())?;
    
    writeln!(report_file, "Configuration:")?;
    writeln!(report_file, "  Pattern: {} bytes", pattern.split_whitespace().count())?;
    writeln!(report_file, "  Pattern String: {}", pattern)?;
    writeln!(report_file, "  Start Address: 0x{:016X}", start_addr)?;
    writeln!(report_file, "  End Address:   0x{:016X}", end_addr)?;
    writeln!(report_file, "  Range Size:    {} GB", (end_addr - start_addr) / 1024 / 1024 / 1024)?;
    writeln!(report_file)?;
    
    writeln!(report_file, "Results:")?;
    writeln!(report_file, "  Total Time: {:?}", elapsed)?;
    writeln!(report_file, "  Matches Found: {}", matches.len())?;
    writeln!(report_file, "  Throughput: {:.2} scans/sec", 1.0 / elapsed.as_secs_f64())?;
    writeln!(report_file)?;
    
    if !matches.is_empty() {
        writeln!(report_file, "Match Addresses:")?;
        for (i, addr) in matches.iter().enumerate() {
            writeln!(report_file, "  [{:3}] 0x{:016X}", i + 1, addr)?;
        }
    }
    
    writeln!(report_file, "\n✅ Report generated successfully")?;
    
    println!("  📄 Report saved to: aobscan_custom_range_report.txt");
    
    Ok(())
}