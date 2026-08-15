<template>
  <div class="wh-full flex-col-center bg-[#0b1220] text-white px-24px">
    <n-card class="w-560px max-w-95vw rounded-16px" :bordered="false">
      <div class="text-center mb-16px">
        <h2 class="text-22px font-semibold m-0">软件授权锁定</h2>
        <p class="text-[#666] mt-8px mb-0">时间锁 / 设备 MAC 绑定未通过，请激活有效 License</p>
      </div>

      <n-alert v-if="status" :type="status.valid ? 'success' : 'error'" class="mb-16px" :bordered="false">
        {{ status.reason }}
      </n-alert>

      <n-descriptions v-if="status" :column="1" label-placement="left" size="small" class="mb-16px">
        <n-descriptions-item label="本机 MAC">{{ status.currentMac }}</n-descriptions-item>
        <n-descriptions-item v-if="status.boundMac" label="已绑定 MAC">{{ status.boundMac }}</n-descriptions-item>
        <n-descriptions-item v-if="status.validUntil" label="有效期至">{{ status.validUntil }}</n-descriptions-item>
        <n-descriptions-item v-if="status.customer" label="客户">{{ status.customer }}</n-descriptions-item>
      </n-descriptions>

      <n-form label-placement="top">
        <n-form-item label="授权码">
          <n-input
            v-model:value="licenseCode"
            type="textarea"
            :rows="4"
            placeholder="粘贴厂商签发的 PN1.... 授权码"
          />
        </n-form-item>
      </n-form>

      <n-space justify="end">
        <n-button :loading="loading" @click="refresh">刷新状态</n-button>
        <n-button type="primary" :loading="activating" @click="activate">激活授权</n-button>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { activateLicense, fetchLicenseStatus } from '@/service';
import type { LicenseStatus } from '@/types';

const router = useRouter();
const status = ref<LicenseStatus | null>(null);
const licenseCode = ref('');
const loading = ref(false);
const activating = ref(false);

async function refresh() {
  loading.value = true;
  const { data } = await fetchLicenseStatus();
  status.value = data;
  loading.value = false;
  if (data?.valid) {
    window.$message?.success('授权有效');
    router.replace('/login');
  }
}

async function activate() {
  if (!licenseCode.value.trim()) {
    window.$message?.warning('请填写授权码');
    return;
  }
  activating.value = true;
  const { data, error } = await activateLicense(licenseCode.value.trim());
  activating.value = false;
  if (error) return;
  status.value = data;
  if (data?.valid) {
    window.$message?.success('激活成功，请登录');
    router.replace('/login');
  }
}

onMounted(refresh);
</script>
