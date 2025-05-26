<template>
  <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
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
    <circle
      id="cat_eye_left_default"
      cx="35"
      cy="45"
      r="8"
      :style="{ fill: eyeColor }"
    />
    <circle
      id="cat_eye_right_default"
      cx="65"
      cy="45"
      r="8"
      :style="{ fill: eyeColor }"
    />
    <text x="10" y="90" font-family="sans-serif" font-size="10" fill="grey">
      Default SVG (Colorable)
    </text>
  </svg>
</template>

<script lang="ts" setup>
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const furColor = computed(() => appStore.furColor)
const eyeColor = computed(() => appStore.eyeColor)
</script>

<style scoped>
/* Add any specific styles for this component if needed */
svg {
  width: 100%;
  height: 100%;
}
</style>
