//! Example: AOB Scan Test
//!
//! This example demonstrates how to use the AOB scan feature to search for
//! specific byte patterns in a target process's memory.
//!
//! # Usage
//! ```bash
//! cargo run --example aobscan_test --features "memory_aobscan"
//! ```
//!
//! Make sure your target process is running before executing this example.
//! Replace "target.exe" with your actual process name.

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AOB Scan Test ===\n");

    // Step 1: Initialize process connection
    println!("[1/4] Initializing process connection...");
    
    // TODO: Replace "target.exe" with your actual process name
    let process_name = "target.exe";
    
    let process = Process::builder(process_name)
        .build();
    
    match process.init() {
        Ok(()) => {
            println!("✓ Successfully connected to {}", process_name);
            println!("  PID: {}", process.get_pid());
            println!("  Handle: {:?}", process.get_handle());
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize process: {}", e);
            eprintln!("  Please make sure {} is running!", process_name);
            return Err(Box::new(e));
        }
    }

    // Step 2: Prepare the byte pattern
    println!("\n[2/4] Preparing byte pattern...");
    
    // TODO: Replace with your actual pattern
    // Pattern format: space-separated hex bytes, use ?? or ? for wildcards
    let pattern_str = "48 89 5C 24 ?? 48 89 6C 24 ??";
    println!("  Pattern: {}", pattern_str);
    
    // Note: The pattern string needs spaces between each byte/group
    // Example pattern with wildcards for flexibility
    let normalized_pattern = pattern_str;
    println!("  Normalized: {}", normalized_pattern);

    // Step 3: Execute AOB scan
    println!("\n[3/4] Executing AOB scan...");
    println!("  Search range: Full process memory");
    println!("  This may take a few seconds...\n");
    
    let start_time = std::time::Instant::now();
    
    let results = AobScanBuilder::new(process.get_handle())
        .pattern_str(normalized_pattern)?
        .find_all(true)       // Find all occurrences
        .scan()?;
    
    let elapsed = start_time.elapsed();
    
    // Step 4: Display results
    println!("\n[4/4] Scan Results:");
    println!("  Time elapsed: {:?}", elapsed);
    println!("  Matches found: {}", results.len());
    
    if results.is_empty() {
        println!("\n⚠ No matches found!");
        println!("\nPossible reasons:");
        println!("  1. The pattern doesn't exist in the target process's memory");
        println!("  2. The pattern format might be incorrect");
        println!("  3. The target process might have different memory layout");
        println!("\nSuggestions:");
        println!("  - Verify the pattern using Cheat Engine or similar tools");
        println!("  - Try searching for a shorter pattern first");
        println!("  - Check if the process has ASLR enabled (affects static addresses)");
    } else {
        println!("\n✓ Found {} matching address(es):", results.len());
        for (i, addr) in results.iter().enumerate() {
            println!("  [{}] 0x{:08X}", i + 1, addr);
        }
        
        // Optional: Verify the first match by reading memory
        if let Some(&first_addr) = results.first() {
            println!("\nVerifying first match at 0x{:08X}...", first_addr);
            
            // Calculate pattern length (excluding wildcards)
            let pattern_len = normalized_pattern.split_whitespace()
                .filter(|&s| s != "??" && s != "?")
                .count();
            
            match win_auto_utils::memory::read_memory_bytes(process.get_handle(), first_addr, pattern_len.max(16)) {
                Ok(bytes) => {
                    println!("  Read bytes: {:02X?}", bytes);
                    println!("  ✓ Verification successful - memory readable!");
                }
                Err(e) => {
                    println!("  ✗ Verification failed: {}", e);
                }
            }
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
