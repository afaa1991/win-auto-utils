//! Ultra-fast DXGI region capture example
//!
//! Demonstrates the optimized region capture functionality.
//!
//! # Running this example
//! ```bash
//! cargo run --example dxgi_capture --features dxgi
//! ```

use win_auto_utils::dxgi::{capture_region_bytes, get_screen_size};
use win_auto_utils::DxgiError;

fn main() {
    println!("=== Ultra-Fast DXGI Region Capture ===\n");

    // ========================================================================
    // 1. Get Screen Dimensions
    // ========================================================================
    println!("1. Screen Information");
    println!("---------------------");

    match get_screen_size() {
        Ok((width, height)) => {
            println!("✓ Screen resolution: {}x{}", width, height);
            println!("  Total pixels: {}", width * height);
        }
        Err(e) => {
            println!("✗ Failed to get screen size: {}", e);
            return;
        }
    }

    println!();

    // ========================================================================
    // 2. Ultra-Fast Region Capture
    // ========================================================================
    println!("2. Fast Region Capture - 200x150 at (100, 100)");
    println!("------------------------------------------------");

    let start = std::time::Instant::now();

    match capture_region_bytes(100, 100, 200, 150) {
        Ok(bytes) => {
            let elapsed = start.elapsed();

            println!("✓ Successfully captured region");
            println!("  Region: 200x150 pixels");
            println!(
                "  Data size: {} bytes ({} pixels)",
                bytes.len(),
                bytes.len() / 4
            );
            println!("  Capture time: {:?}", elapsed);
            println!(
                "  Throughput: {:.2} MB/s",
                (bytes.len() as f64 / 1_000_000.0) / elapsed.as_secs_f64()
            );

            // Verify data integrity
            let expected_size = 200 * 150 * 4;
            if bytes.len() == expected_size {
                println!("  ✓ Data size correct");
            } else {
                println!(
                    "  ✗ Data size mismatch! Expected {}, got {}",
                    expected_size,
                    bytes.len()
                );
            }

            // Show sample pixel data
            if bytes.len() >= 4 {
                println!("\n  Sample pixel at (0, 0):");
                println!(
                    "    B={}, G={}, R={}, A={}",
                    bytes[0], bytes[1], bytes[2], bytes[3]
                );
            }

            // Show center pixel
            let center_x = 100;
            let center_y = 75;
            let offset = (center_y * 200 + center_x) * 4;
            if offset + 3 < bytes.len() {
                println!("  Sample pixel at center (100, 75):");
                println!(
                    "    B={}, G={}, R={}, A={}",
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3]
                );
            }
        }
        Err(e) => {
            println!("✗ Capture failed: {}", e);
            print_error_help(&e);
        }
    }

    println!();

    // ========================================================================
    // 3. Performance Benchmark
    // ========================================================================
    println!("3. Performance Benchmark - 100 captures");
    println!("----------------------------------------");

    let iterations = 100;
    let mut total_time = std::time::Duration::ZERO;
    let mut success_count = 0;

    for i in 0..iterations {
        let start = std::time::Instant::now();

        match capture_region_bytes(50, 50, 100, 100) {
            Ok(_) => {
                total_time += start.elapsed();
                success_count += 1;
            }
            Err(_) => {
                // Skip failed captures
            }
        }

        // Progress indicator every 10 iterations
        if (i + 1) % 10 == 0 {
            print!(".");
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    println!("\n\nBenchmark Results:");
    if success_count > 0 {
        let avg_time = total_time / success_count as u32;
        let fps = 1.0 / avg_time.as_secs_f64();

        println!("  Successful captures: {}/{}", success_count, iterations);
        println!("  Average time: {:?}", avg_time);
        println!("  Estimated FPS: {:.1}", fps);
        println!(
            "  Data rate: {:.2} MB/s",
            (100 * 100 * 4 * success_count) as f64 / 1_000_000.0 / total_time.as_secs_f64()
        );
    } else {
        println!("  All captures failed!");
    }

    println!();

    // ========================================================================
    // 4. Error Handling Examples
    // ========================================================================
    println!("4. Error Handling");
    println!("-----------------");

    // Test invalid dimensions
    println!("Testing zero width:");
    match capture_region_bytes(0, 0, 0, 100) {
        Err(DxgiError::InvalidRegionDimensions) => {
            println!("  ✓ Correctly rejected zero width");
        }
        Err(e) => {
            println!("  Got error: {}", e);
        }
        Ok(_) => {
            println!("  ✗ Should have failed");
        }
    }

    println!("\nTesting negative height:");
    match capture_region_bytes(0, 0, 100, -50) {
        Err(DxgiError::InvalidRegionDimensions) => {
            println!("  ✓ Correctly rejected negative height");
        }
        Err(e) => {
            println!("  Got error: {}", e);
        }
        Ok(_) => {
            println!("  ✗ Should have failed");
        }
    }

    println!("\n=== Example Complete ===");
}

/// Helper function for error troubleshooting
fn print_error_help(error: &DxgiError) {
    println!("\n  Troubleshooting:");
    match error {
        DxgiError::InitializationFailed(msg) => {
            println!("  - Ensure Windows 10/11 with DirectX support");
            println!("  - Try running as administrator");
            println!("  - Details: {}", msg);
        }
        DxgiError::CaptureFailed(msg) => {
            println!("  - Display may be in protected mode (DRM content)");
            println!("  - Another app might be using exclusive fullscreen");
            println!("  - Details: {}", msg);
        }
        DxgiError::RegionOutOfBounds => {
            println!("  - Region extends beyond screen boundaries");
            println!("  - Check coordinates and dimensions");
        }
        DxgiError::InvalidRegionDimensions => {
            println!("  - Width and height must be positive (> 0)");
        }
        DxgiError::LockFailed => {
            println!("  - Internal synchronization error (rare)");
        }
    }
}
