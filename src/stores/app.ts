import type { WindowState } from '@/composables/useWindowState'

import { getName, getVersion } from '@tauri-apps/api/app'
import { defineStore } from 'pinia'
import { onMounted, reactive, ref, watch } from 'vue'

const WORKSHOP_STORAGE_KEY = 'bongo_cat_workshop_settings'

interface WorkshopSettings {
  currentSkinId?: string
  furColor?: string
interface WorkshopSettings {
  currentSkinId?: string
  furColor?: string
  eyeColor?: string
  currentAccessory?: string
  soundEffectsEnabled?: boolean // Added
}

export const useAppStore = defineStore('app', () => {
  const name = ref('')
  const version = ref('')
  const windowState = reactive<WindowState>({})
  const isWorkshopPanelVisible = ref(false)

  // Workshop states
  const currentSkinId = ref('Default')
  const furColor = ref('#FFA500') // Default Orange
  const eyeColor = ref('#0000FF') // Default Blue
  const currentAccessory = ref('None')

  // Idle animation states - not persisted
  const isIdle = ref(false)
  const currentIdleAnimation = ref<string | null>(null)

  // Sound effects state
  const soundEffectsEnabled = ref(true)

  // Cat action state - not persisted
  const catActionState = ref('idle') // e.g., 'idle', 'left_paw_down', 'right_paw_down', 'mouse_move'

  // Load workshop settings from localStorage
  onMounted(()_ => {
    const storedSettings = localStorage.getItem(WORKSHOP_STORAGE_KEY)
    if (storedSettings) {
      try {
        const settings: WorkshopSettings = JSON.parse(storedSettings)
        if (settings.currentSkinId) {
          currentSkinId.value = settings.currentSkinId
        }
        if (settings.furColor) {
          furColor.value = settings.furColor
        }
        if (settings.eyeColor) {
          eyeColor.value = settings.eyeColor
        }
        if (settings.currentAccessory) {
          currentAccessory.value = settings.currentAccessory
        }
        if (typeof settings.soundEffectsEnabled === 'boolean') { // Added
          soundEffectsEnabled.value = settings.soundEffectsEnabled
        }
      } catch (e) {
        console.error('Failed to parse workshop settings from localStorage', e)
        // Initialize with defaults if parsing fails
        localStorage.removeItem(WORKSHOP_STORAGE_KEY) // Clear corrupted data
      }
    }
  })

  // Watch for changes in workshop settings and save to localStorage
  watch([currentSkinId, furColor, eyeColor, currentAccessory, soundEffectsEnabled], () => { // Added soundEffectsEnabled
    const settings: WorkshopSettings = {
      currentSkinId: currentSkinId.value,
      furColor: furColor.value,
      eyeColor: eyeColor.value,
      currentAccessory: currentAccessory.value,
      soundEffectsEnabled: soundEffectsEnabled.value, // Added
    }
    localStorage.setItem(WORKSHOP_STORAGE_KEY, JSON.stringify(settings))
  }, { deep: true })


  onMounted(async () => {
    name.value = await getName()
    version.value = await getVersion()
  })

  function toggleWorkshopPanel() {
    isWorkshopPanelVisible.value = !isWorkshopPanelVisible.value
  }

  function closeWorkshopPanel() {
    isWorkshopPanelVisible.value = false
  }

  function setCurrentSkinId(skinId: string) {
    currentSkinId.value = skinId
  }

  function setFurColor(color: string) { // Added
    furColor.value = color
  }

  function setEyeColor(color: string) {
    eyeColor.value = color
  }

  function setCurrentAccessory(accessory: string) {
    currentAccessory.value = accessory
  }

  // Actions for idle state
  function setIsIdle(status: boolean) {
    isIdle.value = status
  }

  function setCurrentIdleAnimation(animationName: string | null) {
    currentIdleAnimation.value = animationName
  }

  function setSoundEffectsEnabled(enabled: boolean) {
    soundEffectsEnabled.value = enabled
  }

  function setCatActionState(action: string) { // Added
    catActionState.value = action
  }

  return {
    name,
    version,
    windowState,
    isWorkshopPanelVisible,
    toggleWorkshopPanel,
    closeWorkshopPanel,
    // Workshop getters & actions
    currentSkinId,
    setCurrentSkinId,
    furColor,
    setFurColor,
    eyeColor,
    setEyeColor,
    currentAccessory,
    setCurrentAccessory,
    soundEffectsEnabled,
    setSoundEffectsEnabled,
    catActionState, // Added
    setCatActionState, // Added
    // Idle animation states & actions
    isIdle,
    setIsIdle,
    currentIdleAnimation,
    setCurrentIdleAnimation,
  }
})
