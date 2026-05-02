//! Color finder module examples
//!
//! Demonstrates how to use the color_finder module for screen region color searching.
//! The color_finder feature automatically enables dxgi for screen capture.
//!
//! # Running this example
//! ```bash
//! cargo run --example color_reexports --features color_finder
//! ```

use win_auto_utils::color_finder::algorithms;

fn main() {
    println!("=== Color Finder Module Examples ===\n");

    // ========================================================================
    // 1. Using algorithms module (pure Rust, no screen capture)
    // ========================================================================
    println!("1. Pure Algorithm Module (No Screen Capture)");
    println!("----------------------------------------------");

    // Create a sample BGRA buffer
    let width = 100i32;
    let height = 100i32;
    let mut buffer = vec![0u8; (width * height * 4) as usize];

    // Fill with red background
    for i in 0..(width * height) {
        let offset = (i * 4) as usize;
        buffer[offset] = 0; // B
        buffer[offset + 1] = 0; // G
        buffer[offset + 2] = 255; // R
        buffer[offset + 3] = 255; // A
    }

    // Place a blue pixel at position (30, 40)
    let idx = ((40 * width + 30) * 4) as usize;
    buffer[idx] = 255; // B
    buffer[idx + 1] = 0; // G
    buffer[idx + 2] = 0; // R

    println!(
        "Created {}x{} pixel buffer with red background",
        width, height
    );
    println!("Placed blue pixel at (30, 40)");

    // Method 1: Direct access through algorithms module
    let result = algorithms::find_color_in_buffer(&buffer, width, height, (255, 0, 0));

    if result.matched {
        println!(
            "\n✓ Blue pixel found at local coordinates ({}, {})",
            result.x, result.y
        );
        assert_eq!(result.x, 30);
        assert_eq!(result.y, 40);
    } else {
        println!("\n✗ Blue pixel not found!");
    }

    println!();

    // ========================================================================
    // 2. Screen Region Color Finding (with DXGI - automatically enabled)
    // ========================================================================
    #[cfg(feature = "dxgi")]
    {
        println!("2. Screen Region Color Finding (DXGI Integration)");
        println!("---------------------------------------------------");

        println!("Method: color_finder::find_color");
        println!("  (Searches for color in a screen region using DXGI capture)");

        // Example usage (commented out to avoid actual screen capture during demo)
        /*
        match color_finder::find_color(100, 100, 50, 50, (255, 0, 0)) {
            Ok(result) => {
                if result.matched {
                    println!("  ✓ Found red at screen coordinates ({}, {})", result.x, result.y);
                } else {
                    println!("  ✗ Red not found in region (100, 100, 50, 50)");
                }
            }
            Err(e) => eprintln!("  Error: {}", e),
        }
        */

        println!("\n  Note: Uncomment the code above to test actual screen capture.");
        println!("  The dxgi feature is automatically enabled by color_finder.");

        println!("\n✓ Screen region color finding is available!");
    }

    #[cfg(not(feature = "dxgi"))]
    {
        println!("2. Screen Region Color Finding");
        println!("-------------------------------");
        println!("  ⚠ Feature 'dxgi' not enabled");
        println!("  Note: color_finder automatically enables dxgi, so this shouldn't happen.");
    }

    // ========================================================================
    // 3. Performance Comparison
    // ========================================================================
    println!("\n3. Performance Characteristics");
    println!("-------------------------------");

    println!("The color_finder module provides:");
    println!("  ✓ Pure Rust implementation (zero external dependencies)");
    println!("  ✓ Automatic AVX2 detection and optimization (~4-8x faster)");
    println!("  ✓ SWAR technique for remainder handling");
    println!("  ✓ Falls back to scalar on non-x86_64 or no AVX2");
    println!();
    println!("With dxgi feature (automatically enabled):");
    println!("  ✓ Ultra-fast screen region capture");
    println!("  ✓ Direct memory mapping (no full-screen copy)");
    println!("  ✓ Optimized for small region captures");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Summary ===");
    println!("✓ Algorithm functions available via:");
    println!("  - color_finder::algorithms::find_color_in_buffer()");
    println!();
    println!("✓ Screen capture functions available via:");
    println!("  - color_finder::find_color() (requires dxgi, auto-enabled)");
    println!();
    println!("✓ Key features:");
    println!("  - Zero-copy buffer processing");
    println!("  - SIMD acceleration when available");
    println!("  - Automatic coordinate conversion (local → screen)");
}
