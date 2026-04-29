# DXGI 屏幕捕获 (DXGI Screen Capture)

[English](../../en/modules/dxgi.md) | [返回概览](overview.md)

`dxgi` 模块使用 Windows Desktop Duplication API (DXGI) 提供超高性能的屏幕捕获功能。它直接从 GPU 捕获帧，CPU 开销极小，非常适合游戏机器人、流媒体和桌面自动化等实时应用。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["dxgi"] }
```

**平台**: 仅 Windows（需要 DirectX 11+）

## 快速开始

### 基础屏幕捕获

```rust
use win_auto_utils::dxgi::DxgiCapture;

// 创建捕获实例
let mut capture = DxgiCapture::new()?;

// 捕获全屏
let frame = capture.capture_frame()?;
println!("帧大小: {}x{}", frame.width, frame.height);
println!("数据长度: {} 字节", frame.data.len());

// 帧数据采用 BGRA 格式（每像素 4 字节）
```

### 捕获特定区域

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// 在位置 (100, 50) 捕获 200x100 的区域
let region = capture.capture_region(100, 50, 200, 100)?;
println!("区域已捕获: {}x{}", region.width, region.height);
```

## 核心功能

- **GPU 加速**: 直接从 GPU 内存捕获
- **高性能**: 现代硬件上可达 30-60+ FPS
- **低延迟**: 亚毫秒级捕获时间
- **零拷贝**: 直接内存映射，无中间缓冲区
- **区域支持**: 高效捕获特定屏幕区域
- **自动恢复**: 优雅处理显示模式变化

## 使用示例

### 示例 1: 连续帧捕获

```rust
use win_auto_utils::dxgi::DxgiCapture;
use std::time::Instant;

let mut capture = DxgiCapture::new()?;

// 捕获 100 帧并测量性能
let start = Instant::now();
for i in 0..100 {
    let frame = capture.capture_frame()?;
    
    if i % 10 == 0 {
        println!("帧 {}: {}x{}", i, frame.width, frame.height);
    }
}

let elapsed = start.elapsed();
let fps = 100.0 / elapsed.as_secs_f64();
println!("平均 FPS: {:.2}", fps);
```

### 示例 2: 基于区域的监控

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// 监控特定的 UI 元素（如生命条）
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
    
    // 处理区域数据（例如颜色分析）
    analyze_health_bar(&region.data);
    
    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
}
```

### 示例 3: 错误处理和恢复

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
                eprintln!("捕获失败: {}", msg);
                
                // 尝试重新初始化
                eprintln!("重新初始化...");
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

### 示例 4: 多显示器支持

```rust
use win_auto_utils::dxgi::DxgiCapture;

// 默认捕获主显示器
let mut capture = DxgiCapture::new()?;

// 获取屏幕尺寸
let width = capture.get_width();
let height = capture.get_height();

println!("主显示器: {}x{}", width, height);

// 要捕获特定显示器，修改源代码以选择适配器/输出
```

### 示例 5: 帧处理管道

```rust
use win_auto_utils::dxgi::DxgiCapture;

let mut capture = DxgiCapture::new()?;

// 捕获并转换为 RGB
let frame = capture.capture_frame()?;
let bgra_data = &frame.data;

// 将 BGRA 转换为 RGB（跳过 alpha 通道）
let mut rgb_data = Vec::with_capacity(bgra_data.len() * 3 / 4);
for chunk in bgra_data.chunks(4) {
    rgb_data.push(chunk[2]); // R
    rgb_data.push(chunk[1]); // G
    rgb_data.push(chunk[0]); // B
}

// 现在 rgb_data 包含 RGB 像素
println!("RGB 数据大小: {} 字节", rgb_data.len());
```

### 示例 6: 性能基准测试

```rust
use win_auto_utils::dxgi::DxgiCapture;
use std::time::Instant;

let mut capture = DxgiCapture::new()?;

// 预热
for _ in 0..10 {
    let _ = capture.capture_frame()?;
}

// 基准测试
let iterations = 1000;
let start = Instant::now();

for _ in 0..iterations {
    let _ = capture.capture_frame()?;
}

let elapsed = start.elapsed();
let avg_time = elapsed / iterations as u32;
let fps = iterations as f64 / elapsed.as_secs_f64();

