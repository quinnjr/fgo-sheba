//! User settings and team configurations
//!
//! Defines all configurable options for the automation.

use serde::{Deserialize, Serialize};

use crate::game::cards::CardType;
use crate::game::servant::ServantClass;

/// Main settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Card selection priority order
    pub card_priority: CardPriority,
    /// NP threshold (percentage) to consider using NP
    pub np_threshold: u32,
    /// Whether to target low HP enemies first
    pub target_low_hp_first: bool,
    /// Whether to prioritize class advantage
    pub prioritize_class_advantage: bool,
    /// Skill usage settings
    pub skill_settings: SkillSettings,
    /// Team configuration
    pub team_config: Option<TeamConfig>,
    /// General automation settings
    pub automation: AutomationSettings,
    /// Screen timing settings
    pub timings: TimingSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            card_priority: CardPriority::default(),
            np_threshold: 100,
            target_low_hp_first: true,
            prioritize_class_advantage: true,
            skill_settings: SkillSettings::default(),
            team_config: None,
            automation: AutomationSettings::default(),
            timings: TimingSettings::default(),
        }
    }
}

impl Settings {
    /// Create settings optimized for farming
    pub fn farming_preset() -> Self {
        Self {
            card_priority: CardPriority {
                first_choice: CardType::Buster,
                second_choice: CardType::Arts,
                third_choice: CardType::Quick,
                prefer_chains: true,
                prefer_brave_chains: true,
            },
            np_threshold: 100,
            target_low_hp_first: true,
            prioritize_class_advantage: false, // Speed over efficiency
            skill_settings: SkillSettings {
                auto_use_skills: true,
                ..Default::default()
            },
            automation: AutomationSettings {
                auto_continue: true,
                loop_farming: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create settings for boss fights
    pub fn boss_fight_preset() -> Self {
        Self {
            card_priority: CardPriority {
                first_choice: CardType::Arts,
                second_choice: CardType::Buster,
                third_choice: CardType::Quick,
                prefer_chains: true,
                prefer_brave_chains: true,
            },
            np_threshold: 100,
            target_low_hp_first: false, // Focus on boss
            prioritize_class_advantage: true,
            skill_settings: SkillSettings {
                auto_use_skills: false, // Manual control for bosses
                ..Default::default()
            },
            automation: AutomationSettings {
                auto_continue: false,
                loop_farming: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create settings for NP looping
    pub fn np_loop_preset() -> Self {
        Self {
            card_priority: CardPriority {
                first_choice: CardType::Arts,
                second_choice: CardType::Arts,
                third_choice: CardType::Quick,
                prefer_chains: true,
                prefer_brave_chains: false,
            },
            np_threshold: 100,
            target_low_hp_first: true,
            prioritize_class_advantage: false,
            skill_settings: SkillSettings {
                auto_use_skills: true,
                np_charge_priority: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Card selection priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPriority {
    /// First priority card type
    pub first_choice: CardType,
    /// Second priority card type
    pub second_choice: CardType,
    /// Third priority card type
    pub third_choice: CardType,
    /// Whether to prefer forming chains
    pub prefer_chains: bool,
    /// Whether to prefer brave chains over color chains
    pub prefer_brave_chains: bool,
}

impl Default for CardPriority {
    fn default() -> Self {
        Self {
            first_choice: CardType::Buster,
            second_choice: CardType::Arts,
            third_choice: CardType::Quick,
            prefer_chains: true,
            prefer_brave_chains: true,
        }
    }
}

impl CardPriority {
    /// Get the priority score for a card type (higher = better)
    pub fn score(&self, card_type: CardType) -> u32 {
        if card_type == self.first_choice {
            3
        } else if card_type == self.second_choice {
            2
        } else if card_type == self.third_choice {
            1
        } else {
            0
        }
    }
}

/// Skill usage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSettings {
    /// Whether to automatically use skills
    pub auto_use_skills: bool,
    /// Whether to prioritize NP charge skills
    pub np_charge_priority: bool,
    /// Specific skill orders per turn
    pub skill_order: Option<Vec<SkillCommand>>,
    /// Whether to use master skills
    pub use_master_skills: bool,
    /// Master skill order
    pub master_skill_order: Option<Vec<MasterSkillCommand>>,
}

impl Default for SkillSettings {
    fn default() -> Self {
        Self {
            auto_use_skills: true,
            np_charge_priority: true,
            skill_order: None,
            use_master_skills: true,
            master_skill_order: None,
        }
    }
}

/// A skill command (servant skill)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    /// Servant index (0-2)
    pub servant: usize,
    /// Skill index (0-2)
    pub skill: usize,
    /// Target servant index (if applicable)
    pub target: Option<usize>,
    /// Wave to use this skill (1-indexed)
    pub wave: Option<u32>,
    /// Turn to use this skill (1-indexed, within wave)
    pub turn: Option<u32>,
}

/// A master skill command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterSkillCommand {
    /// Skill index (0-2)
    pub skill: usize,
    /// Target servant index (if applicable)
    pub target: Option<usize>,
    /// Wave to use this skill
    pub wave: Option<u32>,
    /// Turn to use this skill
    pub turn: Option<u32>,
}

/// Team configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    /// Servant configurations
    pub servants: Vec<ServantConfig>,
    /// Support servant filter
    pub support_filter: Option<SupportFilter>,
}

/// Individual servant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServantConfig {
    /// Servant ID (from game data)
    pub id: Option<u32>,
    /// Servant name for display
    pub name: Option<String>,
    /// Expected class
    pub class: ServantClass,
    /// Position (0-5)
    pub position: usize,
    /// NP card type
    pub np_type: crate::game::servant::NPType,
    /// Whether this is the main damage dealer
    pub is_dps: bool,
    /// Custom skill order for this servant
    pub skill_order: Option<Vec<u32>>,
}

/// Support servant filter settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportFilter {
    /// Preferred servant class
    pub class: Option<ServantClass>,
    /// Preferred servant IDs
    pub servant_ids: Vec<u32>,
    /// Minimum NP level
    pub min_np_level: Option<u32>,
    /// Required CE (craft essence) ID
    pub ce_id: Option<u32>,
    /// Whether CE must be MLB (max limit broken)
    pub ce_mlb: bool,
}

/// General automation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationSettings {
    /// Auto-continue through result screens
    pub auto_continue: bool,
    /// Loop farming
    pub loop_farming: bool,
    /// Maximum loop count (0 = unlimited)
    pub max_loops: u32,
    /// Use apples for AP recovery
    pub use_apples: bool,
    /// Apple type preference (gold, silver, bronze)
    pub apple_preference: ApplePreference,
    /// Stop on error
    pub stop_on_error: bool,
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            auto_continue: true,
            loop_farming: false,
            max_loops: 0,
            use_apples: false,
            apple_preference: ApplePreference::Bronze,
            stop_on_error: true,
        }
    }
}

/// Apple type preference for AP recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplePreference {
    /// Golden apples (full AP)
    Gold,
    /// Silver apples (half AP)
    Silver,
    /// Bronze apples (10 AP)
    Bronze,
    /// Don't use apples
    None,
}

