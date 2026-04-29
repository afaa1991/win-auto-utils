//! Benchmark: Serial vs Parallel Template Matching
//!
//! This benchmark compares the performance of single-threaded vs multi-threaded
//! template matching to find the crossover point where parallel execution becomes faster.
//!
//! # Usage
//! ```bash
//! cargo bench --features "template_matcher" --bench template_matching_benchmark
//! ```
//!
//! Or run as a regular example for quick testing:
//! ```bash
//! cargo run --example template_matching_benchmark --features "template_matcher"
//! ```

use image::{GrayImage, Luma};
use imageproc::template_matching::{match_template, match_template_parallel, MatchTemplateMethod};
use std::time::Instant;

/// Benchmark result for a specific image size
#[derive(Debug, Clone)]
struct BenchmarkResult {
    screen_width: u32,
    screen_height: u32,
    #[allow(dead_code)]
    template_width: u32,
    #[allow(dead_code)]
    template_height: u32,
    total_pixels: u64,
    serial_time_ms: f64,
    parallel_time_ms: f64,
    speedup: f64, // serial / parallel
    winner: &'static str,
}

impl BenchmarkResult {
    fn new(
        screen_w: u32,
        screen_h: u32,
        tpl_w: u32,
        tpl_h: u32,
        serial_ms: f64,
        parallel_ms: f64,
    ) -> Self {
        let total_pixels = (screen_w * screen_h) as u64;
        let speedup = if parallel_ms > 0.0 {
            serial_ms / parallel_ms
        } else {
            0.0
        };
        
        let winner = if serial_ms < parallel_ms {
            "Serial"
        } else {
            "Parallel"
        };
        
        Self {
            screen_width: screen_w,
            screen_height: screen_h,
            template_width: tpl_w,
            template_height: tpl_h,
            total_pixels,
            serial_time_ms: serial_ms,
            parallel_time_ms: parallel_ms,
            speedup,
            winner,
        }
    }
}

/// Generate a grayscale image with gradient pattern for testing
/// Uses deterministic pattern to avoid external dependencies
fn generate_test_gray_image(width: u32, height: u32) -> GrayImage {
    let mut img = GrayImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Create a gradient pattern with some variation
        let value = ((x as u32 + y as u32 * 7) % 256) as u8;
        *pixel = Luma([value]);
    }
    img
}

