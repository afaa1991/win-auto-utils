# DXGI Screen Capture

[中文文档](../../zh/modules/dxgi.md) | [Back to Overview](overview.md)

The `dxgi` module provides ultra-high-performance screen capture using Windows Desktop Duplication API (DXGI). It captures frames directly from the GPU with minimal CPU overhead, making it ideal for real-time applications like game bots, streaming, and desktop automation.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.2.6", features = ["dxgi"] }
```

**Platform**: Windows only (requires DirectX 11+)

## Quick Start

### Basic Screen Capture

```rust
use win_auto_utils::dxgi::{capture_region_bytes, get_screen_size};

// Get screen dimensions
let (width, height) = get_screen_size()?;
println!("Screen resolution: {}x{}", width, height);

// Capture full screen (using large region)
let bytes = capture_region_bytes(0, 0, width as i32, height as i32)?;
println!("Data length: {} bytes", bytes.len());

// Frame data is in BGRA format (4 bytes per pixel)
```

### Capture Specific Region

```rust
use win_auto_utils::dxgi::capture_region_bytes;

// Capture a 200x100 region at position (100, 50)
let bytes = capture_region_bytes(100, 50, 200, 100)?;
println!("Region captured: {} bytes", bytes.len());
```

## Key Features

- **GPU-Accelerated**: Captures directly from GPU memory
- **High Performance**: 30-60+ FPS on modern hardware
- **Low Latency**: Sub-millisecond capture time
- **Zero Copy**: Direct memory mapping without intermediate buffers
- **Region Support**: Capture specific screen regions efficiently
- **Auto-Recovery**: Handles display mode changes gracefully

## Usage Examples

### Example 1: Continuous Frame Capture

```rust
use win_auto_utils::dxgi::capture_region_bytes;
use std::time::Instant;

// Capture 100 frames and measure performance
let start = Instant::now();

for i in 0..100 {
    let bytes = capture_region_bytes(0, 0, 1920, 1080)?;

    if i % 10 == 0 {
        println!("Frame {}: {} bytes", i, bytes.len());
    }
}

let elapsed = start.elapsed();
let fps = 100.0 / elapsed.as_secs_f64();
println!("Average FPS: {:.2}", fps);
```

### Example 2: Region-Based Monitoring

```rust
use win_auto_utils::dxgi::capture_region_bytes;

// Monitor a specific UI element (e.g., health bar)
let health_bar_x = 100;
let health_bar_y = 50;
let health_bar_width = 200;
let health_bar_height = 20;

loop {
    let bytes = capture_region_bytes(
        health_bar_x,
        health_bar_y,
        health_bar_width,
        health_bar_height,
    )?;

    // Process region data (e.g., color analysis)

    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
}
```

### Example 3: Error Handling and Recovery

```rust
use win_auto_utils::dxgi::{capture_region_bytes, DxgiError};

fn capture_with_recovery() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match capture_region_bytes(0, 0, 1920, 1080) {
            Ok(bytes) => {
                process_bytes(&bytes);
            }
            Err(DxgiError::CaptureFailed(msg)) => {
                eprintln!("Capture failed: {}", msg);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }
}
```

### Example 4: Multi-Monitor Support

```rust
use win_auto_utils::dxgi::get_screen_size;

// Get primary screen dimensions
let (width, height) = get_screen_size()?;

println!("Primary monitor: {}x{}", width, height);

// To capture specific monitor, use lower-level API
```

### Example 5: Frame Processing Pipeline

```rust
use win_auto_utils::dxgi::capture_region_bytes;

// Capture and convert to RGB
let bgra_data = capture_region_bytes(0, 0, 1920, 1080)?;

// Convert BGRA to RGB (skip alpha channel)
let mut rgb_data = Vec::with_capacity(bgra_data.len() * 3 / 4);
for chunk in bgra_data.chunks(4) {
    rgb_data.push(chunk[2]); // R
    rgb_data.push(chunk[1]); // G
    rgb_data.push(chunk[0]); // B
}

// Now rgb_data contains RGB pixels
println!("RGB data size: {} bytes", rgb_data.len());
```

### Example 6: Performance Benchmarking

```rust
use win_auto_utils::dxgi::capture_region_bytes;
use std::time::Instant;

// Warm-up
for _ in 0..10 {
    let _ = capture_region_bytes(0, 0, 100, 100)?;
}

// Benchmark
let iterations = 100;
let start = Instant::now();

for _ in 0..iterations {
    let _ = capture_region_bytes(0, 0, 100, 100)?;
}

let elapsed = start.elapsed();
let avg_time = elapsed / iterations as u32;
let fps = iterations as f64 / elapsed.as_secs_f64();

