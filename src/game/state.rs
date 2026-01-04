//! Game state representation
//!
//! Tracks the overall game state including current screen, battle state,
//! and navigation context.

use serde::{Deserialize, Serialize};

use super::battle::BattleState;
use crate::vision::BattleInfo;

/// Current UI state/screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIState {
    /// Unknown screen
    Unknown,
    /// Main menu / My Room
    MainMenu,
    /// Quest selection screen
    QuestSelection,
    /// Party setup screen
    PartySetup,
    /// Support selection screen
    SupportSelection,
    /// Loading screen
    Loading,
    /// In battle - command phase
    BattleCommand,
    /// In battle - card selection
    BattleCards,
    /// In battle - attack animation
    BattleAttack,
    /// In battle - enemy turn
    BattleEnemy,
    /// Battle result screen
    BattleResult,
    /// Friend point summon result
    FPSummonResult,
    /// Bond/EXP result screen
    BondResult,
    /// Quest complete screen
    QuestComplete,
    /// Item drop screen
    ItemDrops,
    /// Dialog/story scene
    Dialog,
    /// Connection error
    ConnectionError,
    /// AP recovery dialog
    APRecovery,
    /// Mystic Code selection
    MysticCodeSelect,
}

impl UIState {
    /// Check if this is a battle-related state
    pub fn is_battle(&self) -> bool {
        matches!(
            self,
            UIState::BattleCommand
                | UIState::BattleCards
                | UIState::BattleAttack
                | UIState::BattleEnemy
        )
    }

    /// Check if this is a result/reward screen
    pub fn is_result_screen(&self) -> bool {
        matches!(
            self,
            UIState::BattleResult
                | UIState::BondResult
                | UIState::QuestComplete
                | UIState::ItemDrops
        )
    }

    /// Check if interaction is expected in this state
    pub fn requires_input(&self) -> bool {
        matches!(
            self,
            UIState::BattleCommand
                | UIState::BattleCards
                | UIState::PartySetup
                | UIState::SupportSelection
                | UIState::Dialog
                | UIState::APRecovery
                | UIState::BattleResult
                | UIState::BondResult
                | UIState::QuestComplete
                | UIState::ItemDrops
        )
    }
}

/// Overall game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current UI state
    pub ui_state: UIState,
    /// Previous UI state (for transition detection)
    pub previous_ui_state: UIState,
    /// Current battle state (if in battle)
    pub battle: Option<BattleState>,
    /// Number of frames in current UI state
    pub frames_in_state: u32,
    /// Whether automation is paused
    pub is_paused: bool,
    /// Current quest name (if known)
    pub quest_name: Option<String>,
    /// Farming loop count
    pub loop_count: u32,
    /// Total runs completed
    pub runs_completed: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ui_state: UIState::Unknown,
            previous_ui_state: UIState::Unknown,
            battle: None,
            frames_in_state: 0,
            is_paused: false,
            quest_name: None,
            loop_count: 0,
            runs_completed: 0,
        }
    }
}

impl GameState {
    /// Create a new game state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the UI state
    pub fn update_ui_state(&mut self, new_state: UIState) {
        if new_state != self.ui_state {
            self.previous_ui_state = self.ui_state;
            self.ui_state = new_state;
            self.frames_in_state = 0;

            // Handle state transitions
            self.on_state_transition(self.previous_ui_state, new_state);
        } else {
            self.frames_in_state += 1;
        }
    }

    /// Handle state transitions
    fn on_state_transition(&mut self, from: UIState, to: UIState) {
        match (from, to) {
            // Entering battle
            (_, UIState::BattleCommand) if !from.is_battle() => {
                self.battle = Some(BattleState::new());
            }
            // Battle complete
            (UIState::BattleAttack, UIState::BattleResult) => {
                if let Some(ref mut battle) = self.battle {
                    battle.phase = super::battle::BattlePhase::Victory;
                }
            }
            // Quest complete
            (UIState::BattleResult | UIState::BondResult | UIState::ItemDrops, UIState::QuestComplete) => {
                self.runs_completed += 1;
                self.battle = None;
            }
            // Leaving quest complete
            (UIState::QuestComplete, UIState::MainMenu | UIState::QuestSelection) => {
                self.loop_count += 1;
            }
            _ => {}
        }
    }

    /// Check if currently in battle
    pub fn is_in_battle(&self) -> bool {
        self.ui_state.is_battle()
    }

    /// Get the current battle state
    pub fn battle_state(&self) -> Option<&BattleState> {
        self.battle.as_ref()
    }

    /// Get mutable battle state
    pub fn battle_state_mut(&mut self) -> Option<&mut BattleState> {
        self.battle.as_mut()
    }

    /// Update battle state from vision analysis
    pub fn update_battle_state(&mut self, info: &BattleInfo) {
        if let Some(ref mut battle) = self.battle {
            // Update wave info
            battle.current_wave.wave_number = info.wave_number;
            battle.total_waves = info.total_waves;
            battle.current_wave.total_waves = info.total_waves;

            // Update servants
            battle.servants = info.servants.clone();

            // Update enemies
            battle.current_wave.enemies = info.enemies.clone();

            // Update cards if in card selection
            if !info.available_cards.is_empty() {
                battle.enter_card_selection(info.available_cards.clone());
            }

            // Update critical stars
            battle.critical_stars = info.critical_stars;

            // Update NP availability
            for (i, servant) in battle.servants.iter().enumerate() {
                if i < 3 {
                    battle.np_available[i] = servant.can_np();
                }
            }
        }
    }

    /// Pause automation
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume automation
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Check if the game seems stuck (many frames in same state)
    pub fn is_stuck(&self, threshold: u32) -> bool {
        self.frames_in_state > threshold && self.ui_state.requires_input()
    }

    /// Reset for a new farming loop
    pub fn reset_for_new_run(&mut self) {
        self.battle = None;
        self.frames_in_state = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut state = GameState::new();

        state.update_ui_state(UIState::MainMenu);
        assert_eq!(state.ui_state, UIState::MainMenu);
        assert_eq!(state.frames_in_state, 0);

        state.update_ui_state(UIState::MainMenu);
        assert_eq!(state.frames_in_state, 1);

        state.update_ui_state(UIState::BattleCommand);
        assert!(state.battle.is_some());
    }

    #[test]
    fn test_battle_detection() {
        let mut state = GameState::new();
        assert!(!state.is_in_battle());

        state.update_ui_state(UIState::BattleCommand);
        assert!(state.is_in_battle());

        state.update_ui_state(UIState::BattleCards);
        assert!(state.is_in_battle());
    }
}
