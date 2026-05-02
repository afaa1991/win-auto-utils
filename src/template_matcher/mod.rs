//! Image template matching module
//!
//! Provides high-performance template matching using DXGI screen capture
//! and image processing libraries (image, imageproc, rayon).

//! - **Layer 1**: Maximum performance, pre-converted pixel data
//! - **Layer 2**: Flexible, accepts any image type
//! - **Layer 3**: Works with raw image file bytes (e.g., from network/memory)
//! - **Layer 4**: Most convenient, loads from file path
//!
//! # Quick Start
//!
//! ## Layer 1: Raw Pixel Data (Maximum Performance)
//! ```no_run
//! use win_auto_utils::template_matcher::match_region_from_gray;
//!
//! // Pre-load and convert template once (outside hot loop)
//! let template = image::open("button.png")?.to_luma8();
//!
//! // Match repeatedly with zero conversion overhead
//! for _ in 0..100 {
//!     let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
//! }
//! ```
//!
//! ## Layer 2: DynamicImage (Flexible)
//! ```no_run
//! use win_auto_utils::template_matcher::match_region_from_dynamic;
//!
//! // Accepts any image type, converts internally
//! let img = image::open("icon.png")?;
//! let result = match_region_from_dynamic(0, 0, 1920, 1080, &img, 0.85)?;
//! ```
//!
//! ## Layer 3: Image Bytes (Network/Memory)
//! ```no_run
//! use win_auto_utils::template_matcher::match_region_from_bytes;
//!
//! // Load from memory, network, or embedded resources
//! let png_data = std::fs::read("button.png")?;
//! let result = match_region_from_bytes(0, 0, 1920, 1080, &png_data, 0.85)?;
//! ```
//!
//! ## Layer 4: File Path (Most Convenient)
//! ```no_run
//! use win_auto_utils::template_matcher::match_region_from_path;
//!
//! // One-liner for quick testing
//! let result = match_region_from_path(0, 0, 1920, 1080, "button.png", 0.85)?;
//! ```
//!
//! # Performance Comparison
//! | Layer | Conversion Overhead | Use Case |
//! |-------|-------------------|----------|
//! | **Layer 1** | None (zero-copy) | Hot loops, real-time matching |
//! | **Layer 2** | One-time conversion | Mixed image types |
//! | **Layer 3** | Decode + convert | Network/embedded images |
//! | **Layer 4** | File I/O + decode + convert | Quick prototyping |
//!
//! # Image Type Selection Guide
//!
//! ## 1. GrayImage (Recommended for Most Cases)
//! ```no_run
//! use image::GrayImage;
//!
//! // Best for: UI elements, buttons, icons
//! // Advantages:
//! // - Smallest memory footprint (1 byte/pixel)
//! // - Fastest matching performance
//! // - Robust to brightness variations
//! let template: GrayImage = image::open("button.png")?.to_luma8();
//! ```
//!
//! ## 2. RgbImage (When Color Matters)
//! ```no_run
//! use image::{RgbImage, Rgb};
//!
//! // Best for: Color-coded UI elements, game icons
//! // Use when: You need to distinguish by color
//! let template: RgbImage = image::open("icon.png")?.to_rgb8();
//! ```
//!
//! # Architecture Details
//! The module provides pure functions that:
//! 1. Accept screen coordinates and template image data
//! 2. Capture screen region via DXGI
//! 3. Perform template matching in parallel
//! 4. Return match result with position and similarity score
//!
//! # Algorithm Details
//! Uses `CrossCorrelationNormalized` which:
//! - Produces values in range [-1, 1]
//! - Is invariant to brightness and contrast changes
//! - Ideal for real-world UI matching scenarios
//!
//! # Feature Flag
//! Enable with: `--features "template_matcher"`
//!
//! Dependencies (automatically included):
//! - `dxgi`: High-performance screen capture
//! - `image`: Image loading and conversion
//! - `imageproc`: Template matching algorithms (with rayon support)
//! - `rayon`: Parallel processing
//! - `dirs`: Directory utilities

use image::{DynamicImage, GrayImage};
use imageproc::template_matching::{find_extremes, match_template_parallel, MatchTemplateMethod};

/// Template matching result
///
/// Contains similarity score and position information.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Similarity score in range [-1, 1]
    /// - 1.0: Perfect match
    /// - 0.0: No correlation
    /// - -1.0: Inverse correlation
    pub similarity: f32,

    /// Whether the match meets the threshold
    pub matched: bool,

    /// Center X coordinate of matched region (0 if not matched)
    pub x: u32,

    /// Center Y coordinate of matched region (0 if not matched)
    pub y: u32,
}

impl Default for MatchResult {
    fn default() -> Self {
        Self {
            similarity: 0.0,
            matched: false,
            x: 0,
            y: 0,
        }
    }
}

// ============================================================================
// Layer 1: Raw Pixel Data (Maximum Performance)
// ============================================================================

