<template>
  <n-card title="系统控制" :bordered="false" class="rounded-16px shadow-sm">
    <n-alert type="warning" class="mb-16px" title="高风险操作">
      Restart / Shutdown 会影响 Server 与 Clients，需管理员确认。桌面端会记录审计，实际 OS 动作由客户端确认执行。
    </n-alert>
    <n-space vertical :size="12">
      <n-button type="default" :loading="busy === 'closeApp'" @click="run('closeApp')">关闭应用 (Close App)</n-button>
      <n-button type="warning" :loading="busy === 'restart'" @click="run('restart')">重启 (Restart)</n-button>
      <n-button type="error" :loading="busy === 'shutdown'" @click="run('shutdown')">关机 (Shutdown)</n-button>
      <n-button :loading="busy === 'shutdownDisconnectedClient'" @click="run('shutdownDisconnectedClient')">
        断开客户端关机
      </n-button>
    </n-space>
    <n-text v-if="lastNote" depth="3" class="block mt-16px">{{ lastNote }}</n-text>

    <ElectronicSignatureModal ref="signRef" />
  </n-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { runSystemCommand } from '@/service';
import { useAuthStore } from '@/store';
import ElectronicSignatureModal from '@/components/business/ElectronicSignatureModal.vue';
import type { SystemCommandAction } from '@/types';

const auth = useAuthStore();
const signRef = ref<InstanceType<typeof ElectronicSignatureModal> | null>(null);
const busy = ref<string | null>(null);
const lastNote = ref('');

async function run(action: SystemCommandAction) {
  const signed = await signRef.value?.open({
    action: 'system.command',
    targetType: 'system',
    targetId: action,
    label: `系统命令: ${action}`,
    defaultUser: auth.userInfo.userName
  });
  if (!signed) return;

  busy.value = action;
  const { data, error } = await runSystemCommand({
    action,
    signedBy: signed.performedBy,
    reason: `signature=${signed.signatureId}`
  });
  busy.value = null;
  if (error) return;
  lastNote.value = data?.note || `已接受命令 ${action}`;
  window.$message?.success(`已记录命令：${action}`);
}
</script>
