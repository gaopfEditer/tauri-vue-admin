<template>
  <n-card title="采样报告" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px">
      <n-input v-model:value="dateFrom" class="w-220px" placeholder="开始 YYYY-MM-DD HH:mm:ss" />
      <n-input v-model:value="dateTo" class="w-220px" placeholder="结束 YYYY-MM-DD HH:mm:ss" />
      <n-button type="primary" :loading="generating" @click="handleGenerate">生成采样报告</n-button>
    </n-space>

    <n-empty v-if="!document" description="尚未生成报告" />
    <template v-else>
      <n-descriptions bordered :column="2" size="small" class="mb-16px">
        <n-descriptions-item label="标题">{{ document.header?.title }}</n-descriptions-item>
        <n-descriptions-item label="设施">{{ document.header?.facility }}</n-descriptions-item>
        <n-descriptions-item label="区间">{{ document.header?.dateRange }}</n-descriptions-item>
        <n-descriptions-item label="生成人">{{ document.generatedBy }}</n-descriptions-item>
      </n-descriptions>
      <n-data-table :columns="columns" :data="rows" />
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { DataTableColumns } from 'naive-ui';
import { generateReport } from '@/service';
import { useAuthStore } from '@/store';
import type { ReportDocument } from '@/types';

const auth = useAuthStore();
const generating = ref(false);
const document = ref<ReportDocument | null>(null);
const dateFrom = ref('2026-01-01 00:00:00');
const dateTo = ref('2026-12-31 23:59:59');

const rows = computed(() => document.value?.body?.[0]?.rows ?? []);
const columns = computed<DataTableColumns>(() => {
  const cols = document.value?.body?.[0]?.columns ?? [];
  return cols.map(key => ({ key, title: key, align: 'center' as const }));
});

async function handleGenerate() {
  generating.value = true;
  const { data, error } = await generateReport({
    kind: 'sampling',
    mode: 'bySampling',
    dateFrom: dateFrom.value,
    dateTo: dateTo.value,
    generatedBy: auth.userInfo.userName
  });
  generating.value = false;
  if (error) return;
  document.value = data ?? null;
  window.$message?.success('采样报告已生成');
}
</script>
