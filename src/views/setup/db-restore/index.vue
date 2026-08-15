<template>
  <n-card title="数据库恢复" :bordered="false" class="rounded-16px shadow-sm">
    <n-form label-placement="left" :label-width="140" class="max-w-640px">
      <n-form-item label="备份文件路径" required>
        <n-input v-model:value="filePath" placeholder="./backups/pharma_backup_xxx.json" />
      </n-form-item>
      <n-form-item label="FacilityPro 恢复">
        <n-switch v-model:value="facilityPro" />
      </n-form-item>
      <n-space>
        <n-button type="primary" :loading="loading" @click="handleRestore">执行恢复</n-button>
      </n-space>
    </n-form>
    <n-alert v-if="summary" class="mt-16px" type="success" title="恢复摘要">
      <pre class="text-12px">{{ summary }}</pre>
    </n-alert>
    <ElectronicSignatureModal ref="signRef" />
  </n-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { runRestore } from '@/service';
import { useAuthStore } from '@/store';
import ElectronicSignatureModal from '@/components/business/ElectronicSignatureModal.vue';

const auth = useAuthStore();
const signRef = ref<InstanceType<typeof ElectronicSignatureModal> | null>(null);
const filePath = ref('');
const facilityPro = ref(false);
const loading = ref(false);
const summary = ref('');

async function handleRestore() {
  if (!filePath.value.trim()) {
    window.$message?.warning('请填写备份路径');
    return;
  }
  const signed = await signRef.value?.open({
    action: 'system.restore',
    targetType: 'backup',
    label: 'Restore Database',
    defaultUser: auth.userInfo.userName
  });
  if (!signed) return;

  loading.value = true;
  const { data, error } = await runRestore({
    backupFilePath: filePath.value,
    facilityPro: facilityPro.value,
    performedBy: signed.performedBy
  });
  loading.value = false;
  if (error) return;
  summary.value = JSON.stringify(data?.summary ?? data, null, 2);
  window.$message?.success('恢复请求已记录');
}
</script>
