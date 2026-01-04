//! Screen capture handling
//!
//! Receives and processes raw screen capture data from Android.

use image::{ImageBuffer, Rgba, RgbaImage};

use super::VisionError;

/// Screen capture handler
pub struct ScreenCapture {
    /// Current frame as RGBA image
    current_frame: Option<RgbaImage>,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
    /// Frame counter
    frame_count: u64,
}

impl ScreenCapture {
    /// Create a new screen capture handler
    pub fn new() -> Self {
        Self {
            current_frame: None,
            width: 0,
            height: 0,
            frame_count: 0,
        }
    }

    /// Update with new frame data
    pub fn update(
        &mut self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), VisionError> {
        // Validate frame data size
        let expected_size = (width * height * 4) as usize; // RGBA
        if frame_data.len() != expected_size {
            return Err(VisionError::InvalidFrameData);
        }

        // Create image from raw data
        let image: RgbaImage =
            ImageBuffer::from_raw(width, height, frame_data.to_vec())
                .ok_or(VisionError::InvalidFrameData)?;

        self.current_frame = Some(image);
        self.width = width;
        self.height = height;
        self.frame_count += 1;

        Ok(())
    }

    /// Get the current frame
    pub fn current_frame(&self) -> Option<&RgbaImage> {
        self.current_frame.as_ref()
    }

    /// Get a mutable reference to the current frame
    pub fn current_frame_mut(&mut self) -> Option<&mut RgbaImage> {
        self.current_frame.as_mut()
    }

    /// Get screen dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Check if we have a valid frame
    pub fn has_frame(&self) -> bool {
        self.current_frame.is_some()
    }

    /// Extract a region of the current frame
    pub fn extract_region(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<RgbaImage> {
        let frame = self.current_frame.as_ref()?;

        // Validate bounds
        if x + width > self.width || y + height > self.height {
            return None;
        }

        // Extract sub-image
        let sub_image = image::imageops::crop_imm(frame, x, y, width, height);
        Some(sub_image.to_image())
    }

    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Rgba<u8>> {
        let frame = self.current_frame.as_ref()?;
        if x < self.width && y < self.height {
            Some(*frame.get_pixel(x, y))
        } else {
            None
        }
    }

    /// Check if a region matches a color within tolerance
    pub fn region_matches_color(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        target_color: Rgba<u8>,
        tolerance: u8,
    ) -> bool {
        let Some(region) = self.extract_region(x, y, width, height) else {
            return false;
        };

        // Sample some pixels to check for color match
        let samples = [
            (width / 4, height / 4),
            (width / 2, height / 2),
            (3 * width / 4, 3 * height / 4),
        ];

        samples.iter().all(|&(sx, sy)| {
            let pixel = region.get_pixel(sx, sy);
            color_matches(pixel, &target_color, tolerance)
        })
    }

    /// Scale coordinates from a reference resolution to current resolution
    pub fn scale_coords(&self, x: i32, y: i32, ref_width: u32, ref_height: u32) -> (i32, i32) {
        let scale_x = self.width as f32 / ref_width as f32;
        let scale_y = self.height as f32 / ref_height as f32;
        ((x as f32 * scale_x) as i32, (y as f32 * scale_y) as i32)
    }
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two colors match within tolerance
fn color_matches(a: &Rgba<u8>, b: &Rgba<u8>, tolerance: u8) -> bool {
    let dr = (a[0] as i16 - b[0] as i16).unsigned_abs() as u8;
    let dg = (a[1] as i16 - b[1] as i16).unsigned_abs() as u8;
    let db = (a[2] as i16 - b[2] as i16).unsigned_abs() as u8;

    dr <= tolerance && dg <= tolerance && db <= tolerance
}

/// Predefined regions on the FGO battle screen
pub mod regions {
    /// Card region coordinates (for 1920x1080 reference)
    pub const CARD_REGIONS: [(u32, u32, u32, u32); 5] = [
        (88, 700, 256, 380),   // Card 0
        (400, 700, 256, 380),  // Card 1
        (712, 700, 256, 380),  // Card 2
        (1024, 700, 256, 380), // Card 3
        (1336, 700, 256, 380), // Card 4
    ];

    /// NP card regions (for 1920x1080 reference)
    pub const NP_REGIONS: [(u32, u32, u32, u32); 3] = [
        (296, 200, 180, 220),  // NP 0
        (616, 200, 180, 220),  // NP 1
        (936, 200, 180, 220),  // NP 2
    ];

    /// Enemy regions (for 1920x1080 reference)
    pub const ENEMY_REGIONS: [(u32, u32, u32, u32); 3] = [
        (200, 50, 300, 200),  // Enemy 0
        (550, 50, 300, 200),  // Enemy 1
        (900, 50, 300, 200),  // Enemy 2
    ];

    /// Servant portrait regions for card matching
    pub const SERVANT_PORTRAIT_REGIONS: [(u32, u32, u32, u32); 3] = [
        (40, 850, 100, 100),   // Servant 0 portrait on battle screen
        (390, 850, 100, 100),  // Servant 1
        (740, 850, 100, 100),  // Servant 2
    ];

    /// Wave indicator region
    pub const WAVE_REGION: (u32, u32, u32, u32) = (1750, 20, 150, 50);

    /// Critical stars region
    pub const STARS_REGION: (u32, u32, u32, u32) = (100, 50, 80, 40);

    /// Attack button region
    pub const ATTACK_BUTTON: (u32, u32, u32, u32) = (1650, 450, 200, 150);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_update() {
        let mut capture = ScreenCapture::new();

        // Create dummy frame data (10x10 RGBA)
        let width = 10u32;
        let height = 10u32;
        let frame_data = vec![255u8; (width * height * 4) as usize];

        let result = capture.update(&frame_data, width, height);
        assert!(result.is_ok());
        assert!(capture.has_frame());
        assert_eq!(capture.dimensions(), (width, height));
    }

    #[test]
    fn test_invalid_frame_data() {
        let mut capture = ScreenCapture::new();

        // Wrong size data
        let frame_data = vec![255u8; 100];
        let result = capture.update(&frame_data, 10, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_matching() {
        let color_a = Rgba([100, 100, 100, 255]);
        let color_b = Rgba([105, 95, 100, 255]);

        assert!(color_matches(&color_a, &color_b, 10));
        assert!(!color_matches(&color_a, &color_b, 3));
    }
}
