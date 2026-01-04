//! ONNX model loading and inference
//!
//! Manages ML models for card recognition, UI detection, and enemy detection.

use image::RgbaImage;
use std::path::Path;

use crate::game::cards::CardType;
use crate::game::enemy::{Enemy, EnemyClass};
use crate::game::servant::{Servant, ServantClass};
use crate::game::state::UIState;

use super::VisionError;

/// Manages all ML models for the vision system
pub struct ModelManager {
    /// Card classifier model
    card_classifier: Option<CardClassifier>,
    /// UI state classifier model
    ui_classifier: Option<UIClassifier>,
    /// Enemy detector model
    enemy_detector: Option<EnemyDetector>,
    /// Whether models are loaded
    models_loaded: bool,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new() -> Self {
        Self {
            card_classifier: None,
            ui_classifier: None,
            enemy_detector: None,
            models_loaded: false,
        }
    }

    /// Load all models from a directory
    pub fn load_models(&mut self, model_dir: &str) -> Result<(), VisionError> {
        let path = Path::new(model_dir);

        // Try to load card classifier
        let card_model_path = path.join("card_classifier.onnx");
        if card_model_path.exists() {
            self.card_classifier = Some(CardClassifier::load(&card_model_path)?);
        }

        // Try to load UI classifier
        let ui_model_path = path.join("ui_state.onnx");
        if ui_model_path.exists() {
            self.ui_classifier = Some(UIClassifier::load(&ui_model_path)?);
        }

        // Try to load enemy detector
        let enemy_model_path = path.join("enemy_detector.onnx");
        if enemy_model_path.exists() {
            self.enemy_detector = Some(EnemyDetector::load(&enemy_model_path)?);
        }

        self.models_loaded = true;
        Ok(())
    }

    /// Check if models are loaded
    pub fn is_loaded(&self) -> bool {
        self.models_loaded
    }

    /// Classify a card image
    pub fn classify_card(&self, image: &RgbaImage) -> (CardType, f32) {
        if let Some(ref classifier) = self.card_classifier {
            classifier.classify(image)
        } else {
            // Fallback to color-based classification
            classify_card_by_color(image)
        }
    }

    /// Classify the UI state
    pub fn classify_ui_state(&self, image: &RgbaImage) -> UIState {
        if let Some(ref classifier) = self.ui_classifier {
            classifier.classify(image)
        } else {
            // Fallback to heuristic detection
            detect_ui_state_heuristic(image)
        }
    }

    /// Detect enemies in the screen
    pub fn detect_enemies(&self, image: &RgbaImage) -> Vec<Enemy> {
        if let Some(ref detector) = self.enemy_detector {
            detector.detect(image)
        } else {
            // Return empty for now - need model for accurate detection
            Vec::new()
        }
    }

