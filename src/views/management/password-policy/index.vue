<template>
  <n-card title="密码策略" :bordered="false" class="rounded-16px shadow-sm">
    <n-form label-placement="left" :label-width="160" class="max-w-560px">
      <n-form-item label="密码过期天数">
        <n-input-number v-model:value="form.expireDays" :min="0" :max="3650" class="w-full" />
      </n-form-item>
      <n-form-item label="最小密码长度">
        <n-input-number v-model:value="form.minLength" :min="4" :max="64" class="w-full" />
      </n-form-item>
      <n-form-item label="历史密码不可复用">
        <n-input-number v-model:value="form.rememberOldCount" :min="0" :max="20" class="w-full" />
      </n-form-item>
      <n-form-item label="自动登出(秒)">
        <n-input-number v-model:value="form.autoLogoffSeconds" :min="60" :max="86400" class="w-full" />
      </n-form-item>
      <n-form-item label="启用电子签名">
        <n-switch v-model:value="form.electronicSignatureEnabled" />
      </n-form-item>
      <n-button type="primary" :loading="saving" @click="handleSave">保存策略</n-button>
    </n-form>

    <n-divider />
    <n-card size="small" title="用户组功能矩阵 (Table 11-2)" embedded>
      <n-tabs type="line">
        <n-tab-pane v-for="(fns, role) in matrix" :key="role" :name="String(role)" :tab="String(role)">
          <n-space>
            <n-tag v-for="fn in fns" :key="fn" size="small">{{ fn }}</n-tag>
          </n-space>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { fetchGroupFunctions, fetchPasswordPolicy, updatePasswordPolicy } from '@/service';
import type { PasswordPolicy } from '@/types';

const saving = ref(false);
const matrix = ref<Record<string, string[]>>({});
const form = reactive<PasswordPolicy>({
  expireDays: 90,
  minLength: 6,
  rememberOldCount: 3,
  autoLogoffSeconds: 1800,
  electronicSignatureEnabled: true
});

async function loadData() {
  const [policyRes, matrixRes] = await Promise.all([fetchPasswordPolicy(), fetchGroupFunctions()]);
  if (policyRes.data) Object.assign(form, policyRes.data);
  if (matrixRes.data) matrix.value = matrixRes.data;
}

async function handleSave() {
  saving.value = true;
  const { error } = await updatePasswordPolicy({ ...form });
  saving.value = false;
  if (error) return;
  window.$message?.success('密码策略已保存');
}

onMounted(loadData);
</script>
