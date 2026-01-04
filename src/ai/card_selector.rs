//! Card selection algorithm
//!
//! Handles optimal card selection including chain formation,
//! class advantage, and first card bonus considerations.

use crate::config::Settings;
use crate::game::cards::{Card, CardType, Chain, ChainType, calculate_possible_chains};
use crate::game::enemy::EnemyWave;
use crate::game::servant::Servant;

/// Card selection engine
pub struct CardSelector {
    /// Weight for chain bonus
    chain_weight: f32,
    /// Weight for brave chain bonus
    brave_chain_weight: f32,
    /// Weight for class advantage
    class_advantage_weight: f32,
    /// Weight for first card bonus
    first_card_weight: f32,
}

impl CardSelector {
    /// Create a new card selector
    pub fn new() -> Self {
        Self {
            chain_weight: 1.5,
            brave_chain_weight: 1.3,
            class_advantage_weight: 1.2,
            first_card_weight: 1.1,
        }
    }

    /// Select optimal cards from available cards
    pub fn select_cards(
        &self,
        available_cards: &[Card],
        servants: &[Servant],
        enemies: &EnemyWave,
        np_decisions: &[usize],
        settings: &Settings,
    ) -> Vec<Card> {
        if available_cards.is_empty() {
            return Vec::new();
        }

        // Calculate scores for all possible card combinations
        let chains = calculate_possible_chains(available_cards);

        if chains.is_empty() {
            // Not enough cards for a chain, just select best individual cards
            return self.select_best_individual_cards(available_cards, servants, enemies, settings);
        }

        // Score each chain
        let mut best_chain: Option<(Chain, f32)> = None;

        for chain in chains {
            let score = self.score_chain(&chain, servants, enemies, np_decisions, settings);

            if best_chain.is_none() || score > best_chain.as_ref().unwrap().1 {
                best_chain = Some((chain, score));
            }
        }

        // Return the cards from the best chain
        if let Some((chain, _)) = best_chain {
            chain.cards.to_vec()
        } else {
            self.select_best_individual_cards(available_cards, servants, enemies, settings)
        }
    }

    /// Score a card chain based on various factors
    fn score_chain(
        &self,
        chain: &Chain,
        servants: &[Servant],
        enemies: &EnemyWave,
        np_decisions: &[usize],
        settings: &Settings,
    ) -> f32 {
        let mut score = 0.0;

        // Base score from card priorities
        for card in &chain.cards {
            score += settings.card_priority.score(card.card_type) as f32;
        }

        // Chain type bonus
        match chain.chain_type {
            ChainType::Buster => {
                score *= self.chain_weight * 1.2; // Buster chains deal extra damage
            }
            ChainType::Arts => {
                score *= self.chain_weight * 1.1; // Arts chains give NP
            }
            ChainType::Quick => {
                score *= self.chain_weight * 1.0; // Quick chains give stars
            }
            ChainType::Brave => {
                score *= self.brave_chain_weight; // Brave chains give extra attack
            }
            ChainType::None => {}
        }

        // Brave chain bonus
        if chain.is_brave {
            score *= self.brave_chain_weight;
        }

        // First card bonus consideration
        let first_card = &chain.cards[0];
        match first_card.card_type {
            CardType::Buster if settings.card_priority.first_choice == CardType::Buster => {
                score *= self.first_card_weight;
            }
            CardType::Arts if settings.card_priority.first_choice == CardType::Arts => {
                score *= self.first_card_weight;
            }
            _ => {}
        }

        // Class advantage scoring
        if settings.prioritize_class_advantage {
            for card in &chain.cards {
                if let Some(servant) = servants.get(card.servant_idx) {
                    for enemy in enemies.alive_enemies() {
                        let advantage = servant.damage_multiplier(&enemy.class);
                        if advantage > 1.0 {
                            score *= self.class_advantage_weight;
                        }
                    }
                }
            }
        }

        // Boost score if chain includes cards from servants with NP decisions
        for card in &chain.cards {
            if np_decisions.contains(&card.servant_idx) {
                score *= 1.1;
            }
        }

        score
    }