/// Match template in screen region from grayscale image (Layer 1 - Highest Performance)
///
/// This is the **fastest** matching function as it operates directly on pre-converted
/// grayscale pixel data with zero conversion overhead.
///
/// # Why Grayscale Only?
/// The underlying `imageproc::match_template_parallel` algorithm only supports grayscale images.
/// This is actually optimal for most automation scenarios because:
/// - Grayscale matching is ~3x faster than RGB
/// - More robust to lighting variations
/// - Smaller memory footprint (1 byte/pixel vs 3 bytes/pixel)
/// - UI elements are typically distinguished by shape/contrast, not color
///
/// If you need color-sensitive matching, consider:
/// 1. Pre-filtering by color before template matching
/// 2. Using multiple grayscale templates for different color states
/// 3. Post-processing match results with color verification
///
/// # Arguments
/// - `x`: Left coordinate of search region
/// - `y`: Top coordinate of search region
/// - `width`: Width of search region
/// - `height`: Height of search region
/// - `template`: Pre-converted grayscale template image
/// - `threshold`: Minimum similarity score (0.0 to 1.0, typically 0.8-0.95)
///
/// # Returns
/// - `Ok(MatchResult)`: Matching result with position and similarity
/// - `Err(String)`: Error message
///
/// # Performance Tips
/// - Pre-convert templates outside hot loops: `let tpl = img.to_luma8();`
/// - Reuse the same template for multiple matches
/// - Minimize search region size to reduce DXGI capture time
///
/// # Example
/// ```no_run
/// use win_auto_utils::template_matcher::match_region_from_gray;
///
/// // Pre-convert once
/// let template = image::open("button.png")?.to_luma8();
///
/// // Match repeatedly with zero overhead
/// let result = match_region_from_gray(0, 0, 1920, 1080, &template, 0.85)?;
/// ```
pub fn match_region_from_gray(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    template: &GrayImage,
    threshold: f32,
) -> Result<MatchResult, String> {
    validate_dimensions(x, y, width, height)?;

    let template_width = template.width();
    let template_height = template.height();

    if template_width == 0 || template_height == 0 {
        return Err("Template image is empty".to_string());
    }

    if template_width > width as u32 || template_height > height as u32 {
        return Err(format!(
            "Template ({}x{}) is larger than search region ({}x{})",
            template_width, template_height, width, height
        ));
    }

    // Capture screen region using DXGI (zero-copy)
    let _capture_start = std::time::Instant::now();
    let bgra_data = crate::dxgi::capture_region_bytes(x, y, width, height)
        .map_err(|e| format!("DXGI capture failed: {}", e))?;

    // Convert BGRA to grayscale manually for better performance
    // Formula: Y = 0.299*R + 0.587*G + 0.114*B
    let mut gray_data = Vec::with_capacity((width * height) as usize);
    for i in (0..bgra_data.len()).step_by(4) {
        if i + 2 < bgra_data.len() {
            let b = bgra_data[i] as f32;
            let g = bgra_data[i + 1] as f32;
            let r = bgra_data[i + 2] as f32;
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            gray_data.push(gray);
        }
    }

    let gray_capture = image::GrayImage::from_raw(width as u32, height as u32, gray_data)
        .ok_or_else(|| "Failed to create grayscale image from captured data".to_string())?;

    // Perform template matching using parallel algorithm
    let _match_start = std::time::Instant::now();
    let result_map = match_template_parallel(
        &gray_capture,
        template,
        MatchTemplateMethod::CrossCorrelationNormalized,
    );

    // Find best match location
    let extremes = find_extremes(&result_map);
    let similarity = extremes.max_value;
    let (pos_x, pos_y) = extremes.max_value_location;

    // Determine if match meets threshold
    let matched = similarity >= threshold;

    // Calculate center position if matched
    let (center_x, center_y) = if matched {
        (
            pos_x + x as u32 + template_width / 2,
            pos_y + y as u32 + template_height / 2,
        )
    } else {
        (0, 0)
    };

    Ok(MatchResult {
        similarity,
        matched,
        x: center_x,
        y: center_y,
    })
}

// ============================================================================
// Layer 2: DynamicImage (Flexible)
// ============================================================================

/// Match template in screen region from DynamicImage (Layer 2 - Flexible)
///
/// Accepts any image type and automatically converts to optimal format (grayscale).
/// Adds one-time conversion overhead compared to Layer 1.
///
/// # Arguments
/// - `x`: Left coordinate of search region
/// - `y`: Top coordinate of search region
/// - `width`: Width of search region
/// - `height`: Height of search region
/// - `template`: DynamicImage (any format: PNG, JPEG, BMP, etc.)
/// - `threshold`: Minimum similarity score
///
/// # Returns
/// - `Ok(MatchResult)`: Matching result
/// - `Err(String)`: Error message
///
/// # Performance Note
/// This function converts the template to grayscale internally. For repeated matching,
/// use `match_region_from_gray` with pre-converted templates.
///
/// # Example
/// ```no_run
/// use win_auto_utils::template_matcher::match_region_from_dynamic;
///
/// let img = image::open("mixed_format.png")?;
/// let result = match_region_from_dynamic(0, 0, 1920, 1080, &img, 0.85)?;
/// ```
pub fn match_region_from_dynamic(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    template: &DynamicImage,
    threshold: f32,
) -> Result<MatchResult, String> {
    // Convert to grayscale for optimal performance
    let gray_template = template.to_luma8();

    // Delegate to Layer 1
    match_region_from_gray(x, y, width, height, &gray_template, threshold)
}

