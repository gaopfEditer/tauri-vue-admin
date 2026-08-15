<template>
  <n-card title="采样器校准配置" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="end">
      <n-button type="primary" @click="openCreate">新增 / 更新</n-button>
      <n-button size="small" @click="loadData">刷新</n-button>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" />

    <n-modal v-model:show="visible" preset="card" title="校准参数" class="w-520px">
      <n-form label-placement="left" :label-width="100">
        <n-form-item label="Sampler ID" required>
          <n-input v-model:value="form.samplerId" />
        </n-form-item>
        <n-form-item label="参数 JSON" required>
          <n-input v-model:value="form.parametersText" type="textarea" :rows="6" />
        </n-form-item>
      </n-form>
      <n-space justify="end" class="pt-12px">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" :loading="saving" @click="handleSave">保存</n-button>
      </n-space>
    </n-modal>
  </n-card>
</template>

<script setup lang="tsx">
import { onMounted, reactive, ref } from 'vue';
import { NButton } from 'naive-ui';
import type { DataTableColumns } from 'naive-ui';
import { fetchCalibrations, upsertCalibration } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';

type Row = {
  id?: string;
  samplerId: string;
  parameters: Record<string, number | string>;
  updatedBy?: string;
  updatedAt?: string;
};

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const tableData = ref<Row[]>([]);
const visible = ref(false);
const saving = ref(false);
const form = reactive({
  samplerId: '',
  parametersText: '{\n  "flowRate": 28.3,\n  "volume": 1000\n}'
});

const columns: DataTableColumns<Row> = [
  { key: 'samplerId', title: 'Sampler ID', width: 140 },
  {
    key: 'parameters',
    title: '参数',
    render: row => JSON.stringify(row.parameters)
  },
  { key: 'updatedBy', title: '更新人', width: 100 },
  { key: 'updatedAt', title: '更新时间', width: 170 },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    render: row => (
      <NButton
        size="small"
        onClick={() => {
          form.samplerId = row.samplerId;
          form.parametersText = JSON.stringify(row.parameters ?? {}, null, 2);
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
  const { data } = await fetchCalibrations();
  if (data) tableData.value = data as Row[];
  endLoading();
}

function openCreate() {
  form.samplerId = '';
  form.parametersText = '{\n  "flowRate": 28.3,\n  "volume": 1000\n}';
  visible.value = true;
}

async function handleSave() {
  let parameters: Record<string, number | string> = {};
  try {
    parameters = JSON.parse(form.parametersText);
  } catch {
    window.$message?.error('参数 JSON 无效');
    return;
  }
  if (!form.samplerId) {
    window.$message?.warning('请填写 Sampler ID');
    return;
  }
  saving.value = true;
  await upsertCalibration({
    samplerId: form.samplerId,
    parameters,
    updatedBy: auth.userInfo.userName
  });
  saving.value = false;
  visible.value = false;
  window.$message?.success('已保存');
  await loadData();
}

onMounted(loadData);
</script>
