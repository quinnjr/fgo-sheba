//! Android JNI bridge module
//!
//! Provides JNI bindings for communication between the Rust core
//! and the Android Accessibility Service.

pub mod bridge;
pub mod input;

pub use bridge::*;
pub use input::*;
