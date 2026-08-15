<template>
  <div class="realtime-board h-full min-h-560px flex flex-col gap-8px">
    <n-card :bordered="false" class="rounded-12px shadow-sm flex-1" content-style="padding: 12px 16px">
      <template #header>
        <n-space justify="space-between" align="center" class="py-2px">
          <n-space align="center" :size="8">
            <n-text strong class="text-16px">实时信号</n-text>
            <n-tag v-if="currentRoom" type="info" size="small">
              {{ currentRoom.name }}
              <span v-if="checkedRoomIds.length > 1">等 {{ checkedRoomIds.length }} 间</span>
            </n-tag>
            <n-text v-if="checkedRoomIds.length" depth="3" class="text-12px">
              已选 {{ checkedRoomIds.length }} 房间 / {{ selectedSensorCount }} 台设备
            </n-text>
          </n-space>
          <n-space :size="8">
            <n-button
              size="tiny"
              type="primary"
              :disabled="!checkedRoomIds.length"
              :loading="batching"
              @click="batchPower('on')"
            >
              批量开启
            </n-button>
            <n-button size="tiny" :disabled="!checkedRoomIds.length" :loading="batching" @click="batchPower('off')">
              批量关闭
            </n-button>
            <n-button size="tiny" @click="refreshAll">刷新</n-button>
          </n-space>
        </n-space>
      </template>

      <n-spin :show="loading">
        <n-grid v-if="displayCards.length" cols="1 s:2 m:3 l:4" responsive="screen" :x-gap="10" :y-gap="10">
          <n-gi v-for="card in displayCards" :key="card.sensorId">
            <div
              class="signal-card rounded-8px px-10px py-8px h-full transition-all"
              :class="`is-${card.cardStatus || 'normal'}`"
            >
              <n-space justify="space-between" align="center" class="mb-4px" :size="4">
                <n-space align="center" :size="6">
                  <n-checkbox
                    :checked="checkedSensorIds.includes(card.sensorId)"
                    @update:checked="v => toggleSensor(card.sensorId, v)"
                  />
                  <n-tag size="tiny" :type="statusTagType(card.cardStatus)" :bordered="false">
                    {{ card.deviceCode }}
                  </n-tag>
                  <span class="meta-text">{{ card.roomCode || card.roomName }}</span>
                </n-space>
                <n-tag size="tiny" :type="statusTagType(card.cardStatus)">
                  {{ statusText(card.cardStatus) }}
                </n-tag>
              </n-space>

              <div class="card-title mb-4px truncate">{{ card.deviceName }}</div>

              <div class="mb-6px metric-list">
                <div v-for="m in card.metrics" :key="m.key" class="metric-row">
                  <span class="metric-label">{{ m.label }}</span>
                  <n-space align="center" :size="6">
                    <span class="metric-value" :class="`tone-${m.status}`">{{ m.displayValue }}</span>
                    <n-tag size="tiny" :type="statusTagType(m.status)" :bordered="false">
                      {{ m.statusLabel }}
                    </n-tag>
                  </n-space>
                </div>
                <div v-if="card.flowRate != null" class="metric-row">
                  <span class="metric-label">Flow</span>
                  <span class="metric-value">{{ card.flowRate }} L/min</span>
                </div>
              </div>

              <div class="meta-line mb-6px">
                <span>{{ card.cleanroomClass || '-' }}</span>
                ·
                <span>{{ card.serialNumber || '-' }}</span>
                ·
                <span>{{ card.calibrationDate || '-' }}</span>
                ·
                <span>{{ card.operatingMode || '-' }}</span>
              </div>

              <n-space :size="6">
                <n-button
                  size="tiny"
                  type="primary"
                  secondary
                  :disabled="card.powerOn"
                  @click="power(card.sensorId, 'on')"
                >
                  On
                </n-button>
                <n-button size="tiny" secondary :disabled="!card.powerOn" @click="power(card.sensorId, 'off')">
                  Off
                </n-button>
              </n-space>
            </div>
          </n-gi>
        </n-grid>
        <n-empty v-else class="py-40px" description="请勾选或点击底部房间查看设备" />
      </n-spin>
    </n-card>

    <n-card size="small" :bordered="false" class="rounded-12px shadow-sm room-dock" content-style="padding: 8px 12px">
      <n-space justify="space-between" align="center" class="mb-6px">
        <n-space :size="8" align="center">
          <n-text depth="3" class="text-12px">房间导航（可多选）</n-text>
          <n-button size="tiny" quaternary @click="selectAllRooms">全选</n-button>
          <n-button size="tiny" quaternary @click="clearRoomSelection">清空</n-button>
        </n-space>
        <n-space :size="6">
          <n-button
            size="tiny"
            type="primary"
            :disabled="!checkedRoomIds.length"
            :loading="batching"
            @click="batchPower('on')"
          >
            所选房间全部开启
          </n-button>
          <n-button size="tiny" :disabled="!checkedRoomIds.length" :loading="batching" @click="batchPower('off')">
            所选房间全部关闭
          </n-button>
        </n-space>
      </n-space>

      <n-space wrap :size="6">
        <div
          v-for="room in rooms"
          :key="room.id"
          class="room-chip"
          :class="[roomChipClass(room), { active: focusRoomId === room.id, checked: checkedRoomIds.includes(room.id) }]"
          @click="focusRoom(room.id)"
        >
          <n-checkbox
            :checked="checkedRoomIds.includes(room.id)"
            class="mr-4px"
            @click.stop
            @update:checked="v => toggleRoom(room.id, v)"
          />
          <div class="leading-tight">
            <div class="text-12px font-medium">{{ room.name }}</div>
            <div class="text-10px opacity-75">{{ room.code }} · {{ room.deviceCount }}台</div>
          </div>
        </div>
        <n-empty v-if="!rooms.length" description="暂无房间节点" class="w-full" />
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { batchRealtimePower, fetchRealtimeRooms, fetchRealtimeSignals, switchSensorPower } from '@/service';
import { useLoading } from '@/hooks';
import type { RealtimeRoomNav, RealtimeSignalCard, SensorPowerAction } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const rooms = ref<RealtimeRoomNav[]>([]);
const cardsByRoom = ref<Record<string, RealtimeSignalCard[]>>({});
const focusRoomId = ref<string | null>(null);
const checkedRoomIds = ref<string[]>([]);
const checkedSensorIds = ref<string[]>([]);
const batching = ref(false);
let timer: number | undefined;

