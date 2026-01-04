//! FGO Sheba - AI-powered Fate/Grand Order automation for Android
//!
//! This library provides the core functionality for automating FGO gameplay,
//! including screen capture processing, card recognition, battle AI, and
//! touch input generation.
//!
//! ## Anti-Detection
//!
//! The `stealth` module provides humanization features to make automation
//! less detectable by adding realistic variance to timing and positions.

pub mod ai;
pub mod android;
pub mod config;
pub mod game;
pub mod stealth;
pub mod vision;

use once_cell::sync::OnceCell;
use std::sync::Mutex;

use crate::ai::BattleAI;
use crate::config::Settings;
use crate::game::state::GameState;
use crate::vision::VisionSystem;

/// Global application state
pub struct Sheba {
    pub vision: VisionSystem,
    pub ai: BattleAI,
    pub game_state: GameState,
    pub settings: Settings,
}

impl Sheba {
    /// Create a new Sheba instance with the given settings
    pub fn new(settings: Settings) -> Self {
        Self {
            vision: VisionSystem::new(),
            ai: BattleAI::new(),
            game_state: GameState::new(),
            settings,
        }
    }

    /// Process a frame from the screen capture
    pub fn process_frame(&mut self, frame_data: &[u8], width: u32, height: u32) -> ShebaAction {
        // Update vision system with new frame
        if let Err(e) = self.vision.update_frame(frame_data, width, height) {
            log::error!("Failed to process frame: {}", e);
            return ShebaAction::None;
        }

        // Detect current UI state
        let ui_state = self.vision.detect_ui_state();
        self.game_state.update_ui_state(ui_state);

        // If in battle, process battle logic
        if self.game_state.is_in_battle() {
            self.process_battle()
        } else {
            ShebaAction::None
        }
    }

    /// Process battle logic and return the next action
    fn process_battle(&mut self) -> ShebaAction {
        // Get current battle state from vision
        let battle_info = self.vision.analyze_battle_screen();

        // Update game state with battle info
        self.game_state.update_battle_state(&battle_info);

        // Get AI decision
        if let Some(battle_state) = self.game_state.battle_state() {
            self.ai.decide_action(battle_state, &self.settings)
        } else {
            ShebaAction::None
        }
    }
}

/// Actions that Sheba can perform
#[derive(Debug, Clone)]
pub enum ShebaAction {
    /// No action needed
    None,
    /// Tap at a specific screen coordinate
    Tap { x: i32, y: i32 },
    /// Swipe from one point to another
    Swipe {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        duration_ms: u32,
    },
    /// Wait for a specified duration
    Wait { duration_ms: u32 },
    /// Select cards in order (indices 0-4)
    SelectCards { card_indices: Vec<usize> },
    /// Use a skill (servant index 0-2, skill index 0-2, optional target 0-2)
    UseSkill {
        servant_idx: usize,
        skill_idx: usize,
        target: Option<usize>,
    },
    /// Use Noble Phantasm (servant index 0-2)
    UseNP { servant_idx: usize },
    /// Target an enemy (index 0-2)
    TargetEnemy { enemy_idx: usize },
    /// Tap the Attack button to enter card selection
    TapAttack,
    /// Confirm Master Skill usage
    UseMasterSkill { skill_idx: usize, target: Option<usize> },
}

/// Global Sheba instance for JNI access
static SHEBA_INSTANCE: OnceCell<Mutex<Sheba>> = OnceCell::new();

/// Initialize the global Sheba instance
pub fn init_sheba(settings: Settings) {
    let _ = SHEBA_INSTANCE.set(Mutex::new(Sheba::new(settings)));
}

/// Get a reference to the global Sheba instance
pub fn get_sheba() -> Option<&'static Mutex<Sheba>> {
    SHEBA_INSTANCE.get()
}
