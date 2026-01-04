//! Skill usage decision engine
//!
//! Determines when and how to use servant skills and master skills
//! for optimal battle performance.

use crate::config::Settings;
use crate::game::battle::BattleState;
use crate::game::servant::Servant;

/// Skill decision engine
pub struct SkillDecisionEngine {
    /// Priority for NP charge skills
    np_charge_priority: f32,
    /// Priority for damage buff skills
    damage_buff_priority: f32,
    /// Priority for defensive skills
    defensive_priority: f32,
}

impl SkillDecisionEngine {
    /// Create a new skill decision engine
    pub fn new() -> Self {
        Self {
            np_charge_priority: 2.0,
            damage_buff_priority: 1.5,
            defensive_priority: 1.0,
        }
    }

    /// Recommend skills to use based on current state
    pub fn recommend_skills(
        &self,
        state: &BattleState,
        settings: &Settings,
    ) -> Vec<SkillRecommendation> {
        let mut recommendations = Vec::new();

        // Check each servant's skills
        for (servant_idx, servant) in state.servants.iter().enumerate() {
            if !servant.is_alive {
                continue;
            }

            for (skill_idx, skill) in servant.skills.iter().enumerate() {
                if !skill.is_ready() {
                    continue;
                }

                // Score this skill usage
                if let Some(rec) = self.evaluate_skill(
                    servant_idx,
                    skill_idx,
                    servant,
                    state,
                    settings,
                ) {
                    recommendations.push(rec);
                }
            }
        }

        // Check master skills
        if settings.skill_settings.use_master_skills {
            for skill_idx in 0..3 {
                if state.master_skills.is_ready(skill_idx) {
                    if let Some(rec) = self.evaluate_master_skill(skill_idx, state, settings) {
                        recommendations.push(rec);
                    }
                }
            }
        }

        // Sort by priority (descending)
        recommendations.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        recommendations
    }

    /// Evaluate a servant skill for usage
    fn evaluate_skill(
        &self,
        servant_idx: usize,
        skill_idx: usize,
        servant: &Servant,
        state: &BattleState,
        settings: &Settings,
    ) -> Option<SkillRecommendation> {
        let skill = &servant.skills[skill_idx];
        let mut priority = 0.0;
        let mut target = None;

        // NP charge skills - high priority if close to NP
        if skill.is_np_charge {
            let current_np = servant.np_gauge;
            let charge_amount = skill.np_charge_amount;

            // High priority if this would enable NP
            if current_np < 100 && current_np + charge_amount >= 100 {
                priority = self.np_charge_priority * 3.0;
            } else if settings.skill_settings.np_charge_priority {
                priority = self.np_charge_priority;
            }

            // Target self for self-charge skills
            if skill.requires_target {
                target = Some(servant_idx);
            }
        }

        // Damage buff skills - use before NP
        if skill.is_damage_buff {
            // Higher priority on final wave or when NP is ready
            if state.is_final_wave() || servant.can_np() {
                priority = self.damage_buff_priority * 2.0;
            } else {
                priority = self.damage_buff_priority;
            }
        }

        // Only recommend if priority is significant
        if priority > 0.5 {
            Some(SkillRecommendation {
                servant_idx,
                skill_idx,
                target,
                priority,
                is_master_skill: false,
                reason: self.get_skill_reason(skill.is_np_charge, skill.is_damage_buff),
            })
        } else {
            None
        }
    }

