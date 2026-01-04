//! Enemy prioritization system
//!
//! Determines which enemy to target based on HP, class advantage,
//! threat level, and strategic considerations.

use crate::config::Settings;
use crate::game::enemy::{Enemy, EnemyWave, ThreatLevel};
use crate::game::servant::Servant;

/// Enemy prioritization engine
pub struct EnemyPrioritizer {
    /// Weight for low HP targeting
    hp_weight: f32,
    /// Weight for class advantage
    class_advantage_weight: f32,
    /// Weight for threat level
    threat_weight: f32,
    /// Weight for break bars
    break_bar_weight: f32,
}

impl EnemyPrioritizer {
    /// Create a new enemy prioritizer
    pub fn new() -> Self {
        Self {
            hp_weight: 1.0,
            class_advantage_weight: 1.2,
            threat_weight: 1.5,
            break_bar_weight: 0.8,
        }
    }

    /// Prioritize enemies and return the index of the best target
    pub fn prioritize(
        &self,
        wave: &EnemyWave,
        servants: &[Servant],
        settings: &Settings,
    ) -> Option<usize> {
        let alive_enemies: Vec<&Enemy> = wave.alive_enemies().collect();

        if alive_enemies.is_empty() {
            return None;
        }

        // If only one enemy, target it
        if alive_enemies.len() == 1 {
            return Some(alive_enemies[0].position);
        }

        // Score each enemy
        let mut scores: Vec<(usize, f32)> = alive_enemies
            .iter()
            .map(|enemy| {
                let score = self.score_enemy(enemy, servants, settings);
                (enemy.position, score)
            })
            .collect();

        // Sort by score (descending - higher score = higher priority)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scores.first().map(|(idx, _)| *idx)
    }

    /// Score an enemy for targeting priority
    fn score_enemy(&self, enemy: &Enemy, servants: &[Servant], settings: &Settings) -> f32 {
        let mut score = 0.0;

        // HP-based scoring (lower HP = higher score if targeting low HP first)
        if settings.target_low_hp_first {
            score += (1.0 - enemy.hp_percent) * self.hp_weight * 10.0;
        } else {
            // For boss fights, might want to focus on highest HP
            score += enemy.hp_percent * self.hp_weight;
        }

        // Threat level scoring
        score += match enemy.threat_level() {
            ThreatLevel::Critical => 10.0 * self.threat_weight,
            ThreatLevel::High => 7.0 * self.threat_weight,
            ThreatLevel::Medium => 4.0 * self.threat_weight,
            ThreatLevel::Low => 1.0 * self.threat_weight,
        };

        // Class advantage scoring
        if settings.prioritize_class_advantage {
            let mut max_advantage = 1.0f32;
            for servant in servants.iter().filter(|s| s.is_alive) {
                let advantage = servant.damage_multiplier(&enemy.class);
                if advantage > max_advantage {
                    max_advantage = advantage;
                }
            }
            score *= max_advantage * self.class_advantage_weight;
        }

        // Break bar penalty (takes multiple kills)
        if enemy.has_break_bars() {
            score *= self.break_bar_weight.powf(enemy.break_bars as f32);
        }

        // Boss bonus (often the primary target)
        if enemy.is_boss {
            score *= 1.5;
        }

        // Dangerous NP bonus (kill before they NP)
        if enemy.has_dangerous_np {
            score *= 2.0;
        }

        score
    }

    /// Get priority-sorted list of enemies
    pub fn get_priority_list(
        &self,
        wave: &EnemyWave,
        servants: &[Servant],
        settings: &Settings,
    ) -> Vec<EnemyPriority> {
        let mut priorities: Vec<EnemyPriority> = wave
            .alive_enemies()
            .map(|enemy| {
                let score = self.score_enemy(enemy, servants, settings);
                EnemyPriority {
                    position: enemy.position,
                    score,
                    hp_percent: enemy.hp_percent,
                    threat_level: enemy.threat_level(),
                    has_break_bars: enemy.has_break_bars(),
                }
            })
            .collect();

        priorities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        priorities
    }

