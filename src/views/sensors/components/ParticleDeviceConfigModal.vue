<template>
  <n-modal
    v-model:show="visible"
    preset="card"
    title="粒子设备配置"
    class="particle-device-config-modal w-820px"
    :mask-closable="false"
    :block-scroll="false"
    style="max-width: 92vw"
  >
    <n-spin :show="loading">
      <n-form label-placement="left" :label-width="120">
        <n-divider title-placement="left">基础信息</n-divider>
        <n-grid cols="1 s:2" responsive="screen" :x-gap="12">
          <n-gi>
            <n-form-item label="所属厂房/空间" required>
              <n-tree-select
                v-model:value="form.facilityNodeId"
                :options="facilityOptions"
                key-field="key"
                label-field="label"
                children-field="children"
                filterable
                clearable
                placeholder="选择房间节点"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="设备名称" required>
              <n-input v-model:value="form.deviceName" placeholder="如：粒1_0.5立方英尺" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="设备编号" required>
              <n-input v-model:value="form.deviceCode" placeholder="如：001 / PC27" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="仪表类型">
              <n-select v-model:value="form.instrumentType" :options="instrumentOptions" filterable tag />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="更新频率(秒)">
              <n-input-number v-model:value="form.updateIntervalSec" :min="1" :max="3600" class="w-full" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="当前工况">
              <n-select v-model:value="form.productionState" :options="stateOptions" />
            </n-form-item>
          </n-gi>
        </n-grid>

        <n-divider title-placement="left">通讯与数据解析</n-divider>
        <n-grid cols="1 s:2" responsive="screen" :x-gap="12">
          <n-gi>
            <n-form-item label="从机地址">
              <n-input v-model:value="form.slaveAddress" placeholder="如：40001" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="PLC IP">
              <n-select v-model:value="form.plcIp" :options="plcOptions" filterable tag />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="数据单位">
              <n-input v-model:value="form.dataUnit" placeholder="如：pt/3、L/min" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="数据类型">
              <n-select v-model:value="form.dataType" :options="dataTypeOptions" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="数据长度">
              <n-input-number v-model:value="form.dataLength" :min="1" :max="16" class="w-full" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="通讯协议">
              <n-select v-model:value="form.protocol" :options="protocolOptions" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="小数位数">
              <n-input-number v-model:value="form.decimalPlaces" :min="0" :max="6" class="w-full" />
            </n-form-item>
          </n-gi>
        </n-grid>

        <n-divider title-placement="left">台账信息</n-divider>
        <n-grid cols="1 s:2" responsive="screen" :x-gap="12">
          <n-gi>
            <n-form-item label="洁净等级">
              <n-input v-model:value="form.cleanroomClass" placeholder="Class A" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="设备序列号">
              <n-input v-model:value="form.serialNumber" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="校准日期">
              <n-input v-model:value="form.calibrationDate" placeholder="YYYY-MM-DD" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item label="运行模式">
              <n-select v-model:value="form.operatingMode" :options="modeOptions" />
            </n-form-item>
          </n-gi>
        </n-grid>

        <n-divider title-placement="left">多工况状态与告警阈值</n-divider>
        <n-tabs type="line" animated size="small" pane-class="pt-8px">
          <n-tab-pane v-for="group in stateGroups" :key="group.value" :name="group.value" :tab="group.label">
            <template v-for="metric in metricKeys" :key="`${group.value}-${metric}`">
              <n-divider dashed title-placement="left">{{ metric }}</n-divider>
              <n-grid cols="1 s:2" responsive="screen" :x-gap="12">
                <n-gi>
                  <n-form-item label="告警判定">
                    <n-select
                      v-model:value="thresholdMap[`${group.value}::${metric}`].alarmEnable"
                      :options="enableOptions"
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="预警高">
                    <n-input-number
                      v-model:value="thresholdMap[`${group.value}::${metric}`].warnHigh"
                      class="w-full"
                      clearable
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="预警低">
                    <n-input-number
                      v-model:value="thresholdMap[`${group.value}::${metric}`].warnLow"
                      class="w-full"
                      clearable
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="报警高">
                    <n-input-number
                      v-model:value="thresholdMap[`${group.value}::${metric}`].alarmHigh"
                      class="w-full"
                      clearable
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="报警低">
                    <n-input-number
                      v-model:value="thresholdMap[`${group.value}::${metric}`].alarmLow"
                      class="w-full"
                      clearable
                    />
                  </n-form-item>
                </n-gi>
              </n-grid>
            </template>
          </n-tab-pane>
        </n-tabs>
      </n-form>
    </n-spin>

    <n-space justify="end" class="pt-16px">
      <n-button @click="visible = false">取消</n-button>
      <n-button type="primary" :loading="saving" @click="handleSave">保存配置</n-button>
    </n-space>
  </n-modal>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import type { SelectOption, TreeSelectOption } from 'naive-ui';