println!("平均捕获时间: {:?}", avg_time);
println!("达到的 FPS: {:.2}", fps);
```

## API 参考

### 主要类型

#### DxgiCapture

DXGI 屏幕捕获的主要结构体。

**构造函数**:
- `DxgiCapture::new() -> Result<Self, DxgiError>` - 创建新的捕获实例

**方法**:
- `capture_frame() -> Result<CapturedFrame, DxgiError>` - 捕获全屏
- `capture_region(x, y, w, h) -> Result<CapturedFrame, DxgiError>` - 捕获特定区域
- `get_width() -> usize` - 获取屏幕宽度
- `get_height() -> usize` - 获取屏幕高度

#### CapturedFrame

表示捕获的帧。

**字段**:
- `data: Vec<u8>` - BGRA 格式的像素数据（每像素 4 字节）
- `width: usize` - 帧宽度（像素）
- `height: usize` - 帧高度（像素）

**方法**:
- `get_pixel(x, y) -> Option<(u8, u8, u8, u8)>` - 获取像素颜色（B, G, R, A）

#### DxgiError

DXGI 操作的错误类型。

**变体**:
- `CaptureFailed(String)` - 捕获操作失败
- `RegionOutOfBounds` - 请求的区域超出屏幕边界
- `InvalidRegionDimensions` - 无效的宽度或高度（零或负数）
- `InitializationFailed(String)` - 初始化 DXGI 失败
- `LockFailed` - 获取锁失败

## 性能特征

### 不同分辨率的捕获时间

| 分辨率 | 典型时间 | 最大 FPS |
|--------|---------|---------|
| 1920x1080 | 1-3ms | 300-1000 |
| 2560x1440 | 2-5ms | 200-500 |
| 3840x2160 | 5-10ms | 100-200 |
| 区域 (200x100) | <1ms | 1000+ |

*在 Intel i7-10700K 和 NVIDIA RTX 3070 上测量*

### 内存使用

- **全高清 (1920x1080)**: 每帧约 8 MB（BGRA）
- **QHD (2560x1440)**: 每帧约 14 MB
- **4K (3840x2160)**: 每帧约 32 MB
- **区域 (200x100)**: 约 80 KB

### CPU 开销

- **空闲**: <1% CPU 使用率
- **活动捕获**: 2-5% CPU（单核）
- **GPU 使用**: 最小（硬件加速）

## 最佳实践

1. **重用捕获实例**
   ```rust
   // 好: 重用实例
   let mut capture = DxgiCapture::new()?;
   for _ in 0..1000 {
       let frame = capture.capture_frame()?;
   }
   
   // 不好: 每次都创建新实例
   for _ in 0..1000 {
       let capture = DxgiCapture::new()?;  // 慢！
       let frame = capture.capture_frame()?;
   }
   ```

2. **仅捕获需要的区域**
   ```rust
   // 好: 小区域
   let region = capture.capture_region(100, 50, 200, 100)?;
   
   // 不好: 只需要小区域时捕获全屏
   let frame = capture.capture_frame()?;  // 浪费资源
   ```

3. **优雅地处理错误**
   ```rust
   match capture.capture_frame() {
       Ok(frame) => process(frame),
       Err(DxgiError::CaptureFailed(_)) => {
           // 访问丢失时重新初始化
           capture = DxgiCapture::new()?;
       }
       Err(e) => return Err(e),
   }
   ```

4. **使用适当的帧率**
   ```rust
   // 对于 UI 监控: 10-30 FPS 足够
   std::thread::sleep(Duration::from_millis(33)); // 30 FPS
   
   // 对于游戏: 可能需要 60+ FPS
   std::thread::sleep(Duration::from_millis(16)); // 60 FPS
   ```

5. **高效处理帧**
   ```rust
   // 避免复制大向量
   let frame = capture.capture_frame()?;
   process_in_place(&frame.data);  // 借用而非克隆
   ```

## 常见陷阱

### ❌ 不处理访问丢失

```rust
// 错误: 显示模式改变时崩溃
let frame = capture.capture_frame()?;

// 正确: 处理并恢复
match capture.capture_frame() {
    Ok(frame) => use_frame(frame),
    Err(DxgiError::CaptureFailed(msg)) if msg.contains("Access lost") => {
        capture = DxgiCapture::new()?;  // 重新初始化
    }
    Err(e) => return Err(e),
}
```

### ❌ 捕获越界区域

```rust
// 错误: 可能 panic 或返回错误
let region = capture.capture_region(10000, 10000, 200, 100)?;

// 正确: 先检查边界
let width = capture.get_width();
let height = capture.get_height();
if x + w <= width && y + h <= height {
    let region = capture.capture_region(x, y, w, h)?;
}
```

### ❌ 阻塞捕获线程

```rust
// 错误: 繁重的处理阻塞下一次捕获
let frame = capture.capture_frame()?;
heavy_image_processing(&frame);  // 耗时 100ms - 降低 FPS

// 正确: 使用单独的线程或队列
let frame = capture.capture_frame()?;
tx.send(frame)?;  // 发送到工作线程
```

## 平台要求

- **操作系统**: Windows 8 或更高版本
- **DirectX**: DirectX 11+
- **GPU**: 任何兼容 DirectX 11 的 GPU
- **权限**: 不需要特殊权限

### 限制

- ❌ 不适用于 Windows 7 或更早版本
- ❌ 无法捕获受保护的内容（DRM 视频）
- ❌ 在 UAC 提示或安全桌面期间可能失败
- ⚠️ 性能因 GPU 驱动程序质量而异

## 相关模块

- [`template_matcher`](template_matcher.md): 在捕获的帧中查找图像
- [`color_finder`](../utils.md): 分析捕获区域中的颜色
- [`process_window`](process_window.md): 获取窗口坐标以进行目标捕获
- [`memory_hook`](memory_hook.md): 与内存读取结合以实现完整自动化

---

**语言**: [English](../../en/modules/dxgi.md) | [中文](dxgi.md)
