<template>
  <n-card title="Aventino Setup 向导" :bordered="false" class="rounded-16px shadow-sm">
    <n-steps :current="step" class="mb-24px">
      <n-step title="选择模式" />
      <n-step title="数据库连接" />
      <n-step title="执行" />
      <n-step title="完成" />
    </n-steps>

    <div v-if="step === 0">
      <n-radio-group v-model:value="form.mode">
        <n-space vertical>
          <n-radio value="newInstallation">New Installation</n-radio>
          <n-radio value="upgrade">Upgrade</n-radio>
          <n-radio value="restoreFromBackup">Restore from Backup</n-radio>
          <n-radio value="modifyConfiguration">Modify Configuration</n-radio>
          <n-radio value="restoreFacilityPro">Restore FacilityPro</n-radio>
        </n-space>
      </n-radio-group>
    </div>

    <div v-else-if="step === 1" class="max-w-480px">
      <n-form label-placement="left" :label-width="100">
        <n-form-item label="Host"><n-input v-model:value="form.dbConnection.host" /></n-form-item>
        <n-form-item label="Port"><n-input-number v-model:value="form.dbConnection.port" class="w-full" /></n-form-item>
        <n-form-item label="Database"><n-input v-model:value="form.dbConnection.database" /></n-form-item>
        <n-form-item label="User"><n-input v-model:value="form.dbConnection.user" /></n-form-item>
      </n-form>
    </div>

    <div v-else-if="step === 2">
      <n-alert type="info" :title="`模式: ${form.mode}`">
        将标记数据库初始化状态，并写入安装向导进度（复用现有 MySQL schema）。
      </n-alert>
      <n-button class="mt-12px" type="primary" :loading="running" @click="handleExecute">执行</n-button>
    </div>

    <div v-else>
      <n-result status="success" title="向导完成" :description="statusText" />
    </div>

    <n-space class="mt-24px" justify="space-between">
      <n-button :disabled="step === 0 || step >= 3" @click="step -= 1">上一步</n-button>
      <n-button v-if="step < 2" type="primary" @click="nextStep">下一步</n-button>
      <n-button v-else-if="step === 3" type="primary" @click="finish">完成</n-button>
    </n-space>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { fetchSetupStatus, initDatabase, updateSetupWizard } from '@/service';
import { useAuthStore } from '@/store';
import type { SetupWizardMode } from '@/types';

const auth = useAuthStore();
const step = ref(0);
const running = ref(false);
const statusText = ref('');
const form = reactive({
  mode: 'modifyConfiguration' as SetupWizardMode,
  dbConnection: {
    host: '127.0.0.1',
    port: 3306,
    database: 'soybean_admin',
    user: 'root',
    password: ''
  }
});

async function loadStatus() {
  const { data } = await fetchSetupStatus();
  if (!data) return;
  if (data.mode) form.mode = data.mode as SetupWizardMode;
  if (data.dbConnection) Object.assign(form.dbConnection, data.dbConnection);
  statusText.value = data.dbHealthy ? `数据库健康 · initialized=${Boolean(data.initialized)}` : '数据库连接异常';
}

async function nextStep() {
  await updateSetupWizard({
    mode: form.mode,
    step: step.value + 1,
    dbConnection: form.dbConnection
  });
  step.value += 1;
}

async function handleExecute() {
  running.value = true;
  const { data, error } = await initDatabase({
    performedBy: auth.userInfo.userName,
    mode: form.mode
  });
  running.value = false;
  if (error) return;
  statusText.value = data?.note || '初始化完成';
  await updateSetupWizard({ mode: form.mode, step: 3, completed: true, dbConnection: form.dbConnection });
  step.value = 3;
}

async function finish() {
  await updateSetupWizard({ completed: true, step: 3 });
  window.$message?.success('向导已完成');
}

onMounted(loadStatus);
</script>
