//! Stealth and anti-detection module
//!
//! This module provides functionality to make automation less detectable:
//! - Humanized timing with random variance
//! - Humanized tap positions with slight offsets
//! - Random micro-pauses to simulate human attention drift


pub mod humanize;

pub use humanize::*;

/// Configuration for stealth behavior
#[derive(Debug, Clone)]
pub struct StealthConfig {
    /// Enable humanized timing
    pub humanize_timing: bool,
    /// Enable humanized tap positions
    pub humanize_position: bool,
    /// Enable random micro-pauses
    pub enable_micro_pauses: bool,
    /// Base delay variance percentage (0-100)
    pub timing_variance_percent: u32,
    /// Maximum tap position offset in pixels
    pub position_offset_max: i32,
    /// Probability of micro-pause (0.0-1.0)
    pub micro_pause_probability: f32,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            humanize_timing: true,
            humanize_position: true,
            enable_micro_pauses: true,
            timing_variance_percent: 30,
            position_offset_max: 8,
            micro_pause_probability: 0.08,
        }
    }
}

impl StealthConfig {
    /// Create a config with no stealth (for testing)
    pub fn disabled() -> Self {
        Self {
            humanize_timing: false,
            humanize_position: false,
            enable_micro_pauses: false,
            timing_variance_percent: 0,
            position_offset_max: 0,
            micro_pause_probability: 0.0,
        }
    }

    /// Create a highly stealthy config
    pub fn maximum() -> Self {
        Self {
            humanize_timing: true,
            humanize_position: true,
            enable_micro_pauses: true,
            timing_variance_percent: 40,
            position_offset_max: 12,
            micro_pause_probability: 0.12,
        }
    }
}
