//! Card types and chain calculations
//!
//! FGO has three main card types (Buster, Arts, Quick) plus Noble Phantasms.
//! This module handles card representation and chain detection.

use serde::{Deserialize, Serialize};

/// The type of a command card
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    /// Red card - high damage, low NP gain
    Buster,
    /// Blue card - medium damage, high NP gain
    Arts,
    /// Green card - low damage, generates stars
    Quick,
    /// Noble Phantasm - special attack
    NP,
    /// Unknown/unrecognized card
    Unknown,
}

impl CardType {
    /// Get the first card bonus multiplier for this card type
    pub fn first_card_bonus(&self) -> FirstCardBonus {
        match self {
            CardType::Buster => FirstCardBonus {
                damage_bonus: 0.5,
                np_bonus: 0.0,
                star_bonus: 0.0,
            },
            CardType::Arts => FirstCardBonus {
                damage_bonus: 0.0,
                np_bonus: 1.0,
                star_bonus: 0.0,
            },
            CardType::Quick => FirstCardBonus {
                damage_bonus: 0.0,
                np_bonus: 0.0,
                star_bonus: 0.2,
            },
            _ => FirstCardBonus::default(),
        }
    }

    /// Get the card type effectiveness against the given class
    pub fn effectiveness(&self, _enemy_class: &super::EnemyClass) -> f32 {
        // Card types don't have class effectiveness - that's servant class
        1.0
    }
}

/// First card bonus effects
#[derive(Debug, Clone, Default)]
pub struct FirstCardBonus {
    pub damage_bonus: f32,
    pub np_bonus: f32,
    pub star_bonus: f32,
}

/// A command card in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    /// The type of card
    pub card_type: CardType,
    /// Index of the servant this card belongs to (0-2, or 3-5 for backline)
    pub servant_idx: usize,
    /// Position in the card selection (0-4)
    pub position: usize,
    /// Confidence of the card recognition (0.0-1.0)
    pub confidence: f32,
}

impl Card {
    /// Create a new card
    pub fn new(card_type: CardType, servant_idx: usize, position: usize) -> Self {
        Self {
            card_type,
            servant_idx,
            position,
            confidence: 1.0,
        }
    }

    /// Check if this card is an NP
    pub fn is_np(&self) -> bool {
        matches!(self.card_type, CardType::NP)
    }
}

/// Type of card chain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainType {
    /// Three Buster cards - extra damage
    Buster,
    /// Three Arts cards - extra NP gain
    Arts,
    /// Three Quick cards - extra stars
    Quick,
    /// Three cards from the same servant - extra attack
    Brave,
    /// No chain
    None,
}

/// A chain of three cards
#[derive(Debug, Clone)]
pub struct Chain {
    /// The cards in the chain (always 3)
    pub cards: [Card; 3],
    /// The type of chain formed
    pub chain_type: ChainType,
    /// Whether this is also a brave chain
    pub is_brave: bool,
}

impl Chain {
    /// Create a new chain from three cards
    pub fn new(cards: [Card; 3]) -> Self {
        let chain_type = Self::detect_chain_type(&cards);
        let is_brave = Self::detect_brave_chain(&cards);

        Self {
            cards,
            chain_type,
            is_brave,
        }
    }

    /// Detect what type of chain these cards form
    fn detect_chain_type(cards: &[Card; 3]) -> ChainType {
        let types: Vec<CardType> = cards.iter().map(|c| c.card_type).collect();

        // Check for same-type chain (excluding NPs for chain detection)
        let non_np_types: Vec<CardType> = types
            .iter()
            .filter(|t| !matches!(t, CardType::NP))
            .copied()
            .collect();

        if non_np_types.len() == 3 && non_np_types.iter().all(|t| *t == non_np_types[0]) {
            match non_np_types[0] {
                CardType::Buster => ChainType::Buster,
                CardType::Arts => ChainType::Arts,
                CardType::Quick => ChainType::Quick,
                _ => ChainType::None,
            }
        } else {
            ChainType::None
        }
    }

    /// Detect if this is a brave chain (all cards from same servant)
    fn detect_brave_chain(cards: &[Card; 3]) -> bool {
        let servant_idx = cards[0].servant_idx;
        cards.iter().all(|c| c.servant_idx == servant_idx)
    }

    /// Calculate the estimated damage bonus from this chain
    pub fn damage_bonus(&self) -> f32 {
        let mut bonus = 1.0;

        match self.chain_type {
            ChainType::Buster => bonus *= 1.2,
            ChainType::Arts => {}  // Arts chains give NP, not damage
            ChainType::Quick => {} // Quick chains give stars, not damage
            ChainType::Brave => {} // Brave chains add extra attack, handled below
            ChainType::None => {}
        }

        if self.is_brave {
            bonus *= 1.0; // Brave chains add an extra attack
        }

        bonus
    }

    /// Calculate the estimated NP gain bonus from this chain
    pub fn np_gain_bonus(&self) -> f32 {
        match self.chain_type {
            ChainType::Arts => 1.2,
            _ => 1.0,
        }
    }

    /// Calculate the estimated star generation bonus from this chain
    pub fn star_bonus(&self) -> f32 {
        match self.chain_type {
            ChainType::Quick => 1.2,
            _ => 1.0,
        }
    }
}

/// Calculate all possible card combinations from available cards
pub fn calculate_possible_chains(cards: &[Card]) -> Vec<Chain> {
    let mut chains = Vec::new();

    // We need to select 3 cards from the available cards (usually 5)
    let n = cards.len();
    if n < 3 {
        return chains;
    }

    // Generate all combinations of 3 cards with ordering
    for i in 0..n {
        for j in 0..n {
            if j == i {
                continue;
            }
            for k in 0..n {
                if k == i || k == j {
                    continue;
                }
                let chain = Chain::new([cards[i].clone(), cards[j].clone(), cards[k].clone()]);
                chains.push(chain);
            }
        }
    }

    chains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buster_chain_detection() {
        let cards = [
            Card::new(CardType::Buster, 0, 0),
            Card::new(CardType::Buster, 1, 1),
            Card::new(CardType::Buster, 2, 2),
        ];
        let chain = Chain::new(cards);
        assert_eq!(chain.chain_type, ChainType::Buster);
        assert!(!chain.is_brave);
    }

    #[test]
    fn test_brave_chain_detection() {
        let cards = [
            Card::new(CardType::Buster, 0, 0),
            Card::new(CardType::Arts, 0, 1),
            Card::new(CardType::Quick, 0, 2),
        ];
        let chain = Chain::new(cards);
        assert_eq!(chain.chain_type, ChainType::None);
        assert!(chain.is_brave);
    }

    #[test]
    fn test_arts_brave_chain() {
        let cards = [
            Card::new(CardType::Arts, 0, 0),
            Card::new(CardType::Arts, 0, 1),
            Card::new(CardType::Arts, 0, 2),
        ];
        let chain = Chain::new(cards);
        assert_eq!(chain.chain_type, ChainType::Arts);
        assert!(chain.is_brave);
    }
}