/// Run benchmark for a specific configuration
fn run_benchmark(
    screen_width: u32,
    screen_height: u32,
    template_width: u32,
    template_height: u32,
    iterations: u32,
) -> BenchmarkResult {
    println!(
        "\nBenchmarking: {}x{} screen with {}x{} template ({:.2}M pixels)",
        screen_width,
        screen_height,
        template_width,
        template_height,
        (screen_width * screen_height) as f64 / 1_000_000.0
    );
    
    // Generate test images
    let screen = generate_test_gray_image(screen_width, screen_height);
    let template = generate_test_gray_image(template_width, template_height);
    
    // Warm-up (avoid cold start effects)
    let _ = match_template(&screen, &template, MatchTemplateMethod::CrossCorrelationNormalized);
    let _ = match_template_parallel(&screen, &template, MatchTemplateMethod::CrossCorrelationNormalized);
    
    // Benchmark serial version
    let serial_start = Instant::now();
    for _ in 0..iterations {
        let _ = match_template(&screen, &template, MatchTemplateMethod::CrossCorrelationNormalized);
    }
    let serial_elapsed = serial_start.elapsed();
    let serial_ms = serial_elapsed.as_secs_f64() / iterations as f64 * 1000.0;
    
    // Benchmark parallel version
    let parallel_start = Instant::now();
    for _ in 0..iterations {
        let _ = match_template_parallel(&screen, &template, MatchTemplateMethod::CrossCorrelationNormalized);
    }
    let parallel_elapsed = parallel_start.elapsed();
    let parallel_ms = parallel_elapsed.as_secs_f64() / iterations as f64 * 1000.0;
    
    let result = BenchmarkResult::new(
        screen_width,
        screen_height,
        template_width,
        template_height,
        serial_ms,
        parallel_ms,
    );
    
    println!(
        "  Serial:   {:8.2} ms | Parallel: {:8.2} ms | Speedup: {:.2}x | Winner: {}",
        result.serial_time_ms,
        result.parallel_time_ms,
        result.speedup,
        result.winner
    );
    
    result
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Template Matching: Serial vs Parallel Performance Test   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Testing different screen sizes with FIXED template size (50x50)");
    println!("This simulates real-world UI automation scenarios.");
    println!();
    
    let mut results = Vec::new();
    
    // Test configurations: varying screen sizes with FIXED template size
    // This is more realistic for UI automation (buttons, icons are usually 20-100px)
    let test_configs = vec![
        // Very small screens (< 50K pixels) - Expected: Serial might win
        (100, 100),      // 10K pixels
        (160, 120),      // 19.2K pixels
        (200, 150),      // 30K pixels
        
        // Small screens (50K - 100K pixels) - Potential crossover zone
        (320, 240),      // 76.8K pixels
        (400, 300),      // 120K pixels
        (480, 360),      // 172.8K pixels
        
        // Medium screens (100K - 500K pixels)
        (640, 480),      // 307K pixels
        (800, 600),      // 480K pixels
        (1024, 768),     // 786K pixels
        
        // Large screens (> 500K pixels) - Expected: Parallel wins
        (1280, 720),     // 921K pixels (720p)
        (1920, 1080),    // 2.07M pixels (1080p)
    ];
    
    // Fixed template size (realistic UI element)
    let template_width = 50;
    let template_height = 50;
    
    let iterations = 5; // More iterations for better accuracy
    
    for (sw, sh) in test_configs {
        let result = run_benchmark(sw, sh, template_width, template_height, iterations);
        results.push(result);
    }
    
    // Print summary table
    println!("\n\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                        PERFORMANCE SUMMARY                                       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("{:<12} | {:<10} | {:<10} | {:<10} | {:<8} | {}", 
             "Pixels", "Serial(ms)", "Parallel(ms)", "Speedup", "Winner", "Resolution");
    println!("{:-<12}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<8}-+-{:-<20}", 
             "", "", "", "", "", "");
    
    for r in &results {
        println!(
            "{:<12.2}M | {:<10.2} | {:<10.2} | {:<10.2}x | {:<8} | {}x{}",
            r.total_pixels as f64 / 1_000_000.0,
            r.serial_time_ms,
            r.parallel_time_ms,
            r.speedup,
            r.winner,
            r.screen_width,
            r.screen_height
        );
    }
    
    // Find crossover point
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════════════════════════");
    println!("ANALYSIS & RECOMMENDATIONS");
    println!("═══════════════════════════════════════════════════════════════════════════════════════════════════");
    
    let crossover = results.iter()
        .find(|r| r.parallel_time_ms < r.serial_time_ms);
    
    if let Some(cross) = crossover {
        println!();
        println!("✅ Crossover Point Found:");
        println!("   - At approximately {:.2}M pixels", cross.total_pixels as f64 / 1_000_000.0);
        println!("   - Resolution: {}x{}", cross.screen_width, cross.screen_height);
        println!("   - Parallel becomes faster by {:.2}x", cross.speedup);
        println!();
        println!("📊 Recommendation:");
        println!("   - Use SERIAL matching for images < {:.0}K pixels", 
                 cross.total_pixels as f64 / 1_000.0);
        println!("   - Use PARALLEL matching for images > {:.0}K pixels", 
                 cross.total_pixels as f64 / 1_000.0);
    } else {
        println!();
        println!("⚠️  No clear crossover point found in tested range.");
        println!("   All tests favored one method over the other.");
    }
    
    // Find best speedup
    if let Some(best) = results.iter().max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap()) {
        println!();
        println!("🚀 Maximum Speedup:");
        println!("   - {:.2}x faster at {}x{} ({:.2}M pixels)",
                 best.speedup,
                 best.screen_width,
                 best.screen_height,
                 best.total_pixels as f64 / 1_000_000.0);
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════════════════════════");
    println!("IMPLEMENTATION SUGGESTION");
    println!("═══════════════════════════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("For production code, consider adaptive selection:");
    println!();
    println!("fn smart_match(image: &GrayImage, template: &GrayImage) {{");
    println!("    let pixels = (image.width() * image.height()) as u64;");
    println!("    ");
    println!("    if pixels < CROSSOVER_THRESHOLD {{");
    println!("        // Small image: use serial (lower overhead)");
    println!("        match_template(image, template, method)");
    println!("    }} else {{");
    println!("        // Large image: use parallel (better throughput)");
    println!("        match_template_parallel(image, template, method)");
    println!("    }}");
    println!("}}");
    println!();
}
