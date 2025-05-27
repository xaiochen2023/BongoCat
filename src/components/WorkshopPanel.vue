<template>
  <div v-if="appStore.isWorkshopPanelVisible" class="workshop-panel">
    <div class="workshop-panel-header">
      <h3>Creative Workshop</h3>
      <a-button type="text" @click="handleClose" title="Close">
        <template #icon><CloseOutlined /></template>
      </a-button>
    </div>
    <div class="workshop-panel-content">
      <!-- Skin Selection Section -->
      <div class="workshop-section">
        <h4>皮肤选择 (Skin Selection)</h4>
        <a-radio-group v-model:value="selectedSkin" @change="onSkinChange">
          <a-radio-button value="Default">默认 (Default)</a-radio-button>
          <a-radio-button value="Calico">三花 (Calico)</a-radio-button>
          <a-radio-button value="Siamese">暹罗 (Siamese)</a-radio-button>
        </a-radio-group>
      </div>

      <!-- Color Customization Section -->
      <div class="workshop-section">
        <h4>颜色定制 (Color Customization)</h4>
        <div v-if="isDefaultSkinSelected">
          <div class="color-picker-row">
            <label for="furColor">毛色 (Fur Color):</label>
            <input type="color" id="furColor" v-model="selectedFurColor" @input="onFurColorChange" />
            <span>{{ selectedFurColor }}</span>
          </div>
          <div class="color-picker-row">
            <label for="eyeColor">眼睛颜色 (Eye Color):</label>
            <input type="color" id="eyeColor" v-model="selectedEyeColor" @input="onEyeColorChange" />
            <span>{{ selectedEyeColor }}</span>
          </div>
        </div>
        <div v-else>
          <p class="color-customization-note">颜色定制仅适用于“默认”皮肤。(Color customization is only available for the "Default" skin.)</p>
        </div>
      </div>

      <!-- Accessory Selection Section - Extended -->
      <div class="workshop-section">
        <h4>饰品选择 (Accessory Selection)</h4>
        
        <!-- Head Accessories -->
        <div class="accessory-category">
          <h5>头部 (Head)</h5>
          <a-radio-group v-model:value="selectedAccessories.head" @change="onAccessoryCategoryChange('head', $event.target.value)">
            <a-radio-button value="None">无 (None)</a-radio-button>
            <a-radio-button value="Top Hat">礼帽 (Top Hat)</a-radio-button>
            <a-radio-button value="Flower Crown">花环 (Flower Crown)</a-radio-button>
          </a-radio-group>
        </div>

        <!-- Eyes Accessories -->
        <div class="accessory-category">
          <h5>眼部 (Eyes)</h5>
          <a-radio-group v-model:value="selectedAccessories.eyes" @change="onAccessoryCategoryChange('eyes', $event.target.value)">
            <a-radio-button value="None">无 (None)</a-radio-button>
            <a-radio-button value="Sunglasses">太阳镜 (Sunglasses)</a-radio-button>
            <a-radio-button value="Monocle">单片眼镜 (Monocle)</a-radio-button>
          </a-radio-group>
        </div>

        <!-- Neck Accessories -->
        <div class="accessory-category">
          <h5>颈部 (Neck)</h5>
          <a-radio-group v-model:value="selectedAccessories.neck" @change="onAccessoryCategoryChange('neck', $event.target.value)">
            <a-radio-button value="None">无 (None)</a-radio-button>
            <a-radio-button value="Red Collar">红色项圈 (Red Collar)</a-radio-button>
            <a-radio-button value="Blue Scarf">蓝色围巾 (Blue Scarf)</a-radio-button>
          </a-radio-group>
        </div>
      </div>

      <!-- Sound Effects Section -->
      <div class="workshop-section">
        <h4>音效设置 (Sound Settings)</h4>
        <div class="settings-row">
          <label for="soundEffects">启用音效 (Enable Sound Effects):</label>
          <a-switch id="soundEffects" v-model:checked="soundEffectsOn" @change="onSoundEffectsToggle" />
        </div>
      </div>

      <!-- Test Cat Action State Section -->
      <div class="workshop-section">
        <h4>测试猫咪动作 (Test Cat Actions)</h4>
        <div class="action-buttons">
          <a-button @click="appStore.setCatActionState('idle')">Idle</a-button>
          <a-button @click="appStore.setCatActionState('left_paw_down')">Left Paw</a-button>
          <a-button @click="appStore.setCatActionState('right_paw_down')">Right Paw</a-button>
          <a-button @click="appStore.setCatActionState('mouse_move')">Mouse Move</a-button>
        </div>
      </div>

      <!-- Paw Style Section -->
      <div class="workshop-section">
        <h4>爪子样式 (Paw Style)</h4>
        <a-radio-group v-model:value="selectedPawStyle" @change="onPawStyleChange">
          <a-radio-button value="Default Paws">默认爪子 (Default)</a-radio-button>
          <a-radio-button value="Clawed Paws">尖爪爪子 (Claws)</a-radio-button>
          <a-radio-button value="PinkPads Paws">粉嫩肉垫 (Pink Pads)</a-radio-button>
        </a-radio-group>
      </div>
      <!-- Future workshop controls will be added here -->
    </div>
  </div>
