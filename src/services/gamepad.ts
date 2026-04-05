import { GAMEPAD_BUTTON_COUNT, GAMEPAD_AXIS_COUNT } from '../types'

export function getGamepads(): Gamepad[] {
  return navigator.getGamepads ? [...navigator.getGamepads()].filter((g): g is Gamepad => g !== null) : []
}

export function getActiveGamepad(): Gamepad | null {
  const gamepads = getGamepads()
  return gamepads.length > 0 ? gamepads[0] : null
}

export function readButton(gamepad: Gamepad, index: number): boolean {
  if (index >= GAMEPAD_BUTTON_COUNT) return false
  return gamepad.buttons[index]?.pressed ?? false
}

export function readAxis(gamepad: Gamepad, axisIndex: number): number {
  if (axisIndex >= GAMEPAD_AXIS_COUNT) return 0
  return gamepad.axes[axisIndex] ?? 0
}

export function getGamepadInfo(gamepad: Gamepad): { id: string; index: number; buttonCount: number; axisCount: number } {
  return {
    id: gamepad.id,
    index: gamepad.index,
    buttonCount: gamepad.buttons.length,
    axisCount: gamepad.axes.length,
  }
}
