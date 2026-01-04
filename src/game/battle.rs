//! Battle state tracking
//!
//! Manages the state of an ongoing battle, including servants, enemies,
//! cards, and turn progression.

use serde::{Deserialize, Serialize};

use super::cards::{Card, CardType};
use super::enemy::EnemyWave;
use super::servant::Servant;

/// Current phase of the battle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePhase {
    /// Waiting for battle to start
    PreBattle,
    /// In the skill/attack selection phase
    CommandPhase,
    /// Selecting cards
    CardSelection,
    /// Watching attack animations
    AttackPhase,
    /// Enemy turn
    EnemyPhase,
    /// Battle complete (victory)
    Victory,
    /// Battle complete (defeat)
    Defeat,
    /// Unknown state
    Unknown,
}

/// Master skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterSkills {
    /// Master skill cooldowns (3 skills)
    pub cooldowns: [u32; 3],
    /// Whether the master skills are available in this quest
    pub available: bool,
}

impl Default for MasterSkills {
    fn default() -> Self {
        Self {
            cooldowns: [0, 0, 0],
            available: true,
        }
    }
}

impl MasterSkills {
    /// Check if a skill is ready
    pub fn is_ready(&self, skill_idx: usize) -> bool {
        skill_idx < 3 && self.cooldowns[skill_idx] == 0 && self.available
    }

    /// Use a master skill
    pub fn use_skill(&mut self, skill_idx: usize, cooldown: u32) {
        if skill_idx < 3 {
            self.cooldowns[skill_idx] = cooldown;
        }
    }

    /// Tick all cooldowns
    pub fn tick_cooldowns(&mut self) {
        for cd in &mut self.cooldowns {
            if *cd > 0 {
                *cd -= 1;
            }
        }
    }
}

/// Complete state of an ongoing battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    /// Current battle phase
    pub phase: BattlePhase,
    /// Current turn number
    pub turn: u32,
    /// Current wave
    pub current_wave: EnemyWave,
    /// Front-line servants (up to 3)
    pub servants: Vec<Servant>,
    /// Back-line servants (up to 3 more)
    pub backline: Vec<Servant>,
    /// Available command cards (5 cards)
    pub available_cards: Vec<Card>,
    /// Currently selected cards (up to 3)
    pub selected_cards: Vec<Card>,
    /// Whether NP cards are available (per servant)
    pub np_available: [bool; 3],
    /// Currently targeted enemy index
    pub target_enemy: Option<usize>,
    /// Master skills
    pub master_skills: MasterSkills,
    /// Critical stars available
    pub critical_stars: u32,
    /// Total waves in this battle
    pub total_waves: u32,
}

impl Default for BattleState {
    fn default() -> Self {
        Self {
            phase: BattlePhase::Unknown,
            turn: 1,
            current_wave: EnemyWave::new(1, 3),
            servants: Vec::new(),
            backline: Vec::new(),
            available_cards: Vec::new(),
            selected_cards: Vec::new(),
            np_available: [false, false, false],
            target_enemy: None,
            master_skills: MasterSkills::default(),
            critical_stars: 0,
            total_waves: 3,
        }
    }
}

impl BattleState {
    /// Create a new battle state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new turn
    pub fn start_turn(&mut self) {
        self.phase = BattlePhase::CommandPhase;
        self.selected_cards.clear();

        // Tick cooldowns
        for servant in &mut self.servants {
            servant.tick_cooldowns();
        }
        self.master_skills.tick_cooldowns();
    }

    /// Enter card selection phase
    pub fn enter_card_selection(&mut self, cards: Vec<Card>) {
        self.phase = BattlePhase::CardSelection;
        self.available_cards = cards;
        self.selected_cards.clear();

        // Update NP availability
        for (i, servant) in self.servants.iter().enumerate() {
            if i < 3 {
                self.np_available[i] = servant.can_np();
            }
        }
    }

    /// Select a card
    pub fn select_card(&mut self, card: Card) -> bool {
        if self.selected_cards.len() < 3 {
            self.selected_cards.push(card);
            true
        } else {
            false
        }
    }

