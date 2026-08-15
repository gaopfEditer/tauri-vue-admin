<template>
  <div
    class="p-12px border-1px border-[#efeff5] dark:border-[#ffffff17] rounded-8px hover:shadow-sm cursor-pointer h-full"
    @click="emit('click')"
  >
    <header class="flex-y-center">
      <svg-icon :icon="icon" :style="{ color: iconColor }" class="text-28px" />
      <h3 class="pl-10px text-16px font-semibold">{{ name }}</h3>
    </header>
    <p class="pt-8px text-13px text-[#999] leading-20px">{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DashboardWorkbenchModuleCard' });

defineProps<{
  name: string;
  description: string;
  icon: string;
  iconColor?: string;
}>();

const emit = defineEmits<{
  (e: 'click'): void;
}>();
</script>
