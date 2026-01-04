//! JNI function exports for Android integration
//!
//! These functions are called from Kotlin via JNI to interact with
//! the Rust automation core.

use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint, jlong, JNI_TRUE};
use jni::JNIEnv;

use crate::config::Settings;
use crate::{get_sheba, init_sheba, ShebaAction};

/// Initialize the Sheba automation engine
///
/// Called once when the Android service starts.
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_init<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config_json: JString<'local>,
) -> jboolean {
    // Initialize Android logger
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Sheba"),
    );

    log::info!("Initializing Sheba automation engine");

    // Parse configuration
    let settings = if config_json.is_null() {
        Settings::default()
    } else {
        match env.get_string(&config_json) {
            Ok(config_str) => {
                let config: String = config_str.into();
                serde_json::from_str(&config).unwrap_or_default()
            }
            Err(e) => {
                log::error!("Failed to get config string: {}", e);
                Settings::default()
            }
        }
    };

    init_sheba(settings);
    log::info!("Sheba initialized successfully");

    JNI_TRUE
}

/// Process a screen frame and get the next action
///
/// Returns an action code that the Android side interprets.
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_processFrame<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    frame_data: JByteArray<'local>,
    width: jint,
    height: jint,
) -> jlong {
    let Some(sheba) = get_sheba() else {
        log::error!("Sheba not initialized");
        return 0;
    };

    // Get frame data from Java byte array
    let frame_bytes = match env.convert_byte_array(frame_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Failed to convert frame data: {}", e);
            return 0;
        }
    };

    // Process the frame
    let mut sheba = match sheba.lock() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to lock Sheba: {}", e);
            return 0;
        }
    };

    let action = sheba.process_frame(&frame_bytes, width as u32, height as u32);

    // Encode action as a long for efficient JNI transfer
    encode_action(&action)
}

/// Get the current game state as JSON
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_getGameState<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    let Some(sheba) = get_sheba() else {
        return env.new_string("{}").unwrap();
    };

    let sheba = match sheba.lock() {
        Ok(s) => s,
        Err(_) => return env.new_string("{}").unwrap(),
    };

    let state_json = serde_json::to_string(&sheba.game_state).unwrap_or_else(|_| "{}".to_string());

    env.new_string(state_json).unwrap()
}

/// Update settings
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_updateSettings<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    settings_json: JString<'local>,
) -> jboolean {
    let Some(sheba) = get_sheba() else {
        return 0;
    };

    let settings_str: String = match env.get_string(&settings_json) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    let settings: Settings = match serde_json::from_str(&settings_str) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse settings: {}", e);
            return 0;
        }
    };

    let mut sheba = match sheba.lock() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    sheba.settings = settings;
    JNI_TRUE
}

/// Pause/resume automation
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_setPaused<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    paused: jboolean,
) {
    let Some(sheba) = get_sheba() else {
        return;
    };

    let mut sheba = match sheba.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    if paused != 0 {
        sheba.game_state.pause();
    } else {
        sheba.game_state.resume();
    }
}

/// Get the action type from an encoded action
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_getActionType<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    action_code: jlong,
) -> jint {
    ((action_code >> 56) & 0xFF) as jint
}

/// Get action X coordinate
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_getActionX<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    action_code: jlong,
) -> jint {
    ((action_code >> 32) & 0xFFFFFF) as jint
}

/// Get action Y coordinate
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_getActionY<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    action_code: jlong,
) -> jint {
    ((action_code >> 8) & 0xFFFFFF) as jint
}

/// Get action data (duration, index, etc.)
#[no_mangle]
pub extern "system" fn Java_io_sheba_ShebaCore_getActionData<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    action_code: jlong,
) -> jint {
    (action_code & 0xFF) as jint
}

/// Encode a ShebaAction into a long for efficient JNI transfer
///
/// Format (64 bits):
/// - Bits 56-63: Action type (8 bits)
/// - Bits 32-55: X coordinate or param1 (24 bits)
/// - Bits 8-31: Y coordinate or param2 (24 bits)
/// - Bits 0-7: Extra data (8 bits)
fn encode_action(action: &ShebaAction) -> jlong {
    match action {
        ShebaAction::None => 0,

        ShebaAction::Tap { x, y } => {
            let action_type: i64 = 1;
            (action_type << 56) | ((*x as i64 & 0xFFFFFF) << 32) | ((*y as i64 & 0xFFFFFF) << 8)
        }

        ShebaAction::Swipe {
            start_x,
            start_y,
            end_x: _,
            end_y: _,
            duration_ms: _,
        } => {
            // Swipe needs more data, so we encode start coords only
            // End coords and duration are fetched separately
            let action_type: i64 = 2;
            (action_type << 56)
                | ((*start_x as i64 & 0xFFFFFF) << 32)
                | ((*start_y as i64 & 0xFFFFFF) << 8)
        }

        ShebaAction::Wait { duration_ms } => {
            let action_type: i64 = 3;
            (action_type << 56) | (*duration_ms as i64)
        }

        ShebaAction::SelectCards { card_indices } => {
            let action_type: i64 = 4;
            // Encode up to 3 card indices
            let c0 = card_indices.first().copied().unwrap_or(0) as i64;
            let c1 = card_indices.get(1).copied().unwrap_or(0) as i64;
            let c2 = card_indices.get(2).copied().unwrap_or(0) as i64;
            (action_type << 56) | (c0 << 16) | (c1 << 8) | c2
        }

        ShebaAction::UseSkill {
            servant_idx,
            skill_idx,
            target,
        } => {
            let action_type: i64 = 5;
            let t = target.unwrap_or(255) as i64;
            (action_type << 56)
                | ((*servant_idx as i64) << 16)
                | ((*skill_idx as i64) << 8)
                | t
        }

        ShebaAction::UseNP { servant_idx } => {
            let action_type: i64 = 6;
            (action_type << 56) | (*servant_idx as i64)
        }

        ShebaAction::TargetEnemy { enemy_idx } => {
            let action_type: i64 = 7;
            (action_type << 56) | (*enemy_idx as i64)
        }

        ShebaAction::TapAttack => {
            let action_type: i64 = 8;
            action_type << 56
        }

        ShebaAction::UseMasterSkill { skill_idx, target } => {
            let action_type: i64 = 9;
            let t = target.unwrap_or(255) as i64;
            (action_type << 56) | ((*skill_idx as i64) << 8) | t
        }
    }
}

/// Action type constants (must match Kotlin side)
pub mod action_types {
    pub const NONE: i32 = 0;
    pub const TAP: i32 = 1;
    pub const SWIPE: i32 = 2;
    pub const WAIT: i32 = 3;
    pub const SELECT_CARDS: i32 = 4;
    pub const USE_SKILL: i32 = 5;
    pub const USE_NP: i32 = 6;
    pub const TARGET_ENEMY: i32 = 7;
    pub const TAP_ATTACK: i32 = 8;
    pub const USE_MASTER_SKILL: i32 = 9;
}