    /// Select an NP
    pub fn select_np(&mut self, servant_idx: usize) -> bool {
        if servant_idx < 3 && self.np_available[servant_idx] && self.selected_cards.len() < 3 {
            self.selected_cards.push(Card {
                card_type: CardType::NP,
                servant_idx,
                position: 5 + servant_idx, // NP positions are 5, 6, 7
                confidence: 1.0,
            });
            self.np_available[servant_idx] = false;
            true
        } else {
            false
        }
    }

    /// Get the front-line servant at the given index
    pub fn get_servant(&self, idx: usize) -> Option<&Servant> {
        self.servants.get(idx)
    }

    /// Get a mutable reference to the front-line servant at the given index
    pub fn get_servant_mut(&mut self, idx: usize) -> Option<&mut Servant> {
        self.servants.get_mut(idx)
    }

    /// Get alive front-line servants
    pub fn alive_servants(&self) -> impl Iterator<Item = &Servant> {
        self.servants.iter().filter(|s| s.is_alive)
    }

    /// Count alive servants
    pub fn alive_servant_count(&self) -> usize {
        self.servants.iter().filter(|s| s.is_alive).count()
    }

    /// Get cards belonging to a specific servant
    pub fn cards_for_servant(&self, servant_idx: usize) -> Vec<&Card> {
        self.available_cards
            .iter()
            .filter(|c| c.servant_idx == servant_idx && !c.is_np())
            .collect()
    }

    /// Get cards of a specific type
    pub fn cards_of_type(&self, card_type: CardType) -> Vec<&Card> {
        self.available_cards
            .iter()
            .filter(|c| c.card_type == card_type)
            .collect()
    }

    /// Check if we're in the final wave
    pub fn is_final_wave(&self) -> bool {
        self.current_wave.is_final_wave()
    }

    /// Advance to the next wave
    pub fn next_wave(&mut self) {
        let next_wave_num = self.current_wave.wave_number + 1;
        if next_wave_num <= self.total_waves {
            self.current_wave = EnemyWave::new(next_wave_num, self.total_waves);
        }
    }

    /// Use a servant skill
    pub fn use_servant_skill(&mut self, servant_idx: usize, skill_idx: usize) {
        if let Some(servant) = self.servants.get_mut(servant_idx) {
            if skill_idx < 3 {
                servant.skills[skill_idx].use_skill();
            }
        }
    }

    /// Calculate the total NP gauge of all servants
    pub fn total_np_gauge(&self) -> u32 {
        self.servants.iter().map(|s| s.np_gauge).sum()
    }

    /// Get servants that can use NP
    pub fn servants_with_np(&self) -> Vec<usize> {
        self.servants
            .iter()
            .enumerate()
            .filter(|(_, s)| s.can_np())
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::servant::ServantClass;

    #[test]
    fn test_battle_state_initialization() {
        let state = BattleState::new();
        assert_eq!(state.turn, 1);
        assert_eq!(state.phase, BattlePhase::Unknown);
        assert!(state.servants.is_empty());
    }

    #[test]
    fn test_card_selection() {
        let mut state = BattleState::new();
        state.servants.push(Servant::new(ServantClass::Saber, 0));
        state.servants[0].np_gauge = 100;

        let cards = vec![
            Card::new(CardType::Buster, 0, 0),
            Card::new(CardType::Arts, 0, 1),
            Card::new(CardType::Quick, 0, 2),
            Card::new(CardType::Buster, 0, 3),
            Card::new(CardType::Arts, 0, 4),
        ];
        state.enter_card_selection(cards);

        assert_eq!(state.available_cards.len(), 5);
        assert!(state.np_available[0]);

        state.select_np(0);
        assert!(!state.np_available[0]);
        assert_eq!(state.selected_cards.len(), 1);
    }

    #[test]
    fn test_wave_progression() {
        let mut state = BattleState::new();
        state.total_waves = 3;
        state.current_wave = EnemyWave::new(1, 3);

        assert!(!state.is_final_wave());

        state.next_wave();
        assert_eq!(state.current_wave.wave_number, 2);

        state.next_wave();
        assert!(state.is_final_wave());
    }
}
