<template>
  <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg" class="colorable-cat-svg">
    <rect
      id="cat_fur_default"
      width="80"
      height="70"
      x="10"
      y="20"
      rx="10"
      ry="10"
      :style="{ fill: furColor }"
    />

    <!-- Default Expression (Eyes) -->
    <g id="cat_expression_default" :style="{ display: isFocused ? 'none' : 'block' }">
      <circle id="cat_eye_left_default" cx="35" cy="45" r="8" :style="{ fill: eyeColor }" />
      <circle id="cat_eye_right_default" cx="65" cy="45" r="8" :style="{ fill: eyeColor }" />
    </g>
    
    <!-- Focused Expression (Slightly smaller eyes, example) -->
    <g id="cat_expression_focused" :style="{ display: isFocused ? 'block' : 'none' }">
      <circle id="cat_eye_left_focused" cx="35" cy="45" r="6" :style="{ fill: eyeColor }" />
      <circle id="cat_eye_right_focused" cx="65" cy="45" r="6" :style="{ fill: eyeColor }" />
      <path d="M30 35 Q35 32 40 35" stroke="black" stroke-width="1.5" fill="none" />
      <path d="M60 35 Q65 32 70 35" stroke="black" stroke-width="1.5" fill="none" />
    </g>

    <!-- Paws Group - structure from idle.svg -->
    <g id="cat_paws_default">
      <!-- Base Paw Shapes -->
      <ellipse id="cat_left_paw_base_default" cx="30" cy="75" rx="10" ry="5" :fill="pawBaseColor" />
      <ellipse id="cat_right_paw_base_default" cx="70" cy="75" rx="10" ry="5" :fill="pawBaseColor" />

      <!-- Pink Pads -->
      <g id="cat_left_pink_pads" :style="{ display: showPinkPads ? 'block' : 'none' }">
        <circle id="cat_left_paw_pad_main_default" cx="30" cy="75" r="3" fill="#FFC0CB" />
        <circle id="cat_left_paw_pad_1_default" cx="26" cy="72" r="1.5" fill="#FFC0CB" />
        <circle id="cat_left_paw_pad_2_default" cx="30" cy="71" r="1.5" fill="#FFC0CB" />
        <circle id="cat_left_paw_pad_3_default" cx="34" cy="72" r="1.5" fill="#FFC0CB" />
      </g>
      <g id="cat_right_pink_pads" :style="{ display: showPinkPads ? 'block' : 'none' }">
        <circle id="cat_right_paw_pad_main_default" cx="70" cy="75" r="3" fill="#FFC0CB" />
        <circle id="cat_right_paw_pad_1_default" cx="66" cy="72" r="1.5" fill="#FFC0CB" />
        <circle id="cat_right_paw_pad_2_default" cx="70" cy="71" r="1.5" fill="#FFC0CB" />
        <circle id="cat_right_paw_pad_3_default" cx="74" cy="72" r="1.5" fill="#FFC0CB" />
      </g>

      <!-- Claws -->
      <g id="cat_left_claws_default" :style="{ display: showClaws ? 'block' : 'none' }">
        <path d="M23 70 L25 73 L27 70 Z" fill="black" />
        <path d="M28 69 L30 72 L32 69 Z" fill="black" />
        <path d="M33 70 L35 73 L37 70 Z" fill="black" />
      </g>
      <g id="cat_right_claws_default" :style="{ display: showClaws ? 'block' : 'none' }">
        <path d="M63 70 L65 73 L67 70 Z" fill="black" />
        <path d="M68 69 L70 72 L72 69 Z" fill="black" />
        <path d="M73 70 L75 73 L77 70 Z" fill="black" />
      </g>
    </g>

    <text x="10" y="90" font-family="sans-serif" font-size="10" fill="grey">
      Default SVG (Emotions)
    </text>
  </svg>
</template>

<script lang="ts" setup>
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const furColor = computed(() => appStore.furColor)
const eyeColor = computed(() => appStore.eyeColor) // Used for both default and focused eyes
const currentPawStyle = computed(() => appStore.currentPawStyle)
const isFocused = computed(() => appStore.isUserTypingFocused) // Added

const pawBaseColor = computed(() => {
  return currentPawStyle.value === 'PinkPads Paws' ? appStore.furColor : '#D3D3D3'; 
});

const showPinkPads = computed(() => {
  return currentPawStyle.value === 'PinkPads Paws';
});

const showClaws = computed(() => {
  return currentPawStyle.value === 'Clawed Paws';
});

</script>

<style scoped>
/* Add any specific styles for this component if needed */
.colorable-cat-svg { 
  width: 100%;
  height: 100%;
}
</style>