const currentRoom = computed(() => rooms.value.find(r => r.id === focusRoomId.value) || null);

const displayCards = computed(() => {
  const ids = checkedRoomIds.value.length ? checkedRoomIds.value : focusRoomId.value ? [focusRoomId.value] : [];
  return ids.flatMap(id => cardsByRoom.value[id] || []);
});

const selectedSensorCount = computed(() => {
  if (checkedSensorIds.value.length) return checkedSensorIds.value.length;
  return displayCards.value.length;
});

function statusTagType(status: string): 'success' | 'warning' | 'error' | 'default' {
  if (status === 'alarm') return 'error';
  if (status === 'warning') return 'warning';
  if (status === 'normal') return 'success';
  return 'default';
}

function statusText(status: string) {
  if (status === 'alarm') return '报警';
  if (status === 'warning') return '预警';
  return '正常';
}

function roomChipClass(room: RealtimeRoomNav) {
  if (room.alarmLevel === 'alarm') return 'alarm';
  if (room.alarmLevel === 'warning') return 'warning';
  return 'normal';
}

function toggleRoom(id: string, checked: boolean) {
  if (checked) {
    if (!checkedRoomIds.value.includes(id)) checkedRoomIds.value = [...checkedRoomIds.value, id];
    focusRoomId.value = id;
    ensureRoomLoaded(id);
  } else {
    checkedRoomIds.value = checkedRoomIds.value.filter(x => x !== id);
    if (focusRoomId.value === id) {
      focusRoomId.value = checkedRoomIds.value[0] || null;
    }
  }
}

function toggleSensor(id: string, checked: boolean) {
  if (checked) {
    if (!checkedSensorIds.value.includes(id)) checkedSensorIds.value = [...checkedSensorIds.value, id];
  } else {
    checkedSensorIds.value = checkedSensorIds.value.filter(x => x !== id);
  }
}

function selectAllRooms() {
  checkedRoomIds.value = rooms.value.map(r => r.id);
  if (!focusRoomId.value && rooms.value[0]) focusRoomId.value = rooms.value[0].id;
  checkedRoomIds.value.forEach(id => ensureRoomLoaded(id));
}

function clearRoomSelection() {
  checkedRoomIds.value = [];
  checkedSensorIds.value = [];
}

async function ensureRoomLoaded(roomId: string) {
  if (cardsByRoom.value[roomId]) return;
  const { data } = await fetchRealtimeSignals(roomId);
  cardsByRoom.value = { ...cardsByRoom.value, [roomId]: data ?? [] };
}

async function focusRoom(id: string) {
  focusRoomId.value = id;
  if (!checkedRoomIds.value.includes(id)) {
    checkedRoomIds.value = [...checkedRoomIds.value, id];
  }
  startLoading();
  await ensureRoomLoaded(id);
  endLoading();
}

async function loadRooms() {
  const { data } = await fetchRealtimeRooms();
  rooms.value = data ?? [];
  if (!focusRoomId.value && rooms.value[0]) {
    focusRoomId.value = rooms.value[0].id;
    checkedRoomIds.value = [rooms.value[0].id];
  }
}

async function refreshSignals() {
  const ids = new Set<string>([...checkedRoomIds.value, ...(focusRoomId.value ? [focusRoomId.value] : [])]);
  if (!ids.size) {
    cardsByRoom.value = {};
    return;
  }
  startLoading();
  const entries = await Promise.all(
    [...ids].map(async id => {
      const { data } = await fetchRealtimeSignals(id);
      return [id, data ?? []] as const;
    })
  );
  const next: Record<string, RealtimeSignalCard[]> = { ...cardsByRoom.value };
  for (const [id, list] of entries) next[id] = list;
  cardsByRoom.value = next;
  // 清理已不存在的勾选设备
  const allIds = new Set(Object.values(next).flatMap(list => list.map(c => c.sensorId)));
  checkedSensorIds.value = checkedSensorIds.value.filter(id => allIds.has(id));
  endLoading();
}

