<template>
  <n-card title="数据库初始化" :bordered="false" class="rounded-16px shadow-sm">
    <n-descriptions bordered :column="2" size="small" class="mb-16px">
      <n-descriptions-item label="DB 健康">
        <n-tag :type="status?.dbHealthy ? 'success' : 'error'">
          {{ status?.dbHealthy ? 'OK' : 'FAIL' }}
        </n-tag>
      </n-descriptions-item>
      <n-descriptions-item label="已初始化">
        {{ status?.initialized ? '是' : '否' }}
      </n-descriptions-item>
      <n-descriptions-item label="Host">{{ status?.dbConnection?.host }}</n-descriptions-item>
      <n-descriptions-item label="Port">{{ status?.dbConnection?.port }}</n-descriptions-item>
      <n-descriptions-item label="Database">{{ status?.dbConnection?.database }}</n-descriptions-item>
      <n-descriptions-item label="User">{{ status?.dbConnection?.user }}</n-descriptions-item>
    </n-descriptions>

    <n-space>
      <n-button @click="loadStatus">刷新状态</n-button>
      <n-button type="primary" :loading="loading" @click="handleInit">标记 / 执行初始化</n-button>
    </n-space>
    <n-text v-if="note" depth="3" class="block mt-12px">{{ note }}</n-text>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { fetchSetupStatus, initDatabase } from '@/service';
import { useAuthStore } from '@/store';

const auth = useAuthStore();
const loading = ref(false);
const note = ref('');
const status = ref<{
  initialized?: boolean;
  dbHealthy?: boolean;
  dbConnection?: { host?: string; port?: number; database?: string; user?: string };
} | null>(null);

async function loadStatus() {
  const { data } = await fetchSetupStatus();
  status.value = data ?? null;
}

async function handleInit() {
  loading.value = true;
  const { data, error } = await initDatabase({
    performedBy: auth.userInfo.userName,
    mode: 'newInstallation'
  });
  loading.value = false;
  if (error) return;
  note.value = data?.note || '完成';
  window.$message?.success('数据库初始化完成');
  await loadStatus();
}

onMounted(loadStatus);
</script>
