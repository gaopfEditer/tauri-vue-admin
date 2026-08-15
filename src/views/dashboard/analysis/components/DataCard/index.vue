<template>
  <n-grid cols="s:1 m:2 l:4" responsive="screen" :x-gap="16" :y-gap="16">
    <n-grid-item v-for="item in cards" :key="item.id">
      <gradient-bg class="h-100px" :start-color="item.colors[0]" :end-color="item.colors[1]">
        <h3 class="text-16px">{{ item.title }}</h3>
        <div class="flex justify-between pt-12px">
          <svg-icon :icon="item.icon" class="text-32px" />
          <count-to :start-value="0" :end-value="item.value" class="text-30px text-white dark:text-dark" />
        </div>
      </gradient-bg>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { GradientBg } from './components';

defineOptions({ name: 'DashboardAnalysisDataCard' });

const props = defineProps<{
  sensors: number;
  unackedAlarms: number;
  runningSamplings: number;
  reports: number;
}>();

const cards = computed(() => [
  {
    id: 'sensors',
    title: '在线传感器',
    value: props.sensors,
    colors: ['#36ad6a', '#18a058'] as [string, string],
    icon: 'mdi:access-point'
  },
  {
    id: 'alarms',
    title: '未确认报警',
    value: props.unackedAlarms,
    colors: ['#e88080', '#d03050'] as [string, string],
    icon: 'mdi:bell-alert-outline'
  },
  {
    id: 'samplings',
    title: '运行中采样',
    value: props.runningSamplings,
    colors: ['#63e2b7', '#36ad6a'] as [string, string],
    icon: 'mdi:flask-outline'
  },
  {
    id: 'reports',
    title: '报表文档',
    value: props.reports,
    colors: ['#70c0e8', '#2080f0'] as [string, string],
    icon: 'mdi:file-chart-outline'
  }
]);
</script>
