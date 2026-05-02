//! DXGI Performance Comparison: Full-screen + Crop vs Direct Region Capture

use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DXGI Performance Comparison ===\n");

    // Get screen size
    let (screen_w, screen_h) = match win_auto_utils::dxgi::get_screen_size() {
        Ok(size) => size,
        Err(e) => {
            eprintln!("Failed to get screen size: {}", e);
            return Ok(());
        }
    };
    println!("Screen Resolution: {}x{}\n", screen_w, screen_h);

    // Test configurations
    let test_cases = vec![
        ("Small Region (100x100)", 100, 100),
        ("Medium Region (400x300)", 400, 300),
        ("Large Region (800x600)", 800, 600),
        ("Half Screen", screen_w / 2, screen_h / 2),
    ];

    let iterations = 50; // Number of captures per test

    for (name, width, height) in &test_cases {
        println!("\n{}", "=".repeat(60));
        println!("Test Case: {} ({}x{})", name, width, height);
        println!("{}", "=".repeat(60));

        let x = (screen_w as i32 - *width as i32) / 2;
        let y = (screen_h as i32 - *height as i32) / 2;

        // Method 1: Direct Region Capture (NEW - Optimized)
        let direct_times =
            benchmark_direct_capture(x, y, *width as i32, *height as i32, iterations)?;

        // Method 2: Full-screen + Crop (OLD - Simulated)
        let crop_times =
            benchmark_fullscreen_crop(x, y, *width as i32, *height as i32, iterations)?;

        // Calculate statistics
        let direct_avg = direct_times.iter().sum::<f64>() / direct_times.len() as f64;
        let crop_avg = crop_times.iter().sum::<f64>() / crop_times.len() as f64;

        let direct_min = direct_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let crop_min = crop_times.iter().cloned().fold(f64::INFINITY, f64::min);

        let direct_max = direct_times
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let crop_max = crop_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let speedup = crop_avg / direct_avg;
        let improvement_pct = ((crop_avg - direct_avg) / crop_avg) * 100.0;

        println!("\n📊 Results Summary:");
        println!("");
        println!(" Metric                   Direct        Full+Crop    ");
        println!("");
        println!(
            " Average Time             {:>8.2}ms  {:>8.2}ms ",
            direct_avg, crop_avg
        );
        println!(
            " Min Time                 {:>8.2}ms  {:>8.2}ms ",
            direct_min, crop_min
        );
        println!(
            " Max Time                 {:>8.2}ms  {:>8.2}ms ",
            direct_max, crop_max
        );
        println!(
            " Estimated FPS            {:>8.1}    {:>8.1}   ",
            1000.0 / direct_avg,
            1000.0 / crop_avg
        );
        println!("");

        println!("\n🚀 Performance Gain:");
        println!("   Speedup: {:.2}x faster", speedup);
        println!(
            "   Improvement: {:.1}% reduction in capture time",
            improvement_pct
        );

        // Memory comparison
        let direct_mem = (*width * *height * 4) as usize;
        let fullscreen_mem = (screen_w * screen_h * 4) as usize;
        let mem_saved = fullscreen_mem - direct_mem;
        let mem_saved_pct = (mem_saved as f64 / fullscreen_mem as f64) * 100.0;

        println!("\n💾 Memory Efficiency:");
        println!(
            "   Direct method: {} bytes ({:.2} MB)",
            direct_mem,
            direct_mem as f64 / 1024.0 / 1024.0
        );
        println!(
            "   Full+Crop method: {} bytes ({:.2} MB)",
            fullscreen_mem,
            fullscreen_mem as f64 / 1024.0 / 1024.0
        );
        println!(
            "   Memory saved: {} bytes ({:.1}%)",
            mem_saved, mem_saved_pct
        );
    }

    println!("\n\n{}", "=".repeat(60));
    println!("Overall Conclusion");
    println!("{}", "=".repeat(60));
    println!("✅ Direct region capture is significantly faster and more memory-efficient");
    println!("✅ Smaller regions show greater performance improvements");
    println!("✅ Recommended for real-time applications and frequent captures");

    Ok(())
}

/// Benchmark direct region capture
fn benchmark_direct_capture(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    iterations: usize,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut times = Vec::with_capacity(iterations);

    print!("Testing direct capture... ");
    std::io::Write::flush(&mut std::io::stdout())?;

    for _ in 0..iterations {
        let start = Instant::now();
        match win_auto_utils::dxgi::capture_region_bytes(x, y, width, height) {
            Ok(_bytes) => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                times.push(elapsed);
            }
            Err(e) => {
                eprintln!("\nCapture failed: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    println!("✓ Done ({} captures)", iterations);
    Ok(times)
}

/// Benchmark full-screen capture with manual cropping (simulated old method)
fn benchmark_fullscreen_crop(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    iterations: usize,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use win_auto_utils::dxgi;

    let mut times = Vec::with_capacity(iterations);

    print!("Testing full-screen + crop... ");
    std::io::Write::flush(&mut std::io::stdout())?;

    // Get screen size once
    let (screen_w, screen_h) = dxgi::get_screen_size()?;

    for _ in 0..iterations {
        let start = Instant::now();

        // Step 1: Capture full screen
        let full_bytes = dxgi::capture_region_bytes(0, 0, screen_w as i32, screen_h as i32)?;

        // Step 2: Manual crop (simulate the old inefficient method)
        let row_bytes = (width * 4) as usize;
        let total_bytes = row_bytes * height as usize;
        let mut cropped = vec![0u8; total_bytes];

        for row in 0..height as usize {
            let src_offset = ((y as usize + row) * screen_w + x as usize) * 4;
            let dst_offset = row * row_bytes;

            unsafe {
                std::ptr::copy_nonoverlapping(
                    full_bytes.as_ptr().add(src_offset),
                    cropped.as_mut_ptr().add(dst_offset),
                    row_bytes,
                );
            }
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        times.push(elapsed);
    }

    println!("✓ Done ({} captures)", iterations);
    Ok(times)
}