// ============================================================================
// Layer 3: Image Bytes (Network/Memory)
// ============================================================================

/// Match template in screen region from image file bytes (Layer 3 - Bytes)
///
/// Decodes image from raw bytes (PNG, JPEG, etc.) and performs matching.
/// Useful for images loaded from network, embedded resources, or memory buffers.
///
/// # Arguments
/// - `x`: Left coordinate of search region
/// - `y`: Top coordinate of search region
/// - `width`: Width of search region
/// - `height`: Height of search region
/// - `image_data`: Raw image file bytes (PNG, JPEG, etc.)
/// - `threshold`: Minimum similarity score
///
/// # Returns
/// - `Ok(MatchResult)`: Matching result
/// - `Err(String)`: Error message (includes decode errors)
///
/// # Example
/// ```no_run
/// use win_auto_utils::template_matcher::match_region_from_bytes;
///
/// // Load from memory or network
/// let png_bytes = reqwest::blocking::get("https://example.com/icon.png")?.bytes()?;
/// let result = match_region_from_bytes(0, 0, 1920, 1080, &png_bytes, 0.85)?;
/// ```
pub fn match_region_from_bytes(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    image_data: &[u8],
    threshold: f32,
) -> Result<MatchResult, String> {
    // Decode image from bytes
    let img = image::load_from_memory(image_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Delegate to Layer 2
    match_region_from_dynamic(x, y, width, height, &img, threshold)
}

// ============================================================================
// Layer 4: File Path (Most Convenient)
/// Match template in screen region from file path (Layer 4 - Convenience)
///
/// Loads image from file path and performs matching.
/// Most convenient but slowest due to file I/O overhead.
///
/// # Arguments
/// - `x`: Left coordinate of search region
/// - `y`: Top coordinate of search region
/// - `width`: Width of search region
/// - `height`: Height of search region
/// - `path`: Path to image file (PNG, JPEG, etc.)
/// - `threshold`: Minimum similarity score
///
/// # Returns
/// - `Ok(MatchResult)`: Matching result
/// - `Err(String)`: Error message (includes file I/O errors)
///
/// # Example
/// ```no_run
/// use win_auto_utils::template_matcher::match_region_from_path;
///
/// // Quick one-liner for prototyping
/// let result = match_region_from_path(0, 0, 1920, 1080, "assets/button.png", 0.85)?;
/// ```
pub fn match_region_from_path<P: AsRef<std::path::Path>>(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    path: P,
    threshold: f32,
) -> Result<MatchResult, String> {
    let path_str = path.as_ref().display().to_string();

    // Load image from file
    let img =
        image::open(&path).map_err(|e| format!("Failed to load image '{}': {}", path_str, e))?;

    // Delegate to Layer 2
    match_region_from_dynamic(x, y, width, height, &img, threshold)
}

// ============================================================================
// Full Screen Variants (Convenience Wrappers)
// ============================================================================

/// Match template across entire screen from grayscale image
///
/// Convenience wrapper that automatically detects screen size.
pub fn match_full_screen_from_gray(
    template: &GrayImage,
    threshold: f32,
) -> Result<MatchResult, String> {
    let (width, height) =
        crate::dxgi::get_screen_size().map_err(|e| format!("Failed to get screen size: {}", e))?;

    match_region_from_gray(0, 0, width as i32, height as i32, template, threshold)
}

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Validate region dimensions
fn validate_dimensions(x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    if width <= 0 || height <= 0 {
        return Err(format!("Invalid region dimensions: {}x{}", width, height));
    }

    if x < 0 || y < 0 {
        return Err(format!("Invalid region position: ({}, {})", x, y));
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "template_matcher")]
mod tests {
    use super::*;

    #[test]
    fn test_match_result_default() {
        let result = MatchResult::default();
        assert_eq!(result.similarity, 0.0);
        assert!(!result.matched);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_empty_template() {
        let empty_gray = GrayImage::new(0, 0);
        let result = match_region_from_gray(0, 0, 100, 100, &empty_gray, 0.8);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_larger_than_region() {
        let large_template = GrayImage::new(200, 200);
        let result = match_region_from_gray(0, 0, 100, 100, &large_template, 0.8);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_region_dimensions() {
        let template = GrayImage::new(10, 10);
        let result = match_region_from_gray(0, 0, -1, 100, &template, 0.8);
        assert!(result.is_err());
    }
}
