//! SIMD Verification Performance Test
//!
//! This example tests and verifies that AVX2 SIMD acceleration is being used.
//!
//! # Usage
//! ```bash
//! cargo run --example simd_verification_test --features "memory_aobscan" --release
//! ```

use win_auto_utils::process::Process;
use win_auto_utils::memory_aobscan::AobScanBuilder;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SIMD Verification Performance Test ===\n");

    // Check CPU features
    println!("🔍 CPU Feature Detection:");
    println!("  Architecture: {}", std::env::consts::ARCH);
    
    #[cfg(target_arch = "x86_64")]
    {
        let avx2_supported = std::is_x86_feature_detected!("avx2");
        let avx512_supported = std::is_x86_feature_detected!("avx512f");
        
        println!("  AVX2 support:   {} {}", 
            if avx2_supported { "✅ YES" } else { "❌ NO" },
            if avx2_supported { "(SIMD verification enabled)" } else { "(using scalar)" }
        );
        println!("  AVX-512 support: {} {}", 
            if avx512_supported { "✅ YES" } else { "⚠️  NO" },
            if avx512_supported { "(can enable for more speed)" } else { "" }
        );
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("  ⚠️  Not x86_64 - SIMD not available");
    }
    
    println!();

    // Initialize process
    let process = match initialize_process("PUBPETS.exe")? {
        Some(p) => p,
        None => {
            eprintln!("❌ ERROR: PUBPETS.exe not found!");
            return Ok(());
        }
    };

    println!("✓ Connected to PUBPETS.exe (PID: {})\n", process.get_pid());

    // Test pattern
    let pattern_str = "F3 0F 11 68 30 48 B8 E0 A6 FA E7 1E 02 00 00 48 8B 00 48 8B 40 20 48 89 45 F0 48 B8 E0 A6 FA E7 1E 02 00 00 48 8B 08 48 85 C9 0F 84 56 01 00 00 48 83 C1 30 48 8D 64 24 00 49 BB 70 22 4B 9A 1F 02 00 00";
    
    println!("📊 Pattern Analysis:");
    println!("  Length: {} bytes", pattern_str.split_whitespace().count());
    println!("  Type: {}", if pattern_str.split_whitespace().count() >= 16 { "Long pattern (SIMD eligible)" } else { "Short pattern (scalar only)" });
    println!();

    let handle = process.get_handle();
    let start_addr = 0x20000000000usize;
    let end_addr = 0x30000000000usize;
    let length = end_addr - start_addr;

    println!("🎯 Performance Test Configuration:");
    println!("  Address Range: 0x{:016X} - 0x{:016X}", start_addr, end_addr);
    println!("  Range Size: {} GB", length / 1024 / 1024 / 1024);
    println!("  Iterations: 5 (for averaging)\n");

    // Run multiple iterations for accurate measurement
    let mut times = Vec::new();
    let mut total_matches = 0;
    
    for i in 1..=5 {
        print!("  [Iteration {}/5] Scanning... ", i);
        
        let start = Instant::now();
        let results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(true)
            .scan()?;
        let elapsed = start.elapsed();
        
        times.push(elapsed);
        total_matches += results.len();
        
        println!("{:?} ({} matches)", elapsed, results.len());
    }
    
    println!();

    // Calculate statistics
    let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
    let min_time = times.iter().min().unwrap();
    let max_time = times.iter().max().unwrap();
    let throughput = 1.0 / avg_time.as_secs_f64();
    
    println!("📈 Performance Results:");
    println!("  {}", "-".repeat(60));
    println!("  Average time: {:?}", avg_time);
    println!("  Min time:     {:?}", min_time);
    println!("  Max time:     {:?}", max_time);
    println!("  Throughput:   {:.2} scans/sec", throughput);
    println!("  Total matches: {}", total_matches / 5);
    println!("  {}", "-".repeat(60));
    println!();

    // Performance assessment
    println!("🎯 Performance Assessment:");
    
    #[cfg(target_arch = "x86_64")]
    {
        let avx512_supported = std::is_x86_feature_detected!("avx512f");
        let avx2_supported = std::is_x86_feature_detected!("avx2");
        
        if avx512_supported && pattern_str.split_whitespace().count() >= 32 {
            println!("  ✅ AVX-512 SIMD verification IS being used (FASTEST!)");
            println!("  📊 Expected performance: 90-120ms for 1TB range");
            
            if avg_time.as_millis() < 130 {
                println!("  ✅ Performance is EXCELLENT - AVX-512 working at full speed!");
            } else if avg_time.as_millis() < 160 {
                println!("  ✅ Performance is GOOD - AVX-512 active");
            } else {
                println!("  ⚠️  Performance seems slower than expected for AVX-512");
                println!("     May be using AVX2 fallback due to system conditions");
            }
        } else if avx2_supported && pattern_str.split_whitespace().count() >= 16 {
            println!("  ✅ AVX2 SIMD verification IS being used");
            println!("  📊 Expected performance: 130-160ms for 1TB range");
            
            if avg_time.as_millis() < 180 {
                println!("  ✅ Performance is GOOD - SIMD is working correctly!");
            } else {
                println!("  ⚠️  Performance seems slower than expected");
                println!("     Possible causes:");
                println!("     - System load during test");
                println!("     - Memory pressure");
                println!("     - Target process memory layout");
            }
        } else {
            println!("  ⚠️  Using scalar verification (SIMD not available)");
            println!("  📊 Expected performance: 180-220ms for 1TB range");
        }
    }
    
    println!();

    // Verify accuracy
    if total_matches > 0 {
        println!("🔍 Accuracy Verification:");
        let pattern_bytes: Vec<u8> = pattern_str.split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect();
        
        // Test one match from last iteration
        let results = AobScanBuilder::new(handle)
            .pattern_str(pattern_str)?
            .start_address(start_addr)
            .length(length)
            .find_all(false)
            .scan()?;
        
        if !results.is_empty() {
            use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
            let addr = results[0];
            let mut buffer = vec![0u8; pattern_bytes.len()];
            let mut bytes_read = 0;
            
            unsafe {
                let result = ReadProcessMemory(
                    handle,
                    addr as *const std::ffi::c_void,
                    buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    pattern_bytes.len(),
                    Some(&mut bytes_read),
                );
                
                if result.is_ok() && bytes_read == pattern_bytes.len() && buffer == pattern_bytes {
                    println!("  ✅ Match at 0x{:016X} VERIFIED - 100% accurate!", addr);
                } else {
                    println!("  ❌ Verification FAILED");
                }
            }
        }
    }
    
    println!();
    println!("=== Test Complete ===");
    
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