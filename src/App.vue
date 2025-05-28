<script setup lang="ts">
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { error as logError } from '@tauri-apps/plugin-log' // Renamed to avoid conflict
import { openUrl } from '@tauri-apps/plugin-opener'
import { useEventListener } from '@vueuse/core'
import { ConfigProvider } from 'ant-design-vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'
import { isString } from 'es-toolkit'
import isURL from 'is-url'
import { onMounted, onUnmounted, ref, watch } from 'vue' 
import { RouterView } from 'vue-router'

import WorkshopButton from '@/components/WorkshopButton.vue'
import WorkshopPanel from '@/components/WorkshopPanel.vue'
import { useTauriListen } from './composables/useTauriListen' // This seems to be a custom composable, will use direct listen for now
import { useThemeVars } from './composables/useThemeVars'
import { useWindowState } from './composables/useWindowState'
import { LISTEN_KEY } from './constants'
import { hideWindow, showWindow } from './plugins/window'
import { useAppStore } from './stores/app'
import { useCatStore } from './stores/cat'
import { useGeneralStore } from './stores/general'
import { useModelStore } from './stores/model'
import { playSound, preloadSounds } from './utils/audio'
import { listen, type UnlistenFn } from '@tauri-apps/api/event' // Ensure UnlistenFn is typed if needed

const { generateColorVars } = useThemeVars()
const appStore = useAppStore()
const modelStore = useModelStore()
const catStore = useCatStore()
const generalStore = useGeneralStore()
const appWindow = getCurrentWebviewWindow()
const { isRestored, restoreState } = useWindowState()

// --- Idle Animation Logic ---
const INACTIVITY_TIMEOUT_MS = 5000 
const IDLE_ANIMATION_INTERVAL_MS = 3500 
const IDLE_ANIMATIONS = ["Idle: Blinking", "Idle: Tail Wagging", "Idle: Ear Twitching"]
let inactivityTimerId: number | undefined = undefined
let idleAnimationTimerId: number | undefined = undefined

// --- Typing Focus Logic ---
const KEYPRESS_WINDOW_MS = 2000; 
const KEYPRESS_THRESHOLD = 5; 
const FOCUS_TIMEOUT_MS = 1500; 
let keyPressTimestamps: number[] = [];
let focusedStateTimeoutId: number | undefined = undefined;

// --- Sleepy Emotion Logic ---
const LONG_INACTIVITY_DURATION_MS = 60 * 1000; 
let longInactivityTimeoutId: number | undefined = undefined;

const resetAllActivityTimers = () => {
  clearTimeout(inactivityTimerId);
  if (appStore.isIdle) {
    appStore.setIsIdle(false); 
  }
  inactivityTimerId = setTimeout(() => {
    if (appStore.currentEmotion === 'neutral') {
       appStore.setIsIdle(true);
    }
  }, INACTIVITY_TIMEOUT_MS);

  if (appStore.currentEmotion === 'sleepy') {
    appStore.setCurrentEmotion('neutral'); 
  }
  clearTimeout(longInactivityTimeoutId);
  longInactivityTimeoutId = setTimeout(() => {
    appStore.setCurrentEmotion('sleepy');
  }, LONG_INACTIVITY_DURATION_MS);
};

watch(() => appStore.isIdle, (isNowIdle) => {
  clearInterval(idleAnimationTimerId); 
  idleAnimationTimerId = undefined;
  if (isNowIdle && appStore.currentEmotion === 'neutral') { 
    const playRandomIdleAnimation = () => {
      const randomIndex = Math.floor(Math.random() * IDLE_ANIMATIONS.length);
      appStore.setCurrentIdleAnimation(IDLE_ANIMATIONS[randomIndex]);
    };
    playRandomIdleAnimation();
    idleAnimationTimerId = setInterval(playRandomIdleAnimation, IDLE_ANIMATION_INTERVAL_MS);
  } else {
    appStore.setCurrentIdleAnimation(null); 
  }
});

watch(() => appStore.currentEmotion, (newEmotion, oldEmotion) => {
  if (newEmotion !== 'neutral' && newEmotion !== 'focused') { 
    if (appStore.isIdle) {
      appStore.setIsIdle(false); 
    }
    if (oldEmotion === 'focused') {
        clearTimeout(focusedStateTimeoutId);
        keyPressTimestamps = [];
    }
  }
  if (newEmotion === 'focused' && appStore.isIdle) {
      appStore.setIsIdle(false);
  }
});

