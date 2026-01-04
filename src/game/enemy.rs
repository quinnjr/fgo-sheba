//! Enemy data structures
//!
//! Represents enemies in battle, including their class, HP, and break bars.

use serde::{Deserialize, Serialize};

/// Enemy class types (similar to servant classes but includes aggregate types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyClass {
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
    /// Knight class aggregate (Saber, Archer, Lancer)
    Knight,
    /// Cavalry class aggregate (Rider, Caster, Assassin)
    Cavalry,
    Unknown,
}

impl EnemyClass {
    /// Convert to a standard servant class (for reverse lookups)
    pub fn to_servant_class(&self) -> super::ServantClass {
        use super::ServantClass;
        match self {
            EnemyClass::Saber => ServantClass::Saber,
            EnemyClass::Archer => ServantClass::Archer,
            EnemyClass::Lancer => ServantClass::Lancer,
            EnemyClass::Rider => ServantClass::Rider,
            EnemyClass::Caster => ServantClass::Caster,
            EnemyClass::Assassin => ServantClass::Assassin,
            EnemyClass::Berserker => ServantClass::Berserker,
            EnemyClass::Ruler => ServantClass::Ruler,
            EnemyClass::Avenger => ServantClass::Avenger,
            EnemyClass::MoonCancer => ServantClass::MoonCancer,
            EnemyClass::AlterEgo => ServantClass::AlterEgo,
            EnemyClass::Foreigner => ServantClass::Foreigner,
            EnemyClass::Pretender => ServantClass::Pretender,
            EnemyClass::Beast => ServantClass::Beast,
            EnemyClass::Shielder => ServantClass::Shielder,
            EnemyClass::Knight => ServantClass::Unknown,
            EnemyClass::Cavalry => ServantClass::Unknown,
            EnemyClass::Unknown => ServantClass::Unknown,
        }
    }

    /// Check if this is a knight class
    pub fn is_knight(&self) -> bool {
        matches!(
            self,
            EnemyClass::Saber | EnemyClass::Archer | EnemyClass::Lancer | EnemyClass::Knight
        )
    }

    /// Check if this is a cavalry class
    pub fn is_cavalry(&self) -> bool {
        matches!(
            self,
            EnemyClass::Rider | EnemyClass::Caster | EnemyClass::Assassin | EnemyClass::Cavalry
        )
    }
}

/// Enemy threat level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// An enemy in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    /// Enemy name (if recognized)
    pub name: Option<String>,
    /// Enemy class
    pub class: EnemyClass,
    /// Current HP percentage (0.0 - 1.0)
    pub hp_percent: f32,
    /// Number of break bars remaining
    pub break_bars: u32,
    /// Maximum break bars
    pub max_break_bars: u32,
    /// Position in enemy lineup (0-2)
    pub position: usize,
    /// Whether the enemy is alive
    pub is_alive: bool,
    /// Bounding box on screen (x, y, width, height)
    pub screen_bounds: Option<(i32, i32, i32, i32)>,
    /// Whether this is a boss enemy
    pub is_boss: bool,
    /// Whether this enemy has a dangerous NP
    pub has_dangerous_np: bool,
    /// Number of active buffs
    pub buff_count: u32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            name: None,
            class: EnemyClass::Unknown,
            hp_percent: 1.0,
            break_bars: 0,
            max_break_bars: 0,
            position: 0,
            is_alive: true,
            screen_bounds: None,
            is_boss: false,
            has_dangerous_np: false,
            buff_count: 0,
        }
    }
}

impl Enemy {
    /// Create a new enemy with the given class and position
    pub fn new(class: EnemyClass, position: usize) -> Self {
        Self {
            class,
            position,
            ..Default::default()
        }
    }

    /// Calculate the threat level of this enemy
    pub fn threat_level(&self) -> ThreatLevel {
        if self.is_boss && self.has_dangerous_np {
            return ThreatLevel::Critical;
        }

        if self.is_boss || self.has_dangerous_np {
            return ThreatLevel::High;
        }

        if self.break_bars > 0 {
            return ThreatLevel::Medium;
        }

        ThreatLevel::Low
    }

    /// Check if this enemy has break bars
    pub fn has_break_bars(&self) -> bool {
        self.break_bars > 0
    }

    /// Check if this enemy is on its last break bar
    pub fn is_last_break_bar(&self) -> bool {
        self.break_bars == 1
    }

    /// Get the tap coordinates to target this enemy
    pub fn target_coords(&self) -> Option<(i32, i32)> {
        self.screen_bounds
            .map(|(x, y, w, h)| (x + w / 2, y + h / 2))
    }

    /// Calculate how much damage is needed to kill this enemy (as a score)
    pub fn damage_needed_score(&self) -> f32 {
        // Lower HP = lower score = higher priority for killing
        let hp_factor = self.hp_percent;

        // Break bars increase the score significantly
        let break_bar_factor = 1.0 + (self.break_bars as f32 * 0.5);

        hp_factor * break_bar_factor
    }
}

/// Enemy group in a battle wave
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnemyWave {
    /// Enemies in this wave (up to 3)
    pub enemies: Vec<Enemy>,
    /// Wave number (1-indexed)
    pub wave_number: u32,
    /// Total waves in the battle
    pub total_waves: u32,
}

impl EnemyWave {
    /// Create a new wave
    pub fn new(wave_number: u32, total_waves: u32) -> Self {
        Self {
            enemies: Vec::new(),
            wave_number,
            total_waves,
        }
    }

    /// Get the number of alive enemies
    pub fn alive_count(&self) -> usize {
        self.enemies.iter().filter(|e| e.is_alive).count()
    }

    /// Get alive enemies
    pub fn alive_enemies(&self) -> impl Iterator<Item = &Enemy> {
        self.enemies.iter().filter(|e| e.is_alive)
    }

    /// Get the enemy with the lowest HP
    pub fn lowest_hp_enemy(&self) -> Option<&Enemy> {
        self.alive_enemies()
            .min_by(|a, b| a.hp_percent.partial_cmp(&b.hp_percent).unwrap())
    }

    /// Get the enemy with the highest threat
    pub fn highest_threat_enemy(&self) -> Option<&Enemy> {
        self.alive_enemies().max_by_key(|e| e.threat_level())
    }

    /// Check if this is the final wave
    pub fn is_final_wave(&self) -> bool {
        self.wave_number == self.total_waves
    }

    /// Check if any enemy has a dangerous NP
    pub fn has_dangerous_enemy(&self) -> bool {
        self.alive_enemies()
            .any(|e| e.has_dangerous_np || e.is_boss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level() {
        let mut enemy = Enemy::default();
        assert_eq!(enemy.threat_level(), ThreatLevel::Low);

        enemy.break_bars = 2;
        assert_eq!(enemy.threat_level(), ThreatLevel::Medium);

        enemy.is_boss = true;
        assert_eq!(enemy.threat_level(), ThreatLevel::High);

        enemy.has_dangerous_np = true;
        assert_eq!(enemy.threat_level(), ThreatLevel::Critical);
    }

    #[test]
    fn test_wave_alive_count() {
        let mut wave = EnemyWave::new(1, 3);
        wave.enemies.push(Enemy::new(EnemyClass::Saber, 0));
        wave.enemies.push(Enemy::new(EnemyClass::Archer, 1));
        wave.enemies.push(Enemy::new(EnemyClass::Lancer, 2));

        assert_eq!(wave.alive_count(), 3);

        wave.enemies[1].is_alive = false;
        assert_eq!(wave.alive_count(), 2);
    }
}
