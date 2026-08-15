<template>
  <n-drawer v-model:show="visible" :width="760" placement="right" :trap-focus="false" display-directive="show">
    <n-drawer-content title="运行 Debug 日志" closable :native-scrollbar="false">
      <n-space vertical :size="12">
        <n-space align="center" wrap>
          <n-tag :type="status?.enabled ? 'success' : 'warning'" size="small">
            {{ status?.enabled ? '轮询中' : '已暂停' }}
          </n-tag>
          <n-text depth="3" class="text-12px">周期 #{{ status?.cycle ?? 0 }}</n-text>
          <n-switch v-model:value="autoPopup" size="small">
            <template #checked>出错自动弹出</template>
            <template #unchecked>出错仅角标</template>
          </n-switch>
          <n-button size="tiny" @click="refresh">刷新</n-button>
          <n-button size="tiny" secondary type="primary" @click="togglePoll">
            {{ status?.enabled ? '暂停' : '启动' }}
          </n-button>
          <n-button size="tiny" quaternary @click="goPage">完整联调页</n-button>
        </n-space>

        <n-space align="center" wrap>
          <n-select
            v-model:value="sourceFilter"
            clearable
            size="small"
            class="w-120px"
            placeholder="来源"
            :options="sourceOptions"
          />
          <n-select
            v-model:value="levelFilter"
            clearable
            size="small"
            class="w-110px"
            placeholder="级别"
            :options="levelOptions"
          />
          <n-input-number v-model:value="testSensorId" size="small" :min="1" placeholder="传感器ID" class="w-120px" />
          <n-button size="small" type="warning" :loading="testing" @click="runTest">单次测通</n-button>
          <n-checkbox v-model:checked="autoRefresh" size="small">自动刷新</n-checkbox>
        </n-space>

        <div
          ref="listRef"
          class="debug-log-list h-520px overflow-auto rounded-8px bg-[#0f172a] text-[#e2e8f0] text-12px p-10px font-mono"
        >
          <div v-if="!logs.length" class="text-[#94a3b8] py-24px text-center">暂无运行日志</div>
          <div
            v-for="(row, idx) in logs"
            :key="`${row.time}-${row.source}-${idx}`"
            class="mb-8px pb-8px border-b border-[#1e293b] last:border-0"
          >
            <div class="flex gap-8px items-center flex-wrap">
              <span class="text-[#64748b]">{{ row.time }}</span>
              <n-tag size="tiny" :type="levelType(row.level)" :bordered="false">{{ row.level }}</n-tag>
              <n-tag size="tiny" :type="sourceType(row.source)" :bordered="false">{{ row.source }}</n-tag>
              <span class="text-[#38bdf8]">{{ row.action }}</span>
              <span v-if="row.host" class="text-[#a78bfa]">{{ row.host }}</span>
              <span v-if="row.elapsedMs != null" class="text-[#94a3b8]">{{ row.elapsedMs }}ms</span>
            </div>
            <div class="mt-4px leading-18px break-all">
              <span v-if="row.sensorId" class="text-[#fbbf24]">#{{ row.sensorId }}</span>
              <span v-if="row.deviceCode" class="text-[#34d399]">{{ row.deviceCode }}</span>
              {{ row.message }}
            </div>
          </div>
        </div>

        <n-text depth="3" class="text-12px">
          快捷键 Ctrl+Shift+D · 文件 logs/app.log、logs/modbus-poll.log · 现场默认 SQLite
        </n-text>
      </n-space>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import type { SelectOption } from 'naive-ui';
import { fetchDebugLogs, fetchDevicePollStatus, setDevicePollEnabled, testDevicePoll } from '@/service';
import { useDevicePollDebugStore } from '@/store';

defineOptions({ name: 'DevicePollDebugPanel' });

interface DebugLogEntry {
  time: string;
  level: string;
  source: string;
  sensorId?: number | null;
  deviceCode?: string | null;
  host?: string | null;
  action: string;
  message: string;
  elapsedMs?: number | null;
}

interface PollStatus {
  enabled: boolean;
  cycle: number;
  logCount: number;
}

const debug = useDevicePollDebugStore();
const router = useRouter();

const visible = computed({
  get: () => debug.visible,
  set: (v: boolean) => {
    if (v) debug.open();
    else debug.close();
  }
});

const autoPopup = computed({
  get: () => debug.autoPopupOnError,
  set: (v: boolean) => debug.setAutoPopup(v)
});

const logs = ref<DebugLogEntry[]>([]);
const status = ref<PollStatus | null>(null);
const levelFilter = ref<string | null>(null);
const sourceFilter = ref<string | null>(null);
const autoRefresh = ref(true);
const testing = ref(false);
const testSensorId = ref<number | null>(null);
const listRef = ref<HTMLElement | null>(null);
let timer: number | undefined;
let lastErrorKey = '';

const levelOptions: SelectOption[] = [
  { label: 'INFO', value: 'INFO' },
  { label: 'WARN', value: 'WARN' },
  { label: 'ERROR', value: 'ERROR' }
];

const sourceOptions: SelectOption[] = [
  { label: 'system', value: 'system' },
  { label: 'db', value: 'db' },
  { label: 'modbus', value: 'modbus' }
];

function levelType(level: string) {
  if (level === 'ERROR') return 'error';
  if (level === 'WARN') return 'warning';
  return 'info';
}

function sourceType(source: string) {
  if (source === 'db') return 'success';
  if (source === 'modbus') return 'warning';
  return 'default';
}

async function refresh() {
  const [st, lg] = await Promise.all([
    fetchDevicePollStatus(),
    fetchDebugLogs({
      limit: 150,
      level: levelFilter.value || undefined,
      source: sourceFilter.value || undefined
    })
  ]);
  if (st.data) status.value = st.data as PollStatus;
  if (lg.data) {
    const next = lg.data as DebugLogEntry[];
    const newestError = next.find(r => r.level === 'ERROR');
    if (newestError) {
      const key = `${newestError.time}|${newestError.message}`;
      if (key !== lastErrorKey) {
        lastErrorKey = key;
        debug.notifyError();
      }
    }
    logs.value = next;
  }
}

async function togglePoll() {
  const next = !status.value?.enabled;
  await setDevicePollEnabled(next);
  await refresh();
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
  window.$message?.[data?.ok ? 'success' : 'warning']?.(data?.ok ? '测通成功' : '测通失败');
  await refresh();
}

function goPage() {
  debug.close();
  router.push('/system/device-poll');
}

function onKey(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && (e.key === 'D' || e.key === 'd')) {
    e.preventDefault();
    debug.toggle();
  }
}

watch([levelFilter, sourceFilter], () => refresh());
watch(
  () => debug.visible,
  v => {
    if (v) refresh();
  }
);

onMounted(() => {
  refresh();
  window.addEventListener('keydown', onKey);
  timer = window.setInterval(() => {
    if (autoRefresh.value || !debug.visible) {
      refresh();
    }
  }, 3000);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKey);
  if (timer) window.clearInterval(timer);
});
</script>

<style scoped>
.debug-log-list {
  scrollbar-width: thin;
}
</style>