const handleUserActivity = () => {
  resetAllActivityTimers();
};

const handleTypingFocus = (event: KeyboardEvent) => {
  if (!event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
    if (appStore.soundEffectsEnabled) playSound('keypress');
  }
  resetAllActivityTimers(); 
  const now = Date.now();
  keyPressTimestamps.push(now);
  keyPressTimestamps = keyPressTimestamps.filter(timestamp => now - timestamp < KEYPRESS_WINDOW_MS);

  if (keyPressTimestamps.length >= KEYPRESS_THRESHOLD) {
    if (appStore.currentEmotion !== 'focused') {
      appStore.setCurrentEmotion('focused');
    }
    clearTimeout(focusedStateTimeoutId);
    focusedStateTimeoutId = setTimeout(() => {
      if (appStore.currentEmotion === 'focused') {
        appStore.setCurrentEmotion('neutral');
      }
      keyPressTimestamps = [];
    }, FOCUS_TIMEOUT_MS);
  }
};

useEventListener(document, 'mousemove', handleUserActivity);
useEventListener(document, 'mousedown', handleUserActivity);
useEventListener(document, 'keydown', handleTypingFocus); 
useEventListener(document, 'scroll', handleUserActivity);

// --- App Awareness Event Listener (Existing) ---
let unlistenAppDetection: UnlistenFn | undefined;

// --- Local Song Change Event Listener (NEW) ---
interface LocalSongInfoPayload {
  title: string;
  artist: string;
  album: string | null; // Matches Rust Option<String>
  player_source: string;
}
// For Result<Option<LocalSongInfo>, String>
// Ok(Some(data)) -> { Ok: data }
// Ok(None)       -> { Ok: null }
// Err(msg)       -> { Err: msg }
type LocalSongChangeEventPayload = 
  { Ok: LocalSongInfoPayload | null; Err?: undefined } | 
  { Err: string; Ok?: undefined };

let unlistenLocalSongChange: UnlistenFn | undefined;


onMounted(async () => {
  // Existing App Detection Listener
  try {
    unlistenAppDetection = await listen<string | null>('app_detection_change', (event) => {
      appStore.setDetectedAppReaction(event.payload);
    });
  } catch (e) {
    console.error("Failed to listen for app_detection_change event:", e);
  }

  // NEW Local Song Change Listener
  try {
    unlistenLocalSongChange = await listen<LocalSongChangeEventPayload>('local_song_change', (event) => {
      console.log("local_song_change event received:", event.payload); // For debugging
      const payload = event.payload;
      if (payload.Ok !== undefined) { // Check if Ok field exists (covers Ok(Some) and Ok(None))
        if (payload.Ok) { // Ok(Some(data))
          appStore.setLocalPlayerSongInfo(payload.Ok);
        } else { // Ok(None)
          appStore.clearLocalPlayerSongInfo();
          appStore.setLocalPlayerError(''); // Clear any previous error
        }
      } else if (payload.Err) { // Err(msg)
        appStore.setLocalPlayerError(payload.Err);
      }
    });
  } catch (e) {
    console.error("Failed to listen for local_song_change event:", e);
  }

  // Other onMounted logic
  generateColorVars()
  resetAllActivityTimers(); 
  preloadSounds() 
  await appStore.$tauri.start()
  await modelStore.$tauri.start()
  await catStore.$tauri.start()
  await generalStore.$tauri.start()
  restoreState()
})

onUnmounted(() => { 
  if (unlistenAppDetection) unlistenAppDetection();
  if (unlistenLocalSongChange) unlistenLocalSongChange(); // NEW: Cleanup local song listener
  clearTimeout(inactivityTimerId);
  clearInterval(idleAnimationTimerId);
  clearTimeout(focusedStateTimeoutId);
  clearTimeout(longInactivityTimeoutId); 
});

// Original TauriListen composable usage (can be removed if direct listen is preferred for all)
// useTauriListen(LISTEN_KEY.SHOW_WINDOW, ({ payload }) => { ... });
// useTauriListen(LISTEN_KEY.HIDE_WINDOW, ({ payload }) => { ... });

useEventListener('unhandledrejection', ({ reason }) => {
  const message = isString(reason) ? reason : JSON.stringify(reason)
  logError(message) // Use renamed logError
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
</script>

<template>
  <ConfigProvider :locale="zhCN">
    <RouterView v-if="isRestored" />
    <WorkshopButton />
    <WorkshopPanel />
  </ConfigProvider>
</template>
