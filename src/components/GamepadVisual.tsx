import { useState, useEffect, useCallback } from 'react'
import {
  GAMEPAD_BUTTON_NAMES,
  GAMEPAD_AXIS_NAMES,
  BUTTON_DISPLAY_NAMES,
  AXIS_DISPLAY_NAMES,
} from '../types'
import { readButton, readAxis } from '../services/gamepad'
import type { GamepadButton, GamepadAxis } from '../types'
import './GamepadVisual.css'

interface GamepadVisualProps {
  gamepad: Gamepad | null
  pressedButtons?: Set<number>
  axisValues?: number[]
  onButtonClick: (buttonIndex: number, buttonName: GamepadButton) => void
  onAxisClick: (axisIndex: number, axisName: GamepadAxis) => void
}

interface ButtonState {
  pressed: boolean
  value: number
}

interface AxisState {
  value: number
}

export function GamepadVisual({
  gamepad,
  pressedButtons,
  axisValues,
  onButtonClick,
  onAxisClick,
}: GamepadVisualProps) {
  const [buttonStates, setButtonStates] = useState<ButtonState[]>(
    new Array(17).fill({ pressed: false, value: 0 })
  )
  const [axisStates, setAxisStates] = useState<AxisState[]>(
    new Array(4).fill({ value: 0 })
  )

  const updateStates = useCallback(() => {
    if (!gamepad) {
      setButtonStates(new Array(17).fill({ pressed: false, value: 0 }))
      setAxisStates(new Array(4).fill({ value: 0 }))
      return
    }

    const newButtonStates: ButtonState[] = []
    for (let i = 0; i < GAMEPAD_BUTTON_NAMES.length; i++) {
      newButtonStates.push({
        pressed: readButton(gamepad, i),
        value: gamepad.buttons[i]?.value ?? 0,
      })
    }
    setButtonStates(newButtonStates)

    const newAxisStates: AxisState[] = []
    for (let i = 0; i < GAMEPAD_AXIS_NAMES.length; i++) {
      newAxisStates.push({
        value: readAxis(gamepad, i),
      })
    }
    setAxisStates(newAxisStates)
  }, [gamepad])

  useEffect(() => {
    updateStates()
  }, [updateStates])

  if (!gamepad) {
    return (
      <div className="gamepad-visual gamepad-visual--disconnected">
        <div className="gamepad-disconnected-message">
          <span className="gamepad-icon">🎮</span>
          <p>No gamepad detected</p>
          <p className="gamepad-hint">Connect a controller and press any button</p>
        </div>
      </div>
    )
  }

  const handleButtonClick = (index: number) => {
    onButtonClick(index, GAMEPAD_BUTTON_NAMES[index])
  }

  const handleAxisClick = (index: number) => {
    onAxisClick(index, GAMEPAD_AXIS_NAMES[index])
  }

  const getButtonClass = (index: number, baseClass: string) => {
    const isPressed = buttonStates[index]?.pressed ?? false
    const isBackendPressed = pressedButtons?.has(index) ?? false
    return `${baseClass} ${isPressed || isBackendPressed ? 'pressed' : ''}`
  }

  // Use backend axis values as fallback when browser Gamepad API is unavailable
  // axisValues: [LeftStickX, LeftStickY, RightStickX, RightStickY, LeftTrigger, RightTrigger]
  const getAxisValue = (index: number): number => {
    const browserVal = axisStates[index]?.value ?? 0
    if (Math.abs(browserVal) > 0.01) return browserVal
    // Map backend axis indices to frontend axis indices
    // Backend: 0=LX, 1=LY, 2=RX, 3=RY, 4=LT, 5=RT
    // Frontend axisStates: 0=LX, 1=LY, 2=RX, 3=RY (only 4 stick axes)
    return axisValues?.[index] ?? 0
  }

  // Get trigger value from backend (axes 4=LT, 5=RT)
  const getTriggerValue = (buttonIndex: number): number => {
    const browserVal = buttonStates[buttonIndex]?.value ?? 0
    if (browserVal > 0.01) return browserVal
    const axisIdx = buttonIndex === 6 ? 4 : 5 // LT=axis4, RT=axis5
    return axisValues?.[axisIdx] ?? 0
  }

  return (
    <div className="gamepad-visual">
      <div className="gamepad-top-section">
        <button
          className={getButtonClass(6, 'gamepad-trigger gamepad-trigger--left')}
          onClick={() => handleButtonClick(6)}
          type="button"
          title={BUTTON_DISPLAY_NAMES.l_trigger}
        >
          <span className="trigger-label">LT</span>
          <div
            className="trigger-fill"
            style={{
              height: `${getTriggerValue(6) * 100}%`,
            }}
          />
        </button>

        <button
          className={getButtonClass(7, 'gamepad-trigger gamepad-trigger--right')}
          onClick={() => handleButtonClick(7)}
          type="button"
          title={BUTTON_DISPLAY_NAMES.r_trigger}
        >
          <span className="trigger-label">RT</span>
          <div
            className="trigger-fill"
            style={{
              height: `${getTriggerValue(7) * 100}%`,
            }}
          />
        </button>

        <button
          className={getButtonClass(4, 'gamepad-bumper gamepad-bumper--left')}
          onClick={() => handleButtonClick(4)}
          type="button"
          title={BUTTON_DISPLAY_NAMES.l_bumper}
        >
          LB
        </button>

        <button
          className={getButtonClass(5, 'gamepad-bumper gamepad-bumper--right')}
          onClick={() => handleButtonClick(5)}
          type="button"
          title={BUTTON_DISPLAY_NAMES.r_bumper}
        >
          RB
        </button>
      </div>

      <div className="gamepad-body">
        <div className="gamepad-left-section">
          <div className="dpad">
            <button
              className={getButtonClass(12, 'dpad-btn dpad-up')}
              onClick={() => handleButtonClick(12)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.dpad_up}
            >
              ▲
            </button>
            <button
              className={getButtonClass(14, 'dpad-btn dpad-left')}
              onClick={() => handleButtonClick(14)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.dpad_left}
            >
              ◀
            </button>
            <button
              className={getButtonClass(13, 'dpad-btn dpad-down')}
              onClick={() => handleButtonClick(13)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.dpad_down}
            >
              ▼
            </button>
            <button
              className={getButtonClass(15, 'dpad-btn dpad-right')}
              onClick={() => handleButtonClick(15)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.dpad_right}
            >
              ▶
            </button>
            <div className="dpad-center" />
          </div>

          <button
            className={getButtonClass(10, 'gamepad-stick gamepad-stick--left')}
            onClick={() => handleAxisClick(0)}
            type="button"
            title={`${AXIS_DISPLAY_NAMES.l_stick_x} / ${AXIS_DISPLAY_NAMES.l_stick_y}`}
          >
            <div
              className="stick-nub"
              style={{
                transform: `translate(${getAxisValue(0) * 15}px, ${
                  getAxisValue(1) * 15
                }px)`,
              }}
            >
              <span className="stick-label">L</span>
            </div>
          </button>
        </div>

        <div className="gamepad-center-section">
          <div className="center-buttons">
            <button
              className={getButtonClass(8, 'center-btn center-back')}
              onClick={() => handleButtonClick(8)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.select}
            >
              <span className="center-label">BACK</span>
            </button>
            <div className="center-logo">
              <button
                className={getButtonClass(16, 'home-btn')}
                onClick={() => handleButtonClick(16)}
                type="button"
                title={BUTTON_DISPLAY_NAMES.home}
              >
                <span className="home-icon">⌂</span>
              </button>
            </div>
            <button
              className={getButtonClass(9, 'center-btn center-start')}
              onClick={() => handleButtonClick(9)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.start}
            >
              <span className="center-label">START</span>
            </button>
          </div>
        </div>

        <div className="gamepad-right-section">
          <div className="face-buttons">
            <button
              className={getButtonClass(3, 'face-btn face-y')}
              onClick={() => handleButtonClick(3)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.face_top}
            >
              Y
            </button>
            <button
              className={getButtonClass(1, 'face-btn face-b')}
              onClick={() => handleButtonClick(1)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.face_right}
            >
              B
            </button>
            <button
              className={getButtonClass(2, 'face-btn face-x')}
              onClick={() => handleButtonClick(2)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.face_left}
            >
              X
            </button>
            <button
              className={getButtonClass(0, 'face-btn face-a')}
              onClick={() => handleButtonClick(0)}
              type="button"
              title={BUTTON_DISPLAY_NAMES.face_down}
            >
              A
            </button>
          </div>

          <button
            className={getButtonClass(11, 'gamepad-stick gamepad-stick--right')}
            onClick={() => handleAxisClick(2)}
            type="button"
            title={`${AXIS_DISPLAY_NAMES.r_stick_x} / ${AXIS_DISPLAY_NAMES.r_stick_y}`}
          >
            <div
              className="stick-nub"
              style={{
                transform: `translate(${getAxisValue(2) * 15}px, ${
                  getAxisValue(3) * 15
                }px)`,
              }}
            >
              <span className="stick-label">R</span>
            </div>
          </button>
        </div>
      </div>
      {gamepad && gamepad.id.includes('Background') && (
        <div className="backend-status">
          Connected (Background)
        </div>
      )}
    </div>
  )
}
