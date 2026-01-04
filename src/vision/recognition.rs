//! Card and servant recognition
//!
//! Handles the recognition of command cards and matching them to servants.

use image::RgbaImage;

use super::capture::regions;
use super::models::ModelManager;
use crate::game::cards::{Card, CardType};

/// Card recognizer that uses ML models to identify cards
pub struct CardRecognizer {
    /// Confidence threshold for card recognition
    confidence_threshold: f32,
}

impl CardRecognizer {
    /// Create a new card recognizer
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.5,
        }
    }

    /// Set the confidence threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Recognize all cards on the battle screen
    pub fn recognize_cards(&self, frame: &RgbaImage, models: &ModelManager) -> Vec<Card> {
        let mut cards = Vec::new();
        let (frame_width, frame_height) = frame.dimensions();

        // Reference resolution for coordinates
        let ref_width = 1920;
        let ref_height = 1080;

        // Scale factor
        let scale_x = frame_width as f32 / ref_width as f32;
        let scale_y = frame_height as f32 / ref_height as f32;

        // Extract and classify each card region
        for (position, &(x, y, w, h)) in regions::CARD_REGIONS.iter().enumerate() {
            // Scale coordinates
            let scaled_x = (x as f32 * scale_x) as u32;
            let scaled_y = (y as f32 * scale_y) as u32;
            let scaled_w = (w as f32 * scale_x) as u32;
            let scaled_h = (h as f32 * scale_y) as u32;

            // Validate bounds
            if scaled_x + scaled_w > frame_width || scaled_y + scaled_h > frame_height {
                continue;
            }

            // Extract card region
            let card_image =
                image::imageops::crop_imm(frame, scaled_x, scaled_y, scaled_w, scaled_h).to_image();

            // Classify the card
            let (card_type, confidence) = models.classify_card(&card_image);

            if confidence >= self.confidence_threshold && card_type != CardType::Unknown {
                // Try to determine which servant this card belongs to
                let servant_idx = self.match_card_to_servant(&card_image, position);

                cards.push(Card {
                    card_type,
                    servant_idx,
                    position,
                    confidence,
                });
            }
        }

        cards
    }

    /// Match a card to a servant based on the portrait/border
    fn match_card_to_servant(&self, _card_image: &RgbaImage, position: usize) -> usize {
        // TODO: Implement proper servant matching using portrait comparison
        // For now, use a simple heuristic based on position
        // In FGO, cards are somewhat grouped by servant
        position % 3
    }

    /// Recognize NP availability
    pub fn recognize_np_cards(&self, frame: &RgbaImage) -> [bool; 3] {
        let mut np_available = [false; 3];
        let (frame_width, frame_height) = frame.dimensions();

        let ref_width = 1920;
        let ref_height = 1080;
        let scale_x = frame_width as f32 / ref_width as f32;
        let scale_y = frame_height as f32 / ref_height as f32;

        for (idx, &(x, y, w, h)) in regions::NP_REGIONS.iter().enumerate() {
            let scaled_x = (x as f32 * scale_x) as u32;
            let scaled_y = (y as f32 * scale_y) as u32;
            let scaled_w = (w as f32 * scale_x) as u32;
            let scaled_h = (h as f32 * scale_y) as u32;

            if scaled_x + scaled_w > frame_width || scaled_y + scaled_h > frame_height {
                continue;
            }

            // Check if NP region is lit up (not grayed out)
            let np_region =
                image::imageops::crop_imm(frame, scaled_x, scaled_y, scaled_w, scaled_h).to_image();

            np_available[idx] = self.is_np_available(&np_region);
        }

        np_available
    }

    /// Check if an NP card region indicates the NP is available
    fn is_np_available(&self, np_image: &RgbaImage) -> bool {
        // NPs that are available have brighter colors
        // Grayed out NPs are darker
        let (width, height) = np_image.dimensions();
        let mut brightness_sum: u64 = 0;
        let mut count: u64 = 0;

        // Sample the center of the NP region
        let sample_start_x = width / 4;
        let sample_end_x = 3 * width / 4;
        let sample_start_y = height / 4;
        let sample_end_y = 3 * height / 4;

        for y in sample_start_y..sample_end_y {
            for x in sample_start_x..sample_end_x {
                let pixel = np_image.get_pixel(x, y);
                // Calculate perceived brightness
                let brightness =
                    (pixel[0] as u64 * 299 + pixel[1] as u64 * 587 + pixel[2] as u64 * 114) / 1000;
                brightness_sum += brightness;
                count += 1;
            }
        }

        if count == 0 {
            return false;
        }

        let avg_brightness = brightness_sum / count;
        // Threshold for "available" - adjust based on testing
        avg_brightness > 100
    }
}