    /// Evaluate a master skill for usage
    fn evaluate_master_skill(
        &self,
        skill_idx: usize,
        state: &BattleState,
        _settings: &Settings,
    ) -> Option<SkillRecommendation> {
        // Master skills are typically:
        // 0: Party buff
        // 1: Single target buff or heal
        // 2: Utility (stun, swap, etc.)

        let priority = match skill_idx {
            0 => {
                // Party buff - use on final wave with NPs ready
                if state.is_final_wave() && state.servants_with_np().len() >= 2 {
                    2.0
                } else {
                    0.0
                }
            }
            1 => {
                // Single target buff - use on DPS with NP ready
                if let Some(dps_idx) = state.servants.iter().position(|s| s.can_np()) {
                    return Some(SkillRecommendation {
                        servant_idx: 0,
                        skill_idx,
                        target: Some(dps_idx),
                        priority: 1.5,
                        is_master_skill: true,
                        reason: SkillReason::BuffBeforeNP,
                    });
                }
                0.0
            }
            2 => {
                // Utility - context dependent
                0.0
            }
            _ => 0.0,
        };

        if priority > 0.5 {
            Some(SkillRecommendation {
                servant_idx: 0,
                skill_idx,
                target: None,
                priority,
                is_master_skill: true,
                reason: SkillReason::BuffBeforeNP,
            })
        } else {
            None
        }
    }

    /// Get the reason for skill usage
    fn get_skill_reason(&self, is_np_charge: bool, is_damage_buff: bool) -> SkillReason {
        if is_np_charge {
            SkillReason::NPCharge
        } else if is_damage_buff {
            SkillReason::BuffBeforeNP
        } else {
            SkillReason::Utility
        }
    }

    /// Check if any skills should be used this turn
    pub fn should_use_skills(&self, state: &BattleState, settings: &Settings) -> bool {
        if !settings.skill_settings.auto_use_skills {
            return false;
        }

        // Check if any servant has ready skills
        state
            .servants
            .iter()
            .any(|s| s.is_alive && s.skills.iter().any(|skill| skill.is_ready()))
    }

    /// Get skill usage order for a specific wave
    pub fn get_wave_skill_order(
        &self,
        wave: u32,
        settings: &Settings,
    ) -> Vec<SkillRecommendation> {
        // If user has defined a specific skill order, use that
        if let Some(ref skill_order) = settings.skill_settings.skill_order {
            return skill_order
                .iter()
                .filter(|cmd| cmd.wave == Some(wave) || cmd.wave.is_none())
                .map(|cmd| SkillRecommendation {
                    servant_idx: cmd.servant,
                    skill_idx: cmd.skill,
                    target: cmd.target,
                    priority: 10.0, // User-defined skills get high priority
                    is_master_skill: false,
                    reason: SkillReason::UserDefined,
                })
                .collect();
        }

        Vec::new()
    }
}

impl Default for SkillDecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Skill usage recommendation
#[derive(Debug, Clone)]
pub struct SkillRecommendation {
    /// Servant index (0-2 for servant skills, ignored for master skills)
    pub servant_idx: usize,
    /// Skill index (0-2)
    pub skill_idx: usize,
    /// Target servant index (if applicable)
    pub target: Option<usize>,
    /// Priority score (higher = use first)
    pub priority: f32,
    /// Whether this is a master skill
    pub is_master_skill: bool,
    /// Reason for recommendation
    pub reason: SkillReason,
}

/// Reason for skill recommendation
#[derive(Debug, Clone, Copy)]
pub enum SkillReason {
    /// Skill charges NP
    NPCharge,
    /// Buff before NP usage
    BuffBeforeNP,
    /// Defensive skill needed
    Defensive,
    /// General utility
    Utility,
    /// User-defined skill order
    UserDefined,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::servant::{ServantClass, Skill};

    #[test]
    fn test_skill_engine_creation() {
        let engine = SkillDecisionEngine::new();
        assert!(engine.np_charge_priority > 0.0);
    }

    #[test]
    fn test_should_use_skills() {
        let engine = SkillDecisionEngine::new();
        let settings = Settings::default();

        let mut state = BattleState::new();
        state.servants.push(Servant {
            skills: [
                Skill {
                    cooldown: 0,
                    ..Default::default()
                },
                Skill::default(),
                Skill::default(),
            ],
            ..Servant::new(ServantClass::Saber, 0)
        });

        assert!(engine.should_use_skills(&state, &settings));

        // Disable auto skills
        let mut settings = settings;
        settings.skill_settings.auto_use_skills = false;
        assert!(!engine.should_use_skills(&state, &settings));
    }
}
