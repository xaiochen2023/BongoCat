<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { Menu } from '@tauri-apps/api/menu'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useDebounceFn, useEventListener } from '@vueuse/core'
import { computed, onUnmounted, ref, watch, reactive } from 'vue'

import { useDevice } from '@/composables/useDevice'
import { useModel } from '@/composables/useModel'
import { useSharedMenu } from '@/composables/useSharedMenu'
import ColorableCatSvg from '@/components/ColorableCatSvg.vue'
import { useAppStore } from '@/stores/app'
import { useCatStore } from '@/stores/cat'
import { useModelStore } from '@/stores/model'
import { join } from '@/utils/path'

const appWindow = getCurrentWebviewWindow()
const appStore = useAppStore() // Already present, ensure it's used for local player state
const { pressedMouses, mousePosition, pressedLeftKeys, pressedRightKeys } = useDevice()
const { handleDestroy, handleResize, handleMouseDown, handleMouseMove, handleKeyDown } = useModel()
const catStore = useCatStore()
const { getSharedMenu } = useSharedMenu()
const modelStore = useModelStore()

const resizing = ref(false)

// --- Idle Sprite Animation Logic ---
interface AnimationConfig {
  frames: number;
  frameWidth: number;
  frameHeight: number;
  fps: number;
  spriteSheetUrl: string;
}

const defaultIdleAnimations: Record<string, AnimationConfig> = {
  Blinking: { frames: 3, frameWidth: 100, frameHeight: 100, fps: 7, spriteSheetUrl: '/assets/skins/Default/animations/Blinking_spritesheet.png' },
  TailWagging: { frames: 4, frameWidth: 100, frameHeight: 100, fps: 5, spriteSheetUrl: '/assets/skins/Default/animations/TailWagging_spritesheet.png' },
  EarTwitching: { frames: 3, frameWidth: 100, frameHeight: 100, fps: 8, spriteSheetUrl: '/assets/skins/Default/animations/EarTwitching_spritesheet.png' },
};

const animationPlayerStyle = ref<Record<string, string>>({});
const currentFrame = ref(0);
let animationIntervalId: number | undefined = undefined;

const showIdleSpriteAnimation = computed(() => {
  return appStore.isIdle && 
         appStore.currentSkinId === 'Default' && 
         appStore.currentIdleAnimation && 
         defaultIdleAnimations[appStore.currentIdleAnimation.replace('Idle: ', '')];
});

watch([() => appStore.isIdle, () => appStore.currentIdleAnimation, () => appStore.currentSkinId], 
  ([isIdle, currentAnim, skinId]) => {
  clearInterval(animationIntervalId);
  animationIntervalId = undefined;
  currentFrame.value = 0;

  if (isIdle && skinId === 'Default' && currentAnim) {
    const animKey = currentAnim.replace('Idle: ', '');
    const config = defaultIdleAnimations[animKey];

    if (config) {
      animationPlayerStyle.value = {
        width: `${config.frameWidth}px`,
        height: `${config.frameHeight}px`,
        backgroundImage: `url(${config.spriteSheetUrl})`,
        backgroundRepeat: 'no-repeat',
        backgroundPosition: '0px 0px',
      };

      animationIntervalId = setInterval(() => {
        currentFrame.value = (currentFrame.value + 1) % config.frames;
        animationPlayerStyle.value.backgroundPosition = `-${currentFrame.value * config.frameWidth}px 0px`;
      }, 1000 / config.fps);
    }
  }
}, { immediate: true });
// --- End Idle Sprite Animation Logic ---


onUnmounted(() => {
  handleDestroy()
  clearInterval(animationIntervalId); 
});

const handleDebounceResize = useDebounceFn(async () => {
  await handleResize()
  resizing.value = false
}, 100)

useEventListener('resize', () => {
  resizing.value = true
  handleDebounceResize()
})

watch(pressedMouses, handleMouseDown)
watch(mousePosition, handleMouseMove)
watch(pressedLeftKeys, (keys) => { handleKeyDown('left', keys.length > 0) })
watch(pressedRightKeys, (keys) => { handleKeyDown('right', keys.length > 0) })

watch(() => catStore.penetrable, (value) => {
  appWindow.setIgnoreCursorEvents(value)
}, { immediate: true })

const backgroundImage = computed(() => {
  if (!modelStore.currentModel) return
  return convertFileSrc(join(modelStore.currentModel.path, 'resources', 'background.png'))
})

// --- Updated Cat Image & SVG Visibility Logic ---
const showColorableSvg = computed(() => {
  return !showIdleSpriteAnimation.value &&
         appStore.currentSkinId === 'Default' &&
         ['idle', 'left_paw_down', 'right_paw_down', 'mouse_move'].includes(appStore.catActionState);
});

