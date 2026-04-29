# Template Matcher (Image Template Matching)

[中文文档](../../zh/modules/template_matcher.md) | [Back to Overview](overview.md)

The `template_matcher` module provides high-performance image template matching using DXGI screen capture and parallel image processing. It features a four-layer API design for different performance/convenience trade-offs, supporting grayscale and RGB matching with normalized cross-correlation algorithm.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["template_matcher"] }
```

**Platform**: Windows only (requires DXGI support)

## Quick Start

### Layer 1: Raw Pixel Data (Maximum Performance)

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// Pre-load and convert template once (outside hot loop)
let template = image::open("button.png")?.to_luma8();

// Match repeatedly with zero conversion overhead
for _ in 0..100 {
    let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
    
    if let Some(match_info) = result {
        println!("Found at ({}, {}) with similarity {:.2}", 
                 match_info.x, match_info.y, match_info.similarity);
    }
}
```

### Layer 4: File Path (Most Convenient)

```rust
use win_auto_utils::template_matcher::match_region_from_path;

// One-liner for quick testing
let result = match_region_from_path(
    0, 0, 1920, 1080, 
    "button.png", 
    0.85
)?;

if let Some(info) = result {
    println!("Match found!");
}
```

## Key Features

- **Four-Layer API**: From maximum performance to maximum convenience
- **Parallel Processing**: Multi-threaded template matching via Rayon
- **Normalized Cross-Correlation**: Robust to brightness/contrast changes
- **Grayscale & RGB Support**: Choose based on use case
- **DXGI Integration**: High-performance screen capture
- **Flexible Input**: Accept pixels, images, bytes, or file paths
- **Similarity Scoring**: Returns match confidence (0.0 - 1.0)

## Usage Examples

### Example 1: UI Button Detection

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// Load button template
let button_template = image::open("submit_button.png")?.to_luma8();

// Search in specific region (faster than full screen)
let search_x = 800;
let search_y = 600;
let search_width = 300;
let search_height = 100;

let result = match_region_from_gray(
    search_x, search_y, 
    search_width, search_height,
    &button_template,
    0.90  // High threshold for precise match
)?;

if let Some(info) = result {
    println!("Button found at screen position ({}, {})", 
             info.x + search_x, info.y + search_y);
}
```

### Example 2: RGB Color-Sensitive Matching

```rust
use win_auto_utils::template_matcher::match_region_from_rgb;

// Use RGB when color matters (e.g., status indicators)
let health_bar = image::open("health_full.png")?.to_rgb8();

let result = match_region_from_rgb(
    100, 50, 200, 20,
    &health_bar,
    0.85
)?;

if result.is_some() {
    println!("Full health detected");
} else {
    println!("Health is not full");
}
```

### Example 3: Multiple Template Matching

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// Find which icon is present
let templates = vec![
    ("sword", image::open("sword_icon.png")?.to_luma8()),
    ("shield", image::open("shield_icon.png")?.to_luma8()),
    ("potion", image::open("potion_icon.png")?.to_luma8()),
];

for (name, template) in &templates {
    let result = match_region_from_gray(
        0, 0, 1920, 1080,
        template,
        0.80
    )?;
    
    if result.is_some() {
        println!("Found: {}", name);
        break;
    }
}
```

### Example 4: Dynamic Image Type (Layer 2)

```rust
use win_auto_utils::template_matcher::match_region_from_dynamic;

// Accepts any image type, converts internally
let img = image::open("icon.png")?;  // Could be PNG, JPG, BMP, etc.

let result = match_region_from_dynamic(
    0, 0, 1920, 1080,
    &img,
    0.85
)?;
```

### Example 5: Image from Memory/Network (Layer 3)

```rust
use win_auto_utils::template_matcher::match_region_from_bytes;

// Load from memory, network, or embedded resources
let png_data = std::fs::read("button.png")?;

let result = match_region_from_bytes(
    0, 0, 1920, 1080,
    &png_data,
    0.85
)?;
```

### Example 6: Adaptive Threshold Strategy

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

