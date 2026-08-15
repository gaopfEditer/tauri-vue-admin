<template>
  <n-space vertical :size="16">
    <n-card title="当前授权状态" :bordered="false" class="rounded-16px shadow-sm">
      <n-spin :show="loading">
        <n-alert :type="status?.valid ? 'success' : 'warning'" class="mb-12px" :bordered="false">
          {{ status?.reason || '加载中…' }}
          <template v-if="status?.daysLeft != null && status.valid">
            （剩余 {{ status.daysLeft }} 天）
          </template>
        </n-alert>
        <n-descriptions :column="2" label-placement="left">
          <n-descriptions-item label="客户">{{ status?.customer || '-' }}</n-descriptions-item>
          <n-descriptions-item label="授权 ID">{{ status?.licenseId || '-' }}</n-descriptions-item>
          <n-descriptions-item label="生效">{{ status?.validFrom || '-' }}</n-descriptions-item>
          <n-descriptions-item label="到期">{{ status?.validUntil || '-' }}</n-descriptions-item>
          <n-descriptions-item label="设备锁">{{ status?.deviceLock ? '开启' : '关闭' }}</n-descriptions-item>
          <n-descriptions-item label="本机 MAC">{{ status?.currentMac || '-' }}</n-descriptions-item>
          <n-descriptions-item label="绑定 MAC">{{ status?.boundMac || '未绑定' }}</n-descriptions-item>
        </n-descriptions>
        <n-space class="mt-12px">
          <n-button @click="load">刷新</n-button>
          <n-button quaternary type="primary" @click="goLockPage">打开激活页</n-button>
        </n-space>
      </n-spin>
    </n-card>

    <n-card title="导入 / 激活授权码" :bordered="false" class="rounded-16px shadow-sm">
      <n-input v-model:value="activateCode" type="textarea" :rows="3" placeholder="PN1...." class="mb-12px" />
      <n-button type="primary" :loading="activating" @click="doActivate">激活并绑定本机</n-button>
      <n-text depth="3" class="block mt-8px text-12px">
        开启设备锁的授权码在首次激活时会绑定当前 MAC，换机后需重新签发。
      </n-text>
    </n-card>

    <n-card title="厂商签发（需 MASTER KEY）" :bordered="false" class="rounded-16px shadow-sm">
      <n-form label-placement="left" :label-width="110" class="max-w-640px">
        <n-form-item label="客户名称">
          <n-input v-model:value="issueForm.customer" placeholder="现场客户名" />
        </n-form-item>
        <n-form-item label="生效日期">
          <n-input v-model:value="issueForm.validFrom" placeholder="YYYY-MM-DD" />
        </n-form-item>
        <n-form-item label="到期日期">
          <n-input v-model:value="issueForm.validUntil" placeholder="YYYY-MM-DD" />
        </n-form-item>
        <n-form-item label="设备 MAC 锁">
          <n-switch v-model:value="issueForm.deviceLock" />
        </n-form-item>
        <n-form-item label="签发密钥">
          <n-input
            v-model:value="issueForm.masterKey"
            type="password"
            show-password-on="click"
            placeholder="LICENSE_MASTER_KEY"
          />
        </n-form-item>
      </n-form>
      <n-button type="warning" :loading="issuing" @click="doIssue">生成授权码</n-button>
      <n-input
        v-if="issuedCode"
        v-model:value="issuedCode"
        type="textarea"
        :rows="4"
        class="mt-12px"
        readonly
      />
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { useRouter } from 'vue-router';
import { activateLicense, fetchLicenseStatus, issueLicense } from '@/service';
import type { LicenseStatus } from '@/types';

const router = useRouter();
const loading = ref(false);
const activating = ref(false);
const issuing = ref(false);
const status = ref<LicenseStatus | null>(null);
const activateCode = ref('');
const issuedCode = ref('');

const issueForm = reactive({
  customer: '',
  validFrom: '',
  validUntil: '',
  deviceLock: true,
  masterKey: ''
});

async function load() {
  loading.value = true;
  const { data } = await fetchLicenseStatus();
  status.value = data;
  loading.value = false;
}

async function doActivate() {
  if (!activateCode.value.trim()) {
    window.$message?.warning('请填写授权码');
    return;
  }
  activating.value = true;
  const { data, error } = await activateLicense(activateCode.value.trim());
  activating.value = false;
  if (error) return;
  status.value = data;
  window.$message?.success(data?.valid ? '激活成功' : data?.reason || '已提交');
  await load();
}

async function doIssue() {
  if (!issueForm.customer || !issueForm.validFrom || !issueForm.validUntil || !issueForm.masterKey) {
    window.$message?.warning('请完整填写签发信息');
    return;
  }
  issuing.value = true;
  const { data, error } = await issueLicense({ ...issueForm });
  issuing.value = false;
  if (error) return;
  issuedCode.value = data?.licenseCode || '';
  window.$message?.success('已生成授权码');
}

function goLockPage() {
  router.push('/license');
}

onMounted(() => {
  const today = new Date();
  const end = new Date(today);
  end.setFullYear(end.getFullYear() + 1);
  const fmt = (d: Date) => d.toISOString().slice(0, 10);
  issueForm.validFrom = fmt(today);
  issueForm.validUntil = fmt(end);
  load();
});
</script>
