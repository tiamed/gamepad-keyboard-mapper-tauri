import { invoke } from '@tauri-apps/api/core'
import type { Mapping } from '../types'

export function setMappings(mappings: Mapping[], enabled: boolean) {
  // Notify Rust backend of initial mappings and enabled state
  invoke('set_mappings', { mappings }).catch(() => {})
  invoke('set_enabled', { enabled }).catch(() => {})
}