async function refreshAll() {
  await loadRooms();
  await refreshSignals();
}

async function power(id: string, action: SensorPowerAction) {
  const { error } = await switchSensorPower(id, action);
  if (error) {
    window.$message?.error(error.msg || '控制失败');
    return;
  }
  window.$message?.success(action === 'on' ? '已开启' : '已关闭');
  await refreshAll();
}

async function batchPower(action: SensorPowerAction) {
  if (!checkedRoomIds.value.length && !checkedSensorIds.value.length) {
    window.$message?.warning('请先勾选房间或设备');
    return;
  }
  batching.value = true;
  const payload =
    checkedSensorIds.value.length > 0
      ? { action, sensorIds: checkedSensorIds.value }
      : { action, roomIds: checkedRoomIds.value };
  const { data, error } = await batchRealtimePower(payload);
  batching.value = false;
  if (error) return;
  window.$message?.success(`批量${action === 'on' ? '开启' : '关闭'}完成，影响 ${data?.affected ?? 0} 台设备`);
  await refreshAll();
}

onMounted(async () => {
  await refreshAll();
  timer = window.setInterval(refreshAll, 5000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<style scoped>
.realtime-board {
  --rt-text: #333639;
  --rt-muted: #8b949e;
  --rt-line: #e8e9eb;
  --rt-normal-bg: #f7fbf8;
  --rt-normal-border: #b7dfc5;
  --rt-normal-accent: #18a058;
  --rt-warning-bg: #fffaf0;
  --rt-warning-border: #f3d19e;
  --rt-warning-accent: #f0a020;
  --rt-alarm-bg: #fff5f5;
  --rt-alarm-border: #f0b6bf;
  --rt-alarm-accent: #d03050;
  --rt-chip-bg: #fafafa;
  --rt-chip-border: #e5e7eb;
  --rt-active: #2080f0;
}

.signal-card {
  color: var(--rt-text);
  background: #fff;
  border: 1px solid var(--rt-line);
  border-left-width: 3px;
  border-left-color: var(--rt-line);
}

.signal-card.is-normal {
  background: var(--rt-normal-bg);
  border-color: var(--rt-normal-border);
  border-left-color: var(--rt-normal-accent);
}

.signal-card.is-warning {
  background: var(--rt-warning-bg);
  border-color: var(--rt-warning-border);
  border-left-color: var(--rt-warning-accent);
}

.signal-card.is-alarm {
  background: var(--rt-alarm-bg);
  border-color: var(--rt-alarm-border);
  border-left-color: var(--rt-alarm-accent);
  box-shadow: 0 0 0 1px rgba(208, 48, 80, 0.08);
}

.card-title {
  color: var(--rt-text);
  font-size: 13px;
  font-weight: 600;
}

.meta-text,
.meta-line {
  color: var(--rt-muted);
  font-size: 11px;
  line-height: 18px;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 3px 0;
  border-bottom: 1px dashed var(--rt-line);
  font-size: 12px;
}

.metric-row:last-child {
  border-bottom: none;
}

.metric-label {
  color: var(--rt-muted);
}

.metric-value {
  color: var(--rt-text);
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.metric-value.tone-warning {
  color: var(--rt-warning-accent);
}

.metric-value.tone-alarm {
  color: var(--rt-alarm-accent);
}

.metric-value.tone-normal {
  color: var(--rt-normal-accent);
}

.room-dock :deep(.n-card__content) {
  padding-top: 8px;
  padding-bottom: 8px;
}

.room-chip {
  display: inline-flex;
  align-items: center;
  min-width: 108px;
  padding: 4px 8px;
  border-radius: 8px;
  border: 1px solid var(--rt-chip-border);
  background: var(--rt-chip-bg);
  color: var(--rt-text);
  cursor: pointer;
  user-select: none;
  transition: border-color 0.15s ease, background 0.15s ease, box-shadow 0.15s ease;
}

.room-chip:hover {
  border-color: #c2c5cc;
}

.room-chip.active {
  border-color: var(--rt-active);
  box-shadow: 0 0 0 1px rgba(32, 128, 240, 0.2);
}

.room-chip.checked {
  background: #f0f7ff;
  border-color: #a8c7f0;
}

.room-chip.alarm {
  border-color: var(--rt-alarm-border);
  background: var(--rt-alarm-bg);
}

.room-chip.warning {
  border-color: var(--rt-warning-border);
  background: var(--rt-warning-bg);
}

.room-chip.alarm.active,
.room-chip.alarm.checked {
  box-shadow: 0 0 0 1px rgba(208, 48, 80, 0.18);
}
</style>
