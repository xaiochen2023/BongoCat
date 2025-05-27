<script setup lang="ts">
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { error } from '@tauri-apps/plugin-log'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useEventListener } from '@vueuse/core'
import { ConfigProvider } from 'ant-design-vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'
import { isString } from 'es-toolkit'
import isURL from 'is-url'
import { onMounted, ref, watch } from 'vue' // Added ref, watch
import { RouterView } from 'vue-router'

import WorkshopButton from '@/components/WorkshopButton.vue'
import WorkshopPanel from '@/components/WorkshopPanel.vue'
import { useTauriListen } from './composables/useTauriListen'
import { useThemeVars } from './composables/useThemeVars'
import { useWindowState } from './composables/useWindowState'
import { LISTEN_KEY } from './constants'
import { hideWindow, showWindow } from './plugins/window'
import { useAppStore } from './stores/app'
import { useCatStore } from './stores/cat'
import { useGeneralStore } from './stores/general'
import { useModelStore } from './stores/model'
import { playSound, preloadSounds } from './utils/audio'
import { listen } from '@tauri-apps/api/event' // Added for app awareness
import type { UnlistenFn } from '@tauri-apps/api/event' // Added for app awareness

const { generateColorVars } = useThemeVars()
const appStore = useAppStore()
const modelStore = useModelStore()
const catStore = useCatStore()
const generalStore = useGeneralStore()
const appWindow = getCurrentWebviewWindow()
const { isRestored, restoreState } = useWindowState()

// --- Idle Animation Logic ---
const INACTIVITY_TIMEOUT = 5000 // 5 seconds
const IDLE_ANIMATION_INTERVAL = 3500 // 3.5 seconds
const IDLE_ANIMATIONS = ["Idle: Blinking", "Idle: Tail Wagging", "Idle: Ear Twitching"]

let inactivityTimer: number | undefined = undefined
let idleAnimationTimer: number | undefined = undefined

const resetInactivityTimer = () => {
  clearTimeout(inactivityTimer)
  if (appStore.isIdle) {
    appStore.setIsIdle(false)
  }
  inactivityTimer = setTimeout(() => {
    appStore.setIsIdle(true)
  }, INACTIVITY_TIMEOUT)
}

watch(() => appStore.isIdle, (isNowIdle) => {
  if (isNowIdle) {
    // Start cycling idle animations
    const playRandomIdleAnimation = () => {
      const randomIndex = Math.floor(Math.random() * IDLE_ANIMATIONS.length)
      appStore.setCurrentIdleAnimation(IDLE_ANIMATIONS[randomIndex])
    }
    playRandomIdleAnimation() // Play one immediately
    idleAnimationTimer = setInterval(playRandomIdleAnimation, IDLE_ANIMATION_INTERVAL)
  } else {
    // Stop cycling and clear current animation
    clearInterval(idleAnimationTimer)
    appStore.setCurrentIdleAnimation(null)
  }
})

// Global event listeners to reset inactivity timer and play sounds
useEventListener(document, 'mousemove', resetInactivityTimer)
useEventListener(document, 'mousedown', () => {
  resetInactivityTimer()
  if (appStore.soundEffectsEnabled) {
    playSound('mouseclick')
  }
})

// --- Typing Speed Detection for Focused Emotion ---
const KEYPRESS_WINDOW_MS = 2000; // 2 seconds
const KEYPRESS_THRESHOLD = 5; // 5 key presses in the window
const FOCUS_TIMEOUT_MS = 1500; // 1.5 seconds of no typing to lose focus

let keyPressTimestamps: number[] = [];
let focusedStateTimeoutId: number | undefined = undefined;

const handleTypingFocus = (event: KeyboardEvent) => {
  // Play sound (if enabled and not a modifier key)
  if (!event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
    if (appStore.soundEffectsEnabled) {
      playSound('keypress')
    }
  }

  // Inactivity timer reset should still happen for any key press
  resetInactivityTimer(); 

  // Focused state logic using currentEmotion
  const now = Date.now();
  keyPressTimestamps.push(now);
  // Filter out timestamps older than the window
  keyPressTimestamps = keyPressTimestamps.filter(timestamp => now - timestamp < KEYPRESS_WINDOW_MS);

  if (keyPressTimestamps.length >= KEYPRESS_THRESHOLD) {
    if (appStore.currentEmotion !== 'focused') { // Check current emotion
      appStore.setCurrentEmotion('focused');    // Use new action
    }
    // Reset timeout to remove focused state
    clearTimeout(focusedStateTimeoutId);
    focusedStateTimeoutId = setTimeout(() => {
      if (appStore.currentEmotion === 'focused') { // Only revert if still focused
        appStore.setCurrentEmotion('neutral');  // Use new action
      }
      keyPressTimestamps = []; // Reset timestamps after focus is lost
    }, FOCUS_TIMEOUT_MS);
  }
};

useEventListener(document, 'keydown', handleTypingFocus)
// --- End Typing Speed Detection ---

useEventListener(document, 'scroll', resetInactivityTimer)
// --- End Idle Animation Logic ---

// --- App Awareness Event Listener ---
let unlistenAppDetection: UnlistenFn | undefined;

onMounted(async () => {
  try {
    unlistenAppDetection = await listen<string | null>('app_detection_change', (event) => {
      console.log('App detection event received:', event.payload); // For debugging
      appStore.setDetectedAppReaction(event.payload);
    });
  } catch (e) {
    console.error("Failed to listen for app_detection_change event:", e);
  }
})

onUnmounted(() => { // Added onUnmounted for cleanup
  if (unlistenAppDetection) {
    unlistenAppDetection();
  }
});
// --- End App Awareness Event Listener ---


onMounted(async () => {
  generateColorVars()
  resetInactivityTimer() // Initialize the timer on mount
  preloadSounds() // Preload sounds on app mount

  await appStore.$tauri.start()
  await modelStore.$tauri.start()
  await catStore.$tauri.start()
  await generalStore.$tauri.start()

  restoreState()
})

useTauriListen(LISTEN_KEY.SHOW_WINDOW, ({ payload }) => {
  if (appWindow.label !== payload) return

  showWindow()
})

useTauriListen(LISTEN_KEY.HIDE_WINDOW, ({ payload }) => {
  if (appWindow.label !== payload) return

  hideWindow()
})

useEventListener('unhandledrejection', ({ reason }) => {
  const message = isString(reason) ? reason : JSON.stringify(reason)

  error(message)
})

useEventListener('click', (event) => {
  const link = (event.target as HTMLElement).closest('a')

  if (!link) return

  const { href, target } = link

  if (target === '_blank') return

  event.preventDefault()

  if (!isURL(href)) return

  openUrl(href)
})

onMounted(() => { // Ensure this onMounted for inactivity timer reset is distinct or merged
  resetInactivityTimer()
})
</script>

<template>
  <ConfigProvider :locale="zhCN">
    <RouterView v-if="isRestored" />
    <WorkshopButton />
    <WorkshopPanel />
  </ConfigProvider>
</template>
