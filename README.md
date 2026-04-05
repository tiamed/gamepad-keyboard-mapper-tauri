# Gamepad Keyboard Mapper

Cross-platform gamepad to keyboard mapper built with Tauri v2. Turn your game controller into a virtual keyboard that works in browsers, games, and any application, even when the app is running in the background.

Works exactly like DS4Windows, but for Windows, Linux and macOS.

---

## ✅ Features

| Feature | Status |
|---|---|
| ✅ **Native Windows XInput support** | No gilrs bugs, reliable Xbox controller detection |
| ✅ **Background operation** | Works even when minimized or hidden |
| ✅ **Real-time UI feedback** | Visual gamepad controller that animates when buttons are pressed |
| ✅ **Zero latency** | 16ms polling loop, same performance as DS4Windows |
| ✅ **Cross platform** | Windows, Linux, macOS support |
| ✅ **Full controller support** | All 17 buttons + 6 axes (sticks + triggers) |
| ✅ **Custom button mappings** | Map any controller button/axis to any keyboard key |
| ✅ **Deadzone configuration** | Adjustable axis deadzones |
| ✅ **Debug logging toggle** | Optional console logging for button events |
| ✅ **Works everywhere** | Browsers, games, Steam, desktop applications |

---

## 🎯 Purpose

This app allows you to:
- Use a game controller to navigate web browsers
- Play browser games that don't natively support gamepads
- Control desktop applications with your controller
- Create custom control schemes for any software
- Use your controller even when the gamepad mapper itself is in the background or minimized

---

## 🛠️ Technology Stack

| Component | Library |
|---|---|
| Backend | **Rust + Tauri v2** |
| Controller Input | Native Windows XInput / gilrs (Linux/macOS) |
| Keyboard Emulation | **enigo v0.6** |
| Frontend | React 18 + TypeScript + Vite |
| UI | Vanilla CSS with CSS Grid + CSS animations |

---

## 🚀 How it works

```
Hardware Button Press
       ↓
Native OS controller polling (Rust backend thread)
       ↓
Button state change detected
       ↓
✅ Enigo sends native OS keyboard event
       ↓
✅ Tauri `button_event` IPC event sent to frontend
       ↓
✅ Gamepad UI animates with button press visual feedback
```

All controller polling happens in a dedicated background thread with zero dependency on window focus. The frontend UI exists only for configuration and visual feedback.

---

## 📦 Building from source

```bash
# Install dependencies
npm install

# Run development build
npm run tauri dev

# Build release
npm run tauri build
```

---

## 🔧 Platform Notes

### Windows
✅ Uses native XInput directly
✅ No window focus required
✅ Works with all Xbox and XInput compatible controllers
✅ Steam Remote Play compatible

### Linux
✅ Uses evdev via gilrs
✅ Requires udev rules for controller access
✅ Works with most standard controllers

### macOS
✅ Uses IOKit via gilrs
✅ No force feedback support
