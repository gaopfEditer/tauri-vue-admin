<template>
  <n-modal
    v-model:show="visible"
    preset="card"
    title="电子签名"
    class="w-480px"
    :mask-closable="false"
    @after-leave="resetForm"
  >
    <n-alert type="info" class="mb-12px" title="合规确认">操作：{{ actionLabel }}</n-alert>
    <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" :label-width="90">
      <n-form-item label="用户" path="userName">
        <n-input v-model:value="form.userName" placeholder="签名用户名" />
      </n-form-item>
      <n-form-item label="密码" path="password">
        <n-input v-model:value="form.password" type="password" show-password-on="click" />
      </n-form-item>
      <n-form-item label="原因" path="reason">
        <n-input v-model:value="form.reason" type="textarea" :rows="2" placeholder="可选" />
      </n-form-item>
    </n-form>
    <n-space justify="end" class="pt-8px">
      <n-button @click="handleCancel">取消</n-button>
      <n-button type="primary" :loading="loading" @click="handleSubmit">确认签名</n-button>
    </n-space>
  </n-modal>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import type { FormInst, FormRules } from 'naive-ui';
import { submitElectronicSignature } from '@/service';
import type { ElectronicSignatureAction, ElectronicSignatureResult } from '@/types';

const visible = ref(false);
const loading = ref(false);
const formRef = ref<FormInst | null>(null);
const actionLabel = ref('');
let resolveFn: ((v: ElectronicSignatureResult | null) => void) | null = null;

const form = reactive({
  userName: '',
  password: '',
  reason: '',
  action: '' as ElectronicSignatureAction,
  targetType: '',
  targetId: undefined as string | undefined
});

const rules: FormRules = {
  userName: { required: true, message: '请输入用户名', trigger: 'blur' },
  password: { required: true, message: '请输入密码', trigger: 'blur' }
};

function resetForm() {
  form.password = '';
  form.reason = '';
}

function open(options: {
  action: ElectronicSignatureAction;
  targetType: string;
  targetId?: string;
  label?: string;
  defaultUser?: string;
}): Promise<ElectronicSignatureResult | null> {
  form.action = options.action;
  form.targetType = options.targetType;
  form.targetId = options.targetId;
  form.userName = options.defaultUser || '';
  actionLabel.value = options.label || String(options.action);
  visible.value = true;
  return new Promise(resolve => {
    resolveFn = resolve;
  });
}

function handleCancel() {
  visible.value = false;
  resolveFn?.(null);
  resolveFn = null;
}

async function handleSubmit() {
  await formRef.value?.validate();
  loading.value = true;
  const { data, error } = await submitElectronicSignature({
    userName: form.userName,
    password: form.password,
    action: form.action,
    targetType: form.targetType,
    targetId: form.targetId,
    reason: form.reason || undefined
  });
  loading.value = false;
  if (error || !data) return;
  window.$message?.success('电子签名成功');
  visible.value = false;
  resolveFn?.(data);
  resolveFn = null;
}

defineExpose({ open });
</script>
