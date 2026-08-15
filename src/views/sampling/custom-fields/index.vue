<template>
  <n-card title="采样自定义字段配置" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-button type="primary" @click="addRow">
        <icon-ic-round-plus class="mr-4px text-20px" />
        新增字段
      </n-button>
      <n-space>
        <n-button @click="loadData">刷新</n-button>
        <n-button type="primary" :loading="saving" @click="handleSave">保存配置</n-button>
      </n-space>
    </n-space>
    <n-data-table :columns="columns" :data="rows" :loading="loading" />
  </n-card>
</template>

<script setup lang="tsx">
import { onMounted, ref } from 'vue';
import { NButton, NInput, NSwitch } from 'naive-ui';
import type { DataTableColumns } from 'naive-ui';
import { fetchSamplingCustomFields, saveSamplingCustomFields } from '@/service';
import { useLoading } from '@/hooks';
import type { SamplingCustomFieldDef } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const rows = ref<SamplingCustomFieldDef[]>([]);
const saving = ref(false);

const columns: DataTableColumns<SamplingCustomFieldDef> = [
  {
    title: '字段 Key',
    key: 'key',
    render: row => <NInput v-model:value={row.key} placeholder="batchNumber" />
  },
  {
    title: '显示名',
    key: 'displayName',
    render: row => <NInput v-model:value={row.displayName} placeholder="Batch Number" />
  },
  {
    title: '启用',
    key: 'enabled',
    width: 90,
    align: 'center',
    render: row => <NSwitch v-model:value={row.enabled} />
  },
  {
    title: '可编辑',
    key: 'editable',
    width: 90,
    align: 'center',
    render: row => <NSwitch v-model:value={row.editable} />
  },
  {
    title: '默认选项(逗号分隔)',
    key: 'defaultEntries',
    render: row => (
      <NInput
        value={(row.defaultEntries || []).join(',')}
        onUpdateValue={v => {
          row.defaultEntries = v
            .split(',')
            .map(s => s.trim())
            .filter(Boolean);
        }}
        placeholder="B001,B002"
      />
    )
  },
  {
    title: '操作',
    key: 'actions',
    width: 90,
    align: 'center',
    render: (_row, index) => (
      <NButton size="small" onClick={() => rows.value.splice(index, 1)}>
        删除
      </NButton>
    )
  }
];

function addRow() {
  rows.value.push({
    key: `field${rows.value.length + 1}`,
    displayName: '',
    enabled: true,
    editable: true,
    defaultEntries: [],
    sortOrder: rows.value.length
  });
}

async function loadData() {
  startLoading();
  const { data } = await fetchSamplingCustomFields();
  rows.value = (data ?? []).map(item => ({ ...item, defaultEntries: item.defaultEntries ?? [] }));
  endLoading();
}

async function handleSave() {
  saving.value = true;
  const { error } = await saveSamplingCustomFields(rows.value);
  saving.value = false;
  if (error) {
    window.$message?.error(error.msg || '保存失败');
    return;
  }
  window.$message?.success('保存成功');
  loadData();
}

onMounted(loadData);
</script>
