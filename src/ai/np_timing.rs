//! NP timing decision engine
//!
//! Determines when to use Noble Phantasms and in what order
//! for maximum effectiveness.

use crate::config::Settings;
use crate::game::battle::BattleState;
use crate::game::servant::{NPType, Servant};

/// NP timing decision engine
pub struct NPTimingEngine {
    /// Threshold multiplier for using NP on non-final waves
    early_wave_threshold: f32,
    /// Priority bonus for AoE NPs on multiple enemies
    aoe_multi_enemy_bonus: f32,
}

impl NPTimingEngine {
    /// Create a new NP timing engine
    pub fn new() -> Self {
        Self {
            early_wave_threshold: 0.7,
            aoe_multi_enemy_bonus: 1.5,
        }
    }

    /// Decide which servants should use their NPs this turn
    pub fn decide_np_usage(&self, state: &BattleState, settings: &Settings) -> Vec<usize> {
        let mut np_users = Vec::new();

        // Get servants that can NP
        let available_nps: Vec<(usize, &Servant)> = state
            .servants
            .iter()
            .enumerate()
            .filter(|(idx, s)| s.can_np() && state.np_available[*idx])
            .collect();

        if available_nps.is_empty() {
            return np_users;
        }

        // Evaluate each potential NP user
        let mut scored_nps: Vec<(usize, f32)> = available_nps
            .iter()
            .map(|(idx, servant)| {
                let score = self.score_np_usage(*idx, servant, state, settings);
                (*idx, score)
            })
            .filter(|(_, score)| *score > 0.5)
            .collect();

        // Sort by score (descending)
        scored_nps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Decide NP order based on type and score
        for (idx, _) in scored_nps {
            if np_users.len() < 3 {
                np_users.push(idx);
            }
        }

        // Optimize NP order for chains
        self.optimize_np_order(&mut np_users, state);

        np_users
    }

    /// Score NP usage for a servant
    fn score_np_usage(
        &self,
        _servant_idx: usize,
        servant: &Servant,
        state: &BattleState,
        settings: &Settings,
    ) -> f32 {
        let mut score = 0.0;

        // Base score for having NP ready
        if servant.np_gauge >= settings.np_threshold {
            score += 1.0;
        }

        // Final wave bonus
        if state.is_final_wave() {
            score += 2.0;
        }

        // Enemy count consideration
        let enemy_count = state.current_wave.alive_count();
        if enemy_count > 1 {
            score += self.aoe_multi_enemy_bonus;
        }

        // Class advantage bonus
        let mut has_advantage = false;
        for enemy in state.current_wave.alive_enemies() {
            if servant.damage_multiplier(&enemy.class) > 1.0 {
                has_advantage = true;
                break;
            }
        }
        if has_advantage {
            score *= 1.3;
        }

        // Buff stacking bonus (if servant has buffs)
        if servant.buff_count > 0 {
            score *= 1.0 + (servant.buff_count as f32 * 0.2);
        }

        // Overcharge bonus
        if servant.can_overcharge() {
            score *= 1.1;
        }

        // Reduce score on early waves if not necessary
        if !state.is_final_wave() {
            score *= self.early_wave_threshold;
        }

        score
    }

    /// Optimize NP order for maximum effectiveness
    fn optimize_np_order(&self, np_order: &mut Vec<usize>, state: &BattleState) {
        if np_order.len() < 2 {
            return;
        }

        // Get NP types for each user
        let np_types: Vec<(usize, NPType)> = np_order
            .iter()
            .filter_map(|&idx| {
                state
                    .servants
                    .get(idx)
                    .map(|s| (idx, s.np_type))
            })
            .collect();

        // Optimize order based on NP types
        // General rules:
        // 1. Support NPs first (buffs, debuffs)
        // 2. Arts NPs early for NP chain bonus
        // 3. Buster NPs last for damage

        let mut optimized: Vec<usize> = Vec::new();

        // First: Arts NPs (for NP gain bonus)
        for (idx, np_type) in &np_types {
            if *np_type == NPType::Arts {
                optimized.push(*idx);
            }
        }

        // Second: Quick NPs
        for (idx, np_type) in &np_types {
            if *np_type == NPType::Quick && !optimized.contains(idx) {
                optimized.push(*idx);
            }
        }

        // Last: Buster NPs (for damage bonus from first-card bonus)
        for (idx, np_type) in &np_types {
            if *np_type == NPType::Buster && !optimized.contains(idx) {
                optimized.push(*idx);
            }
        }

        // If we have a 3-NP chain, the order matters less
        // but we still want Arts first for the NP chain bonus
        if optimized.len() == np_order.len() {
            *np_order = optimized;
        }
    }

