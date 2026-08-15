<template>
  <n-card title="运行逻辑" :bordered="false" class="rounded-16px shadow-sm">
    <n-grid cols="1 s:2" responsive="screen" :x-gap="16" :y-gap="16">
      <n-gi>
        <n-card size="small" title="Available Tags" embedded>
          <n-data-table
            size="small"
            :columns="tagColumns"
            :data="tags"
            :loading="loading"
            max-height="360"
            :row-key="r => r.id"
          />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" title="Rules" embedded>
          <n-space class="pb-8px" justify="end">
            <n-button size="small" type="primary" @click="openCreate">新建规则</n-button>
            <n-button size="small" @click="loadData">刷新</n-button>
          </n-space>
          <n-data-table
            size="small"
            :columns="ruleColumns"
            :data="rules"
            :loading="loading"
            max-height="360"
            :row-key="r => r.id"
          />
        </n-card>
      </n-gi>
    </n-grid>

    <n-modal v-model:show="visible" preset="card" :title="editingId ? '编辑规则' : '新建规则'" class="w-560px">
      <n-form label-placement="left" :label-width="90">
        <n-form-item label="名称" required>
          <n-input v-model:value="form.name" />
        </n-form-item>
        <n-form-item label="类型">
          <n-select v-model:value="form.ruleType" :options="ruleTypeOptions" />
        </n-form-item>
        <n-form-item label="启用">
          <n-switch v-model:value="form.enabled" />
        </n-form-item>
        <n-form-item label="条件 JSON">
          <n-input v-model:value="form.conditionsText" type="textarea" :rows="4" />
        </n-form-item>
        <n-form-item label="动作 JSON">
          <n-input v-model:value="form.actionsText" type="textarea" :rows="4" />
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
import type { DataTableColumns, SelectOption } from 'naive-ui';
import {
  createRuntimeRule,
  deleteRuntimeRule,
  fetchRuntimeRules,
  fetchRuntimeTags,
  testRuntimeOutput,
  updateRuntimeRule
} from '@/service';
import { useLoading } from '@/hooks';
import type { RuntimeRule, RuntimeTag } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const tags = ref<RuntimeTag[]>([]);
const rules = ref<RuntimeRule[]>([]);
const visible = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);

const ruleTypeOptions: SelectOption[] = [
  { label: 'condition', value: 'condition' },
  { label: 'towerLight', value: 'towerLight' },
  { label: 'samplingOnInput', value: 'samplingOnInput' },
  { label: 'child', value: 'child' }
];

const form = reactive({
  name: '',
  ruleType: 'condition',
  enabled: true,
  conditionsText: '[]',
  actionsText: '[]'
});

const tagColumns: DataTableColumns<RuntimeTag> = [
  { key: 'folder', title: 'Folder', width: 140 },
  { key: 'name', title: 'Name' },
  { key: 'dataType', title: 'Type', width: 80 },
  {
    key: 'value',
    title: 'Value',
    width: 100,
    render: row => String(row.value ?? '')
  },
  {
    title: 'Test',
    key: 'test',
    width: 90,
    render: row => (
      <NButton size="tiny" onClick={() => handleTest(row)}>
        切换
      </NButton>
    )
  }
];

const ruleColumns: DataTableColumns<RuntimeRule> = [
  { key: 'name', title: '规则名' },
  { key: 'ruleType', title: '类型', width: 130 },
  {
    key: 'enabled',
    title: '启用',
    width: 80,
    render: row => <NTag type={row.enabled ? 'success' : 'default'}>{row.enabled ? 'ON' : 'OFF'}</NTag>
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    render: row => (
      <div class="flex gap-8px justify-center">
        <NButton size="tiny" onClick={() => openEdit(row)}>
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
  const [tagRes, ruleRes] = await Promise.all([fetchRuntimeTags(), fetchRuntimeRules()]);
  if (tagRes.data) tags.value = tagRes.data;
  if (ruleRes.data) rules.value = ruleRes.data;
  endLoading();
}

function openCreate() {
  editingId.value = null;
  form.name = '';
  form.ruleType = 'condition';
  form.enabled = true;
  form.conditionsText = '[]';
  form.actionsText = '[]';
  visible.value = true;
}

function openEdit(row: RuntimeRule) {
  editingId.value = row.id;
  form.name = row.name;
  form.ruleType = String(row.ruleType);
  form.enabled = row.enabled;
  form.conditionsText = JSON.stringify(row.conditions ?? [], null, 2);
  form.actionsText = JSON.stringify(row.actions ?? [], null, 2);
  visible.value = true;
}

async function handleSave() {
  let conditions: RuntimeRule['conditions'] = [];
  let actions: RuntimeRule['actions'] = [];
  try {
    conditions = JSON.parse(form.conditionsText);
    actions = JSON.parse(form.actionsText);
  } catch {
    window.$message?.error('条件/动作 JSON 无效');
    return;
  }
  saving.value = true;
  const payload = {
    name: form.name,
    ruleType: form.ruleType,
    enabled: form.enabled,
    conditions,
    actions
  };
  if (editingId.value) {
    await updateRuntimeRule(editingId.value, payload);
  } else {
    await createRuntimeRule(payload);
  }
  saving.value = false;
  visible.value = false;
  window.$message?.success('已保存');
  await loadData();
}

async function handleDelete(id: string) {
  await deleteRuntimeRule(id);
  window.$message?.success('已删除');
  await loadData();
}

async function handleTest(tag: RuntimeTag) {
  const ruleId = rules.value[0]?.id ?? '0';
  const next = tag.dataType === 'bool' ? !tag.value : typeof tag.value === 'number' ? Number(tag.value) + 1 : tag.value;
  await testRuntimeOutput(ruleId, { tagId: tag.id, value: next as boolean | number | string });
  window.$message?.success(`已测试输出 ${tag.name}`);
  await loadData();
}

onMounted(loadData);
</script>
