import { useState } from 'react'
import type { Mapping } from '../types'
import {
  BUTTON_DISPLAY_NAMES,
  AXIS_DISPLAY_NAMES,
  type GamepadButton,
  type GamepadAxis,
} from '../types'
import { KeyPicker } from './KeyPicker'
import './MappingRow.css'

interface MappingRowProps {
  mapping: Mapping
  onUpdate: (id: string, keyCode: string) => void
  onDelete: (id: string) => void
}

function getSourceDisplayName(mapping: Mapping): string {
  if (mapping.sourceType === 'button') {
    return BUTTON_DISPLAY_NAMES[mapping.sourceName as GamepadButton] || mapping.sourceName
  }
  return AXIS_DISPLAY_NAMES[mapping.sourceName as GamepadAxis] || mapping.sourceName
}

function formatKeyCode(keyCode: string): string {
  return keyCode
    .replace(/^Key/, '')
    .replace(/^Digit/, '')
    .replace(/^Arrow/, '↑')
    .replace(/^Control/, 'Ctrl')
    .replace(/^Shift/, 'Shift')
    .replace(/^Alt/, 'Alt')
    .replace(/^Meta/, 'Win')
    .replace(/^Space$/, 'Space')
    .replace(/^Enter$/, 'Enter')
    .replace(/^Escape$/, 'Esc')
    .replace(/^Backspace$/, 'Bksp')
    .replace(/^Tab$/, 'Tab')
    .replace(/^Delete$/, 'Del')
}

export function MappingRow({ mapping, onUpdate, onDelete }: MappingRowProps) {
  const [showKeyPicker, setShowKeyPicker] = useState(false)

  const sourceName = getSourceDisplayName(mapping)
  const axisDirection = mapping.axisDirection
    ? ` (${mapping.axisDirection})`
    : ''

  const handleKeySelect = (keyCode: string) => {
    onUpdate(mapping.id, keyCode)
    setShowKeyPicker(false)
  }

  const handleCancel = () => {
    setShowKeyPicker(false)
  }

  if (showKeyPicker) {
    return (
      <div className="mapping-row mapping-row--editing">
        <span className="mapping-source">
          {sourceName}
          {axisDirection}
        </span>
        <span className="mapping-arrow">→</span>
        <KeyPicker onSelect={handleKeySelect} onCancel={handleCancel} />
      </div>
    )
  }

  return (
    <div className="mapping-row">
      <span className="mapping-source">
        {sourceName}
        {axisDirection}
      </span>
      <span className="mapping-arrow">→</span>
      <div className="mapping-key-section">
        <kbd className="mapping-key">{formatKeyCode(mapping.keyCode)}</kbd>
        <div className="mapping-actions">
          <button
            className="mapping-edit-btn"
            onClick={() => setShowKeyPicker(true)}
            type="button"
            title="Change key"
          >
            ✏️
          </button>
          <button
            className="mapping-delete-btn"
            onClick={() => onDelete(mapping.id)}
            type="button"
            title="Delete mapping"
          >
            🗑️
          </button>
        </div>
      </div>
    </div>
  )
}
