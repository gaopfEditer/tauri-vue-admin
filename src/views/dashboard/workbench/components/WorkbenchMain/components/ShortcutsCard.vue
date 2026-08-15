<template>
  <div
    class="flex-col-center p-12px border-1px border-[#efeff5] dark:border-[#ffffff17] rounded-4px hover:shadow-sm cursor-pointer"
    @click="emit('click')"
  >
    <svg-icon :icon="icon" :style="{ color: iconColor }" class="text-30px" />
    <p class="py-8px text-14px text-center">{{ label }}</p>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DashboardWorkbenchMainShortcutsCard' });

defineProps<{
  label: string;
  icon: string;
  iconColor: string;
}>();

const emit = defineEmits<{
  (e: 'click'): void;
}>();
</script>
