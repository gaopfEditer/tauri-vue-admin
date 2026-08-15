<template>
  <n-card title="报警中心" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-space>
        <n-tag type="warning">未确认 {{ unacked }}</n-tag>
        <n-tag>合计 {{ tableData.length }}</n-tag>
      </n-space>
      <n-space>
        <n-button type="warning" :disabled="!unacked" :loading="acking" @click="handleAckAll">全部确认</n-button>
        <n-button size="small" @click="loadData">刷新</n-button>
      </n-space>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" :row-key="r => r.id" />
  </n-card>
</template>

<script setup lang="tsx">
import { computed, onMounted, ref } from 'vue';
import { NButton, NTag } from 'naive-ui';
import type { DataTableColumns } from 'naive-ui';
import { ackAlarm, ackAllAlarms, fetchAlarmList } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { AlarmEvent, AlarmSeverity } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const tableData = ref<AlarmEvent[]>([]);
const acking = ref(false);

const unacked = computed(() => tableData.value.filter(i => !i.acknowledged).length);

const severityMap: Record<AlarmSeverity, 'warning' | 'error' | 'info' | 'default'> = {
  warning: 'warning',
  alarm: 'error',
  communication: 'info',
  flow: 'default'
};

const columns: DataTableColumns<AlarmEvent> = [
  { key: 'raisedAt', title: '时间', align: 'center', width: 170 },
  { key: 'sensorName', title: '传感器', align: 'center', width: 100 },
  {
    key: 'severity',
    title: '级别',
    align: 'center',
    width: 120,
    render: row => <NTag type={severityMap[row.severity] || 'default'}>{row.severity}</NTag>
  },
  { key: 'message', title: '消息', ellipsis: { tooltip: true } },
  { key: 'dataType', title: '数据类型', align: 'center', width: 100 },
  { key: 'value', title: '当前值', align: 'center', width: 90 },
  { key: 'limitValue', title: '限值', align: 'center', width: 90 },
  {
    key: 'acknowledged',
    title: '状态',
    align: 'center',
    width: 100,
    render: row =>
      row.acknowledged ? (
        <NTag type="success" size="small">
          已确认
        </NTag>
      ) : (
        <NTag type="error" size="small">
          未确认
        </NTag>
      )
  },
  {
    title: '操作',
    key: 'actions',
    align: 'center',
    width: 100,
    render: row =>
      row.acknowledged ? (
        <span class="text-gray-400">—</span>
      ) : (
        <NButton size="small" type="primary" onClick={() => handleAck(row.id)}>
          确认
        </NButton>
      )
  }
];

async function loadData() {
  startLoading();
  const { data } = await fetchAlarmList();
  if (data) tableData.value = data;
  endLoading();
}

async function handleAck(id: string) {
  await ackAlarm(id, { acknowledgedBy: auth.userInfo.userName, reason: 'operator ack' });
  window.$message?.success('已确认报警');
  await loadData();
}

async function handleAckAll() {
  acking.value = true;
  await ackAllAlarms({ acknowledgedBy: auth.userInfo.userName, reason: 'ack all' });
  acking.value = false;
  window.$message?.success('已全部确认');
  await loadData();
}

onMounted(loadData);
</script>