    /// Detect servants on screen
    pub fn detect_servants(&self, _image: &RgbaImage) -> Vec<Servant> {
        // For now, return default servants
        // In practice, this would use portrait matching
        vec![
            Servant::new(ServantClass::Unknown, 0),
            Servant::new(ServantClass::Unknown, 1),
            Servant::new(ServantClass::Unknown, 2),
        ]
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Card classifier using ONNX model
pub struct CardClassifier {
    // TODO: Add ONNX session when ort is properly configured
    _placeholder: (),
}

impl CardClassifier {
    /// Load the model from a file
    pub fn load(_path: &Path) -> Result<Self, VisionError> {
        // TODO: Implement actual model loading with ort
        // For now, return a placeholder
        Ok(Self { _placeholder: () })
    }

    /// Classify a card image
    pub fn classify(&self, image: &RgbaImage) -> (CardType, f32) {
        // TODO: Implement actual inference
        // For now, use color-based fallback
        classify_card_by_color(image)
    }
}

/// UI state classifier using ONNX model
pub struct UIClassifier {
    _placeholder: (),
}

impl UIClassifier {
    /// Load the model from a file
    pub fn load(_path: &Path) -> Result<Self, VisionError> {
        Ok(Self { _placeholder: () })
    }

    /// Classify the UI state
    pub fn classify(&self, image: &RgbaImage) -> UIState {
        detect_ui_state_heuristic(image)
    }
}

/// Enemy detector using ONNX model
pub struct EnemyDetector {
    _placeholder: (),
}

impl EnemyDetector {
    /// Load the model from a file
    pub fn load(_path: &Path) -> Result<Self, VisionError> {
        Ok(Self { _placeholder: () })
    }

    /// Detect enemies in the image
    pub fn detect(&self, _image: &RgbaImage) -> Vec<Enemy> {
        // TODO: Implement actual detection
        Vec::new()
    }
}

/// Classify a card by its dominant color (fallback method)
fn classify_card_by_color(image: &RgbaImage) -> (CardType, f32) {
    let (width, height) = image.dimensions();

    // Sample the center region
    let center_x = width / 2;
    let center_y = height / 2;
    let sample_size = 20;

    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;
    let mut count: u64 = 0;

    for y in (center_y.saturating_sub(sample_size))..(center_y + sample_size).min(height) {
        for x in (center_x.saturating_sub(sample_size))..(center_x + sample_size).min(width) {
            let pixel = image.get_pixel(x, y);
            total_r += pixel[0] as u64;
            total_g += pixel[1] as u64;
            total_b += pixel[2] as u64;
            count += 1;
        }
    }

    if count == 0 {
        return (CardType::Unknown, 0.0);
    }

    let avg_r = (total_r / count) as u8;
    let avg_g = (total_g / count) as u8;
    let avg_b = (total_b / count) as u8;

    // Classify based on dominant color
    // Buster: Red
    // Arts: Blue
    // Quick: Green

    let is_red = avg_r > 150 && avg_r > avg_g + 30 && avg_r > avg_b + 30;
    let is_blue = avg_b > 150 && avg_b > avg_r + 30 && avg_b > avg_g + 30;
    let is_green = avg_g > 150 && avg_g > avg_r + 30 && avg_g > avg_b + 30;

    if is_red {
        (CardType::Buster, 0.7)
    } else if is_blue {
        (CardType::Arts, 0.7)
    } else if is_green {
        (CardType::Quick, 0.7)
    } else {
        (CardType::Unknown, 0.3)
    }
}

/// Detect UI state using heuristics (fallback method)
fn detect_ui_state_heuristic(image: &RgbaImage) -> UIState {
    let (width, height) = image.dimensions();

    // Check for attack button (bottom right, red-ish)
    let attack_region = sample_region(image, width - 200, height - 200, 100, 100);
    if is_reddish(&attack_region) {
        return UIState::BattleCommand;
    }

    // Check for card selection (bottom area with colorful cards)
    let card_region = sample_region(image, width / 4, height - 300, width / 2, 200);
    if is_colorful(&card_region) {
        return UIState::BattleCards;
    }

    UIState::Unknown
}

/// Sample average color of a region
fn sample_region(image: &RgbaImage, x: u32, y: u32, w: u32, h: u32) -> (u8, u8, u8) {
    let (img_w, img_h) = image.dimensions();
    let x = x.min(img_w.saturating_sub(1));
    let y = y.min(img_h.saturating_sub(1));
    let w = w.min(img_w - x);
    let h = h.min(img_h - y);

    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;
    let mut count: u64 = 0;

    for py in y..(y + h) {
        for px in x..(x + w) {
            let pixel = image.get_pixel(px, py);
            total_r += pixel[0] as u64;
            total_g += pixel[1] as u64;
            total_b += pixel[2] as u64;
            count += 1;
        }
    }

    if count == 0 {
        return (0, 0, 0);
    }

    (
        (total_r / count) as u8,
        (total_g / count) as u8,
        (total_b / count) as u8,
    )
}

/// Check if a color is reddish
fn is_reddish(color: &(u8, u8, u8)) -> bool {
    color.0 > 150 && color.0 > color.1 + 30 && color.0 > color.2 + 30
}

/// Check if a region is colorful (high variance)
fn is_colorful(color: &(u8, u8, u8)) -> bool {
    let max = color.0.max(color.1).max(color.2);
    let min = color.0.min(color.1).min(color.2);
    (max - min) > 50
}

/// Class icon colors for enemy detection
pub mod class_colors {
    use image::Rgba;

    /// Saber class icon color (blue/white)
    pub const SABER: Rgba<u8> = Rgba([100, 150, 255, 255]);
    /// Archer class icon color (red)
    pub const ARCHER: Rgba<u8> = Rgba([255, 100, 100, 255]);
    /// Lancer class icon color (blue)
    pub const LANCER: Rgba<u8> = Rgba([100, 100, 255, 255]);
    /// Rider class icon color (pink)
    pub const RIDER: Rgba<u8> = Rgba([255, 150, 200, 255]);
    /// Caster class icon color (purple)
    pub const CASTER: Rgba<u8> = Rgba([200, 100, 255, 255]);
    /// Assassin class icon color (dark purple)
    pub const ASSASSIN: Rgba<u8> = Rgba([100, 50, 150, 255]);
    /// Berserker class icon color (dark red)
    pub const BERSERKER: Rgba<u8> = Rgba([150, 50, 50, 255]);
}

/// Map a class color to EnemyClass
pub fn color_to_class(r: u8, g: u8, b: u8) -> EnemyClass {
    // Simple heuristic based on dominant color
    if r > 200 && g < 150 && b < 150 {
        EnemyClass::Archer
    } else if b > 200 && r < 150 {
        EnemyClass::Lancer
    } else if r > 150 && g > 100 && b > 200 {
        EnemyClass::Caster
    } else if r < 100 && g < 100 && b < 150 {
        EnemyClass::Assassin
    } else if r > 150 && r < 200 && g < 100 && b < 100 {
        EnemyClass::Berserker
    } else {
        EnemyClass::Unknown
    }
}
