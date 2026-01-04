//! Human behavior simulation for anti-detection
//!
//! This module adds realistic variance to automated actions to avoid
//! bot detection heuristics used by games.

use rand::Rng;

/// Human reaction time range in milliseconds
const MIN_REACTION_TIME_MS: u64 = 180;
const MAX_REACTION_TIME_MS: u64 = 350;

/// Visual processing time range
const MIN_PROCESSING_TIME_MS: u64 = 100;
const MAX_PROCESSING_TIME_MS: u64 = 300;

/// Tap duration range (how long finger stays on screen)
const MIN_TAP_DURATION_MS: u64 = 50;
const MAX_TAP_DURATION_MS: u64 = 150;

/// Humanizer for generating realistic timing and positions
pub struct Humanizer {
    rng: rand::rngs::ThreadRng,
}

impl Default for Humanizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Humanizer {
    /// Create a new humanizer
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    /// Get a humanized delay for an action
    ///
    /// Combines reaction time + visual processing time + occasional hesitation
    pub fn get_action_delay(&mut self) -> u64 {
        let reaction_time = self
            .rng
            .gen_range(MIN_REACTION_TIME_MS..=MAX_REACTION_TIME_MS);
        let processing_time = self
            .rng
            .gen_range(MIN_PROCESSING_TIME_MS..=MAX_PROCESSING_TIME_MS);

        // 5% chance of hesitation (200-800ms)
        let hesitation = if self.rng.gen::<f32>() < 0.05 {
            self.rng.gen_range(200..=800)
        } else {
            0
        };

        reaction_time + processing_time + hesitation
    }

    /// Get delay between consecutive quick actions
    pub fn get_consecutive_delay(&mut self) -> u64 {
        self.rng.gen_range(80..=250)
    }

    /// Get tap hold duration
    pub fn get_tap_duration(&mut self) -> u64 {
        self.rng
            .gen_range(MIN_TAP_DURATION_MS..=MAX_TAP_DURATION_MS)
    }

    /// Humanize a delay with variance
    pub fn humanize_delay(&mut self, base_delay_ms: u64, variance_percent: u32) -> u64 {
        if variance_percent == 0 {
            return base_delay_ms;
        }

        let variance = (base_delay_ms as f64 * variance_percent as f64 / 100.0) as i64;
        let offset = self.rng.gen_range(-variance..=variance);

        (base_delay_ms as i64 + offset).max(50) as u64
    }

    /// Humanize tap position with slight offset
    /// Returns (offset_x, offset_y) to add to the target position
    pub fn humanize_position(&mut self, max_offset: i32) -> (i32, i32) {
        if max_offset == 0 {
            return (0, 0);
        }

        // Use gaussian-like distribution for more realistic spread
        let offset_x = self.gaussian_offset(max_offset);
        let offset_y = self.gaussian_offset(max_offset);

        (offset_x, offset_y)
    }

    /// Generate gaussian-distributed offset
    fn gaussian_offset(&mut self, max_offset: i32) -> i32 {
        // Simple approximation using sum of uniform randoms
        let sum: f32 = (0..3).map(|_| self.rng.gen::<f32>() - 0.5).sum();

        (sum * max_offset as f32 * 0.67) as i32
    }

    /// Check if a micro-pause should occur
    pub fn should_micro_pause(&mut self, probability: f32) -> bool {
        self.rng.gen::<f32>() < probability
    }

    /// Get micro-pause duration
    pub fn get_micro_pause_duration(&mut self) -> u64 {
        self.rng.gen_range(500..=2000)
    }

    /// Get card selection thinking time (first card takes longer)
    pub fn get_card_selection_time(&mut self, is_first: bool) -> u64 {
        if is_first {
            self.rng.gen_range(300..=800)
        } else {
            self.rng.gen_range(100..=300)
        }
    }

    /// Get NP recognition delay
    pub fn get_np_recognition_delay(&mut self) -> u64 {
        self.rng.gen_range(200..=500)
    }

