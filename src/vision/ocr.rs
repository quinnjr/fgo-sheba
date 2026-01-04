//! OCR functionality for reading text from the game
//!
//! Handles reading HP bars, NP gauges, damage numbers, and other text.

use image::{GrayImage, RgbaImage};

/// OCR engine for reading game text
pub struct OCREngine {
    /// Whether the OCR is initialized
    initialized: bool,
}

impl OCREngine {
    /// Create a new OCR engine
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the OCR engine
    pub fn init(&mut self) -> Result<(), OCRError> {
        // TODO: Initialize actual OCR backend (Tesseract or custom)
        self.initialized = true;
        Ok(())
    }

    /// Read a number from an image region
    pub fn read_number(&self, image: &RgbaImage) -> Option<u32> {
        if !self.initialized {
            return None;
        }

        // Convert to grayscale for processing
        let gray = image::imageops::grayscale(image);

        // Use digit recognition
        self.recognize_digits(&gray)
    }

    /// Read HP percentage from HP bar region
    pub fn read_hp_percent(&self, hp_bar_image: &RgbaImage) -> Option<f32> {
        // HP bars in FGO are color-coded
        // Full HP: Green
        // Low HP: Yellow/Red

        let (width, height) = hp_bar_image.dimensions();
        if width == 0 || height == 0 {
            return None;
        }

        // Find the rightmost green/yellow pixel
        let mut filled_width = 0;

        for x in 0..width {
            let mut has_hp_color = false;
            for y in 0..height {
                let pixel = hp_bar_image.get_pixel(x, y);
                // Check for HP bar colors (green to red)
                if is_hp_bar_color(pixel[0], pixel[1], pixel[2]) {
                    has_hp_color = true;
                    break;
                }
            }
            if has_hp_color {
                filled_width = x + 1;
            }
        }

        Some(filled_width as f32 / width as f32)
    }

    /// Read NP gauge percentage
    pub fn read_np_gauge(&self, np_region: &RgbaImage) -> Option<u32> {
        // NP gauge in FGO displays as a number (0-300)
        // The gauge itself is blue when filling

        // Try to read the number directly
        self.read_number(np_region)
    }

    /// Read critical star count
    pub fn read_star_count(&self, star_region: &RgbaImage) -> Option<u32> {
        // Stars are displayed as a yellow number
        self.read_number(star_region)
    }

    /// Read wave information (e.g., "1/3")
    pub fn read_wave_info(&self, wave_region: &RgbaImage) -> Option<(u32, u32)> {
        // Wave display format: "Battle X/Y"
        // We need to extract X and Y

        let text = self.read_text(wave_region)?;

        // Parse "X/Y" format
        let parts: Vec<&str> = text.split('/').collect();
        if parts.len() == 2 {
            let current = parts[0].trim().parse().ok()?;
            let total = parts[1].trim().parse().ok()?;
            return Some((current, total));
        }

        None
    }

    /// Read general text from an image
    fn read_text(&self, _image: &RgbaImage) -> Option<String> {
        // TODO: Implement actual OCR
        None
    }

    /// Recognize digits in a grayscale image
    fn recognize_digits(&self, _image: &GrayImage) -> Option<u32> {
        // TODO: Implement digit recognition
        // For now, return None
        // This would use template matching or a trained digit classifier
        None
    }
}

impl Default for OCREngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a color is part of the HP bar
fn is_hp_bar_color(r: u8, g: u8, b: u8) -> bool {
    // Green HP (full)
    let is_green = g > 150 && g > r && g > b;
    // Yellow HP (medium)
    let is_yellow = r > 150 && g > 150 && b < 100;
    // Red HP (low)
    let is_red = r > 150 && g < 100 && b < 100;

    is_green || is_yellow || is_red
}

/// Digit templates for template matching
pub mod digit_templates {
    /// Template dimensions
    pub const DIGIT_WIDTH: u32 = 20;
    pub const DIGIT_HEIGHT: u32 = 30;

    // In a real implementation, these would be actual template images
    // loaded from files or embedded in the binary
}

/// HP bar reader specifically for FGO's HP display
pub struct HPBarReader {
    /// HP bar width in pixels (reference resolution)
    bar_width: u32,
    /// HP bar height in pixels
    bar_height: u32,
}

impl HPBarReader {
    /// Create a new HP bar reader
    pub fn new() -> Self {
        Self {
            bar_width: 200,
            bar_height: 10,
        }
    }

