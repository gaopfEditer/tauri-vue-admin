<template>
  <n-grid :x-gap="16" :y-gap="16" :item-responsive="true">
    <n-grid-item span="0:24 640:24 1024:16">
      <n-card :bordered="false" class="rounded-16px shadow-sm" :loading="loading">
        <div class="flex w-full h-360px flex-wrap">
          <div class="w-200px h-full py-12px shrink-0">
            <h3 class="text-16px font-bold">运行分析</h3>
            <p class="text-[#aaa] text-13px">近 7 日报警与设备概况</p>
            <h3 class="pt-28px text-24px font-bold">
              <count-to :start-value="0" :end-value="unackedAlarms" />
            </h3>
            <p class="text-[#aaa]">未确认报警</p>
            <h3 class="pt-28px text-24px font-bold">
              <count-to :start-value="0" :end-value="onlineSensors" />
            </h3>
            <p class="text-[#aaa]">在线传感器</p>
            <n-button class="mt-24px" type="primary" @click="goAlarms">查看报警</n-button>
          </div>
          <div class="flex-1-hidden h-full min-w-0">
            <div ref="lineRef" class="wh-full"></div>
          </div>
        </div>
      </n-card>
    </n-grid-item>
    <n-grid-item span="0:24 640:24 1024:8">
      <n-card title="传感器分布" :bordered="false" class="rounded-16px shadow-sm" :loading="loading">
        <div ref="pieRef" class="w-full h-360px"></div>
      </n-card>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Ref } from 'vue';
import { useRouter } from 'vue-router';
import { type ECOption, useEcharts } from '@/composables';

defineOptions({ name: 'DashboardAnalysisTopCard' });

const props = defineProps<{
  loading?: boolean;
  unackedAlarms: number;
  onlineSensors: number;
  trendDays: string[];
  trendCounts: number[];
  category: { particle: number; biocapt: number; analog: number };
}>();

const router = useRouter();
function goAlarms() {
  router.push('/alarms');
}

const lineOptions = ref<ECOption>({
  tooltip: { trigger: 'axis' },
  legend: { data: ['报警次数'] },
  grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
  xAxis: [{ type: 'category', boundaryGap: false, data: props.trendDays }],
  yAxis: [{ type: 'value', minInterval: 1 }],
  series: [
    {
      color: '#d03050',
      name: '报警次数',
      type: 'line',
      smooth: true,
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0.25, color: 'rgba(208,48,80,0.35)' },
            { offset: 1, color: 'rgba(208,48,80,0.02)' }
          ]
        }
      },
      data: props.trendCounts
    }
  ]
}) as Ref<ECOption>;
const { domRef: lineRef } = useEcharts(lineOptions);

const pieOptions = ref<ECOption>({
  tooltip: { trigger: 'item' },
  legend: { bottom: '1%', left: 'center', itemStyle: { borderWidth: 0 } },
  series: [
    {
      color: ['#2080f0', '#18a058', '#f0a020'],
      name: '传感器类型',
      type: 'pie',
      radius: ['45%', '75%'],
      avoidLabelOverlap: false,
      itemStyle: { borderRadius: 8, borderColor: '#fff', borderWidth: 1 },
      label: { show: false, position: 'center' },
      emphasis: { label: { show: true, fontSize: '12' } },
      labelLine: { show: false },
      data: [
        { value: props.category.particle, name: '粒子' },
        { value: props.category.biocapt, name: '生物' },
        { value: props.category.analog, name: '模拟量' }
      ]
    }
  ]
}) as Ref<ECOption>;
const { domRef: pieRef } = useEcharts(pieOptions);

watch(
  () => [props.trendDays, props.trendCounts, props.category],
  () => {
    lineOptions.value = {
      ...lineOptions.value,
      xAxis: [{ type: 'category', boundaryGap: false, data: [...props.trendDays] }],
      series: [
        {
          ...(lineOptions.value.series as any[])[0],
          data: [...props.trendCounts]
        }
      ]
    };
    pieOptions.value = {
      ...pieOptions.value,
      series: [
        {
          ...(pieOptions.value.series as any[])[0],
          data: [
            { value: props.category.particle, name: '粒子' },
            { value: props.category.biocapt, name: '生物' },
            { value: props.category.analog, name: '模拟量' }
          ]
        }
      ]
    };
  },
  { deep: true }
);
</script>
