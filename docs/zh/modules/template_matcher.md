# 模板匹配器 (Template Matcher - Image Template Matching)

[English](../../en/modules/template_matcher.md) | [返回概览](overview.md)

`template_matcher` 模块使用 DXGI 屏幕捕获和并行图像处理提供高性能的图像模板匹配功能。它具有四层 API 设计，适用于不同的性能/便利性权衡，支持灰度和 RGB 匹配，采用归一化互相关算法。

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["template_matcher"] }
```

**平台**: 仅 Windows（需要 DXGI 支持）

## 快速开始

### 第一层：原始像素数据（最高性能）

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// 预先加载并转换模板一次（在热循环外）
let template = image::open("button.png")?.to_luma8();

// 重复匹配，零转换开销
for _ in 0..100 {
    let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
    
    if let Some(match_info) = result {
        println!("找到于 ({}, {}) 相似度 {:.2}", 
                 match_info.x, match_info.y, match_info.similarity);
    }
}
```

### 第四层：文件路径（最方便）

```rust
use win_auto_utils::template_matcher::match_region_from_path;

// 快速测试的一行代码
let result = match_region_from_path(
    0, 0, 1920, 1080, 
    "button.png", 
    0.85
)?;

if let Some(info) = result {
    println!("找到匹配！");
}
```

## 核心功能

- **四层 API**: 从最高性能到最方便
- **并行处理**: 通过 Rayon 进行多线程模板匹配
- **归一化互相关**: 对亮度/对比度变化具有鲁棒性
- **灰度和 RGB 支持**: 根据用例选择
- **DXGI 集成**: 高性能屏幕捕获
- **灵活输入**: 接受像素、图像、字节或文件路径
- **相似度评分**: 返回匹配置信度（0.0 - 1.0）

## 使用示例

### 示例 1: UI 按钮检测

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// 加载按钮模板
let button_template = image::open("submit_button.png")?.to_luma8();

// 在特定区域搜索（比全屏快）
let search_x = 800;
let search_y = 600;
let search_width = 300;
let search_height = 100;

let result = match_region_from_gray(
    search_x, search_y, 
    search_width, search_height,
    &button_template,
    0.90  // 高精度匹配的高阈值
)?;

if let Some(info) = result {
    println!("按钮找到于屏幕位置 ({}, {})", 
             info.x + search_x, info.y + search_y);
}
```

### 示例 2: RGB 颜色敏感匹配

```rust
use win_auto_utils::template_matcher::match_region_from_rgb;

// 当颜色重要时使用 RGB（例如状态指示器）
let health_bar = image::open("health_full.png")?.to_rgb8();

let result = match_region_from_rgb(
    100, 50, 200, 20,
    &health_bar,
    0.85
)?;

if result.is_some() {
    println!("检测到满血");
} else {
    println!("血量未满");
}
```

### 示例 3: 多模板匹配

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

// 查找哪个图标存在
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
        println!("找到: {}", name);
        break;
    }
}
```

### 示例 4: 动态图像类型（第二层）

```rust
use win_auto_utils::template_matcher::match_region_from_dynamic;

// 接受任何图像类型，内部转换
let img = image::open("icon.png")?;  // 可以是 PNG、JPG、BMP 等

let result = match_region_from_dynamic(
    0, 0, 1920, 1080,
    &img,
    0.85
)?;
```

### 示例 5: 来自内存/网络的图像（第三层）

```rust
use win_auto_utils::template_matcher::match_region_from_bytes;

// 从内存、网络或嵌入资源加载
let png_data = std::fs::read("button.png")?;

let result = match_region_from_bytes(
    0, 0, 1920, 1080,
    &png_data,
    0.85
)?;
```

### 示例 6: 自适应阈值策略

