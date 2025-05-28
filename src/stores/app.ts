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
  currentAccessories?: { head: string; eyes: string; neck: string; } // Changed
  soundEffectsEnabled?: boolean
  currentPawStyle?: string
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
  const currentAccessories = ref({ head: "None", eyes: "None", neck: "None" }) // Changed

  // Idle animation states - not persisted
  const isIdle = ref(false)
  const currentIdleAnimation = ref<string | null>(null)

  // Sound effects state
  const soundEffectsEnabled = ref(true)

  // Cat action state - not persisted
  const catActionState = ref('idle') 

  // Paw style state
  const currentPawStyle = ref('Default Paws')

  // Emotion system state - not persisted
  const currentEmotion = ref('neutral')

  // Application Awareness state - not persisted
  const detectedAppReaction = ref<string | null>(null)

  // Local Music Player state - not persisted
  const localPlayerSongTitle = ref('') // Added
  const localPlayerArtistName = ref('') // Added
  const localPlayerAlbumName = ref<string | null>(null) // Added
  const localPlayerSource = ref('') // Added
  const localPlayerError = ref('') // Added

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
        if (settings.currentAccessories) { // Changed
          currentAccessories.value = settings.currentAccessories
        }
        if (typeof settings.soundEffectsEnabled === 'boolean') {
          soundEffectsEnabled.value = settings.soundEffectsEnabled
        }
        if (settings.currentPawStyle) {
          currentPawStyle.value = settings.currentPawStyle
        }
      } catch (e) {
        console.error('Failed to parse workshop settings from localStorage', e)
        // Initialize with defaults if parsing fails
        localStorage.removeItem(WORKSHOP_STORAGE_KEY) // Clear corrupted data
      }
    }
  })

  // Watch for changes in workshop settings and save to localStorage
  watch([currentSkinId, furColor, eyeColor, currentAccessories, soundEffectsEnabled, currentPawStyle], () => { // Changed currentAccessory to currentAccessories
    const settings: WorkshopSettings = {
      currentSkinId: currentSkinId.value,
      furColor: furColor.value,
      eyeColor: eyeColor.value,
      currentAccessories: currentAccessories.value, // Changed
      soundEffectsEnabled: soundEffectsEnabled.value,
      currentPawStyle: currentPawStyle.value,
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

  // Removed setCurrentAccessory, replaced by setCurrentAccessoryForCategory
  function setCurrentAccessoryForCategory(payload: { category: 'head' | 'eyes' | 'neck'; accessoryName: string }) { // Added
    if (currentAccessories.value.hasOwnProperty(payload.category)) {
      currentAccessories.value[payload.category] = payload.accessoryName;
    } else {
      console.warn(`Invalid accessory category: ${payload.category}`);
    }
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

  function setCatActionState(action: string) { 
    catActionState.value = action
  }

  function setCurrentPawStyle(style: string) {
    currentPawStyle.value = style
  }

  // setUserTypingFocused is removed, replaced by setCurrentEmotion
  
  function setDetectedAppReaction(reactionType: string | null) {
    detectedAppReaction.value = reactionType
  }

  function setCurrentEmotion(emotion: string) {
    currentEmotion.value = emotion
  }

  // Actions for Local Music Player
  function setLocalPlayerSongInfo(payload: { title: string; artist: string; album: string | null; source: string } | null) { // Added
    if (payload) {
      localPlayerSongTitle.value = payload.title;
      localPlayerArtistName.value = payload.artist;
      localPlayerAlbumName.value = payload.album;
      localPlayerSource.value = payload.source;
      localPlayerError.value = ''; // Clear error on new song info
    } else {
      clearLocalPlayerSongInfo();
    }
  }

  function clearLocalPlayerSongInfo() { // Added
    localPlayerSongTitle.value = '';
    localPlayerArtistName.value = '';
    localPlayerAlbumName.value = null;
    localPlayerSource.value = '';
  }

  function setLocalPlayerError(errorMsg: string) { // Added
    localPlayerError.value = errorMsg;
    clearLocalPlayerSongInfo();
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
    currentAccessories,
    setCurrentAccessoryForCategory,
    soundEffectsEnabled,
    setSoundEffectsEnabled,
    currentPawStyle,
    setCurrentPawStyle,
    catActionState, 
    setCatActionState, 
    currentEmotion, 
    setCurrentEmotion, 
    detectedAppReaction,
    setDetectedAppReaction,
    localPlayerSongTitle, // Added
    localPlayerArtistName, // Added
    localPlayerAlbumName, // Added
    localPlayerSource, // Added
    localPlayerError, // Added
    setLocalPlayerSongInfo, // Added
    clearLocalPlayerSongInfo, // Added
    setLocalPlayerError, // Added
    // Idle animation states & actions
    isIdle,
    setIsIdle,
    currentIdleAnimation,
    setCurrentIdleAnimation,
  }
})
