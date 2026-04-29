//! Template matching example - Layered Architecture Demonstration
//!
//! This example demonstrates the **layered performance architecture** of the
//! template matching module. Each layer adds convenience at the cost of performance.
//!
//! # Usage
//! ```bash
//! cargo run --example template_matching --features "template_matcher"
//! ```
//!
//! # Architecture Layers
//! ```text
//! Layer 4: File Path (most convenient, slowest)
//!     ↓
//! Layer 3: Image Bytes (network/memory)
//!     ↓
//! Layer 2: DynamicImage (flexible)
//!     ↓
//! Layer 1: Raw Pixel Data (fastest, zero overhead)
//! ```

use win_auto_utils::template_matcher::{
    match_region_from_gray,
    match_region_from_dynamic,
    match_region_from_bytes,
    match_region_from_path,
    MatchResult,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Template Matching - Layered Architecture Demo ===\n");

    // Prepare a test template
    let template_path = "test_template.png";
    println!("Using template: {}\n", template_path);

    // ========================================================================
    // Layer 1: Raw Pixel Data (Maximum Performance)
    // ========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Layer 1: GrayImage - MAXIMUM PERFORMANCE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use case: Hot loops, real-time matching, pre-converted templates");
    println!("Overhead: ZERO (direct pixel access)\n");

    match demonstrate_layer_1(template_path) {
        Ok(result) => print_result("Layer 1", &result),
        Err(e) => println!("Layer 1 failed: {}\n", e),
    }

    // ========================================================================
    // Layer 2: DynamicImage (Flexible)
    // ========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Layer 2: DynamicImage - FLEXIBLE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use case: Mixed image types, one-time conversion acceptable");
    println!("Overhead: One-time grayscale conversion\n");

    match demonstrate_layer_2(template_path) {
        Ok(result) => print_result("Layer 2", &result),
        Err(e) => println!("Layer 2 failed: {}\n", e),
    }

    // ========================================================================
    // Layer 3: Image Bytes (Network/Memory)
    // ========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Layer 3: Image Bytes - NETWORK/MEMORY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use case: Embedded resources, network downloads, memory buffers");
    println!("Overhead: Decode + convert\n");

    match demonstrate_layer_3(template_path) {
        Ok(result) => print_result("Layer 3", &result),
        Err(e) => println!("Layer 3 failed: {}\n", e),
    }

    // ========================================================================
    // Layer 4: File Path (Most Convenient)
    // ========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Layer 4: File Path - MOST CONVENIENT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use case: Quick prototyping, simple scripts");
    println!("Overhead: File I/O + decode + convert\n");

    match demonstrate_layer_4(template_path) {
        Ok(result) => print_result("Layer 4", &result),
        Err(e) => println!("Layer 4 failed: {}\n", e),
    }

    // ========================================================================
    // Performance Comparison & Best Practices
    // ========================================================================
    println!("\n=== Performance Comparison ===\n");
    println!("| Layer | Type           | Overhead          | Use Case              |");
    println!("|-------|----------------|-------------------|-----------------------|");
    println!("| 1     | GrayImage      | None (zero-copy)  | Hot loops, real-time  |");
    println!("| 2     | DynamicImage   | One-time convert  | Mixed formats         |");
    println!("| 3     | Bytes          | Decode + convert  | Network/embedded      |");
    println!("| 4     | Path           | I/O + decode      | Prototyping           |");
    println!();

    println!("=== Why Grayscale Only? ===\n");
    println!("The underlying imageproc algorithm only supports grayscale images.");
    println!("This is actually optimal because:");
    println!("✅ ~3x faster than RGB matching");
    println!("✅ More robust to lighting variations");
    println!("✅ Smaller memory footprint (1 byte/pixel vs 3 bytes/pixel)");
    println!("✅ UI elements are distinguished by shape/contrast, not color");
    println!();
    println!("For color-sensitive scenarios:");
    println!("1. Pre-filter by color before template matching");
    println!("2. Use multiple grayscale templates for different states");
    println!("3. Post-process match results with color verification");
    println!();

    println!("=== Best Practices ===\n");
    println!("✅ DO: Pre-convert templates outside hot loops (Layer 1)");
    println!("✅ DO: Minimize search region to reduce DXGI capture time");
    println!("❌ DON'T: Load from file path in performance-critical code");
    println!("❌ DON'T: Convert the same template repeatedly");
    println!();

    println!("=== User-Managed Caching Pattern ===\n");
    println!("For production use, implement your own cache:");
    println!();
    println!("struct TemplateCache {{");
    println!("    templates: HashMap<String, GrayImage>,");
    println!("}}");
    println!();
    println!("impl TemplateCache {{");
    println!("    fn load(&mut self, name: &str, path: &str) {{");
    println!("        let img = image::open(path).unwrap().to_luma8();");
    println!("        self.templates.insert(name.to_string(), img);");
    println!("    }}");
    println!();
    println!("    fn match_region(&self, x, y, w, h, name, threshold) {{");
    println!("        let template = self.templates.get(name).unwrap();");
    println!("        match_region_from_gray(x, y, w, h, template, threshold)");
    println!("    }}");
    println!("}}");

    Ok(())
}

