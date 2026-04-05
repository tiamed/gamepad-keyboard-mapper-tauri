use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use gilrs::ev::Button;
use gilrs::{EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tauri::{AppHandle, Emitter, State};

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
fn set_mappings(mappings: Vec<Mapping>, state: State<AppState>) {
    let mut m = state.mappings.write().unwrap();
    *m = mappings;
}

#[tauri::command]
fn set_enabled(enabled: bool, state: State<AppState>) {
    let mut e = state.enabled.write().unwrap();
    *e = enabled;
}

#[tauri::command]
fn get_status(state: State<AppState>) -> (bool, usize, bool) {
    let enabled = *state.enabled.read().unwrap();
    let len = state.mappings.read().unwrap().len();
    let active = state.active_gamepad.read().unwrap().is_some();
    (enabled, len, active)
}

#[tauri::command]
fn list_gamepads(state: State<AppState>) -> Vec<String> {
    let active = state.active_gamepad.read().unwrap();
    match *active {
        Some(id) => vec![format!("Active gamepad ID: {:?}", id)],
        None => vec!["No gamepad detected by backend".to_string()],
    }
}

#[tauri::command]
fn test_key(key_code: &str, state: State<AppState>) -> Result<(), String> {
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

    loop {
        let mappings = state.mappings.read().unwrap();
        let enabled = *state.enabled.read().unwrap();
        let key_map = state.key_map.lock().unwrap();

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

            if enabled {
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
                        if let Some(w3c_idx) = gilrs_btn_to_w3c(btn) {
                            log::info!("Button {:?} pressed (W3C {})", btn, w3c_idx);
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
                                        }
                                    }
                                }
                            }
                        }
                    } else if !is_now && was {
                        if let Some(w3c_idx) = gilrs_btn_to_w3c(btn) {
                            log::info!("Button {:?} released (W3C {})", btn, w3c_idx);
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
        .manage(state)
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