impl Default for CardRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Servant portrait matcher for card-to-servant matching
pub struct ServantMatcher {
    /// Stored servant portraits for matching
    portraits: Vec<RgbaImage>,
}

impl ServantMatcher {
    /// Create a new servant matcher
    pub fn new() -> Self {
        Self {
            portraits: Vec::new(),
        }
    }

    /// Store a servant portrait for matching
    pub fn add_portrait(&mut self, portrait: RgbaImage) {
        self.portraits.push(portrait);
    }

    /// Clear stored portraits
    pub fn clear_portraits(&mut self) {
        self.portraits.clear();
    }

    /// Match a card border/portrait to stored servants
    pub fn match_portrait(&self, card_portrait: &RgbaImage) -> Option<usize> {
        if self.portraits.is_empty() {
            return None;
        }

        let mut best_match = 0;
        let mut best_score = 0.0f32;

        for (idx, portrait) in self.portraits.iter().enumerate() {
            let score = self.compare_portraits(card_portrait, portrait);
            if score > best_score {
                best_score = score;
                best_match = idx;
            }
        }

        // Only return a match if confidence is high enough
        if best_score > 0.7 {
            Some(best_match)
        } else {
            None
        }
    }

    /// Compare two portraits and return a similarity score (0-1)
    fn compare_portraits(&self, a: &RgbaImage, b: &RgbaImage) -> f32 {
        let (_w_a, _h_a) = a.dimensions();
        let (_w_b, _h_b) = b.dimensions();

        // Resize to common size for comparison
        let target_size = 50;
        let resized_a = image::imageops::resize(
            a,
            target_size,
            target_size,
            image::imageops::FilterType::Nearest,
        );
        let resized_b = image::imageops::resize(
            b,
            target_size,
            target_size,
            image::imageops::FilterType::Nearest,
        );

        // Calculate pixel-wise similarity
        let mut total_diff: u64 = 0;
        let mut count: u64 = 0;

        for y in 0..target_size {
            for x in 0..target_size {
                let pixel_a = resized_a.get_pixel(x, y);
                let pixel_b = resized_b.get_pixel(x, y);

                let diff_r = (pixel_a[0] as i32 - pixel_b[0] as i32).unsigned_abs() as u64;
                let diff_g = (pixel_a[1] as i32 - pixel_b[1] as i32).unsigned_abs() as u64;
                let diff_b = (pixel_a[2] as i32 - pixel_b[2] as i32).unsigned_abs() as u64;

                total_diff += diff_r + diff_g + diff_b;
                count += 3;
            }
        }

        if count == 0 {
            return 0.0;
        }

        // Convert to similarity (0-1)
        let avg_diff = total_diff as f32 / count as f32;
        1.0 - (avg_diff / 255.0)
    }
}

impl Default for ServantMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_card_recognizer_creation() {
        let recognizer = CardRecognizer::new();
        assert_eq!(recognizer.confidence_threshold, 0.5);

        let recognizer = CardRecognizer::new().with_threshold(0.8);
        assert_eq!(recognizer.confidence_threshold, 0.8);
    }

    #[test]
    fn test_np_availability_detection() {
        let recognizer = CardRecognizer::new();

        // Create a bright image (NP available)
        let bright: RgbaImage = ImageBuffer::from_fn(100, 100, |_, _| Rgba([200, 200, 200, 255]));
        assert!(recognizer.is_np_available(&bright));

        // Create a dark image (NP not available)
        let dark: RgbaImage = ImageBuffer::from_fn(100, 100, |_, _| Rgba([50, 50, 50, 255]));
        assert!(!recognizer.is_np_available(&dark));
    }

    #[test]
    fn test_servant_matcher() {
        let mut matcher = ServantMatcher::new();

        // Add a red portrait
        let red_portrait: RgbaImage = ImageBuffer::from_fn(50, 50, |_, _| Rgba([255, 0, 0, 255]));
        matcher.add_portrait(red_portrait);

        // Add a blue portrait
        let blue_portrait: RgbaImage = ImageBuffer::from_fn(50, 50, |_, _| Rgba([0, 0, 255, 255]));
        matcher.add_portrait(blue_portrait);

        // Test matching a red card
        let red_card: RgbaImage = ImageBuffer::from_fn(50, 50, |_, _| Rgba([250, 10, 10, 255]));
        let result = matcher.match_portrait(&red_card);
        assert_eq!(result, Some(0));

        // Test matching a blue card
        let blue_card: RgbaImage = ImageBuffer::from_fn(50, 50, |_, _| Rgba([10, 10, 250, 255]));
        let result = matcher.match_portrait(&blue_card);
        assert_eq!(result, Some(1));
    }
}