/// Demonstrate Layer 1: GrayImage (Maximum Performance)
fn demonstrate_layer_1(template_path: &str) -> Result<MatchResult, String> {
    println!("Step 1: Load and convert template ONCE (outside hot loop)");
    let start = std::time::Instant::now();
    
    let template = image::open(template_path)
        .map_err(|e| format!("Failed to load template: {}", e))?
        .to_luma8(); // Convert to grayscale
    
    println!("  Conversion time: {:?}", start.elapsed());
    println!("  Template size: {}x{} ({} bytes)", 
             template.width(), template.height(),
             template.width() * template.height());
    
    println!("\nStep 2: Match repeatedly with ZERO conversion overhead");
    let start = std::time::Instant::now();
    
    let result = match_region_from_gray(
        0, 0,
        1920, 1080,
        &template,
        0.85
    )?;
    
    println!("  Match time: {:?}", start.elapsed());
    
    Ok(result)
}

/// Demonstrate Layer 2: DynamicImage (Flexible)
fn demonstrate_layer_2(template_path: &str) -> Result<MatchResult, String> {
    println!("Loading as DynamicImage (auto-detects format)");
    
    let img = image::open(template_path)
        .map_err(|e| format!("Failed to load template: {}", e))?;
    
    println!("  Image type: {:?}", img.color());
    println!("  Size: {}x{}", img.width(), img.height());
    
    println!("\nConverting internally to grayscale...");
    let start = std::time::Instant::now();
    
    let result = match_region_from_dynamic(
        0, 0,
        1920, 1080,
        &img,
        0.85
    )?;
    
    println!("  Conversion + match time: {:?}", start.elapsed());
    
    Ok(result)
}

/// Demonstrate Layer 3: Image Bytes (Network/Memory)
fn demonstrate_layer_3(template_path: &str) -> Result<MatchResult, String> {
    println!("Reading file to memory buffer (simulating network download)");
    
    let image_data = std::fs::read(template_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    println!("  Buffer size: {} bytes", image_data.len());
    
    println!("\nDecoding from bytes and matching...");
    let start = std::time::Instant::now();
    
    let result = match_region_from_bytes(
        0, 0,
        1920, 1080,
        &image_data,
        0.85
    )?;
    
    println!("  Decode + match time: {:?}", start.elapsed());
    
    Ok(result)
}

/// Demonstrate Layer 4: File Path (Most Convenient)
fn demonstrate_layer_4(template_path: &str) -> Result<MatchResult, String> {
    println!("One-liner: load from path and match");
    
    let start = std::time::Instant::now();
    
    let result = match_region_from_path(
        0, 0,
        1920, 1080,
        template_path,
        0.85
    )?;
    
    println!("  Total time (I/O + decode + match): {:?}", start.elapsed());
    
    Ok(result)
}

fn print_result(layer_name: &str, result: &MatchResult) {
    println!("\n[{}] Match Result:", layer_name);
    println!("  Similarity: {:.4}", result.similarity);
    println!("  Matched: {}", result.matched);
    if result.matched {
        println!("  Position (center): ({}, {})", result.x, result.y);
    } else {
        println!("  (No match found above threshold)");
    }
    println!();
}