import { fetchFacilityTree, fetchSensorDeviceConfig, saveSensorDeviceConfig } from '@/service';
import type { FacilityNode, SensorDeviceConfig, SensorStateThreshold } from '@/types';

const visible = ref(false);
const loading = ref(false);
const saving = ref(false);
const sensorId = ref('');
const facilityOptions = ref<TreeSelectOption[]>([]);

const emit = defineEmits<{
  (e: 'saved'): void;
}>();

const stateGroups = [
  { value: 'production', label: '生产状态-生产' },
  { value: 'disinfection', label: '生产状态-消毒' },
  { value: 'static', label: '生产状态-静态' },
  { value: 'post_vent', label: '生产状态-消毒后通风' },
  { value: 'shutdown', label: '生产状态-停产' }
];
const metricKeys = ['0.5um', '5.0um'];

const instrumentOptions: SelectOption[] = [
  { label: '尘埃粒子0.5', value: '尘埃粒子0.5' },
  { label: '尘埃粒子5.0', value: '尘埃粒子5.0' }
];
const plcOptions: SelectOption[] = [
  { label: 'PLC1', value: 'PLC1' },
  { label: 'PLC2', value: 'PLC2' }
];
const dataTypeOptions: SelectOption[] = [
  { label: 'ulong', value: 'ulong' },
  { label: 'float', value: 'float' },
  { label: 'int', value: 'int' }
];
const protocolOptions: SelectOption[] = [
  { label: 'ModbusTCP', value: 'ModbusTCP' },
  { label: 'ModbusRTU', value: 'ModbusRTU' },
  { label: 'OPC UA', value: 'OPCUA' }
];
const modeOptions: SelectOption[] = [
  { label: 'Operational', value: 'Operational' },
  { label: 'AtRest', value: 'AtRest' },
  { label: 'Maintenance', value: 'Maintenance' }
];
const stateOptions: SelectOption[] = stateGroups.map(s => ({ label: s.label, value: s.value }));
const enableOptions: SelectOption[] = [
  { label: '不限制', value: 'unlimited' },
  { label: '启用阈值', value: 'enabled' },
  { label: '关闭判定', value: 'disabled' }
];

type ThrDraft = {
  alarmEnable: string;
  warnHigh: number | null;
  warnLow: number | null;
  alarmHigh: number | null;
  alarmLow: number | null;
};

const thresholdMap = reactive<Record<string, ThrDraft>>({});

const form = reactive({
  facilityNodeId: null as string | null,
  deviceName: '',
  deviceCode: '',
  instrumentType: '尘埃粒子0.5' as string | null,
  updateIntervalSec: 60,
  slaveAddress: '',
  plcIp: 'PLC1' as string | null,
  dataUnit: 'pt/3',
  dataType: 'ulong',
  dataLength: 4,
  protocol: 'ModbusTCP',
  decimalPlaces: 0,
  cleanroomClass: 'Class A',
  serialNumber: '',
  calibrationDate: '',
  operatingMode: 'Operational' as string | null,
  productionState: 'production' as string | null,
  flowRate: null as number | null
});

function ensureThresholdDefaults() {
  for (const g of stateGroups) {
    for (const m of metricKeys) {
      const key = `${g.value}::${m}`;
      if (!thresholdMap[key]) {
        thresholdMap[key] = {
          alarmEnable: 'unlimited',
          warnHigh: null,
          warnLow: null,
          alarmHigh: null,
          alarmLow: null
        };
      }
    }
  }
}

function toFacilityOptions(nodes: FacilityNode[]): TreeSelectOption[] {
  return nodes.map(n => ({
    key: n.id,
    label: `${n.name}${n.code ? ` (${n.code})` : ''}`,
    children: n.children?.length ? toFacilityOptions(n.children) : undefined
  }));
}

