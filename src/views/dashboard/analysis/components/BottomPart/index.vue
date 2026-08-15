<template>
  <n-grid :x-gap="16" :y-gap="16" :item-responsive="true">
    <n-grid-item span="0:24 640:24 1024:8">
      <n-card title="报警 / 采样状态" :bordered="false" class="rounded-16px shadow-sm">
        <div class="h-360px overflow-auto">
          <n-timeline v-if="timelineItems.length">
            <n-timeline-item
              v-for="item in timelineItems"
              :key="item.key"
              :type="item.type"
              :title="item.title"
              :content="item.content"
              :time="item.time"
            />
          </n-timeline>
          <n-empty v-else description="暂无事件" size="small" class="pt-48px" />
        </div>
      </n-card>
    </n-grid-item>
    <n-grid-item span="0:24 640:24 1024:16">
      <n-card title="未确认报警" :bordered="false" class="rounded-16px shadow-sm">
        <template #header-extra>
          <a class="text-primary cursor-pointer" @click="go('/alarms')">报警管理</a>
        </template>
        <div class="h-360px">
          <n-data-table
            size="small"
            :columns="columns"
            :data="alarmRows"
            :max-height="320"
            :row-key="(r: AlarmRow) => r.key"
          />
        </div>
      </n-card>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { computed, h } from 'vue';
import { useRouter } from 'vue-router';
import { NTag } from 'naive-ui';
import type { DataTableColumns } from 'naive-ui';
import type { AlarmEvent, SamplingTask } from '@/types';

defineOptions({ name: 'DashboardAnalysisBottomPart' });

const props = defineProps<{
  alarms: AlarmEvent[];
  samplings: SamplingTask[];
}>();

const router = useRouter();
function go(path: string) {
  router.push(path);
}

const severityLabel: Record<string, string> = {
  warning: '预警',
  alarm: '报警',
  communication: '通信',
  flow: '流量'
};

const severityType: Record<string, 'default' | 'info' | 'success' | 'warning' | 'error'> = {
  warning: 'warning',
  alarm: 'error',
  communication: 'info',
  flow: 'warning'
};

interface AlarmRow {
  key: string;
  sensorName: string;
  severity: string;
  message: string;
  raisedAt: string;
}

const alarmRows = computed<AlarmRow[]>(() =>
  props.alarms
    .filter(a => !a.acknowledged)
    .slice(0, 20)
    .map(a => ({
      key: String(a.id),
      sensorName: a.sensorName,
      severity: a.severity,
      message: a.message,
      raisedAt: a.raisedAt
    }))
);

const columns: DataTableColumns<AlarmRow> = [
  { title: '传感器', key: 'sensorName', width: 140 },
  {
    title: '级别',
    key: 'severity',
    width: 90,
    render(row) {
      return h(
        NTag,
        { size: 'small', type: severityType[row.severity] || 'error', bordered: false },
        { default: () => severityLabel[row.severity] || row.severity }
      );
    }
  },
  { title: '消息', key: 'message', ellipsis: { tooltip: true } },
  { title: '时间', key: 'raisedAt', width: 170 }
];

const timelineItems = computed(() => {
  const items: Array<{
    key: string;
    type: 'default' | 'info' | 'success' | 'warning' | 'error';
    title: string;
    content: string;
    time: string;
  }> = [];

  for (const a of props.alarms.slice(0, 6)) {
    items.push({
      key: `a-${a.id}`,
      type: severityType[a.severity] || 'error',
      title: a.acknowledged ? '已确认报警' : '未确认报警',
      content: `${a.sensorName}：${a.message}`,
      time: a.raisedAt
    });
  }

  for (const s of props.samplings.filter(x => x.status === 'running' || x.status === 'scheduled').slice(0, 4)) {
    items.push({
      key: `s-${s.id}`,
      type: s.status === 'running' ? 'success' : 'info',
      title: s.status === 'running' ? '采样运行中' : '采样已调度',
      content: `${s.recipeName || s.recipeId} · ${s.samplingMode}`,
      time: s.scheduledAt || s.startedAt || ''
    });
  }

  return items.slice(0, 10);
});
</script>
