<template>
  <n-card title="限值编辑器" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-radio-group v-model:value="category" @update:value="loadData">
        <n-radio-button value="">全部</n-radio-button>
        <n-radio-button value="particle">粒子</n-radio-button>
        <n-radio-button value="biocapt">生物</n-radio-button>
        <n-radio-button value="analog">模拟量</n-radio-button>
      </n-radio-group>
      <n-space>
        <n-button @click="showHistory = true">变更历史</n-button>
        <n-button type="primary" @click="openCreate">新增限值</n-button>
        <n-button size="small" @click="loadData">刷新</n-button>
      </n-space>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" />

    <n-modal v-model:show="visible" preset="card" title="编辑限值" class="w-520px">
      <n-form label-placement="left" :label-width="100">
        <n-form-item label="传感器" required>
          <n-select v-model:value="form.sensorId" :options="sensorOptions" filterable />
        </n-form-item>
        <n-form-item label="数据类型" required>
          <n-input v-model:value="form.dataType" placeholder="如 0.5um / value" />
        </n-form-item>
        <n-form-item label="预警限">
          <n-input-number v-model:value="form.warningLimit" class="w-full" clearable />
        </n-form-item>
        <n-form-item label="报警限">
          <n-input-number v-model:value="form.alarmLimit" class="w-full" clearable />
        </n-form-item>
        <n-form-item label="原因">
          <n-input v-model:value="form.reason" />
        </n-form-item>
      </n-form>
      <n-space justify="end" class="pt-12px">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" :loading="saving" @click="handleSave">保存</n-button>
      </n-space>
    </n-modal>

    <n-drawer v-model:show="showHistory" :width="520" title="限值变更历史">
      <n-list bordered>
        <n-list-item v-for="item in history" :key="item.id">
          <n-thing :title="`${item.signedBy} @ ${item.signedAt}`" :description="item.reason || '无备注'">
            <template #footer>
              <pre class="text-12px">{{ JSON.stringify({ before: item.before, after: item.after }, null, 2) }}</pre>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-drawer>
  </n-card>
</template>

<script setup lang="tsx">
import { onMounted, reactive, ref } from 'vue';
import { NButton } from 'naive-ui';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { fetchSensorLimitHistory, fetchSensorLimits, fetchSensorList, upsertSensorLimit } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { SensorCategory, SensorLimit, SensorLimitHistory } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const category = ref<'' | SensorCategory>('');
const tableData = ref<SensorLimit[]>([]);
const history = ref<SensorLimitHistory[]>([]);
const sensorOptions = ref<SelectOption[]>([]);
const visible = ref(false);
const showHistory = ref(false);
const saving = ref(false);

const form = reactive({
  sensorId: null as string | null,
  dataType: '',
  warningLimit: null as number | null,
  alarmLimit: null as number | null,
  reason: ''
});

const columns: DataTableColumns<SensorLimit> = [
  { key: 'sensorId', title: '传感器ID', align: 'center' },
  { key: 'dataType', title: '数据类型', align: 'center' },
  { key: 'warningLimit', title: '预警限', align: 'center' },
  { key: 'alarmLimit', title: '报警限', align: 'center' },
  { key: 'changedBy', title: '修改人', align: 'center' },
  { key: 'effectiveFrom', title: '生效时间', align: 'center' },
  {
    title: '操作',
    key: 'actions',
    align: 'center',
    width: 100,
    render: row => (
      <NButton
        size="small"
        onClick={() => {
          form.sensorId = row.sensorId;
          form.dataType = row.dataType;
          form.warningLimit = row.warningLimit ?? null;
          form.alarmLimit = row.alarmLimit ?? null;
          form.reason = '';
          visible.value = true;
        }}
      >
        编辑
      </NButton>
    )
  }
];

async function loadData() {
  startLoading();
  const cat = category.value || undefined;
  const [{ data: limits }, { data: sensors }, { data: hist }] = await Promise.all([
    fetchSensorLimits(cat),
    fetchSensorList(cat),
    fetchSensorLimitHistory()
  ]);
  tableData.value = limits ?? [];
  sensorOptions.value = (sensors ?? []).map(s => ({
    label: `${s.customId} (${s.category})`,
    value: s.id
  }));
  history.value = hist ?? [];
  endLoading();
}

function openCreate() {
  form.sensorId = null;
  form.dataType = '';
  form.warningLimit = null;
  form.alarmLimit = null;
  form.reason = '';
  visible.value = true;
}

async function handleSave() {
  if (!form.sensorId || !form.dataType) {
    window.$message?.warning('请填写传感器和数据类型');
    return;
  }
  saving.value = true;
  const { error } = await upsertSensorLimit({
    sensorId: form.sensorId,
    dataType: form.dataType,
    warningLimit: form.warningLimit ?? undefined,
    alarmLimit: form.alarmLimit ?? undefined,
    changedBy: auth.userInfo.userName,
    reason: form.reason || undefined
  });
  saving.value = false;
  if (error) {
    window.$message?.error(error.msg || '保存失败');
    return;
  }
  window.$message?.success('保存成功');
  visible.value = false;
  loadData();
}

onMounted(loadData);
</script>