async function open(id: string) {
  sensorId.value = id;
  visible.value = true;
  loading.value = true;
  ensureThresholdDefaults();
  const [cfgRes, treeRes] = await Promise.all([fetchSensorDeviceConfig(id), fetchFacilityTree()]);
  if (treeRes.data) facilityOptions.value = toFacilityOptions(treeRes.data);
  const cfg = cfgRes.data;
  if (cfg) {
    form.facilityNodeId = cfg.facilityNodeId ?? null;
    form.deviceName = cfg.deviceName || '';
    form.deviceCode = cfg.deviceCode || '';
    form.instrumentType = cfg.instrumentType || '尘埃粒子0.5';
    form.updateIntervalSec = cfg.updateIntervalSec || 60;
    form.slaveAddress = cfg.slaveAddress || '';
    form.plcIp = cfg.plcIp || 'PLC1';
    form.dataUnit = cfg.dataUnit || 'pt/3';
    form.dataType = cfg.dataType || 'ulong';
    form.dataLength = cfg.dataLength || 4;
    form.protocol = cfg.protocol || 'ModbusTCP';
    form.decimalPlaces = cfg.decimalPlaces ?? 0;
    form.cleanroomClass = cfg.cleanroomClass || 'Class A';
    form.serialNumber = cfg.serialNumber || '';
    form.calibrationDate = cfg.calibrationDate || '';
    form.operatingMode = cfg.operatingMode || 'Operational';
    form.productionState = cfg.productionState || 'production';
    form.flowRate = cfg.flowRate ?? null;
    for (const t of cfg.thresholds || []) {
      const key = `${t.stateGroup}::${t.metricKey}`;
      thresholdMap[key] = {
        alarmEnable: t.alarmEnable || 'unlimited',
        warnHigh: t.warnHigh ?? null,
        warnLow: t.warnLow ?? null,
        alarmHigh: t.alarmHigh ?? null,
        alarmLow: t.alarmLow ?? null
      };
    }
  }
  ensureThresholdDefaults();
  loading.value = false;
}

async function handleSave() {
  if (!form.deviceName.trim() || !form.deviceCode.trim()) {
    window.$message?.warning('请填写设备名称与编号');
    return;
  }
  const thresholds: SensorStateThreshold[] = [];
  for (const g of stateGroups) {
    for (const m of metricKeys) {
      const key = `${g.value}::${m}`;
      const t = thresholdMap[key];
      thresholds.push({
        stateGroup: g.value,
        metricKey: m,
        alarmEnable: t.alarmEnable,
        warnHigh: t.warnHigh,
        warnLow: t.warnLow,
        alarmHigh: t.alarmHigh,
        alarmLow: t.alarmLow
      });
    }
  }
  const body: SensorDeviceConfig = {
    sensorId: sensorId.value,
    facilityNodeId: form.facilityNodeId,
    deviceName: form.deviceName.trim(),
    deviceCode: form.deviceCode.trim(),
    instrumentType: form.instrumentType,
    updateIntervalSec: form.updateIntervalSec,
    slaveAddress: form.slaveAddress,
    plcIp: form.plcIp,
    dataUnit: form.dataUnit,
    dataType: form.dataType,
    dataLength: form.dataLength,
    protocol: form.protocol,
    decimalPlaces: form.decimalPlaces,
    cleanroomClass: form.cleanroomClass,
    serialNumber: form.serialNumber,
    calibrationDate: form.calibrationDate || null,
    operatingMode: form.operatingMode,
    productionState: form.productionState || 'production',
    flowRate: form.flowRate,
    thresholds
  };
  saving.value = true;
  const { error } = await saveSensorDeviceConfig(sensorId.value, body);
  saving.value = false;
  if (error) return;
  window.$message?.success('设备配置已保存');
  visible.value = false;
  emit('saved');
}

defineExpose({ open });
</script>

<style>
/* 弹窗内容过高时：滚动发生在遮罩层外层，而不是卡片内部 */
.n-modal-container:has(.particle-device-config-modal) {
  overflow-y: auto !important;
  align-items: flex-start !important;
  padding: 32px 16px !important;
}

.particle-device-config-modal.n-card,
.n-card.particle-device-config-modal {
  max-height: none !important;
  margin: 0 auto 32px !important;
}

.particle-device-config-modal .n-card__content {
  max-height: none !important;
  overflow: visible !important;
}
</style>