    /// Read HP percentage from an enemy HP bar
    pub fn read_enemy_hp(&self, hp_bar: &RgbaImage) -> f32 {
        let (width, _height) = hp_bar.dimensions();
        if width == 0 {
            return 0.0;
        }

        // Enemy HP bars are typically red when damaged
        let mut filled_pixels = 0;
        let center_y = hp_bar.dimensions().1 / 2;

        for x in 0..width {
            let pixel = hp_bar.get_pixel(x, center_y);
            if is_hp_bar_color(pixel[0], pixel[1], pixel[2]) {
                filled_pixels += 1;
            }
        }

        filled_pixels as f32 / width as f32
    }

    /// Detect break bars on an enemy HP bar
    pub fn detect_break_bars(&self, hp_region: &RgbaImage) -> u32 {
        // Break bars appear as yellow markers below the HP bar
        // Count the number of bright yellow segments

        let (_width, height) = hp_region.dimensions();
        if height < 5 {
            return 0;
        }

        // Sample the bottom portion where break bars appear
        let break_bar_y = height - 3;
        let mut break_count = 0;
        let mut in_break = false;

        for x in 0..hp_region.dimensions().0 {
            let pixel = hp_region.get_pixel(x, break_bar_y);
            let is_break_color = pixel[0] > 200 && pixel[1] > 200 && pixel[2] < 100; // Yellow

            if is_break_color && !in_break {
                break_count += 1;
                in_break = true;
            } else if !is_break_color {
                in_break = false;
            }
        }

        break_count
    }
}

impl Default for HPBarReader {
    fn default() -> Self {
        Self::new()
    }
}

/// NP gauge reader
pub struct NPGaugeReader;

impl NPGaugeReader {
    /// Read NP gauge from the displayed number
    pub fn read_gauge(np_region: &RgbaImage) -> u32 {
        // The NP gauge displays numbers 0-100+ at the bottom of each servant
        // This is typically shown in a specific font

        // For now, estimate based on the blue fill of the gauge
        let (width, height) = np_region.dimensions();
        if width == 0 || height == 0 {
            return 0;
        }

        let mut blue_pixels = 0;
        let mut total_pixels = 0;

        for y in 0..height {
            for x in 0..width {
                let pixel = np_region.get_pixel(x, y);
                // NP gauge is blue when filling
                if pixel[2] > pixel[0] && pixel[2] > pixel[1] && pixel[2] > 100 {
                    blue_pixels += 1;
                }
                total_pixels += 1;
            }
        }

        if total_pixels == 0 {
            return 0;
        }

        // Scale to 0-100
        ((blue_pixels as f32 / total_pixels as f32) * 100.0) as u32
    }
}

/// OCR error types
#[derive(Debug, thiserror::Error)]
pub enum OCRError {
    #[error("OCR not initialized")]
    NotInitialized,
    #[error("Failed to process image: {0}")]
    ProcessingError(String),
    #[error("No text found")]
    NoTextFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_hp_bar_color_detection() {
        assert!(is_hp_bar_color(0, 200, 0)); // Green
        assert!(is_hp_bar_color(200, 200, 50)); // Yellow
        assert!(is_hp_bar_color(200, 50, 50)); // Red
        assert!(!is_hp_bar_color(50, 50, 200)); // Blue - not HP color
    }

    #[test]
    fn test_hp_bar_reading() {
        let reader = HPBarReader::new();

        // Create a half-filled HP bar (green left, black right)
        let hp_bar: RgbaImage = ImageBuffer::from_fn(100, 10, |x, _| {
            if x < 50 {
                Rgba([0, 200, 0, 255]) // Green
            } else {
                Rgba([0, 0, 0, 255]) // Black
            }
        });

        let hp = reader.read_enemy_hp(&hp_bar);
        assert!((hp - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_np_gauge_reading() {
        // Create a blue NP gauge region (50% filled)
        let np_region: RgbaImage = ImageBuffer::from_fn(50, 20, |x, _| {
            if x < 25 {
                Rgba([50, 50, 200, 255]) // Blue
            } else {
                Rgba([50, 50, 50, 255]) // Gray
            }
        });

        let gauge = NPGaugeReader::read_gauge(&np_region);
        assert!(gauge >= 40 && gauge <= 60);
    }
}
