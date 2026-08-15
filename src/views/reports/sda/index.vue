<template>
  <n-card title="统计分析器 (SDA)" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-select
        v-model:value="sessionId"
        class="w-280px"
        placeholder="选择 SDA 会话"
        :options="sessionOptions"
        @update:value="loadSession"
      />
      <n-space>
        <n-button type="primary" :disabled="!sessionId" :loading="analyzing" @click="handleAnalyze">分析</n-button>
        <n-button size="small" @click="loadSessions">刷新会话</n-button>
      </n-space>
    </n-space>

    <n-alert v-if="session" type="info" class="mb-12px" :title="session.configName || `Session #${session.id}`">
      文件：{{ (session.csvFiles || []).join(', ') || '无' }} · Markers：
      {{ session.markersEnabled ? 'ON' : 'OFF' }}
    </n-alert>

    <n-descriptions v-if="stats" bordered size="small" :column="3" class="mb-12px">
      <n-descriptions-item v-for="(v, k) in stats" :key="k" :label="String(k)">{{ v }}</n-descriptions-item>
    </n-descriptions>

    <n-data-table :columns="columns" :data="points" :loading="analyzing" max-height="420" />
  </n-card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { analyzeSda, fetchSdaSession, fetchSdaSessions } from '@/service';
import type { SdaSession } from '@/types';

const sessions = ref<SdaSession[]>([]);
const sessionId = ref<string | null>(null);
const session = ref<SdaSession | null>(null);
const analyzing = ref(false);
const columnsList = ref<string[]>([]);
const points = ref<Array<Record<string, string | number>>>([]);
const stats = ref<Record<string, number> | null>(null);

const sessionOptions = computed<SelectOption[]>(() =>
  sessions.value.map(s => ({
    label: s.configName || `Session #${s.id}`,
    value: s.id
  }))
);

const columns = computed<DataTableColumns>(() =>
  columnsList.value.map(key => ({ key, title: key, align: 'center' as const }))
);

async function loadSessions() {
  const { data } = await fetchSdaSessions();
  if (data) {
    sessions.value = data;
    if (!sessionId.value && data[0]) {
      sessionId.value = data[0].id;
      await loadSession(data[0].id);
    }
  }
}

async function loadSession(id: string) {
  const { data } = await fetchSdaSession(id);
  session.value = data ?? null;
}

async function handleAnalyze() {
  if (!sessionId.value) return;
  analyzing.value = true;
  const { data, error } = await analyzeSda({ sessionId: sessionId.value });
  analyzing.value = false;
  if (error || !data) return;
  columnsList.value = data.columns || [];
  points.value = data.points || [];
  stats.value = data.stats || null;
  window.$message?.success('分析完成');
}

onMounted(loadSessions);
</script>
