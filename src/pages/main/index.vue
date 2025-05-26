<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { Menu } from '@tauri-apps/api/menu'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useDebounceFn, useEventListener } from '@vueuse/core'
import { computed, onUnmounted, ref, watch, reactive } from 'vue' // Added reactive

import { useDevice } from '@/composables/useDevice'
import { useModel } from '@/composables/useModel'
import { useSharedMenu } from '@/composables/useSharedMenu'
import ColorableCatSvg from '@/components/ColorableCatSvg.vue'
import { useAppStore } from '@/stores/app'
import { useCatStore } from '@/stores/cat'
import { useModelStore } from '@/stores/model'
import { join } from '@/utils/path'

const appWindow = getCurrentWebviewWindow()
const appStore = useAppStore()
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
    const animKey = currentAnim.replace('Idle: ', ''); // e.g., "Blinking"
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
  clearInterval(animationIntervalId); // Clear interval on component unmount
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

const catImageSrc = computed(() => {
  const skin = appStore.currentSkinId
  let action = appStore.catActionState
  
  // If showing sprite animation, catImageSrc is not directly used for the Default/idle state
  if (showIdleSpriteAnimation.value) {
     // When idle animation is playing for Default skin, we don't want the img tag for cat skin to show.
     // The ColorableCatSvg also has its own v-if. This ensures that if sprite anim is active,
     // neither ColorableCatSvg nor the base img tag for Default/idle shows.
    return 'sprite_anim_active'; 
  }

  if (appStore.isIdle && skinId !== 'Default') { // For non-default skins, show their static idle.png when app is idle
    action = 'idle'
  } else if (appStore.isIdle && skinId === 'Default' && !showIdleSpriteAnimation.value) { 
    // If app isIdle, skin is Default, but not showing sprite animation (e.g. no currentIdleAnimation string)
    // then it should fall back to the colorable SVG if action is 'idle', or specific action PNG.
    action = appStore.catActionState // Keep current action or let showColorableSvg handle 'idle'
  }


  if (skin === 'Default' && action === 'idle') {
    return 'use_svg_component'; // Handled by showColorableSvg for Default idle (non-sprite)
  }
  return `/assets/skins/${skin}/${action}.png`;
})

const showColorableSvg = computed(() => {
  // Only show if NOT showing sprite animation, and it's Default skin in idle action state
  return !showIdleSpriteAnimation.value &&
         appStore.currentSkinId === 'Default' && 
         (appStore.catActionState === 'idle' || (appStore.isIdle && appStore.catActionState === 'idle'));
});

const accessoryImageSrc = computed(() => {
  if (appStore.currentAccessory === 'None') return '';
  const fileName = appStore.currentAccessory.replace(/\s+/g, '') + '.png';
  return `/assets/accessories/${fileName}`;
});

const accessoryStyle = computed(() => {
  const styles: Record<string, any> = {
    position: 'absolute', zIndex: 2, left: '50%', transform: 'translateX(-50%)',
    width: '50px', height: 'auto', objectFit: 'contain', pointerEvents: 'none',
  };
  switch (appStore.currentAccessory) {
    case 'Top Hat': styles.top = '-15px'; styles.width = '50px'; styles.height = '40px'; break;
    case 'Sunglasses': styles.top = '20px'; styles.width = '70px'; styles.height = '25px'; break;
    case 'Red Collar': styles.top = '65px'; styles.width = '60px'; styles.height = '20px'; break;
  }
  return styles;
});

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
      <!-- NEW: Idle Sprite Animation Player -->
      <div v-if="showIdleSpriteAnimation" class="idle-animation-player" :style="animationPlayerStyle"></div>
      
      <!-- Existing Cat Skin Display (Colorable SVG or Image) -->
      <ColorableCatSvg v-else-if="showColorableSvg" class="cat-image" />
      <img v-else-if="catImageSrc !== 'sprite_anim_active'" :src="catImageSrc" alt="Bongo Cat" class="cat-image" />

      <!-- Accessory Image -->
      <img 
        v-if="appStore.currentAccessory !== 'None'" 
        :src="accessoryImageSrc" 
        :alt="appStore.currentAccessory" 
        class="accessory-image"
        :style="accessoryStyle"
      />
    </div>

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

  .accessory-image {
    transform-origin: center; 
  }

  .idle-animation-player { // NEW
    position: absolute; // Position it like the cat-image
    left: 0; // Relative to cat-display-area
    top: 0; // Relative to cat-display-area
    // width, height, backgroundImage, backgroundPosition are set by 'animationPlayerStyle'
    z-index: 1; // Same level as cat-image, v-if/v-else handles which is shown
    pointer-events: none;
  }
}
</style>
