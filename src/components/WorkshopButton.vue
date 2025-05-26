<template>
  <div class="workshop-button-container">
    <a-button
      type="primary"
      shape="circle"
      size="large"
      @click="handleClick"
      title="Creative Workshop"
    >
      <template #icon><PaletteOutlined /></template>
    </a-button>
  </div>
</template>

<script lang="ts" setup>
import { PaletteOutlined } from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/app'
import { Button as AButton } from 'ant-design-vue'

const appStore = useAppStore()

const handleClick = () => {
  appStore.toggleWorkshopPanel()
}
</script>

<style lang="scss" scoped>
.workshop-button-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 1000; // Ensure it's above other elements
}
</style>