    /// Get confirmation button delay (humans pause before important clicks)
    pub fn get_confirmation_delay(&mut self) -> u64 {
        self.rng.gen_range(150..=400)
    }

    /// Check if a break should be taken after battles
    pub fn should_take_break(&mut self, battles_completed: u32) -> bool {
        if battles_completed > 0 && battles_completed.is_multiple_of(5) {
            self.rng.gen::<f32>() < 0.15
        } else {
            false
        }
    }

    /// Get break duration
    pub fn get_break_duration(&mut self) -> u64 {
        self.rng.gen_range(3000..=10000)
    }
}

/// Humanized action with timing information
#[derive(Debug, Clone)]
pub struct HumanizedAction {
    /// The action to perform
    pub action_type: ActionType,
    /// X coordinate (possibly humanized)
    pub x: i32,
    /// Y coordinate (possibly humanized)
    pub y: i32,
    /// Pre-action delay in ms
    pub pre_delay_ms: u64,
    /// Tap/press duration in ms
    pub duration_ms: u64,
}

/// Types of actions that can be humanized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Tap,
    Swipe,
    LongPress,
}

impl HumanizedAction {
    /// Create a humanized tap action
    pub fn tap(humanizer: &mut Humanizer, x: i32, y: i32, max_offset: i32) -> Self {
        let (offset_x, offset_y) = humanizer.humanize_position(max_offset);

        Self {
            action_type: ActionType::Tap,
            x: x + offset_x,
            y: y + offset_y,
            pre_delay_ms: humanizer.get_action_delay(),
            duration_ms: humanizer.get_tap_duration(),
        }
    }

    /// Create a humanized consecutive tap (faster, for card selections)
    pub fn consecutive_tap(
        humanizer: &mut Humanizer,
        x: i32,
        y: i32,
        max_offset: i32,
        is_first: bool,
    ) -> Self {
        let (offset_x, offset_y) = humanizer.humanize_position(max_offset);

        Self {
            action_type: ActionType::Tap,
            x: x + offset_x,
            y: y + offset_y,
            pre_delay_ms: humanizer.get_card_selection_time(is_first),
            duration_ms: humanizer.get_tap_duration(),
        }
    }

    /// Create a tap without humanization (for testing)
    pub fn raw_tap(x: i32, y: i32) -> Self {
        Self {
            action_type: ActionType::Tap,
            x,
            y,
            pre_delay_ms: 0,
            duration_ms: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanizer_delays() {
        let mut humanizer = Humanizer::new();

        // Generate multiple delays and check they're in valid range
        for _ in 0..100 {
            let delay = humanizer.get_action_delay();
            assert!(delay >= MIN_REACTION_TIME_MS + MIN_PROCESSING_TIME_MS);
            assert!(delay <= MAX_REACTION_TIME_MS + MAX_PROCESSING_TIME_MS + 800);
        }
    }

    #[test]
    fn test_humanizer_position() {
        let mut humanizer = Humanizer::new();

        // Generate multiple offsets and check they're bounded
        for _ in 0..100 {
            let (x, y) = humanizer.humanize_position(10);
            assert!((-10..=10).contains(&x));
            assert!((-10..=10).contains(&y));
        }
    }

    #[test]
    fn test_humanize_delay_variance() {
        let mut humanizer = Humanizer::new();
        let base = 500u64;
        let variance = 30u32;

        let mut min_seen = base;
        let mut max_seen = base;

        for _ in 0..1000 {
            let delay = humanizer.humanize_delay(base, variance);
            min_seen = min_seen.min(delay);
            max_seen = max_seen.max(delay);
        }

        // Should see variance in both directions
        assert!(min_seen < base);
        assert!(max_seen > base);
    }

    #[test]
    fn test_zero_variance_returns_base() {
        let mut humanizer = Humanizer::new();

        for _ in 0..10 {
            let delay = humanizer.humanize_delay(500, 0);
            assert_eq!(delay, 500);
        }
    }
}
