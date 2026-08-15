<template>
  <hover-container
    class="w-40px"
    tooltip-content="设备轮询 Debug（Ctrl+Shift+D）"
    :inverted="theme.header.inverted"
    @click="debug.toggle()"
  >
    <n-badge :value="debug.unreadErrors" :max="99" :show-zero="false">
      <svg-icon icon="mdi:lan-connect" class="text-18px" :class="{ 'text-error': debug.unreadErrors > 0 }" />
    </n-badge>
  </hover-container>
</template>

<script setup lang="ts">
import { useDevicePollDebugStore, useThemeStore } from '@/store';

defineOptions({ name: 'DevicePollDebugButton' });

const theme = useThemeStore();
const debug = useDevicePollDebugStore();
</script>
