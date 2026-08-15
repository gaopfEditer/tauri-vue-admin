<template>
  <n-card title="实时趋势" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" vertical :size="12">
      <n-space align="center">
        <n-select
          v-model:value="recipeId"
          class="w-280px"
          placeholder="选择配方"
          :options="recipeOptions"
          :disabled="running"
        />
        <n-button type="primary" :disabled="!recipeId || running" :loading="starting" @click="handleStart">
          开始
        </n-button>
        <n-button type="error" :disabled="!running" @click="handleStop">停止</n-button>
        <n-tag :type="running ? 'success' : 'default'">{{ running ? '运行中' : '已停止' }}</n-tag>
        <n-text v-if="session" depth="3">会话 #{{ session.id }} · {{ session.recipeName }}</n-text>
      </n-space>

      <n-data-table :columns="columns" :data="points" :loading="loading" max-height="420" />
    </n-space>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { fetchRecipeList, fetchRtTrendCurrent, fetchRtTrendPoints, startRtTrend, stopRtTrend } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { RealTimeTrendPoint, RealTimeTrendSession } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const recipeId = ref<string | null>(null);
const recipeOptions = ref<SelectOption[]>([]);
const session = ref<RealTimeTrendSession | null>(null);
const points = ref<RealTimeTrendPoint[]>([]);
const running = ref(false);
const starting = ref(false);
let timer: number | undefined;

const columns: DataTableColumns<RealTimeTrendPoint> = [
  { key: 'timestamp', title: '时间', align: 'center' },
  { key: 'sensorId', title: '传感器', align: 'center' },
  { key: 'dataType', title: '数据类型', align: 'center' },
  { key: 'value', title: '数值', align: 'center' }
];

async function loadRecipes() {
  const { data } = await fetchRecipeList();
  if (data) {
    recipeOptions.value = data.map(r => ({ label: r.name, value: r.id }));
  }
}

async function refreshSession() {
  const { data } = await fetchRtTrendCurrent();
  session.value = data ?? null;
  running.value = Boolean(data?.running);
  if (data?.recipeId) recipeId.value = data.recipeId;
}

async function pollPoints() {
  if (!running.value) return;
  startLoading();
  const { data } = await fetchRtTrendPoints();
  if (data) points.value = data;
  endLoading();
}

async function handleStart() {
  if (!recipeId.value) return;
  starting.value = true;
  const { data, error } = await startRtTrend({
    recipeId: recipeId.value,
    startedBy: auth.userInfo.userName
  });
  starting.value = false;
  if (error) return;
  window.$message?.success(`已启动会话 ${data?.id ?? ''}`);
  await refreshSession();
  await pollPoints();
  startPoll();
}

async function handleStop() {
  if (!session.value?.id) return;
  await stopRtTrend({ sessionId: session.value.id });
  window.$message?.success('已停止');
  stopPoll();
  await refreshSession();
}

function startPoll() {
  stopPoll();
  timer = window.setInterval(pollPoints, 3000);
}

function stopPoll() {
  if (timer) {
    clearInterval(timer);
    timer = undefined;
  }
}

onMounted(async () => {
  await loadRecipes();
  await refreshSession();
  if (running.value) {
    await pollPoints();
    startPoll();
  }
});

onUnmounted(stopPoll);
</script>
