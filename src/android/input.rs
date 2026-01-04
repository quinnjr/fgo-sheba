//! Touch event generation and input handling
//!
//! Generates touch events to be executed by the Android Accessibility Service.

use crate::vision::ScreenElement;
use crate::ShebaAction;

/// Screen coordinates for FGO UI elements
/// Based on 1920x1080 reference resolution
pub struct ScreenCoordinates {
    /// Reference width
    pub ref_width: u32,
    /// Reference height
    pub ref_height: u32,
    /// Current screen width
    pub screen_width: u32,
    /// Current screen height
    pub screen_height: u32,
}

impl ScreenCoordinates {
    /// Create new screen coordinates
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            ref_width: 1920,
            ref_height: 1080,
            screen_width,
            screen_height,
        }
    }

    /// Scale X coordinate from reference to actual screen
    pub fn scale_x(&self, x: i32) -> i32 {
        ((x as f32 * self.screen_width as f32) / self.ref_width as f32) as i32
    }

    /// Scale Y coordinate from reference to actual screen
    pub fn scale_y(&self, y: i32) -> i32 {
        ((y as f32 * self.screen_height as f32) / self.ref_height as f32) as i32
    }

    /// Get coordinates for a screen element
    pub fn get_element_coords(&self, element: ScreenElement) -> (i32, i32) {
        let (ref_x, ref_y) = match element {
            ScreenElement::AttackButton => (1700, 500),
            ScreenElement::Card(idx) => {
                let base_x = 180;
                let card_width = 300;
                (base_x + (idx as i32 * card_width) + card_width / 2, 880)
            }
            ScreenElement::NP(idx) => {
                let base_x = 380;
                let np_width = 280;
                (base_x + (idx as i32 * np_width) + np_width / 2, 320)
            }
            ScreenElement::Skill { servant, skill } => {
                let servant_x = 160 + (servant as i32 * 350);
                let skill_x = servant_x + (skill as i32 * 85);
                (skill_x, 950)
            }
            ScreenElement::MasterSkill(idx) => (1820, 340 + (idx as i32 * 80)),
            ScreenElement::Enemy(idx) => {
                let base_x = 400;
                let enemy_width = 350;
                (base_x + (idx as i32 * enemy_width) + enemy_width / 2, 150)
            }
            ScreenElement::DialogNext => (1800, 1000),
            ScreenElement::ResultNext => (1800, 1000),
        };

        (self.scale_x(ref_x), self.scale_y(ref_y))
    }

    /// Get card selection coordinates
    pub fn get_card_coords(&self, card_idx: usize) -> (i32, i32) {
        self.get_element_coords(ScreenElement::Card(card_idx))
    }

    /// Get NP selection coordinates
    pub fn get_np_coords(&self, servant_idx: usize) -> (i32, i32) {
        self.get_element_coords(ScreenElement::NP(servant_idx))
    }

    /// Get skill button coordinates
    pub fn get_skill_coords(&self, servant_idx: usize, skill_idx: usize) -> (i32, i32) {
        self.get_element_coords(ScreenElement::Skill {
            servant: servant_idx,
            skill: skill_idx,
        })
    }

    /// Get enemy target coordinates
    pub fn get_enemy_coords(&self, enemy_idx: usize) -> (i32, i32) {
        self.get_element_coords(ScreenElement::Enemy(enemy_idx))
    }

    /// Get attack button coordinates
    pub fn get_attack_button_coords(&self) -> (i32, i32) {
        self.get_element_coords(ScreenElement::AttackButton)
    }

    /// Get master skill coordinates
    pub fn get_master_skill_coords(&self, skill_idx: usize) -> (i32, i32) {
        self.get_element_coords(ScreenElement::MasterSkill(skill_idx))
    }

    /// Get servant target selection coordinates (when skill requires target)
    pub fn get_servant_target_coords(&self, servant_idx: usize) -> (i32, i32) {
        // Target selection appears in the middle of the screen
        let base_x = 400;
        let target_width = 380;
        let ref_x = base_x + (servant_idx as i32 * target_width) + target_width / 2;
        let ref_y = 540; // Middle of screen

        (self.scale_x(ref_x), self.scale_y(ref_y))
    }
}

