import { Store } from '@tauri-apps/plugin-store'
import { invoke } from '@tauri-apps/api/core'
import type { MappingState } from '../types'

const STORE_PATH = 'store.json'
const STORAGE_KEY = 'gamepad_mappings'

const DEFAULT_STATE: MappingState = {
  mappings: [],
  enabled: true,
  activeGamepadIndex: null,
}

let store: Store | null = null

async function getStore(): Promise<Store> {
  if (store) return store
  store = await Store.load(STORE_PATH)
  return store
}

export async function loadMappings(): Promise<MappingState> {
  try {
    const s = await getStore()
    const value = await s.get<MappingState>(STORAGE_KEY)
    return value ?? DEFAULT_STATE
  } catch {
    return DEFAULT_STATE
  }
}

export async function saveMappings(state: MappingState): Promise<void> {
  const s = await getStore()
  await s.set(STORAGE_KEY, state)
  await s.save()
  // Notify Rust backend of updated mappings and enabled state
  await invoke('set_mappings', { mappings: state.mappings })
  await invoke('set_enabled', { enabled: state.enabled })
}

export async function toggleEnabled(): Promise<boolean> {
  const state = await loadMappings()
  state.enabled = !state.enabled
  await saveMappings(state)
  return state.enabled
}

export async function addMapping(mapping: MappingState['mappings'][number]): Promise<void> {
  const state = await loadMappings()
  state.mappings.push(mapping)
  await saveMappings(state)
}

export async function removeMapping(id: string): Promise<void> {
  const state = await loadMappings()
  state.mappings = state.mappings.filter(m => m.id !== id)
  await saveMappings(state)
}

export async function updateMapping(id: string, updates: Partial<MappingState['mappings'][number]>): Promise<void> {
  const state = await loadMappings()
  const index = state.mappings.findIndex(m => m.id === id)
  if (index !== -1) {
    state.mappings[index] = { ...state.mappings[index], ...updates }
    await saveMappings(state)
  }
}
