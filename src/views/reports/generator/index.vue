<template>
  <n-card title="报表生成器" :bordered="false" class="rounded-16px shadow-sm">
    <n-form label-placement="left" :label-width="100" class="max-w-720px">
      <n-form-item label="报告类型" required>
        <n-select v-model:value="form.kind" :options="kindOptions" />
      </n-form-item>
      <n-form-item label="选择模式">
        <n-select v-model:value="form.mode" :options="modeOptions" />
      </n-form-item>
      <n-form-item label="开始日期" required>
        <n-input v-model:value="form.dateFrom" placeholder="YYYY-MM-DD HH:mm:ss" />
      </n-form-item>
      <n-form-item label="结束日期" required>
        <n-input v-model:value="form.dateTo" placeholder="YYYY-MM-DD HH:mm:ss" />
      </n-form-item>
      <n-form-item label="间隔">
        <n-input v-model:value="form.interval" placeholder="如 1h / 15m" />
      </n-form-item>
      <n-space>
        <n-button type="primary" :loading="generating" @click="handleGenerate">生成报告</n-button>
        <n-button @click="loadList">刷新列表</n-button>
      </n-space>
    </n-form>

    <n-divider />
    <n-data-table :columns="columns" :data="list" :loading="loading" />

    <n-drawer v-model:show="showPreview" :width="640" title="报告预览">
      <pre class="text-12px overflow-auto">{{ previewText }}</pre>
      <n-space class="pt-12px">
        <n-button type="primary" :disabled="!previewId" @click="handleExport">导出 CSV</n-button>
      </n-space>
    </n-drawer>
  </n-card>
</template>

<script setup lang="tsx">
import { onMounted, reactive, ref } from 'vue';
import { NButton } from 'naive-ui';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { exportReport, fetchReport, fetchReportList, generateReport } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import { reportExport } from '@/utils';
import type { ReportKind, ReportSelectMode } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const generating = ref(false);
const list = ref<Array<{ id: string; kind: string; mode: string; generatedAt: string; generatedBy: string }>>([]);
const showPreview = ref(false);
const previewText = ref('');
const previewId = ref<string | null>(null);

const kindOptions: SelectOption[] = [
  { label: 'Audit Trail', value: 'audit' },
  { label: 'Data', value: 'data' },
  { label: 'Trend', value: 'trend' },
  { label: 'Sampling', value: 'sampling' }
];
const modeOptions: SelectOption[] = [
  { label: 'By Group', value: 'byGroup' },
  { label: 'By Sampling', value: 'bySampling' }
];

const form = reactive({
  kind: 'audit' as ReportKind,
  mode: 'byGroup' as ReportSelectMode,
  dateFrom: '2026-01-01 00:00:00',
  dateTo: '2026-12-31 23:59:59',
  interval: '1h'
});

const columns: DataTableColumns<(typeof list)['value'][number]> = [
  { key: 'id', title: 'ID', width: 80 },
  { key: 'kind', title: '类型', width: 100 },
  { key: 'mode', title: '模式', width: 120 },
  { key: 'generatedAt', title: '生成时间' },
  { key: 'generatedBy', title: '生成人', width: 120 },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render: row => (
      <NButton size="small" onClick={() => openPreview(row.id)}>
        查看
      </NButton>
    )
  }
];

async function loadList() {
  startLoading();
  const { data } = await fetchReportList();
  if (data) list.value = data;
  endLoading();
}

async function handleGenerate() {
  generating.value = true;
  const { data, error } = await generateReport({
    ...form,
    generatedBy: auth.userInfo.userName
  });
  generating.value = false;
  if (error) return;
  window.$message?.success(`已生成报告 #${data?.id ?? ''}`);
  await loadList();
  if (data?.id) await openPreview(data.id);
}

async function openPreview(id: string) {
  const { data } = await fetchReport(id);
  previewId.value = id;
  previewText.value = JSON.stringify(data, null, 2);
  showPreview.value = true;
}

async function handleExport() {
  if (!previewId.value) return;
  const { data } = await exportReport(previewId.value, 'csv');
  if (!data?.content) return;
  const blob = new Blob([data.content], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `report-${previewId.value}.csv`;
  a.click();
  URL.revokeObjectURL(url);
  reportExport('report.csv', previewId.value, { format: 'csv' });
}

onMounted(loadList);
</script>
