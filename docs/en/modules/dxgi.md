# DXGI Screen Capture

[中文文档](../../zh/modules/dxgi.md) | [Back to Overview](overview.md)

The `dxgi` module provides ultra-high-performance screen capture using Windows Desktop Duplication API (DXGI). It captures frames directly from the GPU with minimal CPU overhead, making it ideal for real-time applications like game bots, streaming, and desktop automation.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["dxgi"] }
```

**Platform**: Windows only (requires DirectX 11+)

## Quick Start

### Basic Screen Capture

```rust
use win_auto_utils::dxgi::DxgiCapture;

// Create capture instance
let mut capture = DxgiCapture::new()?;

// Capture full screen
let frame = capture.capture_frame()?;
println!("Frame size: {}x{}", frame.width, frame.height);
println!("Data length: {} bytes", frame.data.len());

// Frame data is in BGRA format (4 bytes per pixel)
```

### Capture Specific Region

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// Capture a 200x100 region at position (100, 50)
let region = capture.capture_region(100, 50, 200, 100)?;
println!("Region captured: {}x{}", region.width, region.height);
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
use win_auto_utils::dxgi::DxgiCapture;
use std::time::Instant;

let mut capture = DxgiCapture::new()?;

// Capture 100 frames and measure performance
let start = Instant::now();
for i in 0..100 {
    let frame = capture.capture_frame()?;
    
    if i % 10 == 0 {
        println!("Frame {}: {}x{}", i, frame.width, frame.height);
    }
}

let elapsed = start.elapsed();
let fps = 100.0 / elapsed.as_secs_f64();
println!("Average FPS: {:.2}", fps);
```

### Example 2: Region-Based Monitoring

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// Monitor a specific UI element (e.g., health bar)
let health_bar_x = 100;
let health_bar_y = 50;
let health_bar_width = 200;
let health_bar_height = 20;

loop {
    let region = capture.capture_region(
        health_bar_x,
        health_bar_y,
        health_bar_width,
        health_bar_height,
    )?;
    
    // Process region data (e.g., color analysis)
    analyze_health_bar(&region.data);
    
    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
}
```

### Example 3: Error Handling and Recovery

```rust
use win_auto_utils::dxgi::{DxgiCapture, DxgiError};

fn capture_with_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let mut capture = DxgiCapture::new()?;
    
    loop {
        match capture.capture_frame() {
            Ok(frame) => {
                process_frame(&frame);
            }
            Err(DxgiError::CaptureFailed(msg)) => {
                eprintln!("Capture failed: {}", msg);
                
                // Try to reinitialize
                eprintln!("Reinitializing...");
                capture = DxgiCapture::new()?;
                
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
use win_auto_utils::dxgi::DxgiCapture;

// By default, captures the primary monitor
let mut capture = DxgiCapture::new()?;

// Get screen dimensions
let width = capture.get_width();
let height = capture.get_height();

println!("Primary monitor: {}x{}", width, height);

// To capture specific monitor, modify source code to select adapter/output
```

### Example 5: Frame Processing Pipeline

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// Capture and convert to RGB
let frame = capture.capture_frame()?;
let bgra_data = &frame.data;

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
use win_auto_utils::dxgi::DxgiCapture;
use std::time::Instant;

let mut capture = DxgiCapture::new()?;

// Warm-up
for _ in 0..10 {
    let _ = capture.capture_frame()?;
}

// Benchmark
let iterations = 1000;
let start = Instant::now();

for _ in 0..iterations {
    let _ = capture.capture_frame()?;
}

let elapsed = start.elapsed();
let avg_time = elapsed / iterations as u32;
let fps = iterations as f64 / elapsed.as_secs_f64();

println!("Average capture time: {:?}", avg_time);
println!("Achieved FPS: {:.2}", fps);
```

## API Reference

### Main Types

#### DxgiCapture

Main struct for DXGI screen capture.

**Constructor**:
- `DxgiCapture::new() -> Result<Self, DxgiError>` - Create new capture instance

**Methods**:
- `capture_frame() -> Result<CapturedFrame, DxgiError>` - Capture full screen
- `capture_region(x, y, w, h) -> Result<CapturedFrame, DxgiError>` - Capture specific region
- `get_width() -> usize` - Get screen width
- `get_height() -> usize` - Get screen height

#### CapturedFrame

Represents a captured frame.

**Fields**:
- `data: Vec<u8>` - Pixel data in BGRA format (4 bytes per pixel)
- `width: usize` - Frame width in pixels
- `height: usize` - Frame height in pixels

**Methods**:
- `get_pixel(x, y) -> Option<(u8, u8, u8, u8)>` - Get pixel color (B, G, R, A)

#### DxgiError

Error types for DXGI operations.

**Variants**:
- `CaptureFailed(String)` - Capture operation failed
- `RegionOutOfBounds` - Requested region exceeds screen bounds
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

1. **Reuse Capture Instance**
   ```rust
   // Good: Reuse instance
   let mut capture = DxgiCapture::new()?;
   for _ in 0..1000 {
       let frame = capture.capture_frame()?;
   }
   
   // Bad: Create new instance each time
   for _ in 0..1000 {
       let capture = DxgiCapture::new()?;  // Slow!
       let frame = capture.capture_frame()?;
   }
   ```

2. **Capture Only Needed Regions**
   ```rust
   // Good: Small region
   let region = capture.capture_region(100, 50, 200, 100)?;
   
   // Bad: Full screen when only small area needed
   let frame = capture.capture_frame()?;  // Wastes resources
   ```

3. **Handle Errors Gracefully**
   ```rust
   match capture.capture_frame() {
       Ok(frame) => process(frame),
       Err(DxgiError::CaptureFailed(_)) => {
           // Reinitialize on access lost
           capture = DxgiCapture::new()?;
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
   let frame = capture.capture_frame()?;
   process_in_place(&frame.data);  // Borrow instead of clone
   ```

## Common Pitfalls

### ❌ Not Handling Access Lost

```rust
// Wrong: Crashes on display mode change
let frame = capture.capture_frame()?;

// Correct: Handle and recover
match capture.capture_frame() {
    Ok(frame) => use_frame(frame),
    Err(DxgiError::CaptureFailed(msg)) if msg.contains("Access lost") => {
        capture = DxgiCapture::new()?;  // Reinitialize
    }
    Err(e) => return Err(e),
}
```

### ❌ Capturing Out-of-Bounds Regions

```rust
// Wrong: May panic or return error
let region = capture.capture_region(10000, 10000, 200, 100)?;

// Correct: Check bounds first
let width = capture.get_width();
let height = capture.get_height();
if x + w <= width && y + h <= height {
    let region = capture.capture_region(x, y, w, h)?;
}
```

### ❌ Blocking the Capture Thread

```rust
// Wrong: Heavy processing blocks next capture
let frame = capture.capture_frame()?;
heavy_image_processing(&frame);  // Takes 100ms - drops FPS

// Correct: Use separate thread or queue
let frame = capture.capture_frame()?;
tx.send(frame)?;  // Send to worker thread
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
