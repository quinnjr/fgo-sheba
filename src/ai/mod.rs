//! Battle AI decision engine
//!
//! Handles intelligent decision-making for card selection, enemy targeting,
//! skill usage, and NP timing.

pub mod card_selector;
pub mod enemy_priority;
pub mod np_timing;
pub mod skill_usage;
pub mod strategy;

use crate::config::Settings;
use crate::game::battle::BattleState;
use crate::ShebaAction;

pub use card_selector::CardSelector;
pub use enemy_priority::EnemyPrioritizer;
pub use np_timing::NPTimingEngine;
pub use skill_usage::SkillDecisionEngine;
pub use strategy::{BattleStrategy, StrategyType};

/// Main battle AI that coordinates all decision-making
pub struct BattleAI {
    /// Card selection engine
    card_selector: CardSelector,
    /// Enemy prioritization engine
    enemy_prioritizer: EnemyPrioritizer,
    /// Skill decision engine
    skill_engine: SkillDecisionEngine,
    /// NP timing engine
    np_engine: NPTimingEngine,
    /// Current strategy
    strategy: BattleStrategy,
}

impl BattleAI {
    /// Create a new battle AI
    pub fn new() -> Self {
        Self {
            card_selector: CardSelector::new(),
            enemy_prioritizer: EnemyPrioritizer::new(),
            skill_engine: SkillDecisionEngine::new(),
            np_engine: NPTimingEngine::new(),
            strategy: BattleStrategy::new(StrategyType::Balanced),
        }
    }

    /// Create a battle AI with a specific strategy
    pub fn with_strategy(strategy_type: StrategyType) -> Self {
        Self {
            card_selector: CardSelector::new(),
            enemy_prioritizer: EnemyPrioritizer::new(),
            skill_engine: SkillDecisionEngine::new(),
            np_engine: NPTimingEngine::new(),
            strategy: BattleStrategy::new(strategy_type),
        }
    }

    /// Set the battle strategy
    pub fn set_strategy(&mut self, strategy_type: StrategyType) {
        self.strategy = BattleStrategy::new(strategy_type);
    }

    /// Decide the next action based on battle state and settings
    pub fn decide_action(&self, state: &BattleState, settings: &Settings) -> ShebaAction {
        use crate::game::battle::BattlePhase;

        match state.phase {
            BattlePhase::CommandPhase => {
                // First, check if we should use skills
                if let Some(skill_action) = self.decide_skill_usage(state, settings) {
                    return skill_action;
                }

                // Otherwise, enter card selection
                ShebaAction::TapAttack
            }

            BattlePhase::CardSelection => {
                // Select cards and NPs
                self.decide_card_selection(state, settings)
            }

            BattlePhase::PreBattle | BattlePhase::Unknown => {
                // Wait for battle to start
                ShebaAction::Wait { duration_ms: 500 }
            }

            BattlePhase::AttackPhase | BattlePhase::EnemyPhase => {
                // Wait for animations
                ShebaAction::Wait {
                    duration_ms: settings.timings.attack_wait,
                }
            }

            BattlePhase::Victory | BattlePhase::Defeat => {
                // Battle ended
                ShebaAction::None
            }
        }
    }

    /// Decide which skills to use
    fn decide_skill_usage(&self, state: &BattleState, settings: &Settings) -> Option<ShebaAction> {
        if !settings.skill_settings.auto_use_skills {
            return None;
        }

        // Get skill recommendations from the skill engine
        let skill_recommendations = self.skill_engine.recommend_skills(state, settings);

        // Return the highest priority skill that should be used
        skill_recommendations
            .into_iter()
            .next()
            .map(|rec| ShebaAction::UseSkill {
                servant_idx: rec.servant_idx,
                skill_idx: rec.skill_idx,
                target: rec.target,
            })
    }

    /// Decide card selection
    fn decide_card_selection(&self, state: &BattleState, settings: &Settings) -> ShebaAction {
        // First, decide on enemy targeting
        let target_enemy =
            self.enemy_prioritizer
                .prioritize(&state.current_wave, &state.servants, settings);

        // Check if we should use NPs
        let np_decisions = self.np_engine.decide_np_usage(state, settings);

        // Get card selection
        let card_selection = self.card_selector.select_cards(
            &state.available_cards,
            &state.servants,
            &state.current_wave,
            &np_decisions,
            settings,
        );

        // Build the action with selected cards
        if !card_selection.is_empty() {
            // First, target the enemy if needed
            if let Some(enemy_idx) = target_enemy {
                if state.target_enemy != Some(enemy_idx) {
                    return ShebaAction::TargetEnemy { enemy_idx };
                }
            }

            // Then select NPs and cards
            let mut card_indices = Vec::new();

            // Add NPs first (in the decided order)
            for servant_idx in &np_decisions {
                if state.np_available[*servant_idx] {
                    card_indices.push(5 + servant_idx); // NP positions are 5, 6, 7
                }
            }

            // Add regular cards
            for card in card_selection {
                if card_indices.len() < 3 {
                    card_indices.push(card.position);
                }
            }

            // Ensure we have exactly 3 cards
            while card_indices.len() < 3 && card_indices.len() < state.available_cards.len() {
                // Fill with remaining cards
                for i in 0..5 {
                    if !card_indices.contains(&i) {
                        card_indices.push(i);
                        break;
                    }
                }
            }

            ShebaAction::SelectCards { card_indices }
        } else {
            // No cards available, wait
            ShebaAction::Wait { duration_ms: 500 }
        }
    }

    /// Get the card selector
    pub fn card_selector(&self) -> &CardSelector {
        &self.card_selector
    }

    /// Get the enemy prioritizer
    pub fn enemy_prioritizer(&self) -> &EnemyPrioritizer {
        &self.enemy_prioritizer
    }

    /// Get the skill engine
    pub fn skill_engine(&self) -> &SkillDecisionEngine {
        &self.skill_engine
    }

    /// Get the NP engine
    pub fn np_engine(&self) -> &NPTimingEngine {
        &self.np_engine
    }
}

impl Default for BattleAI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_ai_creation() {
        let ai = BattleAI::new();
        // Just verify it creates successfully
        assert!(matches!(
            ai.strategy.strategy_type(),
            StrategyType::Balanced
        ));
    }

    #[test]
    fn test_strategy_setting() {
        let mut ai = BattleAI::new();
        ai.set_strategy(StrategyType::NPLoop);
        assert!(matches!(ai.strategy.strategy_type(), StrategyType::NPLoop));
    }
}
