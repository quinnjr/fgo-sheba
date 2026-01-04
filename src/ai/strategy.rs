//! High-level battle strategy
//!
//! Defines overall battle strategies and how they affect decision-making.

use serde::{Deserialize, Serialize};

/// Type of battle strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    /// Balanced approach - mix of damage and NP gain
    Balanced,
    /// Maximum damage output
    MaxDamage,
    /// NP looping - focus on NP gain and refund
    NPLoop,
    /// Defensive - prioritize survival
    Defensive,
    /// Speed farming - fastest possible clears
    SpeedFarm,
    /// Custom strategy
    Custom,
}

/// Battle strategy configuration
#[derive(Debug, Clone)]
pub struct BattleStrategy {
    /// Strategy type
    strategy_type: StrategyType,
    /// Card type preference weights
    pub card_weights: CardWeights,
    /// Whether to prioritize AoE attacks
    pub prefer_aoe: bool,
    /// Whether to save NP for boss
    pub save_np_for_boss: bool,
    /// NP usage threshold per wave
    pub np_per_wave: [u32; 3],
    /// Whether to use all skills immediately
    pub front_load_skills: bool,
}

impl BattleStrategy {
    /// Create a new strategy
    pub fn new(strategy_type: StrategyType) -> Self {
        match strategy_type {
            StrategyType::Balanced => Self::balanced(),
            StrategyType::MaxDamage => Self::max_damage(),
            StrategyType::NPLoop => Self::np_loop(),
            StrategyType::Defensive => Self::defensive(),
            StrategyType::SpeedFarm => Self::speed_farm(),
            StrategyType::Custom => Self::balanced(),
        }
    }

    /// Get the strategy type
    pub fn strategy_type(&self) -> StrategyType {
        self.strategy_type
    }

    /// Balanced strategy
    fn balanced() -> Self {
        Self {
            strategy_type: StrategyType::Balanced,
            card_weights: CardWeights {
                buster: 1.0,
                arts: 1.0,
                quick: 1.0,
            },
            prefer_aoe: true,
            save_np_for_boss: false,
            np_per_wave: [1, 1, 2],
            front_load_skills: false,
        }
    }

    /// Maximum damage strategy
    fn max_damage() -> Self {
        Self {
            strategy_type: StrategyType::MaxDamage,
            card_weights: CardWeights {
                buster: 2.0,
                arts: 0.8,
                quick: 0.6,
            },
            prefer_aoe: false,
            save_np_for_boss: true,
            np_per_wave: [0, 1, 3],
            front_load_skills: true,
        }
    }

    /// NP looping strategy
    fn np_loop() -> Self {
        Self {
            strategy_type: StrategyType::NPLoop,
            card_weights: CardWeights {
                buster: 0.6,
                arts: 2.0,
                quick: 0.8,
            },
            prefer_aoe: true,
            save_np_for_boss: false,
            np_per_wave: [1, 1, 1],
            front_load_skills: true,
        }
    }

    /// Defensive strategy
    fn defensive() -> Self {
        Self {
            strategy_type: StrategyType::Defensive,
            card_weights: CardWeights {
                buster: 0.8,
                arts: 1.5,
                quick: 1.0,
            },
            prefer_aoe: true,
            save_np_for_boss: false,
            np_per_wave: [1, 1, 1],
            front_load_skills: false,
        }
    }

    /// Speed farming strategy
    fn speed_farm() -> Self {
        Self {
            strategy_type: StrategyType::SpeedFarm,
            card_weights: CardWeights {
                buster: 1.5,
                arts: 1.2,
                quick: 0.8,
            },
            prefer_aoe: true,
            save_np_for_boss: false,
            np_per_wave: [1, 1, 1],
            front_load_skills: true,
        }
    }

    /// Get the recommended NP count for a wave
    pub fn np_count_for_wave(&self, wave: u32) -> u32 {
        let idx = (wave.saturating_sub(1) as usize).min(2);
        self.np_per_wave[idx]
    }

    /// Check if we should use NP on this wave
    pub fn should_np_on_wave(&self, wave: u32, total_waves: u32, current_np_count: u32) -> bool {
        let target = self.np_count_for_wave(wave);

        if current_np_count < target {
            return true;
        }

        // Always use NP on final wave if available
        wave == total_waves
    }

    /// Get card weight
    pub fn get_card_weight(&self, card_type: &crate::game::cards::CardType) -> f32 {
        match card_type {
            crate::game::cards::CardType::Buster => self.card_weights.buster,
            crate::game::cards::CardType::Arts => self.card_weights.arts,
            crate::game::cards::CardType::Quick => self.card_weights.quick,
            _ => 1.0,
        }
    }
}

/// Card type weights for strategy
#[derive(Debug, Clone)]
pub struct CardWeights {
    /// Buster card weight
    pub buster: f32,
    /// Arts card weight
    pub arts: f32,
    /// Quick card weight
    pub quick: f32,
}

impl Default for CardWeights {
    fn default() -> Self {
        Self {
            buster: 1.0,
            arts: 1.0,
            quick: 1.0,
        }
    }
}

/// Wave-specific strategy adjustments
#[derive(Debug, Clone)]
pub struct WaveStrategy {
    /// Wave number
    pub wave: u32,
    /// Target NP count
    pub target_np_count: u32,
    /// Priority skills to use
    pub priority_skills: Vec<(usize, usize)>,
    /// Whether to focus single target
    pub single_target_focus: bool,
}

impl WaveStrategy {
    /// Create a new wave strategy
    pub fn new(wave: u32) -> Self {
        Self {
            wave,
            target_np_count: 1,
            priority_skills: Vec::new(),
            single_target_focus: false,
        }
    }

    /// Set target NP count
    pub fn with_np_count(mut self, count: u32) -> Self {
        self.target_np_count = count;
        self
    }

    /// Add a priority skill
    pub fn with_skill(mut self, servant: usize, skill: usize) -> Self {
        self.priority_skills.push((servant, skill));
        self
    }

    /// Set single target focus
    pub fn with_single_target(mut self, focus: bool) -> Self {
        self.single_target_focus = focus;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = BattleStrategy::new(StrategyType::Balanced);
        assert_eq!(strategy.strategy_type(), StrategyType::Balanced);
    }

    #[test]
    fn test_np_loop_strategy() {
        let strategy = BattleStrategy::new(StrategyType::NPLoop);
        assert!(strategy.card_weights.arts > strategy.card_weights.buster);
        assert!(strategy.front_load_skills);
    }

    #[test]
    fn test_max_damage_strategy() {
        let strategy = BattleStrategy::new(StrategyType::MaxDamage);
        assert!(strategy.card_weights.buster > strategy.card_weights.arts);
        assert!(strategy.save_np_for_boss);
    }

    #[test]
    fn test_wave_np_count() {
        let strategy = BattleStrategy::new(StrategyType::MaxDamage);
        assert_eq!(strategy.np_count_for_wave(1), 0);
        assert_eq!(strategy.np_count_for_wave(2), 1);
        assert_eq!(strategy.np_count_for_wave(3), 3);
    }

    #[test]
    fn test_should_np() {
        let strategy = BattleStrategy::new(StrategyType::Balanced);
        assert!(strategy.should_np_on_wave(1, 3, 0));
        assert!(!strategy.should_np_on_wave(1, 3, 1));
        assert!(strategy.should_np_on_wave(3, 3, 0)); // Final wave
    }
}
