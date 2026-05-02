//! Color conversion examples
//!
//! Demonstrates color format conversions using the color_finder module.
//! Note: This example requires the color_finder feature which automatically enables dxgi.
//!
//! # Running this example
//! ```bash
//! cargo run --example color_conversion --features color_finder
//! ```

use win_auto_utils::color_finder::algorithms::find_color_in_buffer;

fn main() {
    println!("=== Color Finder Examples ===\n");

    // ========================================================================
    // 1. Basic Color Finding in Buffer
    // ========================================================================
    println!("1. Basic Color Finding in Pixel Buffer");
    println!("---------------------------------------");

    // Create a sample BGRA buffer (100x100 pixels)
    let width = 100i32;
    let height = 100i32;
    let mut buffer = vec![0u8; (width * height * 4) as usize];

    // Fill with blue background (BGRA: B=255, G=0, R=0, A=255)
    for i in 0..(width * height) {
        let offset = (i * 4) as usize;
        buffer[offset] = 255; // B
        buffer[offset + 1] = 0; // G
        buffer[offset + 2] = 0; // R
        buffer[offset + 3] = 255; // A
    }

    // Place a red pixel at position (50, 50) (BGRA: B=0, G=0, R=255, A=255)
    let target_x = 50;
    let target_y = 50;
    let target_idx = ((target_y * width + target_x) * 4) as usize;
    buffer[target_idx] = 0; // B
    buffer[target_idx + 1] = 0; // G
    buffer[target_idx + 2] = 255; // R
    buffer[target_idx + 3] = 255; // A

    println!("Created {}x{} pixel buffer", width, height);
    println!("Background: Blue (BGR: 255, 0, 0)");
    println!(
        "Target pixel at ({}, {}): Red (BGR: 0, 0, 255)",
        target_x, target_y
    );

    // Search for red color (B=0, G=0, R=255)
    let result = find_color_in_buffer(&buffer, width, height, (0, 0, 255));

    if result.matched {
        println!("\n✓ Color found!");
        println!("  Local coordinates: ({}, {})", result.x, result.y);
        assert_eq!(result.x, target_x, "X coordinate mismatch");
        assert_eq!(result.y, target_y, "Y coordinate mismatch");
        println!("  ✓ Coordinates match expected position");
    } else {
        println!("\n✗ Color not found!");
    }

    println!();

    // ========================================================================
    // 2. Multiple Color Search
    // ========================================================================
    println!("2. Multiple Color Search");
    println!("-------------------------");

    // Create a gradient buffer
    let grad_width = 50i32;
    let grad_height = 50i32;
    let mut grad_buffer = vec![0u8; (grad_width * grad_height * 4) as usize];

    // Create a gradient from black to white
    for y in 0..grad_height {
        for x in 0..grad_width {
            let offset = ((y * grad_width + x) * 4) as usize;
            let gray = ((x + y) * 255 / (grad_width + grad_height - 2)) as u8;
            grad_buffer[offset] = gray; // B
            grad_buffer[offset + 1] = gray; // G
            grad_buffer[offset + 2] = gray; // R
            grad_buffer[offset + 3] = 255; // A
        }
    }

    println!("Created {}x{} gradient buffer", grad_width, grad_height);

    // Search for different gray levels
    let test_colors = vec![
        ((0, 0, 0), "Black"),
        ((128, 128, 128), "Gray"),
        ((255, 255, 255), "White"),
    ];

    for (color, name) in test_colors {
        let result = find_color_in_buffer(&grad_buffer, grad_width, grad_height, color);
        if result.matched {
            println!("  ✓ {} found at ({}, {})", name, result.x, result.y);
        } else {
            println!("  ✗ {} not found", name);
        }
    }

    println!();

    // ========================================================================
    // 3. Performance Test
    // ========================================================================
    println!("3. Performance Test");
    println!("--------------------");

    let perf_width = 1920i32;
    let perf_height = 1080i32;
    let perf_buffer = vec![128u8; (perf_width * perf_height * 4) as usize];

    println!(
        "Testing with {}x{} buffer ({} MB)",
        perf_width,
        perf_height,
        perf_buffer.len() / 1024 / 1024
    );

    let iterations = 100;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = find_color_in_buffer(&perf_buffer, perf_width, perf_height, (128, 128, 128));
    }

    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations as u32;

    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", elapsed);
    println!("  Average time: {:?}", avg_time);
    println!(
        "  Throughput: {:.2} searches/sec",
        iterations as f64 / elapsed.as_secs_f64()
    );

    println!("\n=== Example Complete ===");
}
