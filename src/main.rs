//! FGO Sheba CLI - Testing and development entry point
//!
//! This binary is used for testing the automation logic on desktop
//! without requiring an Android device.

use fgo_sheba::config::Settings;
use fgo_sheba::Sheba;

fn main() {
    println!("FGO Sheba - AI-powered FGO Automation");
    println!("=====================================");
    println!();
    println!("This is the CLI testing interface.");
    println!("For Android deployment, build as a library and use the Android app.");
    println!();

    // Initialize with default settings
    let settings = Settings::default();
    let sheba = Sheba::new(settings);

    println!("Sheba initialized successfully!");
    println!();
    println!("Modules loaded:");
    println!("  - Vision System: Ready");
    println!("  - Battle AI: Ready");
    println!("  - Game State: Ready");
    println!();
    println!("To test with screenshots, use the following workflow:");
    println!("  1. Capture a screenshot from FGO");
    println!("  2. Load it using the vision system");
    println!("  3. Process with the battle AI");
    println!();

    // Print current configuration
    println!("Current Configuration:");
    println!("  - Card Priority: {:?}", sheba.settings.card_priority);
    println!("  - NP Threshold: {}%", sheba.settings.np_threshold);
    println!("  - Target Low HP First: {}", sheba.settings.target_low_hp_first);
}