    /// Check if NP chain bonus applies
    pub fn has_np_chain_bonus(&self, np_count: usize) -> bool {
        // NP chains (3 NPs) give overcharge bonus
        np_count >= 3
    }

    /// Calculate overcharge level from chain position
    pub fn chain_overcharge(&self, base_overcharge: u32, chain_position: usize) -> u32 {
        // Each position in NP chain adds 100% overcharge
        (base_overcharge + (chain_position as u32 * 100)).min(500)
    }

    /// Suggest whether to wait for more NP gauge
    pub fn should_wait_for_np(&self, servant: &Servant, state: &BattleState) -> bool {
        // Don't wait on final wave
        if state.is_final_wave() {
            return false;
        }

        // Wait if close to NP but not there yet
        servant.np_gauge >= 80 && servant.np_gauge < 100
    }

    /// Calculate expected damage from NP
    pub fn estimate_np_damage(&self, servant: &Servant, state: &BattleState) -> f32 {
        let base_damage = 1.0;

        // NP type multiplier
        let type_mult = match servant.np_type {
            NPType::Buster => 1.5,
            NPType::Arts => 1.0,
            NPType::Quick => 0.8,
        };

        // Overcharge multiplier
        let oc_mult = 1.0 + ((servant.overcharge_level() - 1) as f32 * 0.1);

        // Buff multiplier (simplified)
        let buff_mult = 1.0 + (servant.buff_count as f32 * 0.2);

        // Class advantage (average across enemies)
        let mut class_mult = 1.0;
        let enemies: Vec<_> = state.current_wave.alive_enemies().collect();
        if !enemies.is_empty() {
            let total: f32 = enemies
                .iter()
                .map(|e| servant.damage_multiplier(&e.class))
                .sum();
            class_mult = total / enemies.len() as f32;
        }

        base_damage * type_mult * oc_mult * buff_mult * class_mult
    }
}

impl Default for NPTimingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::enemy::{Enemy, EnemyWave};
    use crate::game::servant::ServantClass;

    #[test]
    fn test_np_engine_creation() {
        let engine = NPTimingEngine::new();
        assert!(engine.early_wave_threshold > 0.0);
    }

    #[test]
    fn test_np_chain_bonus() {
        let engine = NPTimingEngine::new();
        assert!(!engine.has_np_chain_bonus(2));
        assert!(engine.has_np_chain_bonus(3));
    }

    #[test]
    fn test_chain_overcharge() {
        let engine = NPTimingEngine::new();
        assert_eq!(engine.chain_overcharge(100, 0), 100);
        assert_eq!(engine.chain_overcharge(100, 1), 200);
        assert_eq!(engine.chain_overcharge(100, 2), 300);
        assert_eq!(engine.chain_overcharge(300, 2), 500); // Capped at 500
    }

    #[test]
    fn test_should_wait_for_np() {
        let engine = NPTimingEngine::new();

        let mut servant = Servant::new(ServantClass::Saber, 0);
        let mut state = BattleState::new();
        state.current_wave = EnemyWave::new(1, 3);

        // Not close enough
        servant.np_gauge = 50;
        assert!(!engine.should_wait_for_np(&servant, &state));

        // Close enough to wait
        servant.np_gauge = 85;
        assert!(engine.should_wait_for_np(&servant, &state));

        // Final wave - don't wait
        state.current_wave = EnemyWave::new(3, 3);
        assert!(!engine.should_wait_for_np(&servant, &state));
    }
}