```rust
use win_auto_utils::template_matcher::match_region_from_gray;

let template = image::open("ui_element.png")?.to_luma8();

// 尝试多个阈值以提高鲁棒性
for threshold in [0.95, 0.90, 0.85, 0.80].iter() {
    let result = match_region_from_gray(
        0, 0, 1920, 1080,
        &template,
        *threshold
    )?;
    
    if let Some(info) = result {
        println!("阈值 {} 匹配: 相似度 {:.3}", 
                 threshold, info.similarity);
        break;
    }
}
```

## 示例代码

查看这些示例文件了解使用演示：

- `template_matching.rs`
- `template_matching_benchmark.rs`

运行示例：

```bash
cargo run --example template_matching --features "template_matcher"
cargo run --example template_matching_benchmark --features "template_matcher"
```

## API 参考

### 主要函数

#### 第一层：原始像素数据

- **`match_region_from_gray(x, y, w, h, template: &GrayImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - 将灰度模板与屏幕区域匹配
  - 零转换开销（最快）
  - 最适合热循环和实时匹配

- **`match_region_from_rgb(x, y, w, h, template: &RgbImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - 将 RGB 模板与屏幕区域匹配
  - 当颜色信息重要时使用

#### 第二层：DynamicImage

- **`match_region_from_dynamic(x, y, w, h, template: &DynamicImage, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - 接受任何图像类型
  - 内部转换为适当格式
  - 灵活但有转换开销

#### 第三层：图像字节

- **`match_region_from_bytes(x, y, w, h, image_data: &[u8], threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - 从原始字节解码图像（PNG、JPG 等）
  - 适用于网络/嵌入资源
  - 包括解码 + 转换开销

#### 第四层：文件路径

- **`match_region_from_path(x, y, w, h, path: &str, threshold: f64) -> Result<Option<MatchInfo>, TemplateError>`**
  - 从文件路径加载图像
  - 原型设计最方便
  - 包括文件 I/O + 解码 + 转换开销

### 类型

#### MatchInfo

成功匹配的信息。

**字段**:
- `x: u32` - 匹配的 X 坐标（相对于搜索区域）
- `y: u32` - 匹配的 Y 坐标（相对于搜索区域）
- `similarity: f64` - 相似度分数（0.0 - 1.0）
- `width: u32` - 模板宽度
- `height: u32` - 模板高度

**方法**:
- `screen_x(&self, region_x: u32) -> u32` - 获取绝对屏幕 X 坐标
- `screen_y(&self, region_y: u32) -> u32` - 获取绝对屏幕 Y 坐标

#### TemplateError

模板匹配操作的错误类型。

**变体**:
- `CaptureFailed(String)` - 屏幕捕获失败
- `ImageDecodeError(String)` - 图像解码失败
- `InvalidThreshold(f64)` - 阈值超出范围（必须是 0.0-1.0）
- `TemplateTooLarge` - 模板大于搜索区域
- `ProcessingError(String)` - 图像处理错误

## 算法细节

### 归一化互相关

模块使用 `CrossCorrelationNormalized` 算法，其特点：

- **输出范围**: [-1, 1]，其中 1.0 是完美匹配
- **亮度不变**: 不受均匀亮度变化的影响
- **对比度不变**: 对对比度变化具有鲁棒性
- **理想用途**: 现实世界的 UI 匹配场景

### 并行处理

- **Rayon 集成**: 自动跨 CPU 核心并行化
- **区域分割**: 将搜索空间分成块
- **加速比**: 四核 CPU 上 2-4 倍，更多核心上更高

## 性能特征

### 层级对比

| 层级 | 开销 | 典型时间（1080p） | 用例 |
|------|------|------------------|------|
| **第一层** | 无 | 5-20ms | 热循环、实时 |
| **第二层** | 转换 | 10-30ms | 混合图像类型 |
| **第三层** | 解码 + 转换 | 15-40ms | 网络/嵌入 |
| **第四层** | I/O + 解码 + 转换 | 20-50ms | 原型设计 |

*在 Intel i7-10700K 和 RTX 3070 上测量*

### 优化技巧

1. **尽可能使用灰度**
   ```rust
   // 更快: 灰度（每像素 1 字节）
   let template = img.to_luma8();
   
   // 更慢: RGB（每像素 3 字节）
   let template = img.to_rgb8();
   ```

2. **限制搜索区域**
   ```rust
   // 快: 小区域
   match_region_from_gray(100, 50, 200, 100, &template, 0.85)?;
   
   // 慢: 全屏
   match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   ```

3. **预转换模板**
   ```rust
   // 好: 在循环外转换一次
   let template = image::open("btn.png")?.to_luma8();
   for _ in 0..100 {
       match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   }
   
   // 不好: 每次迭代都转换
   for _ in 0..100 {
       let template = image::open("btn.png")?.to_luma8();  // 慢！
       match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
   }
   ```

4. **选择合适的阈值**
   ```rust
   // 高精度: 0.90-0.95
   // 平衡: 0.80-0.90
   // 宽松: 0.70-0.80
   ```

## 最佳实践

1. **为用例选择正确的层级**
   ```rust
   // 实时机器人: 第一层
   let template = preload_template();
   loop {
       match_region_from_gray(..., &template, 0.85)?;
   }
   
   // 快速测试: 第四层
   match_region_from_path(..., "icon.png", 0.85)?;
   ```

2. **UI 元素使用灰度**
   ```rust
   // UI 按钮、图标、文本: 灰度
   let ui_template = img.to_luma8();
   
   // 颜色编码指示器: RGB
   let status_template = img.to_rgb8();
   ```

3. **缓存模板**
   ```rust
   // 启动时加载模板一次
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

4. **处理无匹配情况**
   ```rust
   match match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)? {
       Some(info) => handle_match(info),
       None => handle_no_match(),
   }
   ```

## 常见陷阱

### ❌ 使用过低的阈值

```rust
// 错误: 太多误报
let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.50)?;

