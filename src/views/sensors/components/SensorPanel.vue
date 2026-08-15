<template>
  <n-card :title="title" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-space align="center">
        <n-radio-group v-model:value="listMode" size="small" @update:value="loadData">
          <n-radio-button value="active">在用</n-radio-button>
          <n-radio-button value="deleted">已删除</n-radio-button>
        </n-radio-group>
      </n-space>
      <n-space>
        <n-button v-if="listMode === 'active'" type="primary" size="small" @click="openCreate">新增设备</n-button>
        <n-button size="small" @click="loadData">
          <icon-mdi-refresh class="mr-4px text-16px" :class="{ 'animate-spin': loading }" />
          刷新
        </n-button>
      </n-space>
    </n-space>

    <n-grid cols="1 s:2 m:3" responsive="screen" :x-gap="16" :y-gap="16">
      <n-gi v-for="item in sensors" :key="item.id">
        <n-card size="small" embedded class="h-full" :class="{ 'opacity-70': item.deleted }">
          <n-space vertical :size="8">
            <n-space justify="space-between">
              <n-text strong>{{ item.customId }}</n-text>
              <n-space>
                <n-tag v-if="item.deleted" type="error" size="small">已删除</n-tag>
                <n-tag v-else :type="item.powerOn ? 'success' : 'default'" size="small">
                  {{ item.runtimeState }}
                </n-tag>
              </n-space>
            </n-space>
            <div class="text-13px text-gray-500">{{ item.description }}</div>
            <div class="text-13px">通道: {{ item.channel || '-' }}</div>
            <div class="text-13px">数值: {{ formatValue(item.lastValue) }} {{ item.unit || '' }}</div>
            <n-space>
              <n-tag v-if="item.comAlarm" type="error" size="small">Com Alarm</n-tag>
              <n-tag v-if="item.flowCalcAlarm" type="error" size="small">Flow Alarm</n-tag>
            </n-space>

            <n-space v-if="!item.deleted">
              <n-button size="tiny" type="primary" @click="power(item.id, 'on')">ON</n-button>
              <n-button size="tiny" @click="power(item.id, 'off')">OFF</n-button>
              <n-button v-if="category === 'particle'" size="tiny" type="warning" @click="power(item.id, 'cleaning')">
                Cleaning
              </n-button>
              <n-button v-if="category === 'particle'" size="tiny" type="info" @click="openConfig(item.id)">
                配置
              </n-button>
              <n-button size="tiny" type="error" @click="handleDelete(item)">删除</n-button>
            </n-space>
            <n-space v-else>
              <n-button size="tiny" type="primary" @click="handleRestore(item)">恢复</n-button>
            </n-space>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
    <n-empty v-if="!loading && !sensors.length" class="py-40px" :description="emptyText" />

    <ParticleDeviceConfigModal v-if="category === 'particle'" ref="configRef" @saved="loadData" />

    <n-modal v-model:show="createVisible" preset="card" title="新增设备" class="w-480px">
      <n-form label-placement="left" :label-width="100">
        <n-form-item label="类别">
          <n-tag>{{ categoryLabel }}</n-tag>
        </n-form-item>
        <n-form-item label="设备编号" required>
          <n-input v-model:value="createForm.customId" placeholder="如 PC-03 / BC-02" />
        </n-form-item>
        <n-form-item label="名称/描述" required>
          <n-input v-model:value="createForm.description" placeholder="设备显示名称" />
        </n-form-item>
        <n-form-item label="通道">
          <n-input v-model:value="createForm.channel" placeholder="可选" />
        </n-form-item>
        <n-form-item label="单位">
          <n-input v-model:value="createForm.unit" placeholder="如 pt/3、L/min" />
        </n-form-item>
      </n-form>
      <n-space justify="end" class="pt-12px">
        <n-button @click="createVisible = false">取消</n-button>
        <n-button type="primary" :loading="creating" @click="handleCreate">创建</n-button>
      </n-space>
    </n-modal>
  </n-card>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue';
import { useDialog } from 'naive-ui';
import { createSensor, deleteSensor, fetchSensorList, restoreSensor, switchSensorPower } from '@/service';
import { useLoading } from '@/hooks';
import type { Sensor, SensorCategory, SensorPowerAction } from '@/types';
import ParticleDeviceConfigModal from './ParticleDeviceConfigModal.vue';

const props = defineProps<{
  category: SensorCategory;
  title: string;
}>();

const dialog = useDialog();
const { loading, startLoading, endLoading } = useLoading(false);
const sensors = ref<Sensor[]>([]);
const listMode = ref<'active' | 'deleted'>('active');
const configRef = ref<InstanceType<typeof ParticleDeviceConfigModal> | null>(null);
const createVisible = ref(false);
const creating = ref(false);
let timer: number | undefined;

const createForm = reactive({
  customId: '',
  description: '',
  channel: '',
  unit: ''
});

const categoryLabel = computed(() => {
  if (props.category === 'particle') return '粒子';
  if (props.category === 'biocapt') return '生物';
  return '模拟量';
});

const emptyText = computed(() => (listMode.value === 'deleted' ? '暂无已删除设备' : '暂无传感器'));

function formatValue(value: Sensor['lastValue']) {
  if (value == null) return '-';
  if (typeof value === 'number') return value;
  return Object.entries(value)
    .map(([k, v]) => `${k}=${v}`)
    .join(', ');
}

async function loadData() {
  startLoading();
  const { data } = await fetchSensorList(props.category, {
    onlyDeleted: listMode.value === 'deleted'
  });
  sensors.value = data ?? [];
  endLoading();
}

async function power(id: string, action: SensorPowerAction) {
  const { error } = await switchSensorPower(id, action);
  if (error) {
    window.$message?.error(error.msg || '操作失败');
    return;
  }
  window.$message?.success(`已切换为 ${action}`);
  loadData();
}

function openConfig(id: string) {
  configRef.value?.open(id);
}

function openCreate() {
  createForm.customId = '';
  createForm.description = '';
  createForm.channel = '';
  createForm.unit = props.category === 'particle' ? 'pt/3' : '';
  createVisible.value = true;
}

async function handleCreate() {
  if (!createForm.customId.trim() || !createForm.description.trim()) {
    window.$message?.warning('请填写设备编号与名称');
    return;
  }
  creating.value = true;
  const { data, error } = await createSensor({
    category: props.category,
    customId: createForm.customId.trim(),
    description: createForm.description.trim(),
    channel: createForm.channel || undefined,
    unit: createForm.unit || undefined
  });
  creating.value = false;
  if (error) return;
  window.$message?.success('已创建设备');
  createVisible.value = false;
  listMode.value = 'active';
  await loadData();
  if (props.category === 'particle' && data?.id) {
    configRef.value?.open(data.id);
  }
}

function handleDelete(item: Sensor) {
  dialog.warning({
    title: '确认删除',
    content: `将软删除设备「${item.customId}」，之后可在「已删除」中恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      const { error } = await deleteSensor(item.id);
      if (error) {
        window.$message?.error(error.msg || '删除失败');
        return;
      }
      window.$message?.success('已删除，可在「已删除」中恢复');
      await loadData();
    }
  });
}

async function handleRestore(item: Sensor) {
  const { error } = await restoreSensor(item.id);
  if (error) {
    window.$message?.error(error.msg || '恢复失败');
    return;
  }
  window.$message?.success('已恢复');
  listMode.value = 'active';
  await loadData();
}

onMounted(() => {
  loadData();
  timer = window.setInterval(() => {
    if (listMode.value === 'active') loadData();
  }, 8000);
});

onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>
