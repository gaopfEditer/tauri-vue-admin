<template>
  <n-card title="厂区组织（三级空间架构）" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-text depth="3">厂房/园区 → 区域/洁净区 → 房间 Room</n-text>
      <n-space>
        <n-button type="primary" @click="openCreateRoot">新增厂房/园区</n-button>
        <n-button size="small" @click="loadTree">刷新</n-button>
      </n-space>
    </n-space>

    <n-grid cols="1 m:2" responsive="screen" :x-gap="16" :y-gap="16">
      <n-gi>
        <n-spin :show="loading">
          <n-tree
            block-line
            selectable
            :data="treeOptions"
            :selected-keys="selectedKeys"
            default-expand-all
            key-field="key"
            label-field="label"
            children-field="children"
            @update:selected-keys="onSelect"
          />
          <n-empty v-if="!loading && !treeData.length" class="py-40px" description="暂无厂区节点" />
        </n-spin>
      </n-gi>
      <n-gi>
        <n-card size="small" embedded title="节点详情">
          <template v-if="current">
            <n-descriptions bordered size="small" :column="1" class="mb-12px">
              <n-descriptions-item label="层级">{{ current.levelLabel }} (L{{ current.level }})</n-descriptions-item>
              <n-descriptions-item label="编码">{{ current.code || '-' }}</n-descriptions-item>
              <n-descriptions-item label="名称">{{ current.name }}</n-descriptions-item>
              <n-descriptions-item label="描述">{{ current.description || '-' }}</n-descriptions-item>
              <n-descriptions-item label="排序">{{ current.sortOrder }}</n-descriptions-item>
              <n-descriptions-item label="状态">
                <n-tag :type="current.status ? 'success' : 'default'" size="small">
                  {{ current.status ? '启用' : '停用' }}
                </n-tag>
              </n-descriptions-item>
            </n-descriptions>
            <n-space>
              <n-button size="small" @click="openEdit">编辑</n-button>
              <n-button v-if="current.level < 3" size="small" type="primary" @click="openCreateChild">
                {{ current.level === 1 ? '新增区域' : '新增房间' }}
              </n-button>
              <n-button size="small" type="error" @click="handleDelete">删除</n-button>
            </n-space>
          </template>
          <n-empty v-else description="请选择左侧节点" />
        </n-card>
      </n-gi>
    </n-grid>

    <n-modal v-model:show="visible" preset="card" :title="modalTitle" class="w-520px" :mask-closable="false">
      <n-form label-placement="left" :label-width="90">
        <n-form-item label="层级">
          <n-tag>{{ levelLabel(form.level) }}</n-tag>
        </n-form-item>
        <n-form-item v-if="form.parentId" label="父节点">
          <n-text>{{ parentName || form.parentId }}</n-text>
        </n-form-item>
        <n-form-item label="编码">
          <n-input v-model:value="form.code" placeholder="如 WS-01 / ISO5 / R-3012" />
        </n-form-item>
        <n-form-item label="名称" required>
          <n-input v-model:value="form.name" placeholder="如 一车间 / 百级洁净区 / Room 3012" />
        </n-form-item>
        <n-form-item label="描述">
          <n-input v-model:value="form.description" type="textarea" :rows="2" />
        </n-form-item>
        <n-form-item label="排序">
          <n-input-number v-model:value="form.sortOrder" :min="0" class="w-full" />
        </n-form-item>
        <n-form-item label="启用">
          <n-switch v-model:value="form.status" />
        </n-form-item>
      </n-form>
      <n-space justify="end" class="pt-8px">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" :loading="saving" @click="handleSave">保存</n-button>
      </n-space>
    </n-modal>
  </n-card>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import type { TreeOption } from 'naive-ui';
import { createFacility, deleteFacility, fetchFacilityTree, updateFacility } from '@/service';
import { useLoading } from '@/hooks';
import type { FacilityLevel, FacilityNode } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const treeData = ref<FacilityNode[]>([]);
const selectedKeys = ref<string[]>([]);
const current = ref<FacilityNode | null>(null);
const visible = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);
const parentName = ref('');

const form = reactive({
  parentId: null as string | null,
  level: 1 as FacilityLevel,
  code: '',
  name: '',
  description: '',
  sortOrder: 0,
  status: true
});

const modalTitle = computed(() => {
  if (editingId.value) return '编辑节点';
  return form.level === 1 ? '新增厂房/园区' : form.level === 2 ? '新增区域/洁净区' : '新增房间';
});

function levelLabel(level: FacilityLevel) {
  return level === 1 ? '厂房/园区' : level === 2 ? '区域/洁净区' : '房间 Room';
}

function toTreeOptions(nodes: FacilityNode[]): TreeOption[] {
  return nodes.map(n => ({
    key: n.id,
    label: `${n.name}${n.code ? ` (${n.code})` : ''} · L${n.level}`,
    children: n.children?.length ? toTreeOptions(n.children) : undefined
  }));
}

const treeOptions = computed(() => toTreeOptions(treeData.value));

function findNode(nodes: FacilityNode[], id: string): FacilityNode | null {
  for (const n of nodes) {
    if (n.id === id) return n;
    if (n.children?.length) {
      const hit = findNode(n.children, id);
      if (hit) return hit;
    }
  }
  return null;
}

function onSelect(keys: Array<string | number>) {
  selectedKeys.value = keys.map(String);
  const id = selectedKeys.value[0];
  current.value = id ? findNode(treeData.value, id) : null;
}

async function loadTree() {
  startLoading();
  const { data } = await fetchFacilityTree();
  treeData.value = data ?? [];
  if (current.value) {
    current.value = findNode(treeData.value, current.value.id);
  }
  endLoading();
}

function resetForm(level: FacilityLevel, parentId: string | null, nameHint = '') {
  editingId.value = null;
  form.parentId = parentId;
  form.level = level;
  form.code = '';
  form.name = '';
  form.description = '';
  form.sortOrder = 0;
  form.status = true;
  parentName.value = nameHint;
}

function openCreateRoot() {
  resetForm(1, null);
  visible.value = true;
}

function openCreateChild() {
  if (!current.value || current.value.level >= 3) return;
  const nextLevel = (current.value.level + 1) as FacilityLevel;
  resetForm(nextLevel, current.value.id, current.value.name);
  visible.value = true;
}

function openEdit() {
  if (!current.value) return;
  editingId.value = current.value.id;
  form.parentId = current.value.parentId ?? null;
  form.level = current.value.level;
  form.code = current.value.code || '';
  form.name = current.value.name;
  form.description = current.value.description || '';
  form.sortOrder = current.value.sortOrder ?? 0;
  form.status = current.value.status !== false;
  parentName.value = '';
  visible.value = true;
}

async function handleSave() {
  if (!form.name.trim()) {
    window.$message?.warning('请填写名称');
    return;
  }
  saving.value = true;
  const payload = {
    parentId: form.parentId,
    level: form.level,
    code: form.code,
    name: form.name.trim(),
    description: form.description || null,
    sortOrder: form.sortOrder,
    status: form.status
  };
  const { error } = editingId.value ? await updateFacility(editingId.value, payload) : await createFacility(payload);
  saving.value = false;
  if (error) return;
  window.$message?.success('已保存');
  visible.value = false;
  await loadTree();
}

async function handleDelete() {
  if (!current.value) return;
  const { error } = await deleteFacility(current.value.id);
  if (error) {
    window.$message?.error(error.msg || '删除失败（可能仍有子节点）');
    return;
  }
  window.$message?.success('已删除');
  current.value = null;
  selectedKeys.value = [];
  await loadTree();
}

onMounted(loadTree);
</script>
