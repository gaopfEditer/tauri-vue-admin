<template>
  <n-card title="采样编辑器" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-button type="primary" @click="openCreate">
        <icon-ic-round-plus class="mr-4px text-20px" />
        调度采样
      </n-button>
      <n-button size="small" type="primary" @click="loadData">
        <icon-mdi-refresh class="mr-4px text-16px" :class="{ 'animate-spin': loading }" />
        刷新
      </n-button>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" :pagination="pagination" />

    <n-modal v-model:show="visible" preset="card" :title="editingId ? '编辑采样' : '调度采样'" class="w-640px">
      <n-form label-placement="left" :label-width="110">
        <n-form-item label="配方" required>
          <n-select v-model:value="form.recipeId" :options="recipeOptions" :disabled="Boolean(editingId)" />
        </n-form-item>
        <n-form-item label="计划时间" required>
          <n-date-picker v-model:value="form.scheduledAt" type="datetime" class="w-full" clearable />
        </n-form-item>
        <n-form-item label="采样模式">
          <n-select v-model:value="form.samplingMode" :options="modeOptions" />
        </n-form-item>
        <n-form-item v-for="field in enabledFields" :key="field.key" :label="field.displayName">
          <n-select
            v-if="field.defaultEntries?.length"
            v-model:value="form.customFields[field.key]"
            :options="field.defaultEntries.map(v => ({ label: v, value: v }))"
            :disabled="!field.editable"
            clearable
          />
          <n-input v-else v-model:value="form.customFields[field.key]" :disabled="!field.editable" />
        </n-form-item>
        <n-form-item label="备注">
          <n-input v-model:value="form.inputNotes" type="textarea" />
        </n-form-item>
      </n-form>
      <n-space justify="end" class="pt-12px">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" :loading="saving" @click="handleSave">确定</n-button>
      </n-space>
    </n-modal>
  </n-card>
</template>

<script setup lang="tsx">
import { computed, onMounted, reactive, ref } from 'vue';
import { NButton, NPopconfirm, NSpace, NTag } from 'naive-ui';
import type { DataTableColumns, PaginationProps, SelectOption } from 'naive-ui';
import {
  abortSampling,
  createSampling,
  deleteSampling,
  fetchRecipeList,
  fetchSamplingCustomFields,
  fetchSamplingList,
  updateSampling
} from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { SamplingCustomFieldDef, SamplingMode, SamplingTask } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const tableData = ref<SamplingTask[]>([]);
const recipeOptions = ref<SelectOption[]>([]);
const customFields = ref<SamplingCustomFieldDef[]>([]);
const visible = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);

const modeOptions: SelectOption[] = [
  { label: 'Operational', value: 'Operational' },
  { label: 'At Rest', value: 'AtRest' },
  { label: 'Normal Operation', value: 'NormalOperation' },
  { label: 'Custom', value: 'Custom' }
];

const form = reactive({
  recipeId: null as string | null,
  scheduledAt: null as number | null,
  samplingMode: 'Operational' as SamplingMode,
  customFields: {} as Record<string, string>,
  inputNotes: ''
});

const enabledFields = computed(() => customFields.value.filter(f => f.enabled));

const statusType: Record<string, NaiveUI.ThemeColor> = {
  scheduled: 'info',
  running: 'warning',
  aborted: 'error',
  finished: 'success',
  failed: 'error'
};

const columns: DataTableColumns<SamplingTask> = [
  { key: 'recipeName', title: '配方', align: 'center' },
  { key: 'scheduledAt', title: '计划时间', align: 'center', width: 180 },
  { key: 'samplingMode', title: '模式', align: 'center', width: 140 },
  {
    key: 'status',
    title: '状态',
    align: 'center',
    width: 110,
    render: row => <NTag type={statusType[row.status] || 'default'}>{row.status}</NTag>
  },
  { key: 'createdBy', title: '创建人', align: 'center', width: 100 },
  {
    key: 'actions',
    title: '操作',
    align: 'center',
    width: 220,
    render: row => (
      <NSpace justify="center">
        {row.status === 'scheduled' && (
          <NButton size="small" onClick={() => openEdit(row)}>
            编辑
          </NButton>
        )}
        {['scheduled', 'running'].includes(row.status) && (
          <NPopconfirm onPositiveClick={() => handleAbort(row.id)}>
            {{
              default: () => '确认中止该采样？',
              trigger: () => (
                <NButton size="small" type="warning">
                  中止
                </NButton>
              )
            }}
          </NPopconfirm>
        )}
        {row.status === 'scheduled' && (
          <NPopconfirm onPositiveClick={() => handleDelete(row.id)}>
            {{
              default: () => '确认删除？',
              trigger: () => <NButton size="small">删除</NButton>
            }}
          </NPopconfirm>
        )}
      </NSpace>
    )
  }
];

const pagination = reactive<PaginationProps>({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 20],
  onChange: page => {
    pagination.page = page;
  },
  onUpdatePageSize: size => {
    pagination.pageSize = size;
    pagination.page = 1;
  }
});

async function loadData() {
  startLoading();
  const [{ data: list }, { data: recipes }, { data: fields }] = await Promise.all([
    fetchSamplingList(),
    fetchRecipeList(),
    fetchSamplingCustomFields()
  ]);
  tableData.value = list ?? [];
  recipeOptions.value = (recipes ?? []).map(r => ({ label: r.name, value: r.id }));
  customFields.value = fields ?? [];
  endLoading();
}

function resetForm() {
  form.recipeId = null;
  form.scheduledAt = Date.now() + 60 * 60 * 1000;
  form.samplingMode = 'Operational';
  form.customFields = {};
  form.inputNotes = '';
  enabledFields.value.forEach(f => {
    form.customFields[f.key] = '';
  });
}

function openCreate() {
  editingId.value = null;
  resetForm();
  visible.value = true;
}

function openEdit(row: SamplingTask) {
  editingId.value = row.id;
  form.recipeId = row.recipeId;
  form.scheduledAt = new Date(row.scheduledAt).getTime();
  form.samplingMode = row.samplingMode;
  form.customFields = { ...(row.customFields || {}) };
  form.inputNotes = row.inputNotes ?? '';
  visible.value = true;
}

async function handleSave() {
  if (!form.recipeId || !form.scheduledAt) {
    window.$message?.warning('请选择配方和计划时间');
    return;
  }
  saving.value = true;
  const payload = {
    recipeId: String(form.recipeId),
    scheduledAt: new Date(form.scheduledAt).toISOString().slice(0, 19).replace('T', ' '),
    samplingMode: form.samplingMode,
    customFields: form.customFields,
    inputNotes: form.inputNotes || undefined,
    createdBy: auth.userInfo.userName
  };
  const result = editingId.value ? await updateSampling(editingId.value, payload) : await createSampling(payload);
  saving.value = false;
  if (result.error) {
    window.$message?.error(result.error.msg || '操作失败');
    return;
  }
  window.$message?.success('操作成功');
  visible.value = false;
  loadData();
}

async function handleAbort(id: string) {
  const { error } = await abortSampling(id, '用户中止');
  if (error) {
    window.$message?.error(error.msg || '中止失败');
    return;
  }
  window.$message?.success('已中止');
  loadData();
}

async function handleDelete(id: string) {
  const { error } = await deleteSampling(id);
  if (error) {
    window.$message?.error(error.msg || '删除失败');
    return;
  }
  window.$message?.success('已删除');
  loadData();
}

onMounted(loadData);
</script>
