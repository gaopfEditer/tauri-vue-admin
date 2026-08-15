<template>
  <n-card title="备份与恢复" :bordered="false" class="rounded-16px shadow-sm">
    <n-form label-placement="left" :label-width="120" class="max-w-640px">
      <n-form-item label="启用日备">
        <n-switch v-model:value="config.enabled" />
      </n-form-item>
      <n-form-item label="日备时间">
        <n-input v-model:value="config.dailyAt" placeholder="HH:mm" class="w-160px" />
      </n-form-item>
      <n-form-item label="备份路径">
        <n-input v-model:value="config.targetPath" />
      </n-form-item>
      <n-form-item label="保留天数">
        <n-input-number v-model:value="config.retainDays" :min="1" :max="3650" />
      </n-form-item>
      <n-space>
        <n-button :loading="saving" @click="saveConfig">保存配置</n-button>
        <n-button type="primary" :loading="backing" @click="handleBackup">立即备份</n-button>
      </n-space>
    </n-form>

    <n-divider />
    <n-space class="pb-8px" justify="space-between">
      <n-text strong>备份任务</n-text>
      <n-button size="small" @click="loadJobs">刷新</n-button>
    </n-space>
    <n-data-table :columns="columns" :data="jobs" :loading="loading" />
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import type { DataTableColumns } from 'naive-ui';
import { fetchBackupConfig, fetchBackupJobs, runBackup, updateBackupConfig } from '@/service';
import { useAuthStore } from '@/store';
import { useLoading } from '@/hooks';
import type { BackupConfig, BackupJob } from '@/types';

const { loading, startLoading, endLoading } = useLoading(false);
const auth = useAuthStore();
const saving = ref(false);
const backing = ref(false);
const jobs = ref<BackupJob[]>([]);
const config = reactive<BackupConfig>({
  enabled: false,
  dailyAt: '02:00',
  targetPath: './backups',
  retainDays: 30
});

const columns: DataTableColumns<BackupJob> = [
  { key: 'id', title: 'ID', width: 70 },
  { key: 'type', title: '类型', width: 90 },
  { key: 'status', title: '状态', width: 100 },
  { key: 'filePath', title: '文件', ellipsis: { tooltip: true } },
  { key: 'createdBy', title: '操作人', width: 100 },
  { key: 'startedAt', title: '开始', width: 170 },
  { key: 'message', title: '消息', ellipsis: { tooltip: true } }
];

async function loadConfig() {
  const { data } = await fetchBackupConfig();
  if (data) Object.assign(config, data);
}

async function loadJobs() {
  startLoading();
  const { data } = await fetchBackupJobs();
  if (data) jobs.value = data;
  endLoading();
}

async function saveConfig() {
  saving.value = true;
  await updateBackupConfig({ ...config });
  saving.value = false;
  window.$message?.success('备份配置已保存');
}

async function handleBackup() {
  backing.value = true;
  const { data, error } = await runBackup(auth.userInfo.userName);
  backing.value = false;
  if (error) return;
  window.$message?.success(`备份完成: ${data?.filePath ?? ''}`);
  await loadJobs();
}

onMounted(async () => {
  await loadConfig();
  await loadJobs();
});
</script>
