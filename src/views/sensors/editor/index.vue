<template>
  <n-card title="传感器编辑器" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="end">
      <n-button size="small" type="primary" @click="loadData">刷新</n-button>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" :pagination="pagination" />
  </n-card>
</template>

<script setup lang="tsx">
import { onMounted, reactive, ref } from 'vue';
import { NButton, NInput, NSelect } from 'naive-ui';
import type { DataTableColumns, PaginationProps, SelectOption } from 'naive-ui';
import { fetchSensorGroups, fetchSensorList, updateSensorMeta } from '@/service';
import { useLoading } from '@/hooks';
import type { Sensor } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const tableData = ref<Sensor[]>([]);
const groupOptions = ref<SelectOption[]>([]);
const drafts = reactive<Record<string, { customId: string; description: string; groupId: string | null }>>({});

const columns: DataTableColumns<Sensor> = [
  { key: 'category', title: '类别', align: 'center', width: 100 },
  {
    title: 'Custom ID',
    key: 'customId',
    render: row => <NInput v-model:value={ensureDraft(row).customId} />
  },
  {
    title: 'Description',
    key: 'description',
    render: row => <NInput v-model:value={ensureDraft(row).description} />
  },
  {
    title: '传感器组',
    key: 'groupId',
    width: 180,
    render: row => (
      <NSelect v-model:value={ensureDraft(row).groupId} options={groupOptions.value} clearable placeholder="未分组" />
    )
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    align: 'center',
    render: row => (
      <NButton size="small" type="primary" onClick={() => handleSave(row.id)}>
        保存
      </NButton>
    )
  }
];

const pagination = reactive<PaginationProps>({
  page: 1,
  pageSize: 10,
  onChange: page => {
    pagination.page = page;
  }
});

function ensureDraft(row: Sensor) {
  if (!drafts[row.id]) {
    drafts[row.id] = {
      customId: row.customId,
      description: row.description,
      groupId: row.groupId ?? null
    };
  }
  return drafts[row.id];
}

async function loadData() {
  startLoading();
  const [{ data: sensors }, { data: groups }] = await Promise.all([fetchSensorList(), fetchSensorGroups()]);
  tableData.value = sensors ?? [];
  groupOptions.value = (groups ?? []).map(g => ({ label: g.name, value: g.id }));
  Object.keys(drafts).forEach(k => delete drafts[k]);
  endLoading();
}

async function handleSave(id: string) {
  const draft = drafts[id];
  if (!draft) return;
  const { error } = await updateSensorMeta(id, {
    customId: draft.customId,
    description: draft.description,
    groupId: draft.groupId
  });
  if (error) {
    window.$message?.error(error.msg || '保存失败');
    return;
  }
  window.$message?.success('已保存');
  loadData();
}

onMounted(loadData);
</script>