const catImageSrc = computed(() => {
  const skin = appStore.currentSkinId;
  const action = appStore.catActionState;
  const emotion = appStore.currentEmotion;

  if (showIdleSpriteAnimation.value) {
    return 'sprite_anim_active'; 
  }

  if (showColorableSvg.value) { // If it's Default SVG state, ColorableCatSvg handles it.
      return 'use_svg_component';
  }
  
  // For Image-Based Skins (e.g., "Calico")
  if (skin !== 'Default' && (emotion === 'happy' || emotion === 'sleepy')) {
    // Attempt to find emotion-specific image for the current action state
    // Fallback to neutral action state if specific emotion_action asset is missing is implicitly handled by browser 404
    return `/assets/skins/${skin}/${emotion}_${action}.png`;
  }

  // Fallback for neutral/focused emotions on image-based skins, or any other non-SVG Default states
  return `/assets/skins/${skin}/${action}.png`;
});
// --- End Updated Cat Image & SVG Visibility Logic ---


// --- Extended Accessory Logic ---
type AccessoryCategory = 'head' | 'eyes' | 'neck';

const getAccessoryImageSrc = (category: AccessoryCategory, accessoryName: string) => {
  if (accessoryName === 'None') return '';
  const fileName = accessoryName.replace(/\s+/g, '') + '.png';
  return `/assets/accessories/${category}/${fileName}`;
};

const getAccessoryStyle = (category: AccessoryCategory, accessoryName: string) => {
  const styles: Record<string, any> = {
    position: 'absolute',
    left: '50%',
    transform: 'translateX(-50%)',
    width: '50px', 
    height: 'auto',
    objectFit: 'contain',
    pointerEvents: 'none',
  };

  switch (category) {
    case 'head':
      styles.zIndex = 5;
      switch (accessoryName) {
        case 'Top Hat': styles.top = '-15px'; styles.width = '50px'; styles.height = '40px'; break;
        case 'Flower Crown': styles.top = '-20px'; styles.width = '60px'; styles.height = '30px'; break;
      }
      break;
    case 'eyes':
      styles.zIndex = 4;
      switch (accessoryName) {
        case 'Sunglasses': styles.top = '20px'; styles.width = '70px'; styles.height = '25px'; break;
        case 'Monocle': styles.top = '15px'; styles.left = '35%'; styles.width = '30px'; styles.height = '30px'; break; 
      }
      break;
    case 'neck':
      styles.zIndex = 3;
      switch (accessoryName) {
        case 'Red Collar': styles.top = '65px'; styles.width = '60px'; styles.height = '20px'; break;
        case 'Blue Scarf': styles.top = '60px'; styles.width = '70px'; styles.height = '35px'; break;
      }
      break;
  }
  return styles;
};
// --- End Extended Accessory Logic ---


// --- Paw Overlay Logic ---
const showPawOverlay = computed(() => {
  return (appStore.currentSkinId !== 'Default' || (appStore.currentSkinId === 'Default' && appStore.catActionState !== 'idle')) &&
         appStore.currentPawStyle !== 'Default Paws' &&
         (appStore.catActionState === 'left_paw_down' || appStore.catActionState === 'right_paw_down');
});

const pawOverlayImageSrc = computed(() => {
  if (!showPawOverlay.value) return '';
  const pawStyleDir = appStore.currentPawStyle.replace(/\s+/g, ''); 
  const side = appStore.catActionState === 'left_paw_down' ? 'left_paw_overlay' : 'right_paw_overlay';
  return `/assets/paws/${pawStyleDir}/${side}.png`;
});

const pawOverlayStyle = computed(() => {
  const styles: Record<string, any> = {
    position: 'absolute', zIndex: 2, width: '30px', height: '20px',
    objectFit: 'contain', pointerEvents: 'none',
  };
  if (appStore.catActionState === 'left_paw_down') {
    styles.left = '25px'; styles.top = '70px';  
  } else if (appStore.catActionState === 'right_paw_down') {
    styles.left = '65px'; styles.top = '70px';  
  }
  return styles;
});
// --- End Paw Overlay Logic ---

// --- App Awareness Reaction Icon Logic ---
const reactionIconSrc = computed(() => {
  if (!appStore.detectedAppReaction) return '';
  return `/assets/reactions/${appStore.detectedAppReaction}_reaction_icon.png`;
});

const reactionIconStyle = computed(() => {
  return {
    position: 'absolute',
    top: '5px', 
    right: '5px', 
    width: '24px',
    height: '24px',
    zIndex: 10, 
    pointerEvents: 'none',
    objectFit: 'contain', 
  };
});
// --- End App Awareness Reaction Icon Logic ---


function handleWindowDrag() { appWindow.startDragging() }
async function handleContextmenu(event: MouseEvent) {
  event.preventDefault()
  const menu = await Menu.new({ items: await getSharedMenu() })
  menu.popup()
}
function resolveImagePath(key: string, side: 'left' | 'right' = 'left') {
  if (!modelStore.currentModel) return
  return convertFileSrc(join(modelStore.currentModel.path, 'resources', `${side}-keys`, `${key}.png`))
}
</script>