    /// Select best individual cards when chains aren't possible
    fn select_best_individual_cards(
        &self,
        available_cards: &[Card],
        servants: &[Servant],
        enemies: &EnemyWave,
        settings: &Settings,
    ) -> Vec<Card> {
        let mut scored_cards: Vec<(Card, f32)> = available_cards
            .iter()
            .map(|card| {
                let score = self.score_individual_card(card, servants, enemies, settings);
                (card.clone(), score)
            })
            .collect();

        // Sort by score (descending)
        scored_cards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top 3
        scored_cards
            .into_iter()
            .take(3)
            .map(|(card, _)| card)
            .collect()
    }

    /// Score an individual card
    fn score_individual_card(
        &self,
        card: &Card,
        servants: &[Servant],
        enemies: &EnemyWave,
        settings: &Settings,
    ) -> f32 {
        let mut score = settings.card_priority.score(card.card_type) as f32;

        // Class advantage bonus
        if settings.prioritize_class_advantage {
            if let Some(servant) = servants.get(card.servant_idx) {
                for enemy in enemies.alive_enemies() {
                    let advantage = servant.damage_multiplier(&enemy.class);
                    if advantage > 1.0 {
                        score *= advantage;
                    }
                }
            }
        }

        // Boost cards from DPS servants
        if let Some(servant) = servants.get(card.servant_idx) {
            if servant.buff_count > 0 {
                score *= 1.2; // Prioritize buffed servants
            }
        }

        score
    }

    /// Find the best first card for maximum bonus
    pub fn find_best_first_card(
        &self,
        available_cards: &[Card],
        settings: &Settings,
    ) -> Option<Card> {
        available_cards
            .iter()
            .filter(|c| c.card_type == settings.card_priority.first_choice)
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .cloned()
    }

    /// Check if a chain can be formed with available cards
    pub fn can_form_chain(&self, available_cards: &[Card], chain_type: ChainType) -> bool {
        let target_type = match chain_type {
            ChainType::Buster => CardType::Buster,
            ChainType::Arts => CardType::Arts,
            ChainType::Quick => CardType::Quick,
            _ => return false,
        };

        available_cards
            .iter()
            .filter(|c| c.card_type == target_type)
            .count()
            >= 3
    }

    /// Check if a brave chain can be formed for a servant
    pub fn can_form_brave_chain(&self, available_cards: &[Card], servant_idx: usize) -> bool {
        available_cards
            .iter()
            .filter(|c| c.servant_idx == servant_idx)
            .count()
            >= 3
    }
}

impl Default for CardSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of card selection
#[derive(Debug, Clone)]
pub struct CardSelectionResult {
    /// Selected cards in order
    pub cards: Vec<Card>,
    /// Type of chain formed (if any)
    pub chain_type: ChainType,
    /// Whether a brave chain is formed
    pub is_brave_chain: bool,
    /// Score of this selection
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_selector_creation() {
        let selector = CardSelector::new();
        assert!(selector.chain_weight > 1.0);
    }

    #[test]
    fn test_chain_detection() {
        let selector = CardSelector::new();

        let cards = vec![
            Card::new(CardType::Buster, 0, 0),
            Card::new(CardType::Buster, 1, 1),
            Card::new(CardType::Buster, 2, 2),
            Card::new(CardType::Arts, 0, 3),
            Card::new(CardType::Quick, 1, 4),
        ];

        assert!(selector.can_form_chain(&cards, ChainType::Buster));
        assert!(!selector.can_form_chain(&cards, ChainType::Arts));
    }

    #[test]
    fn test_brave_chain_detection() {
        let selector = CardSelector::new();

        let cards = vec![
            Card::new(CardType::Buster, 0, 0),
            Card::new(CardType::Arts, 0, 1),
            Card::new(CardType::Quick, 0, 2),
            Card::new(CardType::Buster, 1, 3),
            Card::new(CardType::Arts, 2, 4),
        ];

        assert!(selector.can_form_brave_chain(&cards, 0));
        assert!(!selector.can_form_brave_chain(&cards, 1));
    }
}