    /// Check if enemy can be killed in one turn
    pub fn can_kill_in_one_turn(&self, enemy: &Enemy, estimated_damage: f32) -> bool {
        // If enemy has break bars, we can only break one bar
        if enemy.has_break_bars() {
            return false;
        }

        // Estimate if damage is enough to kill
        // This is simplified - would need actual HP values
        enemy.hp_percent < estimated_damage
    }

    /// Get the weakest enemy (for quick kills)
    pub fn get_weakest_enemy(&self, wave: &EnemyWave) -> Option<usize> {
        wave.lowest_hp_enemy().map(|e| e.position)
    }

    /// Get the most threatening enemy
    pub fn get_most_threatening(&self, wave: &EnemyWave) -> Option<usize> {
        wave.highest_threat_enemy().map(|e| e.position)
    }

    /// Suggest multi-target strategy if enemies have similar HP
    pub fn should_use_aoe(&self, wave: &EnemyWave) -> bool {
        let alive: Vec<&Enemy> = wave.alive_enemies().collect();

        if alive.len() < 2 {
            return false;
        }

        // Check if HP percentages are relatively close
        let hp_values: Vec<f32> = alive.iter().map(|e| e.hp_percent).collect();
        let max_hp = hp_values.iter().cloned().fold(0.0, f32::max);
        let min_hp = hp_values.iter().cloned().fold(1.0, f32::min);

        // If HP spread is small, AoE is good
        (max_hp - min_hp) < 0.3
    }
}

impl Default for EnemyPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Enemy priority information
#[derive(Debug, Clone)]
pub struct EnemyPriority {
    /// Enemy position (0-2)
    pub position: usize,
    /// Priority score
    pub score: f32,
    /// HP percentage
    pub hp_percent: f32,
    /// Threat level
    pub threat_level: ThreatLevel,
    /// Has break bars
    pub has_break_bars: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prioritizer_creation() {
        let prioritizer = EnemyPrioritizer::new();
        assert!(prioritizer.hp_weight > 0.0);
    }

    #[test]
    fn test_low_hp_priority() {
        let prioritizer = EnemyPrioritizer::new();
        let settings = Settings::default();

        let mut wave = EnemyWave::new(1, 3);
        wave.enemies.push(Enemy {
            hp_percent: 0.8,
            position: 0,
            ..Default::default()
        });
        wave.enemies.push(Enemy {
            hp_percent: 0.2, // Low HP
            position: 1,
            ..Default::default()
        });
        wave.enemies.push(Enemy {
            hp_percent: 0.5,
            position: 2,
            ..Default::default()
        });

        let target = prioritizer.prioritize(&wave, &[], &settings);
        assert_eq!(target, Some(1)); // Should target low HP enemy
    }

    #[test]
    fn test_threat_priority() {
        let prioritizer = EnemyPrioritizer::new();
        let mut settings = Settings::default();
        settings.target_low_hp_first = false;

        let mut wave = EnemyWave::new(1, 3);
        wave.enemies.push(Enemy {
            hp_percent: 0.5,
            position: 0,
            ..Default::default()
        });
        wave.enemies.push(Enemy {
            hp_percent: 0.5,
            position: 1,
            is_boss: true,
            has_dangerous_np: true,
            ..Default::default()
        });

        let target = prioritizer.prioritize(&wave, &[], &settings);
        assert_eq!(target, Some(1)); // Should target dangerous boss
    }

    #[test]
    fn test_aoe_suggestion() {
        let prioritizer = EnemyPrioritizer::new();

        let mut wave = EnemyWave::new(1, 3);
        wave.enemies.push(Enemy {
            hp_percent: 0.5,
            position: 0,
            ..Default::default()
        });
        wave.enemies.push(Enemy {
            hp_percent: 0.6,
            position: 1,
            ..Default::default()
        });
        wave.enemies.push(Enemy {
            hp_percent: 0.55,
            position: 2,
            ..Default::default()
        });

        assert!(prioritizer.should_use_aoe(&wave));
    }
}
