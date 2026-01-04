//! Vision and image processing module
//!
//! Handles screen capture processing, card recognition, enemy detection,
//! and OCR for game state analysis.

pub mod capture;
pub mod models;
pub mod ocr;
pub mod recognition;

use crate::game::cards::Card;
use crate::game::enemy::Enemy;
use crate::game::servant::Servant;
use crate::game::state::UIState;

pub use capture::ScreenCapture;
pub use models::ModelManager;
pub use recognition::CardRecognizer;

/// Result of analyzing a battle screen
#[derive(Debug, Clone, Default)]
pub struct BattleInfo {
    /// Current wave number
    pub wave_number: u32,
    /// Total waves
    pub total_waves: u32,
    /// Detected servants
    pub servants: Vec<Servant>,
    /// Detected enemies
    pub enemies: Vec<Enemy>,
    /// Available command cards (in card selection)
    pub available_cards: Vec<Card>,
    /// Critical stars available
    pub critical_stars: u32,
    /// Whether attack button is visible
    pub attack_button_visible: bool,
    /// Whether skills are visible
    pub skills_visible: bool,
}

/// Main vision system that coordinates all recognition
pub struct VisionSystem {
    /// Screen capture handler
    capture: ScreenCapture,
    /// ML model manager
    models: ModelManager,
    /// Card recognizer
    card_recognizer: CardRecognizer,
    /// Last detected UI state
    last_ui_state: UIState,
}

impl VisionSystem {
    /// Create a new vision system
    pub fn new() -> Self {
        Self {
            capture: ScreenCapture::new(),
            models: ModelManager::new(),
            card_recognizer: CardRecognizer::new(),
            last_ui_state: UIState::Unknown,
        }
    }

    /// Initialize the vision system with model paths
    pub fn init(&mut self, model_dir: &str) -> Result<(), VisionError> {
        self.models.load_models(model_dir)?;
        Ok(())
    }

    /// Update with a new frame from screen capture
    pub fn update_frame(
        &mut self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), VisionError> {
        self.capture.update(frame_data, width, height)
    }

    /// Detect the current UI state from the screen
    pub fn detect_ui_state(&mut self) -> UIState {
        // Get the current frame
        let Some(frame) = self.capture.current_frame() else {
            return UIState::Unknown;
        };

        // Use model to classify UI state
        let state = self.models.classify_ui_state(frame);
        self.last_ui_state = state;
        state
    }

    /// Analyze the battle screen to extract game state
    pub fn analyze_battle_screen(&self) -> BattleInfo {
        let Some(frame) = self.capture.current_frame() else {
            return BattleInfo::default();
        };

        let mut info = BattleInfo::default();

        // Detect wave info
        if let Some((current, total)) = self.detect_wave_info(frame) {
            info.wave_number = current;
            info.total_waves = total;
        }

        // Detect servants
        info.servants = self.detect_servants(frame);

        // Detect enemies
        info.enemies = self.detect_enemies(frame);

        // Detect available cards (if in card selection)
        if self.last_ui_state == UIState::BattleCards {
            info.available_cards = self.card_recognizer.recognize_cards(frame, &self.models);
        }

        // Detect critical stars
        info.critical_stars = self.detect_critical_stars(frame);

        // Check for attack button
        info.attack_button_visible = self.detect_attack_button(frame);

        // Check for skills
        info.skills_visible = self.detect_skills_visible(frame);

        info
    }

    /// Detect wave information (current wave / total waves)
    fn detect_wave_info(&self, _frame: &image::RgbaImage) -> Option<(u32, u32)> {
        // TODO: Implement OCR-based wave detection
        // For now, return default
        Some((1, 3))
    }

    /// Detect servants on screen
    fn detect_servants(&self, frame: &image::RgbaImage) -> Vec<Servant> {
        self.models.detect_servants(frame)
    }

    /// Detect enemies on screen
    fn detect_enemies(&self, frame: &image::RgbaImage) -> Vec<Enemy> {
        self.models.detect_enemies(frame)
    }

    /// Detect critical stars count
    fn detect_critical_stars(&self, _frame: &image::RgbaImage) -> u32 {
        // TODO: Implement OCR for star count
        0
    }

    /// Detect if attack button is visible
    fn detect_attack_button(&self, _frame: &image::RgbaImage) -> bool {
        // TODO: Implement template matching for attack button
        false
    }

    /// Detect if servant skills are visible
    fn detect_skills_visible(&self, _frame: &image::RgbaImage) -> bool {
        // TODO: Implement detection
        false
    }

    /// Get the last detected UI state
    pub fn last_ui_state(&self) -> UIState {
        self.last_ui_state
    }

    /// Get screen coordinates for a specific element
    pub fn get_element_coords(&self, element: ScreenElement) -> Option<(i32, i32)> {
        // Return predefined coordinates based on element type
        // These are calibrated for standard FGO screen layout
        match element {
            ScreenElement::AttackButton => Some((1700, 500)),
            ScreenElement::Card(idx) => {
                let base_x = 180;
                let card_width = 300;
                Some((base_x + (idx as i32 * card_width), 850))
            }
            ScreenElement::NP(idx) => {
                let base_x = 380;
                let np_width = 280;
                Some((base_x + (idx as i32 * np_width), 280))
            }
            ScreenElement::Skill { servant, skill } => {
                let servant_x = 160 + (servant as i32 * 350);
                let skill_x = servant_x + (skill as i32 * 85);
                Some((skill_x, 950))
            }
            ScreenElement::MasterSkill(idx) => {
                let base_x = 1800;
                Some((base_x, 340 + (idx as i32 * 80)))
            }
            ScreenElement::Enemy(idx) => {
                let base_x = 400;
                let enemy_width = 350;
                Some((base_x + (idx as i32 * enemy_width), 150))
            }
            ScreenElement::DialogNext => Some((1800, 1000)),
            ScreenElement::ResultNext => Some((1800, 1000)),
        }
    }
}

impl Default for VisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen elements that can be tapped
#[derive(Debug, Clone, Copy)]
pub enum ScreenElement {
    /// Attack button
    AttackButton,
    /// Command card (index 0-4)
    Card(usize),
    /// Noble Phantasm (servant index 0-2)
    NP(usize),
    /// Servant skill (servant index 0-2, skill index 0-2)
    Skill { servant: usize, skill: usize },
    /// Master skill (index 0-2)
    MasterSkill(usize),
    /// Enemy (index 0-2)
    Enemy(usize),
    /// Dialog next button
    DialogNext,
    /// Result screen next button
    ResultNext,
}

/// Vision system errors
#[derive(Debug, thiserror::Error)]
pub enum VisionError {
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    #[error("Failed to process image: {0}")]
    ImageProcessingError(String),
    #[error("Invalid frame data")]
    InvalidFrameData,
    #[error("Model inference failed: {0}")]
    InferenceError(String),
}