</template>

<script lang="ts" setup>
import { CloseOutlined } from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/app'
import { Button as AButton, RadioGroup as ARadioGroup, RadioButton as ARadioButton, Switch as ASwitch } from 'ant-design-vue'
import { ref, watch, computed, reactive } from 'vue' // Added reactive

const appStore = useAppStore()
const selectedSkin = ref(appStore.currentSkinId)
const selectedFurColor = ref(appStore.furColor)
const selectedEyeColor = ref(appStore.eyeColor)
// Use reactive for selectedAccessories to properly bind to the object structure
const selectedAccessories = reactive({ ...appStore.currentAccessories })
const soundEffectsOn = ref(appStore.soundEffectsEnabled)
const selectedPawStyle = ref(appStore.currentPawStyle)

const isDefaultSkinSelected = computed(() => appStore.currentSkinId === 'Default')

// Watch for changes from the store (e.g., initial load from localStorage)
watch(() => appStore.currentSkinId, (newSkinId) => { selectedSkin.value = newSkinId })
watch(() => appStore.furColor, (newColor) => { selectedFurColor.value = newColor })
watch(() => appStore.eyeColor, (newColor) => { selectedEyeColor.value = newColor })
watch(() => appStore.currentAccessories, (newAccessories) => {
  // Ensure local reactive object is updated when store changes (e.g. on load)
  selectedAccessories.head = newAccessories.head;
  selectedAccessories.eyes = newAccessories.eyes;
  selectedAccessories.neck = newAccessories.neck;
}, { deep: true })
watch(() => appStore.soundEffectsEnabled, (newVal) => { soundEffectsOn.value = newVal })
watch(() => appStore.currentPawStyle, (newStyle) => { selectedPawStyle.value = newStyle })


const onSkinChange = () => { appStore.setCurrentSkinId(selectedSkin.value) }
const onFurColorChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  appStore.setFurColor(target.value)
}
const onEyeColorChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  appStore.setEyeColor(target.value)
}

// Updated accessory change handler
const onAccessoryCategoryChange = (category: 'head' | 'eyes' | 'neck', accessoryName: string) => {
  appStore.setCurrentAccessoryForCategory({ category, accessoryName })
}

const onSoundEffectsToggle = () => { appStore.setSoundEffectsEnabled(soundEffectsOn.value) }
const onPawStyleChange = () => { appStore.setCurrentPawStyle(selectedPawStyle.value) }
const handleClose = () => { appStore.closeWorkshopPanel() }
</script>

<style lang="scss" scoped>
.workshop-section {
  margin-bottom: 20px;

  h4 {
    margin-bottom: 10px;
    font-weight: 600;
  }
}

.color-customization-note { // Added
  font-size: 0.9em;
  color: #777;
  margin-top: 5px;
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;

  .ant-btn {
    margin-right: 0;
  }
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;

  label {
    margin-right: 10px;
  }
}

.color-picker-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;

  label {
    margin-right: 10px;
    min-width: 120px;
  }

  input[type="color"] {
    margin-right: 10px;
    min-width: 50px;
    height: 30px;
    border: 1px solid #d9d9d9;
    border-radius: 4px;
    padding: 2px;
  }

  span {
    font-family: monospace;
  }
}

.workshop-panel {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 400px;
  max-width: 90vw;
  height: auto; // Changed to auto for dynamic height
  min-height: 300px; // Ensure a minimum height
  max-height: 80vh;
  background-color: #ffffff;
  border: 1px solid #d9d9d9;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 999; // Ensure it's above other elements but below the button if it overlaps
  display: flex;
  flex-direction: column;

  .workshop-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #f0f0f0;

    h3 {
      margin: 0;
      font-size: 16px;
      font-weight: 600;
    }
  }

  .workshop-panel-content {
    padding: 16px;
    overflow-y: auto;
    flex-grow: 1;
  }
}
</style>
