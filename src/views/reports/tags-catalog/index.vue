<template>
  <n-card title="Tags 目录" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-tabs v-model:value="tab" type="segment" @update:value="loadData">
        <n-tab-pane v-for="t in tabs" :key="t" :name="t" :tab="t" />
      </n-tabs>
      <n-space>
        <n-button type="primary" @click="openCreate">新增</n-button>
        <n-button size="small" @click="loadData">刷新</n-button>
      </n-space>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" />

    <n-modal v-model:show="visible" preset="card" title="Tag 条目" class="w-480px">
      <n-form label-placement="left" :label-width="80">
        <n-form-item label="名称" required>
          <n-input v-model:value="form.name" />
        </n-form-item>
        <n-form-item label="颜色">
          <n-color-picker v-model:value="form.color" :modes="['hex']" />
        </n-form-item>
        <n-form-item label="启用">
          <n-switch v-model:value="form.enabled" />
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
import { NButton, NTag } from 'naive-ui';
import type { DataTableColumns } from 'naive-ui';
import { createTagCatalog, deleteTagCatalog, fetchTagCatalog, updateTagCatalog } from '@/service';
import { useLoading } from '@/hooks';
import type { TagCatalogItem, TagCatalogTab } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const tabs: TagCatalogTab[] = [
  'sensorType',
  'sensors',
  'sensorGroups',
  'recipes',
  'dataTypes',
  'dataTypeGroups',
  'sensorsAlarms',
  'tagsControl'
];
const tab = ref<TagCatalogTab>('sensorType');
const tableData = ref<TagCatalogItem[]>([]);
const visible = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);

const form = reactive({
  name: '',
  color: '#2080f0',
  enabled: true
});

const columns: DataTableColumns<TagCatalogItem> = [
  { key: 'name', title: '名称' },
  {
    key: 'color',
    title: '颜色',
    width: 120,
    render: row => (
      <span class="inline-flex items-center gap-8px">
        <span style={{ background: row.color || '#ccc', width: '14px', height: '14px', display: 'inline-block' }} />
        {row.color}
      </span>
    )
  },
  {
    key: 'enabled',
    title: '启用',
    width: 90,
    render: row => <NTag type={row.enabled ? 'success' : 'default'}>{row.enabled ? 'Y' : 'N'}</NTag>
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    render: row => (
      <div class="flex gap-8px justify-center">
        <NButton
          size="tiny"
          onClick={() => {
            editingId.value = row.id;
            form.name = row.name;
            form.color = row.color || '#2080f0';
            form.enabled = row.enabled !== false;
            visible.value = true;
          }}
        >
          编辑
        </NButton>
        <NButton size="tiny" type="error" onClick={() => handleDelete(row.id)}>
          删除
        </NButton>
      </div>
    )
  }
];

async function loadData() {
  startLoading();
  const { data } = await fetchTagCatalog(tab.value);
  if (data) tableData.value = data;
  endLoading();
}

function openCreate() {
  editingId.value = null;
  form.name = '';
  form.color = '#2080f0';
  form.enabled = true;
  visible.value = true;
}

async function handleSave() {
  saving.value = true;
  if (editingId.value) {
    await updateTagCatalog(editingId.value, { ...form, tab: tab.value });
  } else {
    await createTagCatalog({ tab: tab.value, name: form.name, color: form.color, enabled: form.enabled });
  }
  saving.value = false;
  visible.value = false;
  window.$message?.success('已保存');
  await loadData();
}

async function handleDelete(id: string) {
  await deleteTagCatalog(id);
  window.$message?.success('已删除');
  await loadData();
}

onMounted(loadData);
</script>
