<template>
  <n-card title="重置缓冲 (Reset the Buffer)" :bordered="false" class="rounded-16px shadow-sm">
    <n-alert type="warning" class="mb-16px">
      将清空运行时缓冲（实时趋势会话、运行 Tag 当前值），不影响历史库与审计记录。需电子签名确认。
    </n-alert>
    <n-button type="error" :loading="loading" @click="handleReset">重置缓冲</n-button>
    <ElectronicSignatureModal ref="signRef" />
  </n-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { resetSystemBuffer } from '@/service';
import { useAuthStore } from '@/store';
import ElectronicSignatureModal from '@/components/business/ElectronicSignatureModal.vue';

const auth = useAuthStore();
const signRef = ref<InstanceType<typeof ElectronicSignatureModal> | null>(null);
const loading = ref(false);

async function handleReset() {
  const signed = await signRef.value?.open({
    action: 'system.command',
    targetType: 'buffer',
    label: 'Reset the Buffer',
    defaultUser: auth.userInfo.userName
  });
  if (!signed) return;
  loading.value = true;
  const { error } = await resetSystemBuffer({
    performedBy: signed.performedBy,
    reason: `signature=${signed.signatureId}`
  });
  loading.value = false;
  if (error) return;
  window.$message?.success('缓冲已重置');
}
</script>
