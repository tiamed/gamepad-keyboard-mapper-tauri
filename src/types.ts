// W3C Gamepad standard layout indices
export type GamepadButton =
  | 'face_down'    // 0
  | 'face_right'   // 1
  | 'face_left'    // 2
  | 'face_top'     // 3
  | 'l_bumper'     // 4
  | 'r_bumper'     // 5
  | 'l_trigger'    // 6
  | 'r_trigger'    // 7
  | 'select'       // 8
  | 'start'        // 9
  | 'l_stick'      // 10
  | 'r_stick'      // 11
  | 'dpad_up'      // 12
  | 'dpad_down'    // 13
  | 'dpad_left'    // 14
  | 'dpad_right'   // 15
  | 'home'         // 16

export type GamepadAxis =
  | 'l_stick_x'
  | 'l_stick_y'
  | 'r_stick_x'
  | 'r_stick_y'

export const GAMEPAD_BUTTON_COUNT = 17
export const GAMEPAD_AXIS_COUNT = 4

export type KeyCode = string // e.g. 'KeyA', 'ArrowUp', 'Digit1', etc.

export interface Mapping {
  id: string
  sourceType: 'button' | 'axis' | 'axis_positive' | 'axis_negative'
  sourceIndex: number  // button index or axis index
  sourceName: GamepadButton | GamepadAxis
  axisDirection?: 'positive' | 'negative' // for axis mappings
  keyCode: KeyCode
  deadzone?: number // 0-1, for axis triggers
}

export interface MappingState {
  mappings: Mapping[]
  enabled: boolean
  activeGamepadIndex: number | null
}

export const DEFAULT_DEADZONE = 0.15

export const GAMEPAD_BUTTON_NAMES: GamepadButton[] = [
  'face_down', 'face_right', 'face_left', 'face_top',
  'l_bumper', 'r_bumper', 'l_trigger', 'r_trigger',
  'select', 'start', 'l_stick', 'r_stick',
  'dpad_up', 'dpad_down', 'dpad_left', 'dpad_right',
  'home',
]

export const GAMEPAD_AXIS_NAMES: GamepadAxis[] = [
  'l_stick_x', 'l_stick_y', 'r_stick_x', 'r_stick_y',
]

// Standard Xbox-style layout display names
export const BUTTON_DISPLAY_NAMES: Record<GamepadButton, string> = {
  face_down: 'A',
  face_right: 'B',
  face_left: 'X',
  face_top: 'Y',
  l_bumper: 'LB',
  r_bumper: 'RB',
  l_trigger: 'LT',
  r_trigger: 'RT',
  select: 'Back',
  start: 'Start',
  l_stick: 'L3',
  r_stick: 'R3',
  dpad_up: 'D-Pad ↑',
  dpad_down: 'D-Pad ↓',
  dpad_left: 'D-Pad ←',
  dpad_right: 'D-Pad →',
  home: 'Home',
}

export const AXIS_DISPLAY_NAMES: Record<GamepadAxis, string> = {
  l_stick_x: 'Left Stick X',
  l_stick_y: 'Left Stick Y',
  r_stick_x: 'Right Stick X',
  r_stick_y: 'Right Stick Y',
}
