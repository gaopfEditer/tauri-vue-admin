<template>
  <n-card title="显示语言" :bordered="false" class="rounded-16px shadow-sm">
    <n-form label-placement="left" :label-width="100" class="max-w-480px">
      <n-form-item label="语言">
        <n-select v-model:value="form.locale" :options="options" @update:value="onLocaleChange" />
      </n-form-item>
      <n-form-item label="显示名">
        <n-input v-model:value="form.displayName" />
      </n-form-item>
      <n-button type="primary" :loading="saving" @click="handleSave">保存</n-button>
    </n-form>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import type { SelectOption } from 'naive-ui';
import { fetchDisplayLanguage, updateDisplayLanguage } from '@/service';

const saving = ref(false);
const options: SelectOption[] = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English (US)', value: 'en-US' },
  { label: 'Italiano', value: 'it-IT' }
];
const form = reactive({ locale: 'zh-CN', displayName: '简体中文' });

function onLocaleChange(v: string) {
  const hit = options.find(o => o.value === v);
  form.displayName = String(hit?.label ?? v);
}

async function loadData() {
  const { data } = await fetchDisplayLanguage();
  if (data) Object.assign(form, data);
}

async function handleSave() {
  saving.value = true;
  await updateDisplayLanguage({ ...form });
  saving.value = false;
  window.$message?.success('语言设置已保存');
}

onMounted(loadData);
</script>