<template>
  <div
    class="relative size-screen overflow-hidden children:(absolute size-full)"
    :class="[catStore.mirrorMode ? '-scale-x-100' : 'scale-x-100']"
    :style="{ opacity: catStore.opacity / 100 }"
    @contextmenu="handleContextmenu"
    @mousedown="handleWindowDrag"
  >
    <img :src="backgroundImage" v-if="backgroundImage">

    <div class="cat-display-area">
      <div v-if="showIdleSpriteAnimation" class="idle-animation-player" :style="animationPlayerStyle"></div>
      <ColorableCatSvg 
        v-else-if="showColorableSvg" 
        class="cat-image" 
        :key="appStore.catActionState" 
      />
      <img 
        v-else-if="catImageSrc !== 'sprite_anim_active' && catImageSrc !== 'use_svg_component'" 
        :src="catImageSrc" 
        alt="Bongo Cat" 
        class="cat-image" 
      />

      <img
        v-if="showPawOverlay"
        :src="pawOverlayImageSrc"
        alt="Paw Overlay"
        class="paw-overlay-image" 
        :style="pawOverlayStyle"
      />

      <img 
        v-if="appStore.currentAccessories.neck !== 'None'" 
        :src="getAccessoryImageSrc('neck', appStore.currentAccessories.neck)" 
        :alt="appStore.currentAccessories.neck" 
        class="accessory-image neck-accessory"
        :style="getAccessoryStyle('neck', appStore.currentAccessories.neck)"
      />
      <img 
        v-if="appStore.currentAccessories.eyes !== 'None'" 
        :src="getAccessoryImageSrc('eyes', appStore.currentAccessories.eyes)" 
        :alt="appStore.currentAccessories.eyes" 
        class="accessory-image eyes-accessory"
        :style="getAccessoryStyle('eyes', appStore.currentAccessories.eyes)"
      />
      <img 
        v-if="appStore.currentAccessories.head !== 'None'" 
        :src="getAccessoryImageSrc('head', appStore.currentAccessories.head)" 
        :alt="appStore.currentAccessories.head" 
        class="accessory-image head-accessory"
        :style="getAccessoryStyle('head', appStore.currentAccessories.head)"
      />
      
      <img
        v-if="appStore.detectedAppReaction"
        :src="reactionIconSrc"
        alt="Reaction Icon"
        class="reaction-icon"
        :style="reactionIconStyle"
      />
    </div>

    <!-- Local Music Display (NEW) -->
    <div class="local-music-display" v-if="appStore.localPlayerSongTitle">
      <div class="song-info">
        <p class="song-title">{{ appStore.localPlayerSongTitle }}</p>
        <p class="song-artist">{{ appStore.localPlayerArtistName }}</p>
        <p class="song-album" v-if="appStore.localPlayerAlbumName">{{ appStore.localPlayerAlbumName }}</p>
        <p class="song-source">via {{ appStore.localPlayerSource }}</p>
      </div>
    </div>
    <div class="local-music-error" v-else-if="appStore.localPlayerError">
      <p>Local Music Error: {{ appStore.localPlayerError }}</p>
    </div>
    <!-- End Local Music Display -->

    <div v-show="resizing" class="flex items-center justify-center bg-black">
      <span class="text-center text-5xl text-white">重绘中...</span>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.cat-display-area {
  position: relative; 
  left: 50%; top: 50%;
  transform: translate(-50%, -50%);
  width: 100px; height: 100px;

  .cat-image {
    display: block; width: 100%; height: 100%;
    object-fit: contain; position: relative; z-index: 1; 
  }

  .paw-overlay-image { 
    transform-origin: center;
  }
  
  .accessory-image { 
    transform-origin: center; 
  }

  .idle-animation-player { 
    position: absolute; 
    left: 0; top: 0; 
    z-index: 1; 
    pointer-events: none;
  }

  .reaction-icon { 
    object-fit: contain;
  }
}

/* NEW Styles for Local Music Display */
.local-music-display {
  position: fixed;
  bottom: 10px;
  left: 10px;
  background-color: rgba(0,0,0,0.6);
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.8em;
  display: flex;
  align-items: center;
  z-index: 1000; // Ensure it's above other UI elements like workshop button
  max-width: 250px; // Prevent it from becoming too wide
  pointer-events: none; // Non-interactive
}

.song-info p {
  margin: 0;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-title {
  font-weight: bold;
}

.song-artist, .song-album, .song-source {
  font-size: 0.9em;
  opacity: 0.85;
}

.local-music-error {
  position: fixed;
  bottom: 10px;
  left: 10px;
  background-color: rgba(150,0,0,0.7); /* Reddish background for error */
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.8em;
  z-index: 1000;
  max-width: 250px;
  pointer-events: none;
}
</style>
