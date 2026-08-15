<template>
  <n-card title="配方编辑器" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-button type="primary" @click="openCreate">
        <icon-ic-round-plus class="mr-4px text-20px" />
        新建配方
      </n-button>
      <n-button size="small" type="primary" @click="loadData">
        <icon-mdi-refresh class="mr-4px text-16px" :class="{ 'animate-spin': loading }" />
        刷新
      </n-button>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" :pagination="pagination" />

    <n-modal v-model:show="visible" preset="card" :title="editingId ? '编辑配方' : '新建配方'" class="w-720px">
      <n-form label-placement="left" :label-width="100">
        <n-form-item label="名称" required>
          <n-input v-model:value="form.name" placeholder="配方名称" />
        </n-form-item>
        <n-form-item label="描述">
          <n-input v-model:value="form.description" type="textarea" placeholder="可选描述" />
        </n-form-item>
        <n-form-item label="传感器组">
          <n-select v-model:value="form.sensorGroupIds" multiple :options="groupOptions" />
        </n-form-item>
        <n-form-item label="Pens(JSON)">
          <n-input v-model:value="form.pensJson" type="textarea" :rows="6" placeholder="趋势笔配置 JSON" />
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
import { reactive, ref, onMounted } from 'vue';
import { NButton, NPopconfirm, NSpace } from 'naive-ui';
import type { DataTableColumns, PaginationProps, SelectOption } from 'naive-ui';
import { createRecipe, deleteRecipe, fetchRecipeList, fetchSensorGroups, updateRecipe } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { Recipe } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const tableData = ref<Recipe[]>([]);
const groupOptions = ref<SelectOption[]>([]);
const visible = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);

const form = reactive({
  name: '',
  description: '',
  sensorGroupIds: [] as string[],
  pensJson: '[]'
});

const columns: DataTableColumns<Recipe> = [
  { key: 'name', title: '名称', align: 'center' },
  { key: 'description', title: '描述', align: 'center', ellipsis: { tooltip: true } },
  {
    key: 'sensorGroupIds',
    title: '传感器组',
    align: 'center',
    render: row => (Array.isArray(row.sensorGroupIds) ? row.sensorGroupIds.join(', ') : '-')
  },
  { key: 'createdBy', title: '创建人', align: 'center', width: 100 },
  { key: 'updatedAt', title: '更新时间', align: 'center', width: 180 },
  {
    key: 'actions',
    title: '操作',
    align: 'center',
    width: 180,
    render: row => (
      <NSpace justify="center">
        <NButton size="small" onClick={() => openEdit(row)}>
          编辑
        </NButton>
        <NPopconfirm onPositiveClick={() => handleDelete(row.id)}>
          {{
            default: () => '确认删除该配方？',
            trigger: () => <NButton size="small">删除</NButton>
          }}
        </NPopconfirm>
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
  const [{ data: recipes }, { data: groups }] = await Promise.all([fetchRecipeList(), fetchSensorGroups()]);
  tableData.value = recipes ?? [];
  groupOptions.value = (groups ?? []).map(g => ({ label: g.name, value: g.id }));
  endLoading();
}

function openCreate() {
  editingId.value = null;
  form.name = '';
  form.description = '';
  form.sensorGroupIds = [];
  form.pensJson = '[]';
  visible.value = true;
}

function openEdit(row: Recipe) {
  editingId.value = row.id;
  form.name = row.name;
  form.description = row.description ?? '';
  form.sensorGroupIds = (row.sensorGroupIds as unknown as Array<string | number>).map(String);
  form.pensJson = JSON.stringify(row.pens ?? [], null, 2);
  visible.value = true;
}

async function handleSave() {
  if (!form.name.trim()) {
    window.$message?.warning('请填写配方名称');
    return;
  }
  let pens: unknown;
  try {
    pens = JSON.parse(form.pensJson || '[]');
  } catch {
    window.$message?.error('Pens JSON 格式错误');
    return;
  }
  saving.value = true;
  const payload = {
    name: form.name.trim(),
    description: form.description || undefined,
    sensorGroupIds: form.sensorGroupIds.map(Number),
    pens,
    createdBy: auth.userInfo.userName
  };
  const result = editingId.value ? await updateRecipe(editingId.value, payload) : await createRecipe(payload);
  saving.value = false;
  if (result.error) {
    window.$message?.error(result.error.msg || '保存失败');
    return;
  }
  window.$message?.success('保存成功');
  visible.value = false;
  loadData();
}

async function handleDelete(id: string) {
  const { error } = await deleteRecipe(id);
  if (error) {
    window.$message?.error(error.msg || '删除失败');
    return;
  }
  window.$message?.success('已删除');
  loadData();
}

onMounted(loadData);
</script>
