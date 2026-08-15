<template>
  <n-space vertical :size="16">
    <n-card title="设备轮询联调" :bordered="false" class="rounded-16px shadow-sm">
      <n-alert type="info" :bordered="false" class="mb-12px">
        本页查看 Modbus TCP 主站轮询日志。请在「粒子设备配置」把 PLC IP 填成设备真实 IP（如 192.168.1.50），从机地址填
        Unit ID（通常 1）。文件日志：
        <code>logs/modbus-poll.log</code>
      </n-alert>
      <n-space align="center" class="mb-12px" wrap>
        <n-tag :type="status?.enabled ? 'success' : 'warning'">轮询 {{ status?.enabled ? '运行中' : '已暂停' }}</n-tag>
        <n-text depth="3">周期序号 {{ status?.cycle ?? 0 }} · 内存日志 {{ status?.logCount ?? 0 }}</n-text>
        <n-button size="small" @click="loadStatus">刷新状态</n-button>
        <n-button size="small" type="primary" secondary @click="toggleEnabled">
          {{ status?.enabled ? '暂停轮询' : '启动轮询' }}
        </n-button>
        <n-input-number v-model:value="testSensorId" :min="1" placeholder="传感器 ID" class="w-140px" />
        <n-button size="small" type="warning" :loading="testing" @click="runTest">单次测通</n-button>
      </n-space>
    </n-card>

    <n-card title="轮询日志" :bordered="false" class="rounded-16px shadow-sm">
      <n-space class="mb-12px">
        <n-select v-model:value="levelFilter" clearable placeholder="级别" class="w-120px" :options="levelOptions" />
        <n-button type="primary" :loading="loading" @click="loadLogs">刷新日志</n-button>
        <n-button @click="autoRefresh = !autoRefresh">
          {{ autoRefresh ? '停止自动刷新' : '每 3s 自动刷新' }}
        </n-button>
      </n-space>
      <n-data-table
        size="small"
        :columns="columns"
        :data="logs"
        :max-height="480"
        :row-key="(r: PollLogEntry) => `${r.time}-${r.action}-${r.message}`"
        :scroll-x="1100"
      />
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { h, onMounted, onUnmounted, ref, watch } from 'vue';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { NTag } from 'naive-ui';
import { fetchDevicePollLogs, fetchDevicePollStatus, setDevicePollEnabled, testDevicePoll } from '@/service';

interface PollLogEntry {
  time: string;
  level: string;
  sensorId?: number | null;
  deviceCode?: string | null;
  host?: string | null;
  action: string;
  message: string;
  detail?: Record<string, unknown> | null;
  elapsedMs?: number | null;
}

interface PollStatus {
  enabled: boolean;
  cycle: number;
  logFile?: string;
  logCount: number;
}

const loading = ref(false);
const testing = ref(false);
const autoRefresh = ref(true);
const levelFilter = ref<string | null>(null);
const testSensorId = ref<number | null>(null);
const logs = ref<PollLogEntry[]>([]);
const status = ref<PollStatus | null>(null);
let timer: number | undefined;

const levelOptions: SelectOption[] = [
  { label: 'INFO', value: 'INFO' },
  { label: 'WARN', value: 'WARN' },
  { label: 'ERROR', value: 'ERROR' }
];

const columns: DataTableColumns<PollLogEntry> = [
  { title: '时间', key: 'time', width: 180 },
  {
    title: '级别',
    key: 'level',
    width: 80,
    render(row) {
      const type = row.level === 'ERROR' ? 'error' : row.level === 'WARN' ? 'warning' : 'info';
      return h(NTag, { size: 'small', type, bordered: false }, { default: () => row.level });
    }
  },
  { title: '传感器', key: 'sensorId', width: 80 },
  { title: '编号', key: 'deviceCode', width: 100 },
  { title: '目标', key: 'host', width: 150, ellipsis: { tooltip: true } },
  { title: '动作', key: 'action', width: 110 },
  { title: '耗时ms', key: 'elapsedMs', width: 80 },
  { title: '消息', key: 'message', ellipsis: { tooltip: true } }
];

async function loadStatus() {
  const { data } = await fetchDevicePollStatus();
  if (data) status.value = data as PollStatus;
}

async function loadLogs() {
  loading.value = true;
  const { data } = await fetchDevicePollLogs({
    limit: 200,
    level: levelFilter.value || undefined
  });
  if (data) logs.value = data as PollLogEntry[];
  loading.value = false;
}

async function toggleEnabled() {
  const next = !status.value?.enabled;
  await setDevicePollEnabled(next);
  await loadStatus();
  window.$message?.success(next ? '已启动轮询' : '已暂停轮询');
}

async function runTest() {
  if (!testSensorId.value) {
    window.$message?.warning('请填写传感器 ID');
    return;
  }
  testing.value = true;
  const { data, error } = await testDevicePoll(testSensorId.value);
  testing.value = false;
  if (error) return;
  window.$message?.[data?.ok ? 'success' : 'warning']?.(data?.ok ? '测通成功' : '测通失败，见日志');
  await loadLogs();
}

watch(levelFilter, () => loadLogs());

onMounted(() => {
  loadStatus();
  loadLogs();
  timer = window.setInterval(() => {
    if (autoRefresh.value) {
      loadLogs();
      loadStatus();
    }
  }, 3000);
});

onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>