let template = image::open("ui_element.png")?.to_luma8();

// Try multiple thresholds for robustness
for threshold in [0.95, 0.90, 0.85, 0.80].iter() {
    let result = match_region_from_gray(
        0, 0, 1920, 1080,
        &template,
        *threshold
    )?;
    
    if let Some(info) = result {
        println!("Match at threshold {}: similarity {:.3}", 
                 threshold, info.similarity);
        break;
    }
}
```

## Examples

See these example files for usage demonstrations:

- `template_matching.rs`
- `template_matching_benchmark.rs`

Run examples:

```bash
cargo run --example template_matching --features "template_matcher"
cargo run --example template_matching_benchmark --features "template_matcher"
```

## API Reference

### Main Functions

#### Layer 1: Raw Pixel Data

- **`match_region_from_gray(x, y, w, h, template: &GrayImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - Match grayscale template against screen region
  - Zero conversion overhead (fastest)
  - Best for hot loops and real-time matching

- **`match_region_from_rgb(x, y, w, h, template: &RgbImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - Match RGB template against screen region
  - Use when color information is important

#### Layer 2: DynamicImage

- **`match_region_from_dynamic(x, y, w, h, template: &DynamicImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - Accepts any image type
  - Converts internally to appropriate format
  - Flexible but has conversion overhead

#### Layer 3: Image Bytes

- **`match_region_from_bytes(x, y, w, h, image_data: &[u8], threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - Decode image from raw bytes (PNG, JPG, etc.)
  - Useful for network/embedded resources
  - Includes decode + conversion overhead

#### Layer 4: File Path

- **`match_region_from_path(x, y, w, h, path: &str, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - Load image from file path
  - Most convenient for prototyping
  - Includes file I/O + decode + conversion overhead

### Types

#### MatchInfo

Information about a successful match.

**Fields**:
- `x: u32` - X coordinate of match (relative to search region)
- `y: u32` - Y coordinate of match (relative to search region)
- `similarity: f64` - Similarity score (0.0 - 1.0)
- `width: u32` - Template width
- `height: u32` - Template height

**Methods**:
- `screen_x(&self, region_x: u32) -> u32` - Get absolute screen X coordinate
- `screen_y(&self, region_y: u32) -> u32` - Get absolute screen Y coordinate

#### TemplateError

Error types for template matching operations.

**Variants**:
- `CaptureFailed(String)` - Screen capture failed
- `ImageDecodeError(String)` - Failed to decode image
- `InvalidThreshold(f64)` - Threshold out of range (must be 0.0-1.0)
- `TemplateTooLarge` - Template larger than search region
- `ProcessingError(String)` - Image processing error

## Algorithm Details

### Normalized Cross-Correlation

The module uses `CrossCorrelationNormalized` algorithm which:

- **Output Range**: [-1, 1] where 1.0 is perfect match
- **Brightness Invariant**: Unaffected by uniform brightness changes
- **Contrast Invariant**: Robust to contrast variations
- **Ideal For**: Real-world UI matching scenarios

### Parallel Processing

- **Rayon Integration**: Automatically parallelizes across CPU cores
- **Region Splitting**: Divides search space into chunks
- **Speedup**: 2-4x on quad-core CPUs, more on higher core counts

## Performance Characteristics

### Layer Comparison

| Layer | Overhead | Typical Time (1080p) | Use Case |
|-------|----------|----------------------|----------|
| **Layer 1** | None | 5-20ms | Hot loops, real-time |
| **Layer 2** | Conversion | 10-30ms | Mixed image types |
| **Layer 3** | Decode + Convert | 15-40ms | Network/embedded |
| **Layer 4** | I/O + Decode + Convert | 20-50ms | Prototyping |

*Measured on Intel i7-10700K with RTX 3070*

### Optimization Tips

1. **Use Grayscale When Possible**
   ```rust
   // Faster: Grayscale (1 byte/pixel)
   let template = img.to_luma8();
   
   // Slower: RGB (3 bytes/pixel)
   let template = img.to_rgb8();
   ```

2. **Limit Search Region**
   ```rust
   // Fast: Small region
   match_region_from_gray(100, 50, 200, 100, &template, 0.85)?;
   
   // Slow: Full screen
   match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   ```

3. **Pre-convert Templates**
   ```rust
   // Good: Convert once outside loop
   let template = image::open("btn.png")?.to_luma8();
   for _ in 0..100 {
       match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   }
   
   // Bad: Convert every iteration
   for _ in 0..100 {
       let template = image::open("btn.png")?.to_luma8();  // Slow!
       match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   }
   ```

4. **Choose Appropriate Threshold**
   ```rust
   // High precision: 0.90-0.95
   // Balanced: 0.80-0.90
   // Lenient: 0.70-0.80
   ```

## Best Practices

1. **Select Correct Layer for Use Case**
   ```rust
   // Real-time bot: Layer 1
   let template = preload_template();
   loop {
       match_region_from_gray(..., &template, 0.85)?;
   }
   
   // Quick test: Layer 4
   match_region_from_path(..., "icon.png", 0.85)?;
   ```

2. **Use Grayscale for UI Elements**
   ```rust
   // UI buttons, icons, text: Grayscale
   let ui_template = img.to_luma8();
   
   // Color-coded indicators: RGB
   let status_template = img.to_rgb8();
   ```

3. **Cache Templates**
   ```rust
   // Load templates once at startup
   struct TemplateCache {
       submit_button: GrayImage,
       cancel_button: GrayImage,
   }
   
   impl TemplateCache {
       fn new() -> Self {
           Self {
               submit_button: image::open("submit.png").unwrap().to_luma8(),
               cancel_button: image::open("cancel.png").unwrap().to_luma8(),
           }
       }
   }
   ```

4. **Handle No-Match Cases**
   ```rust
   match match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)? {
       Some(info) => handle_match(info),
       None => handle_no_match(),
   }
   ```

## Common Pitfalls

### ❌ Using Too Low Threshold

```rust
// Wrong: Too many false positives
let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.50)?;

// Correct: Appropriate threshold
let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
```

### ❌ Not Limiting Search Region

```rust
// Wrong: Searches entire screen (slow)
match_region_from_path(0, 0, 1920, 1080, "small_icon.png", 0.85)?;

// Correct: Limit to expected area
match_region_from_path(800, 600, 300, 100, "small_icon.png", 0.85)?;
```

### ❌ Converting Template in Loop

```rust
// Wrong: Converts every iteration
for _ in 0..100 {
    let template = image::open("btn.png")?.to_luma8();
    match_region_from_gray(..., &template, 0.85)?;
}

// Correct: Convert once
let template = image::open("btn.png")?.to_luma8();
for _ in 0..100 {
    match_region_from_gray(..., &template, 0.85)?;
}
```

## Image Type Selection Guide

### Grayscale (GrayImage)

**Best For**:
- UI elements (buttons, menus)
- Icons and symbols
- Text recognition
- Shapes and patterns

**Advantages**:
- Smallest memory footprint (1 byte/pixel)
- Fastest matching performance
- Robust to brightness variations
- Works well for most UI scenarios

```rust
let template: GrayImage = image::open("button.png")?.to_luma8();
```

### RGB (RgbImage)

**Best For**:
- Color-coded UI elements
- Game icons with color significance
- Status indicators (red/green/yellow)
- When color is the distinguishing feature

**Use When**:
- You need to distinguish by color
- Grayscale produces too many false positives
- Color is semantically important

```rust
let template: RgbImage = image::open("status_icon.png")?.to_rgb8();
```

## Related Modules

- [`dxgi`](dxgi.md): Screen capture backend
- [`script_engine`](script_engine.md): Use template matching in scripts
- [`color_finder`](../utils.md): Alternative for simple color detection
- [`input`](input.md): Click at matched positions

---

**Language**: [English](template_matcher.md) | [中文](../../zh/modules/template_matcher.md)
