<template>
  <n-card title="审计轨迹" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" wrap>
      <n-select
        v-model:value="filters.category"
        class="w-160px"
        clearable
        placeholder="类别"
        :options="categoryOptions"
      />
      <n-input v-model:value="filters.action" class="w-180px" placeholder="动作关键词" clearable />
      <n-input v-model:value="filters.targetType" class="w-140px" placeholder="对象类型" clearable />
      <n-input v-model:value="filters.performedBy" class="w-140px" placeholder="操作人" clearable />
      <n-input v-model:value="filters.dateFrom" class="w-200px" placeholder="开始 2026-01-01 00:00:00" />
      <n-input v-model:value="filters.dateTo" class="w-200px" placeholder="结束 2026-12-31 23:59:59" />
      <n-button type="primary" @click="loadData">查询</n-button>
      <n-button @click="openUserEvent">记录 User Event</n-button>
    </n-space>
    <n-alert type="info" class="mb-12px" :bordered="false">
      自动记录：登录/登出、打开页面、配置保存、导出、传感器/配方/采样等写操作，便于追溯。
    </n-alert>
    <n-data-table
      :columns="columns"
      :data="tableData"
      :loading="loading"
      :row-key="r => r.id"
      :scroll-x="1200"
      size="small"
    />

    <n-modal v-model:show="eventVisible" preset="card" title="User Event" class="w-480px">
      <n-form label-placement="left" :label-width="80">
        <n-form-item label="事件" required>
          <n-input v-model:value="eventForm.message" type="textarea" :rows="3" />
        </n-form-item>
        <n-form-item label="原因">
          <n-input v-model:value="eventForm.reason" />
        </n-form-item>
      </n-form>
      <n-space justify="end">
        <n-button @click="eventVisible = false">取消</n-button>
        <n-button type="primary" :loading="eventSaving" @click="submitUserEvent">提交</n-button>
      </n-space>
    </n-modal>
  </n-card>
</template>

<script setup lang="ts">
import { h, onMounted, reactive, ref } from 'vue';
import type { DataTableColumns, SelectOption } from 'naive-ui';
import { NEllipsis } from 'naive-ui';
import { createUserEvent, fetchAuditTrail } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { AuditTrailEntry } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const tableData = ref<AuditTrailEntry[]>([]);
const eventVisible = ref(false);
const eventSaving = ref(false);
const filters = reactive({
  category: null as string | null,
  action: '',
  targetType: '',
  performedBy: '',
  dateFrom: '',
  dateTo: ''
});
const eventForm = reactive({ message: '', reason: '' });

const categoryOptions: SelectOption[] = [
  { label: '登录/登出', value: 'auth.' },
  { label: '打开页面', value: 'page.open' },
  { label: '配置/保存', value: 'save' },
  { label: '导出', value: 'export' },
  { label: '传感器', value: 'sensor.' },
  { label: '配方/采样', value: 'recipe.|sampling.' },
  { label: '报警/趋势', value: 'alarm.|rtTrend.' },
  { label: '系统运维', value: 'system.|setup.|facility.' },
  { label: '用户事件', value: 'userEvent.' }
];

function actionLabel(action: string) {
  const map: Record<string, string> = {
    'auth.login': '登录成功',
    'auth.login.failed': '登录失败',
    'auth.logout': '退出登录',
    'page.open': '打开页面',
    'sensor.deviceConfig.save': '保存粒子设备配置',
    'sensor.limit.save': '保存传感器限值',
    'report.export': '导出报表',
    'export.report.csv': '下载报表 CSV',
    'passwordPolicy.update': '更新密码策略',
    'userEvent.custom': '用户自定义事件'
  };
  return map[action] || action;
}

function detailText(row: AuditTrailEntry) {
  if (row.reason) return row.reason;
  const d = row.detail;
  if (!d) return '';
  if (typeof d.message === 'string') return d.message;
  if (typeof d.title === 'string' && d.title) return String(d.title);
  if (typeof d.path === 'string') return String(d.path);
  try {
    return JSON.stringify(d);
  } catch {
    return '';
  }
}

const columns: DataTableColumns<AuditTrailEntry> = [
  { key: 'eventTime', title: '时间', width: 170 },
  {
    key: 'action',
    title: '动作',
    width: 200,
    render: row => actionLabel(row.action)
  },
  { key: 'targetType', title: '对象类型', width: 110 },
  { key: 'targetId', title: '对象ID', width: 100 },
  { key: 'performedBy', title: '操作人', width: 110 },
  { key: 'role', title: '角色', width: 110 },
  {
    key: 'reason',
    title: '详情',
    ellipsis: { tooltip: true },
    render: row => h(NEllipsis, null, { default: () => detailText(row) })
  }
];

function matchCategory(action: string, category: string | null) {
  if (!category) return true;
  if (category === 'save') {
    return /save|update|create|\.put|Config|policy|wizard|calibration|meta|limit/i.test(action);
  }
  if (category.includes('|')) {
    return category.split('|').some(p => action.includes(p.replace(/\.$/, '')) || action.startsWith(p));
  }
  return action.includes(category) || action.startsWith(category);
}

async function loadData() {
  startLoading();
  const actionFilter = filters.action || undefined;
  const { data } = await fetchAuditTrail({
    action: actionFilter,
    targetType: filters.targetType || undefined,
    performedBy: filters.performedBy || undefined,
    dateFrom: filters.dateFrom || undefined,
    dateTo: filters.dateTo || undefined,
    limit: 500
  });
  if (data) {
    tableData.value = filters.category ? data.filter(r => matchCategory(r.action, filters.category)) : data;
  }
  endLoading();
}

function openUserEvent() {
  eventForm.message = '';
  eventForm.reason = '';
  eventVisible.value = true;
}

async function submitUserEvent() {
  if (!eventForm.message.trim()) {
    window.$message?.warning('请填写事件内容');
    return;
  }
  eventSaving.value = true;
  await createUserEvent({
    message: eventForm.message,
    reason: eventForm.reason || undefined,
    performedBy: auth.userInfo.userName
  });
  eventSaving.value = false;
  eventVisible.value = false;
  window.$message?.success('已记入审计');
  await loadData();
}

onMounted(loadData);
</script>
