import { useState, useEffect, useCallback } from 'react'
import './KeyPicker.css'

interface KeyPickerProps {
  onSelect: (keyCode: string) => void
  onCancel: () => void
}

export function KeyPicker({ onSelect, onCancel }: KeyPickerProps) {
  const [isListening, setIsListening] = useState(false)

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      event.preventDefault()
      onSelect(event.code)
      setIsListening(false)
    },
    [onSelect]
  )

  useEffect(() => {
    if (isListening) {
      window.addEventListener('keydown', handleKeyDown, { once: true })
      return () => {
        window.removeEventListener('keydown', handleKeyDown)
      }
    }
  }, [isListening, handleKeyDown])

  if (!isListening) {
    return (
      <div className="key-picker-trigger">
        <button
          className="key-picker-btn"
          onClick={() => setIsListening(true)}
          type="button"
        >
          Press to set key
        </button>
        <button
          className="key-picker-cancel"
          onClick={onCancel}
          type="button"
        >
          Cancel
        </button>
      </div>
    )
  }

  return (
    <div className="key-picker-listening">
      <div className="key-picker-overlay">
        <div className="key-picker-prompt">
          <span className="key-picker-icon">⌨️</span>
          <p className="key-picker-text">Press any key...</p>
          <button
            className="key-picker-cancel-btn"
            onClick={() => {
              setIsListening(false)
              onCancel()
            }}
            type="button"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}