// 正确: 适当的阈值
let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
```

### ❌ 不限制搜索区域

```rust
// 错误: 搜索整个屏幕（慢）
match_region_from_path(0, 0, 1920, 1080, "small_icon.png", 0.85)?;

// 正确: 限制到预期区域
match_region_from_path(800, 600, 300, 100, "small_icon.png", 0.85)?;
```

### ❌ 在循环中转换模板

```rust
// 错误: 每次迭代都转换
for _ in 0..100 {
    let template = image::open("btn.png")?.to_luma8();
    match_region_from_gray(..., &template, 0.85)?;
}

// 正确: 转换一次
let template = image::open("btn.png")?.to_luma8();
for _ in 0..100 {
    match_region_from_gray(..., &template, 0.85)?;
}
```

## 图像类型选择指南

### 灰度（GrayImage）

**最适合**:
- UI 元素（按钮、菜单）
- 图标和符号
- 文本识别
- 形状和图案

**优势**:
- 最小的内存占用（每像素 1 字节）
- 最快的匹配性能
- 对亮度变化具有鲁棒性
- 适用于大多数 UI 场景

```rust
let template: GrayImage = image::open("button.png")?.to_luma8();
```

### RGB（RgbImage）

**最适合**:
- 颜色编码的 UI 元素
- 具有颜色意义的游戏图标
- 状态指示器（红/绿/黄）
- 当颜色是区分特征时

**使用时机**:
- 需要通过颜色区分
- 灰度产生太多误报
- 颜色在语义上很重要

```rust
let template: RgbImage = image::open("status_icon.png")?.to_rgb8();
```

## 相关模块

- [`dxgi`](dxgi.md): 屏幕捕获后端
- [`script_engine`](script_engine.md): 在脚本中使用模板匹配
- [`color_finder`](../utils.md): 简单颜色检测的替代方案
- [`input`](input.md): 在匹配位置点击

---

**语言**: [English](../../en/modules/template_matcher.md) | [中文](template_matcher.md)