/// Timing settings for screen interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSettings {
    /// Delay between taps (ms)
    pub tap_delay: u32,
    /// Delay after card selection (ms)
    pub card_delay: u32,
    /// Delay after skill use (ms)
    pub skill_delay: u32,
    /// Delay after NP selection (ms)
    pub np_delay: u32,
    /// Wait time for screen transitions (ms)
    pub transition_wait: u32,
    /// Wait time for attack animations (ms)
    pub attack_wait: u32,
}

impl Default for TimingSettings {
    fn default() -> Self {
        Self {
            tap_delay: 300,
            card_delay: 200,
            skill_delay: 500,
            np_delay: 300,
            transition_wait: 2000,
            attack_wait: 3000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.np_threshold, 100);
        assert!(settings.target_low_hp_first);
    }

    #[test]
    fn test_card_priority_score() {
        let priority = CardPriority::default();
        assert_eq!(priority.score(CardType::Buster), 3);
        assert_eq!(priority.score(CardType::Arts), 2);
        assert_eq!(priority.score(CardType::Quick), 1);
        assert_eq!(priority.score(CardType::NP), 0);
    }

    #[test]
    fn test_farming_preset() {
        let settings = Settings::farming_preset();
        assert!(settings.automation.auto_continue);
        assert!(settings.automation.loop_farming);
        assert!(settings.skill_settings.auto_use_skills);
    }

    #[test]
    fn test_boss_preset() {
        let settings = Settings::boss_fight_preset();
        assert!(!settings.automation.loop_farming);
        assert!(!settings.skill_settings.auto_use_skills);
        assert!(settings.prioritize_class_advantage);
    }
}
