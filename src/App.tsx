import { useState, useEffect, useCallback } from 'react'
import {
  type Mapping,
  type MappingState,
  GAMEPAD_BUTTON_NAMES,
  GAMEPAD_AXIS_NAMES,
  BUTTON_DISPLAY_NAMES,
  AXIS_DISPLAY_NAMES,
  type GamepadButton,
  type GamepadAxis,
  DEFAULT_DEADZONE,
} from './types'
import {
  loadMappings,
  saveMappings,
  addMapping,
  removeMapping,
  updateMapping,
} from './services/storage'
import { setMappings } from './services/mapper'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { GamepadVisual } from './components/GamepadVisual'
import { MappingRow } from './components/MappingRow'
import { KeyPicker } from './components/KeyPicker'
import './App.css'

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

function App() {
  const [state, setState] = useState<MappingState>({
    mappings: [],
    enabled: true,
    activeGamepadIndex: null,
  })
  const [gamepad, setGamepad] = useState<Gamepad | null>(null)
  const [isAddingMapping, setIsAddingMapping] = useState(false)
  const [addingStep, setAddingStep] = useState<'source' | 'key' | null>(null)
  const [pendingSource, setPendingSource] = useState<{
    type: 'button' | 'axis'
    index: number
    name: GamepadButton | GamepadAxis
  } | null>(null)
  const [toasts, setToasts] = useState<{ id: string; msg: string; type: 'info' | 'error' | 'success' }[]>([])
  const [debugInfo, setDebugInfo] = useState<string>('')

  const addToast = useCallback((msg: string, type: 'info' | 'error' | 'success' = 'info') => {
    const id = generateId()
    setToasts(prev => [...prev, { id, msg, type }])
    setTimeout(() => setToasts(prev => prev.filter(t => t.id !== id)), 5000)
  }, [])

  const refreshState = useCallback(async () => {
    const loaded = await loadMappings()
    setState(loaded)
    setMappings(loaded.mappings, loaded.enabled)
  }, [])

  useEffect(() => {
    refreshState()
  }, [refreshState])

  useEffect(() => {
    const unlisten = listen('gamepad_status', (event) => {
      const payload = event.payload as { status: 'connected' | 'disconnected', active: boolean }
      if (payload.status === 'connected' && payload.active) {
        const dummyGamepad: Gamepad = {
          id: 'Gamepad (Background)',
          index: 0,
          connected: true,
          timestamp: Date.now(),
          mapping: 'standard',
          axes: [0, 0, 0, 0],
          buttons: Array(17).fill({ pressed: false, touched: false, value: 0 }),
          vibrationActuator: undefined as unknown as Gamepad['vibrationActuator'],
        }
        setGamepad(dummyGamepad)
        addToast('Gamepad detected!', 'success')
      } else {
        setGamepad(null)
        addToast('Gamepad disconnected', 'error')
      }
    })
    return () => {
      unlisten.then(fn => fn())
    }
  }, [addToast])

  // On mount, fetch current backend state to detect already-connected gamepad
  useEffect(() => {
    invoke<[boolean, number, boolean]>('get_status').then(([_enabled, _mappingCount, active]) => {
      if (active) {
        const dummyGamepad: Gamepad = {
          id: 'Gamepad (Background)',
          index: 0,
          connected: true,
          timestamp: Date.now(),
          mapping: 'standard',
          axes: [0, 0, 0, 0],
          buttons: Array(17).fill({ pressed: false, touched: false, value: 0 }),
          vibrationActuator: undefined as unknown as Gamepad['vibrationActuator'],
        }
        setGamepad(dummyGamepad)
        // No toast here because this is just restoring state, not a new detection
      }
    }).catch(e => {
      console.error('Failed to get backend status:', e)
    })
  }, [])

  useEffect(() => {
    invoke<string[]>('list_gamepads').then(names => {
      setDebugInfo(`gilrs sees: ${names.length ? names.join(', ') : 'none'}`)
      if (names.length === 0) {
        addToast('No gamepads found by backend. Check USB connection.', 'error')
      }
    }).catch(e => {
      setDebugInfo(`list_gamepads error: ${e}`)
      addToast(`Backend error: ${e}`, 'error')
    })
  }, [addToast])

  const handleToggleEnabled = async () => {
    const newEnabled = !state.enabled
    setState((prev) => ({ ...prev, enabled: newEnabled }))
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('set_enabled', { enabled: newEnabled })
      saveMappings({ ...state, enabled: newEnabled }).catch((e) =>
        console.error('Failed to save enabled state to storage:', e)
      )
    } catch (e) {
      console.error('Failed to sync enabled state:', e)
    }
  }

  const handleDeleteMapping = async (id: string) => {
    await removeMapping(id)
    await refreshState()
  }

  const handleUpdateMapping = async (id: string, keyCode: string) => {
    await updateMapping(id, { keyCode })
    await refreshState()
  }

  const handleButtonClick = (buttonIndex: number, buttonName: GamepadButton) => {
    setPendingSource({
      type: 'button',
      index: buttonIndex,
      name: buttonName,
    })
    setIsAddingMapping(true)
    setAddingStep('key')
  }

  const handleAxisClick = (axisIndex: number, axisName: GamepadAxis) => {
    setPendingSource({
      type: 'axis',
      index: axisIndex,
      name: axisName,
    })
    setIsAddingMapping(true)
    setAddingStep('key')
  }

  const handleKeySelect = async (keyCode: string) => {
    if (!pendingSource) return

    const mapping: Mapping = {
      id: generateId(),
      sourceType: pendingSource.type === 'button' ? 'button' : 'axis_positive',
      sourceIndex: pendingSource.index,
      sourceName: pendingSource.name,
      keyCode,
      deadzone: DEFAULT_DEADZONE,
    }

    await addMapping(mapping)
    await refreshState()
    setIsAddingMapping(false)
    setAddingStep(null)
    setPendingSource(null)
  }

  const handleCancelAdd = () => {
    setIsAddingMapping(false)
    setAddingStep(null)
    setPendingSource(null)
  }

  const handleStartAddMapping = () => {
    setIsAddingMapping(true)
    setAddingStep('source')
  }

  const handleSourceSelect = (
    type: 'button' | 'axis',
    index: number,
    name: GamepadButton | GamepadAxis
  ) => {
    setPendingSource({ type, index, name })
    setAddingStep('key')
  }

  return (
    <div className="app">
      <div className="toast-container">
        {toasts.map(t => (
          <div key={t.id} className={`toast toast-${t.type}`}>
            {t.msg}
          </div>
        ))}
      </div>

      {debugInfo && (
        <div className="debug-panel">
          <code>{debugInfo}</code>
        </div>
      )}

      <header className="app-header">
        <div className="app-title">
          <span className="app-icon">🎮</span>
          <h1>Gamepad Mapper</h1>
        </div>
        <button
          className={`toggle-btn ${state.enabled ? 'enabled' : 'disabled'}`}
          onClick={handleToggleEnabled}
          type="button"
        >
          {state.enabled ? 'ON' : 'OFF'}
        </button>
      </header>

      <section className="gamepad-section">
        <h2 className="section-title">Controller</h2>
        <GamepadVisual
          gamepad={gamepad}
          onButtonClick={handleButtonClick}
          onAxisClick={handleAxisClick}
        />
        {gamepad && (
          <p className="gamepad-info">
            {gamepad.id.split('(')[0].trim()}
          </p>
        )}
      </section>

      {isAddingMapping && addingStep === 'key' && pendingSource && (
        <div className="add-mapping-overlay">
          <div className="add-mapping-modal">
            <h3>Assign Key</h3>
            <p className="add-mapping-source">
              {pendingSource.type === 'button'
                ? BUTTON_DISPLAY_NAMES[pendingSource.name as GamepadButton]
                : AXIS_DISPLAY_NAMES[pendingSource.name as GamepadAxis]}
            </p>
            <KeyPicker onSelect={handleKeySelect} onCancel={handleCancelAdd} />
          </div>
        </div>
      )}

      <section className="mappings-section">
        <div className="mappings-header">
          <h2 className="section-title">
            Mappings
            <span className="mappings-count">{state.mappings.length}</span>
          </h2>
          <button
            className="add-mapping-btn"
            onClick={handleStartAddMapping}
            disabled={isAddingMapping}
            type="button"
          >
            + Add Mapping
          </button>
        </div>

        {isAddingMapping && addingStep === 'source' && (
          <div className="source-selector">
            <p className="selector-title">Click a button on the controller</p>
            <div className="selector-options">
              <div className="selector-group">
                <p className="selector-group-title">Buttons</p>
                <div className="selector-buttons">
                  {GAMEPAD_BUTTON_NAMES.map((name, index) => (
                    <button
                      key={name}
                      className="selector-btn"
                      onClick={() => handleSourceSelect('button', index, name)}
                      type="button"
                    >
                      {BUTTON_DISPLAY_NAMES[name]}
                    </button>
                  ))}
                </div>
              </div>
              <div className="selector-group">
                <p className="selector-group-title">Axes</p>
                <div className="selector-buttons">
                  {GAMEPAD_AXIS_NAMES.map((name, index) => (
                    <button
                      key={name}
                      className="selector-btn"
                      onClick={() => handleSourceSelect('axis', index, name)}
                      type="button"
                    >
                      {AXIS_DISPLAY_NAMES[name]}
                    </button>
                  ))}
                </div>
              </div>
            </div>
            <button
              className="selector-cancel"
              onClick={handleCancelAdd}
              type="button"
            >
              Cancel
            </button>
          </div>
        )}

        <div className="mappings-list">
          {state.mappings.length === 0 ? (
            <p className="empty-state">
              No mappings yet. Click "Add Mapping" or click a controller button.
            </p>
          ) : (
            state.mappings.map((mapping) => (
              <MappingRow
                key={mapping.id}
                mapping={mapping}
                onUpdate={handleUpdateMapping}
                onDelete={handleDeleteMapping}
              />
            ))
          )}
        </div>
      </section>
    </div>
  )
}

export default App
