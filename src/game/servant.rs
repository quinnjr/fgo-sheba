//! Servant data structures
//!
//! Represents servants in battle, including their class, skills, and NP.

use serde::{Deserialize, Serialize};

/// Servant class types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServantClass {
    Saber,
    Archer,
    Lancer,
    Rider,
    Caster,
    Assassin,
    Berserker,
    Ruler,
    Avenger,
    MoonCancer,
    AlterEgo,
    Foreigner,
    Pretender,
    Beast,
    Shielder,
    Unknown,
}

impl ServantClass {
    /// Get class advantage multiplier against another class
    pub fn advantage_against(&self, enemy: &super::EnemyClass) -> f32 {
        use super::EnemyClass;
        use ServantClass::*;

        match (self, enemy) {
            // Standard triangle
            (Saber, EnemyClass::Lancer) => 2.0,
            (Saber, EnemyClass::Archer) => 0.5,
            (Archer, EnemyClass::Saber) => 2.0,
            (Archer, EnemyClass::Lancer) => 0.5,
            (Lancer, EnemyClass::Archer) => 2.0,
            (Lancer, EnemyClass::Saber) => 0.5,

            // Cavalry triangle
            (Rider, EnemyClass::Caster) => 2.0,
            (Rider, EnemyClass::Assassin) => 0.5,
            (Caster, EnemyClass::Assassin) => 2.0,
            (Caster, EnemyClass::Rider) => 0.5,
            (Assassin, EnemyClass::Rider) => 2.0,
            (Assassin, EnemyClass::Caster) => 0.5,

            // Berserker deals and receives extra damage
            (Berserker, _) => 1.5,

            // Extra classes
            (Ruler, EnemyClass::MoonCancer) => 2.0,
            (Ruler, EnemyClass::Avenger) => 0.5,
            (Avenger, EnemyClass::Ruler) => 2.0,
            (MoonCancer, EnemyClass::Avenger) => 2.0,
            (MoonCancer, EnemyClass::Ruler) => 0.5,
            (AlterEgo, EnemyClass::Cavalry) => 1.5,
            (AlterEgo, EnemyClass::Foreigner) => 2.0,
            (Foreigner, EnemyClass::Foreigner) => 2.0,
            (Foreigner, EnemyClass::Berserker) => 2.0,
            (Pretender, EnemyClass::Knight) => 1.5,

            // Neutral
            _ => 1.0,
        }
    }

    /// Check if this servant takes extra damage from the enemy class
    pub fn weak_against(&self, enemy: &super::EnemyClass) -> bool {
        self.advantage_against(enemy) < 1.0
    }
}

/// A skill that a servant has
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill name (if known)
    pub name: Option<String>,
    /// Current cooldown (0 = ready)
    pub cooldown: u32,
    /// Maximum cooldown after use
    pub max_cooldown: u32,
    /// Whether this skill requires a target
    pub requires_target: bool,
    /// Whether this skill is a damage buff
    pub is_damage_buff: bool,
    /// Whether this skill is an NP charge
    pub is_np_charge: bool,
    /// NP charge amount (percentage)
    pub np_charge_amount: u32,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: None,
            cooldown: 0,
            max_cooldown: 6,
            requires_target: false,
            is_damage_buff: false,
            is_np_charge: false,
            np_charge_amount: 0,
        }
    }
}

impl Skill {
    /// Check if the skill is ready to use
    pub fn is_ready(&self) -> bool {
        self.cooldown == 0
    }

    /// Use the skill, setting it on cooldown
    pub fn use_skill(&mut self) {
        self.cooldown = self.max_cooldown;
    }

    /// Reduce cooldown by 1 turn
    pub fn tick_cooldown(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }
}

/// NP card type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NPType {
    Buster,
    Arts,
    Quick,
}

/// A servant in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servant {
    /// Servant ID (from game data)
    pub id: Option<u32>,
    /// Servant name (if known)
    pub name: Option<String>,
    /// Servant class
    pub class: ServantClass,
    /// Current HP percentage (0.0 - 1.0)
    pub hp_percent: f32,
    /// Current NP gauge (0-300)
    pub np_gauge: u32,
    /// The servant's skills
    pub skills: [Skill; 3],
    /// NP card type
    pub np_type: NPType,
    /// Position in party (0-2 for frontline, 3-5 for backline)
    pub position: usize,
    /// Whether the servant is alive
    pub is_alive: bool,
    /// Number of active buffs
    pub buff_count: u32,
    /// Number of active debuffs
    pub debuff_count: u32,
}

impl Default for Servant {
    fn default() -> Self {
        Self {
            id: None,
            name: None,
            class: ServantClass::Unknown,
            hp_percent: 1.0,
            np_gauge: 0,
            skills: [Skill::default(), Skill::default(), Skill::default()],
            np_type: NPType::Buster,
            position: 0,
            is_alive: true,
            buff_count: 0,
            debuff_count: 0,
        }
    }
}

impl Servant {
    /// Create a new servant with the given class
    pub fn new(class: ServantClass, position: usize) -> Self {
        Self {
            class,
            position,
            ..Default::default()
        }
    }

    /// Check if NP is ready (100% or more)
    pub fn can_np(&self) -> bool {
        self.np_gauge >= 100 && self.is_alive
    }

    /// Check if NP is at overcharge level 2 (200%+)
    pub fn can_overcharge(&self) -> bool {
        self.np_gauge >= 200
    }

    /// Get the NP overcharge level (1-5)
    pub fn overcharge_level(&self) -> u32 {
        (self.np_gauge / 100).clamp(1, 5)
    }

    /// Calculate damage multiplier against an enemy class
    pub fn damage_multiplier(&self, enemy_class: &super::EnemyClass) -> f32 {
        self.class.advantage_against(enemy_class)
    }

    /// Tick all skill cooldowns
    pub fn tick_cooldowns(&mut self) {
        for skill in &mut self.skills {
            skill.tick_cooldown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::EnemyClass;

    #[test]
    fn test_class_advantage() {
        let saber = ServantClass::Saber;
        assert_eq!(saber.advantage_against(&EnemyClass::Lancer), 2.0);
        assert_eq!(saber.advantage_against(&EnemyClass::Archer), 0.5);
        assert_eq!(saber.advantage_against(&EnemyClass::Saber), 1.0);
    }

    #[test]
    fn test_np_gauge() {
        let mut servant = Servant::default();
        assert!(!servant.can_np());

        servant.np_gauge = 100;
        assert!(servant.can_np());
        assert!(!servant.can_overcharge());

        servant.np_gauge = 200;
        assert!(servant.can_overcharge());
        assert_eq!(servant.overcharge_level(), 2);
    }

    #[test]
    fn test_skill_cooldown() {
        let mut skill = Skill {
            cooldown: 0,
            max_cooldown: 5,
            ..Default::default()
        };

        assert!(skill.is_ready());
        skill.use_skill();
        assert!(!skill.is_ready());
        assert_eq!(skill.cooldown, 5);

        skill.tick_cooldown();
        assert_eq!(skill.cooldown, 4);
    }
}