impl Default for ScreenCoordinates {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

/// Input generator for creating touch sequences
pub struct InputGenerator {
    /// Screen coordinates
    coords: ScreenCoordinates,
    /// Delay between actions (ms)
    action_delay: u32,
}

impl InputGenerator {
    /// Create a new input generator
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            coords: ScreenCoordinates::new(screen_width, screen_height),
            action_delay: 200,
        }
    }

    /// Set action delay
    pub fn with_delay(mut self, delay_ms: u32) -> Self {
        self.action_delay = delay_ms;
        self
    }

    /// Generate tap action for a screen element
    pub fn tap_element(&self, element: ScreenElement) -> ShebaAction {
        let (x, y) = self.coords.get_element_coords(element);
        ShebaAction::Tap { x, y }
    }

    /// Generate tap action for coordinates
    pub fn tap_coords(&self, x: i32, y: i32) -> ShebaAction {
        ShebaAction::Tap {
            x: self.coords.scale_x(x),
            y: self.coords.scale_y(y),
        }
    }

    /// Generate action sequence for selecting cards
    pub fn select_cards(&self, card_indices: &[usize]) -> Vec<ShebaAction> {
        let mut actions = Vec::new();

        for &idx in card_indices {
            // Check if this is an NP (indices 5, 6, 7 correspond to servants 0, 1, 2)
            if idx >= 5 {
                let servant_idx = idx - 5;
                let (x, y) = self.coords.get_np_coords(servant_idx);
                actions.push(ShebaAction::Tap { x, y });
            } else {
                let (x, y) = self.coords.get_card_coords(idx);
                actions.push(ShebaAction::Tap { x, y });
            }

            actions.push(ShebaAction::Wait {
                duration_ms: self.action_delay,
            });
        }

        actions
    }

    /// Generate action sequence for using a skill
    pub fn use_skill(
        &self,
        servant_idx: usize,
        skill_idx: usize,
        target: Option<usize>,
    ) -> Vec<ShebaAction> {
        let mut actions = Vec::new();

        // Tap the skill
        let (x, y) = self.coords.get_skill_coords(servant_idx, skill_idx);
        actions.push(ShebaAction::Tap { x, y });
        actions.push(ShebaAction::Wait {
            duration_ms: self.action_delay,
        });

        // If skill requires target, tap the target
        if let Some(target_idx) = target {
            let (tx, ty) = self.coords.get_servant_target_coords(target_idx);
            actions.push(ShebaAction::Tap { x: tx, y: ty });
            actions.push(ShebaAction::Wait {
                duration_ms: self.action_delay,
            });
        }

        actions
    }

    /// Generate action sequence for using master skill
    pub fn use_master_skill(&self, skill_idx: usize, target: Option<usize>) -> Vec<ShebaAction> {
        let mut actions = Vec::new();

        // Open master skill menu (tap master skill icon)
        actions.push(ShebaAction::Tap {
            x: self.coords.scale_x(1880),
            y: self.coords.scale_y(440),
        });
        actions.push(ShebaAction::Wait {
            duration_ms: self.action_delay * 2,
        });

        // Tap the specific skill
        let (x, y) = self.coords.get_master_skill_coords(skill_idx);
        actions.push(ShebaAction::Tap { x, y });
        actions.push(ShebaAction::Wait {
            duration_ms: self.action_delay,
        });

        // If skill requires target, tap the target
        if let Some(target_idx) = target {
            let (tx, ty) = self.coords.get_servant_target_coords(target_idx);
            actions.push(ShebaAction::Tap { x: tx, y: ty });
            actions.push(ShebaAction::Wait {
                duration_ms: self.action_delay,
            });
        }

        actions
    }

    /// Generate action for targeting an enemy
    pub fn target_enemy(&self, enemy_idx: usize) -> ShebaAction {
        let (x, y) = self.coords.get_enemy_coords(enemy_idx);
        ShebaAction::Tap { x, y }
    }

    /// Generate action for tapping attack button
    pub fn tap_attack(&self) -> ShebaAction {
        let (x, y) = self.coords.get_attack_button_coords();
        ShebaAction::Tap { x, y }
    }

    /// Generate swipe action
    pub fn swipe(
        &self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        duration_ms: u32,
    ) -> ShebaAction {
        ShebaAction::Swipe {
            start_x: self.coords.scale_x(start_x),
            start_y: self.coords.scale_y(start_y),
            end_x: self.coords.scale_x(end_x),
            end_y: self.coords.scale_y(end_y),
            duration_ms,
        }
    }

    /// Generate scroll action
    pub fn scroll_down(&self) -> ShebaAction {
        self.swipe(960, 800, 960, 400, 500)
    }

    /// Generate scroll up action
    pub fn scroll_up(&self) -> ShebaAction {
        self.swipe(960, 400, 960, 800, 500)
    }
}

impl Default for InputGenerator {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_scaling() {
        let coords = ScreenCoordinates::new(2560, 1440);

        // 1920x1080 reference -> 2560x1440 actual
        let scaled_x = coords.scale_x(960); // Center X
        let scaled_y = coords.scale_y(540); // Center Y

        assert_eq!(scaled_x, 1280);
        assert_eq!(scaled_y, 720);
    }

    #[test]
    fn test_card_coords() {
        let coords = ScreenCoordinates::new(1920, 1080);

        let (x0, y0) = coords.get_card_coords(0);
        let (x1, y1) = coords.get_card_coords(1);

        // Cards should be horizontally spaced
        assert!(x1 > x0);
        // Y should be the same
        assert_eq!(y0, y1);
    }

    #[test]
    fn test_generator() {
        let gen = InputGenerator::new(1920, 1080);

        let action = gen.tap_attack();
        match action {
            ShebaAction::Tap { x, y } => {
                assert!(x > 1500); // Right side of screen
                assert!(y > 400 && y < 600); // Middle-ish
            }
            _ => panic!("Expected tap action"),
        }
    }

    #[test]
    fn test_card_selection_sequence() {
        let gen = InputGenerator::new(1920, 1080);

        let actions = gen.select_cards(&[0, 1, 2]);

        // Should have 6 actions (3 taps + 3 waits)
        assert_eq!(actions.len(), 6);

        // First action should be a tap
        assert!(matches!(actions[0], ShebaAction::Tap { .. }));
        // Second action should be a wait
        assert!(matches!(actions[1], ShebaAction::Wait { .. }));
    }
}
