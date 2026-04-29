//! Quick AOB Scan Speed Test
//!
//! A simplified test to quickly measure AOB scan performance.
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_quick_test --features "memory_aobscan" --release
//! ```

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quick AOB Scan Speed Test ===\n");

    // Initialize process
    println!("[1/3] Connecting to lf2.exe...");
    let process = Process::builder("lf2.exe").build();
    
    match process.init() {
        Ok(()) => {
            println!("✓ Connected (PID: {})\n", process.get_pid());
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
            eprintln!("  Please ensure lf2.exe is running!");
            return Err(Box::new(e));
        }
    }

    // Test pattern
    let pattern = "2B FA 89 B9 08 03 00 00 8B 96 F0 07 00 00";
    println!("[2/3] Testing pattern: {}", pattern);
    println!("  Length: {} bytes\n", pattern.split_whitespace().count());

    // Benchmark: 16MB range
    println!("[3/3] Running benchmarks...\n");
    
    let range = 0x1000000;  // 16MB
    let iterations = 5;
    
    println!("  Scanning 16MB range, {} iterations...", iterations);
    let start = Instant::now();
    
    let mut total_matches = 0;
    for i in 0..iterations {
        let iter_start = Instant::now();
        let results = AobScanBuilder::new(process.get_handle())
            .pattern_str(pattern)?
            .start_address(0x0)
            .length(range)
            .find_all(false)  // Early exit for speed
            .scan()?;
        
        let iter_elapsed = iter_start.elapsed();
        total_matches += results.len();
        
        println!("    Iteration {}: {:?} (found {} matches)", 
                 i + 1, iter_elapsed, results.len());
    }
    
    let total_elapsed = start.elapsed();
    let avg_elapsed = total_elapsed / iterations;
    
    println!("\n  Results:");
    println!("    Total time:     {:?}", total_elapsed);
    println!("    Average time:   {:?}", avg_elapsed);
    println!("    Total matches:  {}", total_matches);
    println!("    Throughput:     {:.2} MB/s", 
             (range as f64 * iterations as f64 / 1_048_576.0) / total_elapsed.as_secs_f64());
    
    // Full memory scan test
    println!("\n  Full memory scan test (0x0 - 0xFFFFFFFF)...");
    println!("  This may take 10-30 seconds...\n");
    
    let start = Instant::now();
    let results = AobScanBuilder::new(process.get_handle())
        .pattern_str(pattern)?
        .start_address(0x0)
        .length(0xFFFFFFFF)
        .find_all(true)
        .scan()?;
    
    let elapsed = start.elapsed();
    
    println!("  Found {} matches in {:?}", results.len(), elapsed);
    println!("  Estimated speed: {:.2} MB/s", 
             2048.0 / elapsed.as_secs_f64());  // Assume ~2GB actual memory
    
    if !results.is_empty() {
        println!("\n  First 5 matches:");
        for (i, addr) in results.iter().take(5).enumerate() {
            println!("    [{}] 0x{:08X}", i + 1, addr);
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