println!("Average capture time: {:?}", avg_time);
println!("Achieved FPS: {:.2}", fps);
```

## API Reference

### Main Functions

DXGI module provides direct functional API:

#### Capture Functions

- **`capture_region_bytes(x, y, width, height) -> Result<Vec<u8>, DxgiError>`**
  - Captures a screenshot of the specified region
  - Returns BGRA format byte data
  - x, y are starting coordinates, width, height are region dimensions

- **`get_screen_size() -> Result<(i32, i32), DxgiError>`**
  - Gets primary screen dimensions
  - Returns (width, height) tuple

### Main Types

#### DxgiError

Error type for DXGI operations.

**Variants**:
- `CaptureFailed(String)` - Capture operation failed
- `RegionOutOfBounds` - Requested region is outside screen bounds
- `InvalidRegionDimensions` - Invalid width or height (zero or negative)
- `InitializationFailed(String)` - Failed to initialize DXGI
- `LockFailed` - Failed to acquire lock

## Performance Characteristics

### Capture Time by Resolution

| Resolution | Typical Time | Max FPS |
|------------|--------------|---------|
| 1920x1080 | 1-3ms | 300-1000 |
| 2560x1440 | 2-5ms | 200-500 |
| 3840x2160 | 5-10ms | 100-200 |
| Region (200x100) | <1ms | 1000+ |

*Measured on Intel i7-10700K with NVIDIA RTX 3070*

### Memory Usage

- **Full HD (1920x1080)**: ~8 MB per frame (BGRA)
- **QHD (2560x1440)**: ~14 MB per frame
- **4K (3840x2160)**: ~32 MB per frame
- **Region (200x100)**: ~80 KB

### CPU Overhead

- **Idle**: <1% CPU usage
- **Active capture**: 2-5% CPU (single core)
- **GPU usage**: Minimal (hardware-accelerated)

## Best Practices

1. **Use Capture Functions Directly**
   ```rust
   // Good: Call functions directly
   let bytes = capture_region_bytes(100, 50, 200, 100)?;

   // Bad: Reinitializing every time
   let mut capture = DxgiCapture::new()?;
   ```

2. **Capture Only Needed Regions**
   ```rust
   // Good: Small region
   let bytes = capture_region_bytes(100, 50, 200, 100)?;

   // Bad: Full screen when only small area needed
   let bytes = capture_region_bytes(0, 0, 1920, 1080)?; // Waste of resources
   ```

3. **Handle Errors Gracefully**
   ```rust
   match capture_region_bytes(0, 0, 1920, 1080) {
       Ok(bytes) => process_bytes(&bytes),
       Err(DxgiError::CaptureFailed(_)) => {
           // Retry on access loss
           std::thread::sleep(std::time::Duration::from_secs(1));
       }
       Err(e) => return Err(e),
   }
   ```

4. **Use Appropriate Frame Rate**
   ```rust
   // For UI monitoring: 10-30 FPS is sufficient
   std::thread::sleep(Duration::from_millis(33)); // 30 FPS

   // For gaming: 60+ FPS may be needed
   std::thread::sleep(Duration::from_millis(16)); // 60 FPS
   ```

5. **Process Frames Efficiently**
   ```rust
   // Avoid copying large vectors
   let bytes = capture_region_bytes(0, 0, 1920, 1080)?;
   process_in_place(&bytes);  // Borrow instead of clone
   ```

## Common Pitfalls

### ❌ Not Handling Access Lost

```rust
// Wrong: May fail on display mode change
let bytes = capture_region_bytes(0, 0, 1920, 1080)?;

// Correct: Handle and retry
loop {
    match capture_region_bytes(0, 0, 1920, 1080) {
        Ok(bytes) => break,
        Err(DxgiError::CaptureFailed(_)) => {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Err(e) => return Err(e),
    }
}
```

### ❌ Capturing Out-of-Bounds Regions

```rust
// Wrong: May panic or return error
let bytes = capture_region_bytes(10000, 10000, 200, 100)?;

// Correct: Check bounds first
let (width, height) = get_screen_size()?;
if x + w <= width && y + h <= height {
    let bytes = capture_region_bytes(x, y, w, h)?;
}
```

### ❌ Blocking the Capture Thread

```rust
// Wrong: Heavy processing blocks next capture
let bytes = capture_region_bytes(0, 0, 1920, 1080)?;
heavy_image_processing(&bytes);  // Takes 100ms - drops FPS

// Correct: Use separate thread or queue
let bytes = capture_region_bytes(0, 0, 1920, 1080)?;
tx.send(bytes)?;  // Send to worker thread
```

## Platform Requirements

- **OS**: Windows 8 or later
- **DirectX**: DirectX 11+
- **GPU**: Any DirectX 11-compatible GPU
- **Permissions**: No special permissions required

### Limitations

- ❌ Does not work on Windows 7 or earlier
- ❌ Cannot capture protected content (DRM videos)
- ❌ May fail during UAC prompts or secure desktop
- ⚠️ Performance varies by GPU driver quality

## Related Modules

- [`template_matcher`](template_matcher.md): Find images in captured frames
- [`color_finder`](../utils.md): Analyze colors in captured regions
- [`process_window`](process_window.md): Get window coordinates for targeted capture
- [`memory_hook`](memory_hook.md): Combine with memory reading for complete automation

---

**Language**: [English](dxgi.md) | [中文](../../zh/modules/dxgi.md)
