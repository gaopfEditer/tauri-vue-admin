<template>
  <n-card :bordered="false" class="rounded-16px shadow-sm" :loading="loading">
    <div class="flex-y-center justify-between flex-wrap gap-16px">
      <div class="flex-y-center">
        <icon-local-avatar class="text-70px" />
        <div class="pl-12px">
          <h3 class="text-18px font-semibold">{{ greeting }}，{{ auth.userInfo.userName }}</h3>
          <p class="leading-28px text-[#999]">角色 {{ pharmaRole }} · Pharmaceutical Net Pro 洁净区监控工作台</p>
        </div>
      </div>
      <n-space :size="24" :wrap="true">
        <n-statistic label="在线传感器" :value="onlineSensors" />
        <n-statistic label="未确认报警" :value="unackedAlarms">
          <template v-if="unackedAlarms > 0" #suffix>
            <n-tag size="small" type="error" class="ml-6px">需处理</n-tag>
          </template>
        </n-statistic>
        <n-statistic label="运行中采样" :value="runningSamplings" />
        <n-statistic label="配方数" :value="recipes" />
      </n-space>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useAuthStore } from '@/store';
import { mapAuthRoleToPharma } from '@/types';

defineOptions({ name: 'DashboardWorkbenchHeader' });

defineProps<{
  loading?: boolean;
  onlineSensors: number;
  unackedAlarms: number;
  runningSamplings: number;
  recipes: number;
}>();

const auth = useAuthStore();
const pharmaRole = computed(() => mapAuthRoleToPharma(auth.userInfo.userRole));

const greeting = computed(() => {
  const h = new Date().getHours();
  if (h < 6) return '夜深了';
  if (h < 12) return '上午好';
  if (h < 18) return '下午好';
  return '晚上好';
});
</script>
