use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use gilrs::ev::Button;
use gilrs::{EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tauri::{AppHandle, Emitter, State};

#[cfg(windows)]
mod xinput_polling {
    use super::*;
    use std::mem::zeroed;
    use std::sync::OnceLock;

    const XINPUT_GAMEPAD_DPAD_UP: u16 = 0x0001;
    const XINPUT_GAMEPAD_DPAD_DOWN: u16 = 0x0004;
    const XINPUT_GAMEPAD_DPAD_LEFT: u16 = 0x0008;
    const XINPUT_GAMEPAD_DPAD_RIGHT: u16 = 0x0002;
    const XINPUT_GAMEPAD_START: u16 = 0x0010;
    const XINPUT_GAMEPAD_BACK: u16 = 0x0020;
    const XINPUT_GAMEPAD_LEFT_THUMB: u16 = 0x0040;
    const XINPUT_GAMEPAD_RIGHT_THUMB: u16 = 0x0080;
    const XINPUT_GAMEPAD_LEFT_SHOULDER: u16 = 0x0100;
    const XINPUT_GAMEPAD_RIGHT_SHOULDER: u16 = 0x0200;
    const XINPUT_GAMEPAD_A: u16 = 0x1000;
    const XINPUT_GAMEPAD_B: u16 = 0x2000;
    const XINPUT_GAMEPAD_X: u16 = 0x4000;
    const XINPUT_GAMEPAD_Y: u16 = 0x8000;

    const XINPUT_BUTTON_MAP: [(u16, usize); 14] = [
        (XINPUT_GAMEPAD_A, 0),
        (XINPUT_GAMEPAD_B, 1),
        (XINPUT_GAMEPAD_X, 2),
        (XINPUT_GAMEPAD_Y, 3),
        (XINPUT_GAMEPAD_LEFT_SHOULDER, 4),
        (XINPUT_GAMEPAD_RIGHT_SHOULDER, 5),
        (XINPUT_GAMEPAD_BACK, 8),
        (XINPUT_GAMEPAD_START, 9),
        (XINPUT_GAMEPAD_LEFT_THUMB, 10),
        (XINPUT_GAMEPAD_RIGHT_THUMB, 11),
        (XINPUT_GAMEPAD_DPAD_UP, 12),
        (XINPUT_GAMEPAD_DPAD_DOWN, 13),
        (XINPUT_GAMEPAD_DPAD_LEFT, 14),
        (XINPUT_GAMEPAD_DPAD_RIGHT, 15),
    ];

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct XInputGamepad {
        buttons: u16,
        left_trigger: u8,
        right_trigger: u8,
        thumb_lx: i16,
        thumb_ly: i16,
        thumb_rx: i16,
        thumb_ry: i16,
    }

    #[repr(C)]
    struct XInputState {
        packet_number: u32,
        gamepad: XInputGamepad,
    }

    type XInputGetStateFn = unsafe extern "system" fn(u32, *mut XInputState) -> u32;

    fn get_xinput_fn() -> Option<XInputGetStateFn> {
        static FN: OnceLock<Option<XInputGetStateFn>> = OnceLock::new();
        *FN.get_or_init(|| {
            let lib = libloading::Library::new("xinput1_4.dll")
                .or_else(|_| libloading::Library::new("xinput9_1_0.dll"))
                .ok()?;
            unsafe { lib.get(b"XInputGetState").ok() }
        })
    }

    pub fn poll_xinput(
        mappings: &[Mapping],
        key_map: &HashMap<String, Key>,
        pressed: &mut HashMap<String, String>,
        enigo: &Mutex<Enigo>,
        enabled: bool,
        last_buttons: &mut u16,
        last_axes: &mut [f32; 6],
    ) -> Result<(), String> {
        const DEADZONE: f32 = 0.15;
        const XINPUT_DEADZONE: f32 = 7849.0;

        let get_state = get_xinput_fn().ok_or("XInput not available")?;
        let mut state: XInputState = unsafe { zeroed() };
        let result = unsafe { get_state(0, &mut state) };
        if result != 0 {
            return Err(format!("XInputGetState failed: {}", result));
        }

        let buttons = state.gamepad.buttons;
        let mut axes = [0.0f32; 6];
        axes[0] = state.gamepad.thumb_lx as f32 / 32767.0;
        axes[1] = state.gamepad.thumb_ly as f32 / 32767.0;
        axes[2] = state.gamepad.thumb_rx as f32 / 32767.0;
        axes[3] = state.gamepad.thumb_ry as f32 / 32767.0;
        axes[4] = state.gamepad.left_trigger as f32 / 255.0;
        axes[5] = state.gamepad.right_trigger as f32 / 255.0;

        let changed = buttons ^ *last_buttons;
        for &(mask, w3c_idx) in XINPUT_BUTTON_MAP.iter() {
            if changed & mask == 0 {
                continue;
            }
            let is_now = buttons & mask != 0;
            if !enabled {
                continue;
            }
            if is_now {
                for mapping in mappings.iter() {
                    if mapping.source_type == "button" && mapping.source_index == w3c_idx {
                        if let Some(key) = resolve_key(&mapping.key_code, key_map) {
                            if let Ok(mut en) = enigo.lock() {
                                let _ = en.key(key, Direction::Press);
                                pressed.insert(mapping.id.clone(), mapping.key_code.clone());
                            }
                        }
                    }
                }
            } else {
                let to_release: Vec<String> = pressed
                    .iter()
                    .filter(|(mid, _)| {
                        mappings.iter().any(|m| {
                            m.id == **mid && m.source_type == "button" && m.source_index == w3c_idx
                        })
                    })
                    .map(|(mid, _)| mid.clone())
                    .collect();
                for mid in to_release {
                    if let Some(kc) = pressed.remove(&mid) {
                        if let Some(key) = resolve_key(&kc, key_map) {
                            if let Ok(mut en) = enigo.lock() {
                                let _ = en.key(key, Direction::Release);
                            }
                        }
                    }
                }
            }
        }

        if enabled {
            for i in 0..6 {
                let value = if i < 4 {
                    let v = axes[i];
                    if v.abs() < XINPUT_DEADZONE / 32767.0 {
                        0.0
                    } else {
                        v
                    }
                } else {
                    axes[i]
                };
                let last = last_axes[i];
                if (value - last).abs() > 0.01 {
                    if last <= DEADZONE && value > DEADZONE {
                        if let Some(m) = mappings
                            .iter()
                            .find(|m| m.source_type == "axis_positive" && m.source_index == i)
                        {
                            if let Some(key) = resolve_key(&m.key_code, key_map) {
                                if let Ok(mut en) = enigo.lock() {
                                    let _ = en.key(key, Direction::Press);
                                    pressed.insert(m.id.clone(), m.key_code.clone());
                                }
                            }
                        }
                    }
                    if last > DEADZONE && value <= DEADZONE {
                        if let Some(m) = mappings
                            .iter()
                            .find(|m| m.source_type == "axis_positive" && m.source_index == i)
                        {
                            if let Some(key) = resolve_key(&m.key_code, key_map) {
                                if let Ok(mut en) = enigo.lock() {
                                    let _ = en.key(key, Direction::Release);
                                    pressed.remove(&m.id);
                                }
                            }
                        }
                    }
                    if last >= -DEADZONE && value < -DEADZONE {
                        if let Some(m) = mappings
                            .iter()
                            .find(|m| m.source_type == "axis_negative" && m.source_index == i)
                        {
                            if let Some(key) = resolve_key(&m.key_code, key_map) {
                                if let Ok(mut en) = enigo.lock() {
                                    let _ = en.key(key, Direction::Press);
                                    pressed.insert(m.id.clone(), m.key_code.clone());
                                }
                            }
                        }
                    }
                    if last < -DEADZONE && value >= -DEADZONE {
                        if let Some(m) = mappings
                            .iter()
                            .find(|m| m.source_type == "axis_negative" && m.source_index == i)
                        {
                            if let Some(key) = resolve_key(&m.key_code, key_map) {
                                if let Ok(mut en) = enigo.lock() {
                                    let _ = en.key(key, Direction::Release);
                                    pressed.remove(&m.id);
                                }
                            }
                        }
                    }
                    last_axes[i] = value;
                }
            }
        }

        *last_buttons = buttons;
        Ok(())
    }
}

/// Convert gilrs Button enum to W3C Gamepad API button index.
/// Frontend stores mappings using W3C indices, so we must convert.
fn gilrs_btn_to_w3c(btn: Button) -> Option<usize> {
    match btn {
        Button::South => Some(0),        // A
        Button::East => Some(1),         // B
        Button::West => Some(2),         // X
        Button::North => Some(3),        // Y
        Button::LeftTrigger => Some(4),  // LB
        Button::RightTrigger => Some(5), // RB
        Button::Select => Some(8),       // Back/Select
        Button::Start => Some(9),        // Start
        Button::LeftThumb => Some(10),   // L3
        Button::RightThumb => Some(11),  // R3
        Button::DPadUp => Some(12),
        Button::DPadDown => Some(13),
        Button::DPadLeft => Some(14),
        Button::DPadRight => Some(15),
        Button::Mode => Some(16), // Home/Guide
        _ => None,
    }
}

struct AppState {
    enigo: Mutex<Enigo>,
    key_map: Mutex<HashMap<String, Key>>,
    mappings: RwLock<Vec<Mapping>>,
    enabled: RwLock<bool>,
    active_gamepad: RwLock<Option<gilrs::GamepadId>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Mapping {
    id: String,
    source_type: String,
    source_index: usize,
    source_name: String,
    key_code: String,
    deadzone: Option<f32>,
}

fn build_key_map() -> HashMap<String, Key> {
    let mut map = HashMap::new();
    map.insert("KeyA".into(), Key::Unicode('a'));
    map.insert("KeyB".into(), Key::Unicode('b'));
    map.insert("KeyC".into(), Key::Unicode('c'));
    map.insert("KeyD".into(), Key::Unicode('d'));
    map.insert("KeyE".into(), Key::Unicode('e'));
    map.insert("KeyF".into(), Key::Unicode('f'));
    map.insert("KeyG".into(), Key::Unicode('g'));
    map.insert("KeyH".into(), Key::Unicode('h'));
    map.insert("KeyI".into(), Key::Unicode('i'));
    map.insert("KeyJ".into(), Key::Unicode('j'));
    map.insert("KeyK".into(), Key::Unicode('k'));
    map.insert("KeyL".into(), Key::Unicode('l'));
    map.insert("KeyM".into(), Key::Unicode('m'));
    map.insert("KeyN".into(), Key::Unicode('n'));
    map.insert("KeyO".into(), Key::Unicode('o'));
    map.insert("KeyP".into(), Key::Unicode('p'));
    map.insert("KeyQ".into(), Key::Unicode('q'));
    map.insert("KeyR".into(), Key::Unicode('r'));
    map.insert("KeyS".into(), Key::Unicode('s'));
    map.insert("KeyT".into(), Key::Unicode('t'));
    map.insert("KeyU".into(), Key::Unicode('u'));
    map.insert("KeyV".into(), Key::Unicode('v'));
    map.insert("KeyW".into(), Key::Unicode('w'));
    map.insert("KeyX".into(), Key::Unicode('x'));
    map.insert("KeyY".into(), Key::Unicode('y'));
    map.insert("KeyZ".into(), Key::Unicode('z'));
    map.insert("Digit0".into(), Key::Unicode('0'));
    map.insert("Digit1".into(), Key::Unicode('1'));
    map.insert("Digit2".into(), Key::Unicode('2'));
    map.insert("Digit3".into(), Key::Unicode('3'));
    map.insert("Digit4".into(), Key::Unicode('4'));
    map.insert("Digit5".into(), Key::Unicode('5'));
    map.insert("Digit6".into(), Key::Unicode('6'));
    map.insert("Digit7".into(), Key::Unicode('7'));
    map.insert("Digit8".into(), Key::Unicode('8'));
    map.insert("Digit9".into(), Key::Unicode('9'));
    map.insert("ArrowUp".into(), Key::UpArrow);
    map.insert("ArrowDown".into(), Key::DownArrow);
    map.insert("ArrowLeft".into(), Key::LeftArrow);
    map.insert("ArrowRight".into(), Key::RightArrow);
    map.insert("Home".into(), Key::Home);
    map.insert("End".into(), Key::End);
    map.insert("PageUp".into(), Key::PageUp);
    map.insert("PageDown".into(), Key::PageDown);
    #[cfg(not(target_os = "macos"))]
    map.insert("Insert".into(), Key::Insert);
    map.insert("Delete".into(), Key::Delete);
    map.insert("Enter".into(), Key::Return);
    map.insert("Space".into(), Key::Space);
    map.insert("Tab".into(), Key::Tab);
    map.insert("Backspace".into(), Key::Backspace);
    map.insert("Escape".into(), Key::Escape);
    map.insert("ShiftLeft".into(), Key::LShift);
    map.insert("ShiftRight".into(), Key::RShift);
    map.insert("ControlLeft".into(), Key::LControl);
    map.insert("ControlRight".into(), Key::RControl);
    map.insert("AltLeft".into(), Key::Alt);
    map.insert("AltRight".into(), Key::Alt);
    map.insert("MetaLeft".into(), Key::Meta);
    map.insert("MetaRight".into(), Key::Meta);
    map.insert("F1".into(), Key::F1);
    map.insert("F2".into(), Key::F2);
    map.insert("F3".into(), Key::F3);
    map.insert("F4".into(), Key::F4);
    map.insert("F5".into(), Key::F5);
    map.insert("F6".into(), Key::F6);
    map.insert("F7".into(), Key::F7);
    map.insert("F8".into(), Key::F8);
    map.insert("F9".into(), Key::F9);
    map.insert("F10".into(), Key::F10);
    map.insert("F11".into(), Key::F11);
    map.insert("F12".into(), Key::F12);
    map.insert("CapsLock".into(), Key::CapsLock);
    #[cfg(not(target_os = "macos"))]
    map.insert("NumLock".into(), Key::Numlock);
    #[cfg(not(target_os = "macos"))]
    map.insert("PrintScreen".into(), Key::PrintScr);
    #[cfg(not(target_os = "macos"))]
    map.insert("Pause".into(), Key::Pause);
    map.insert("Numpad0".into(), Key::Numpad0);
    map.insert("Numpad1".into(), Key::Numpad1);
    map.insert("Numpad2".into(), Key::Numpad2);
    map.insert("Numpad3".into(), Key::Numpad3);
    map.insert("Numpad4".into(), Key::Numpad4);
    map.insert("Numpad5".into(), Key::Numpad5);
    map.insert("Numpad6".into(), Key::Numpad6);
    map.insert("Numpad7".into(), Key::Numpad7);
    map.insert("Numpad8".into(), Key::Numpad8);
    map.insert("Numpad9".into(), Key::Numpad9);
    map.insert("NumpadAdd".into(), Key::Add);
    map.insert("NumpadSubtract".into(), Key::Subtract);
    map.insert("NumpadMultiply".into(), Key::Multiply);
    map.insert("NumpadDivide".into(), Key::Divide);
    map.insert("NumpadDecimal".into(), Key::Decimal);
    map.insert("NumpadEnter".into(), Key::Return);
    map.insert("Minus".into(), Key::Unicode('-'));
    map.insert("Equal".into(), Key::Unicode('='));
    map.insert("BracketLeft".into(), Key::Unicode('['));
    map.insert("BracketRight".into(), Key::Unicode(']'));
    map.insert("Backslash".into(), Key::Unicode('\\'));
    map.insert("Semicolon".into(), Key::Unicode(';'));
    map.insert("Quote".into(), Key::Unicode('\''));
    map.insert("Backquote".into(), Key::Unicode('`'));
    map.insert("Comma".into(), Key::Unicode(','));
    map.insert("Period".into(), Key::Unicode('.'));
    map.insert("Slash".into(), Key::Unicode('/'));
    map
}

fn resolve_key(name: &str, key_map: &HashMap<String, Key>) -> Option<Key> {
    if let Some(key) = key_map.get(name) {
        return Some(*key);
    }
    let chars: Vec<char> = name.chars().collect();
    if chars.len() == 1 {
        return Some(Key::Unicode(chars[0]));
    }
    None
}

#[tauri::command]
fn set_mappings(mappings: Vec<Mapping>, state: State<'_, Arc<AppState>>) {
    let mut m = state.mappings.write().unwrap();
    *m = mappings;
}

#[tauri::command]
fn set_enabled(enabled: bool, state: State<'_, Arc<AppState>>) {
    let mut e = state.enabled.write().unwrap();
    *e = enabled;
}

#[tauri::command]
fn get_status(state: State<'_, Arc<AppState>>) -> (bool, usize, bool) {
    let enabled = *state.enabled.read().unwrap();
    let len = state.mappings.read().unwrap().len();
    let active = state.active_gamepad.read().unwrap().is_some();
    (enabled, len, active)
}

#[tauri::command]
fn list_gamepads(state: State<'_, Arc<AppState>>) -> Vec<String> {
    let active = state.active_gamepad.read().unwrap();
    match *active {
        Some(id) => vec![format!("Active gamepad ID: {:?}", id)],
        None => vec!["No gamepad detected by backend".to_string()],
    }
}

#[tauri::command]
fn test_key(key_code: &str, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let key_map = state.key_map.lock().map_err(|e| e.to_string())?;
    let key =
        resolve_key(key_code, &key_map).ok_or_else(|| format!("Unknown key: {}", key_code))?;
    drop(key_map);
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;
    enigo
        .key(key, Direction::Click)
        .map_err(|e| format!("{:?}", e))
}

fn gamepad_loop(app: AppHandle, state: Arc<AppState>) {
    let mut gilrs: Gilrs = match Gilrs::new() {
        Ok(g) => {
            log::info!("gilrs initialized successfully");
            g
        }
        Err(e) => {
            log::error!("Failed to init gilrs: {}", e);
            return;
        }
    };

    let mut pressed: HashMap<String, String> = HashMap::new();

    for (id, gp) in gilrs.gamepads() {
        log::info!("Found already-connected gamepad: {:?} ({})", id, gp.name());
        if state.active_gamepad.read().unwrap().is_none() {
            *state.active_gamepad.write().unwrap() = Some(id);
            let _ = app.emit(
                "gamepad_status",
                serde_json::json!({"status": "connected", "active": true}),
            );
        }
        break;
    }

    let mut last_button_states: HashMap<usize, HashMap<gilrs::ev::Button, bool>> = HashMap::new();
    let mut last_axis_values: HashMap<usize, [f32; 6]> = HashMap::new();
    #[cfg(windows)]
    let mut last_xinput_buttons: u16 = 0;
    #[cfg(windows)]
    let mut last_xinput_axes: [f32; 6] = [0.0; 6];

    loop {
        // Process events first to update internal state cache (critical for Windows XInput)
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::Connected => {
                    log::info!("Gamepad connected (event): {:?}", event.id);
                    if state.active_gamepad.read().unwrap().is_none() {
                        *state.active_gamepad.write().unwrap() = Some(event.id);
                        let _ = app.emit(
                            "gamepad_status",
                            serde_json::json!({"status": "connected", "active": true}),
                        );
                    }
                }
                EventType::Disconnected => {
                    log::info!("Gamepad disconnected: {:?}", event.id);
                    let active = state.active_gamepad.read().unwrap();
                    if *active == Some(event.id) {
                        drop(active);
                        *state.active_gamepad.write().unwrap() = None;
                        let _ = app.emit(
                            "gamepad_status",
                            serde_json::json!({"status": "disconnected", "active": false}),
                        );
                    }
                    let eid: usize = event.id.into();
                    last_button_states.remove(&eid);
                    last_axis_values.remove(&eid);
                }
                _ => {}
            }
        }

        let mappings = state.mappings.read().unwrap();
        let enabled = *state.enabled.read().unwrap();
        let key_map = state.key_map.lock().unwrap();

        #[cfg(windows)]
        let xinput_connected = xinput_polling::poll_xinput(
            &mappings,
            &key_map,
            &mut pressed,
            &state.enigo,
            enabled,
            &mut last_xinput_buttons,
            &mut last_xinput_axes,
        )
        .is_ok();
        #[cfg(not(windows))]
        let xinput_connected = false;

        for (id, gp) in gilrs.gamepads() {
            let id_usize: usize = id.into();
            if state.active_gamepad.read().unwrap().is_none() {
                *state.active_gamepad.write().unwrap() = Some(id);
                let _ = app.emit(
                    "gamepad_status",
                    serde_json::json!({"status": "connected", "active": true}),
                );
                log::info!("Gamepad detected: {:?}", id);
            }

            if !gp.is_connected() {
                let active = state.active_gamepad.read().unwrap();
                if *active == Some(id) {
                    drop(active);
                    *state.active_gamepad.write().unwrap() = None;
                    let _ = app.emit(
                        "gamepad_status",
                        serde_json::json!({"status": "disconnected", "active": false}),
                    );
                }
                last_button_states.remove(&id_usize);
                last_axis_values.remove(&id_usize);
                continue;
            }

            let btn_states = last_button_states.entry(id_usize).or_default();
            let axis_vals = last_axis_values.entry(id_usize).or_insert([0.0; 6]);

            if !xinput_connected && enabled {
                let all_buttons = [
                    gilrs::ev::Button::South,
                    gilrs::ev::Button::East,
                    gilrs::ev::Button::North,
                    gilrs::ev::Button::West,
                    gilrs::ev::Button::LeftTrigger,
                    gilrs::ev::Button::RightTrigger,
                    gilrs::ev::Button::Select,
                    gilrs::ev::Button::Start,
                    gilrs::ev::Button::LeftThumb,
                    gilrs::ev::Button::RightThumb,
                    gilrs::ev::Button::DPadUp,
                    gilrs::ev::Button::DPadDown,
                    gilrs::ev::Button::DPadLeft,
                    gilrs::ev::Button::DPadRight,
                    gilrs::ev::Button::Mode,
                ];

                for btn in all_buttons {
                    let is_now = gp.is_pressed(btn);
                    let was = btn_states.get(&btn).copied().unwrap_or(false);

                    if is_now && !was {
                        log::info!("Button {:?} pressed", btn);
                        if let Some(w3c_idx) = gilrs_btn_to_w3c(btn) {
                            log::info!("  -> W3C index {}", w3c_idx);
                            for mapping in mappings.iter() {
                                if mapping.source_type == "button"
                                    && mapping.source_index == w3c_idx
                                {
                                    if let Some(key) = resolve_key(&mapping.key_code, &key_map) {
                                        if let Ok(mut enigo) = state.enigo.lock() {
                                            let _ = enigo.key(key, Direction::Press);
                                            pressed.insert(
                                                mapping.id.clone(),
                                                mapping.key_code.clone(),
                                            );
                                            log::info!("  -> Key pressed: {:?}", key);
                                        }
                                    } else {
                                        log::info!(
                                            "  -> Key not found in key_map for: {}",
                                            mapping.key_code
                                        );
                                    }
                                }
                            }
                            if mappings
                                .iter()
                                .filter(|m| m.source_type == "button" && m.source_index == w3c_idx)
                                .count()
                                == 0
                            {
                                log::info!("  -> No mapping for W3C index {}", w3c_idx);
                            }
                        } else {
                            log::info!("  -> No W3C mapping for button {:?}", btn);
                        }
                    } else if !is_now && was {
                        log::info!("Button {:?} released", btn);
                        if let Some(w3c_idx) = gilrs_btn_to_w3c(btn) {
                            let to_release: Vec<String> = pressed
                                .iter()
                                .filter(|(mid, _)| {
                                    mappings.iter().any(|m| {
                                        m.id == **mid
                                            && m.source_type == "button"
                                            && m.source_index == w3c_idx
                                    })
                                })
                                .map(|(mid, _)| mid.clone())
                                .collect();
                            for mid in to_release {
                                if let Some(kc) = pressed.remove(&mid) {
                                    if let Some(key) = resolve_key(&kc, &key_map) {
                                        if let Ok(mut enigo) = state.enigo.lock() {
                                            let _ = enigo.key(key, Direction::Release);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    btn_states.insert(btn, is_now);
                }

                let axes = [
                    gilrs::ev::Axis::LeftStickX,
                    gilrs::ev::Axis::LeftStickY,
                    gilrs::ev::Axis::RightStickX,
                    gilrs::ev::Axis::RightStickY,
                    gilrs::ev::Axis::LeftZ,
                    gilrs::ev::Axis::RightZ,
                ];

                for (i, axis) in axes.iter().enumerate() {
                    let value = gp.value(*axis);
                    let last = axis_vals[i];
                    let dz = 0.15;

                    if (value - last).abs() > 0.01 {
                        if last <= dz && value > dz {
                            if let Some(m) = mappings
                                .iter()
                                .find(|m| m.source_type == "axis_positive" && m.source_index == i)
                            {
                                if let Some(key) = resolve_key(&m.key_code, &key_map) {
                                    if let Ok(mut enigo) = state.enigo.lock() {
                                        let _ = enigo.key(key, Direction::Press);
                                        pressed.insert(m.id.clone(), m.key_code.clone());
                                    }
                                }
                            }
                        }
                        if last > dz && value <= dz {
                            if let Some(m) = mappings
                                .iter()
                                .find(|m| m.source_type == "axis_positive" && m.source_index == i)
                            {
                                if let Some(key) = resolve_key(&m.key_code, &key_map) {
                                    if let Ok(mut enigo) = state.enigo.lock() {
                                        let _ = enigo.key(key, Direction::Release);
                                        pressed.remove(&m.id);
                                    }
                                }
                            }
                        }
                        if last >= -dz && value < -dz {
                            if let Some(m) = mappings
                                .iter()
                                .find(|m| m.source_type == "axis_negative" && m.source_index == i)
                            {
                                if let Some(key) = resolve_key(&m.key_code, &key_map) {
                                    if let Ok(mut enigo) = state.enigo.lock() {
                                        let _ = enigo.key(key, Direction::Press);
                                        pressed.insert(m.id.clone(), m.key_code.clone());
                                    }
                                }
                            }
                        }
                        if last < -dz && value >= -dz {
                            if let Some(m) = mappings
                                .iter()
                                .find(|m| m.source_type == "axis_negative" && m.source_index == i)
                            {
                                if let Some(key) = resolve_key(&m.key_code, &key_map) {
                                    if let Ok(mut enigo) = state.enigo.lock() {
                                        let _ = enigo.key(key, Direction::Release);
                                        pressed.remove(&m.id);
                                    }
                                }
                            }
                        }
                        axis_vals[i] = value;
                    }
                }
            }
        }

        thread::sleep(std::time::Duration::from_millis(16));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Gamepad Keyboard Mapper starting...");

    let enigo = Enigo::new(&Settings::default()).expect("Failed to init Enigo");
    let key_map = build_key_map();
    let state = Arc::new(AppState {
        enigo: Mutex::new(enigo),
        key_map: Mutex::new(key_map),
        mappings: RwLock::new(Vec::new()),
        enabled: RwLock::new(true),
        active_gamepad: RwLock::new(None),
    });

    let state_clone = Arc::clone(&state);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(Arc::clone(&state))
        .setup(move |app| {
            let handle = app.handle().clone();
            thread::spawn(move || gamepad_loop(handle, state_clone));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_mappings,
            set_enabled,
            get_status,
            test_key,
            list_gamepads,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
