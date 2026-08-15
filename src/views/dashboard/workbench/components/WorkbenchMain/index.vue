<template>
  <n-grid :item-responsive="true" :x-gap="16" :y-gap="16">
    <n-grid-item span="0:24 640:24 1024:16">
      <n-space :vertical="true" :size="16">
        <n-card title="业务模块" :bordered="false" size="small" class="shadow-sm rounded-16px">
          <n-grid :item-responsive="true" responsive="screen" cols="m:2 l:3" :x-gap="8" :y-gap="8">
            <n-grid-item v-for="item in modules" :key="item.id">
              <technology-card
                :name="item.name"
                :description="item.description"
                :icon="item.icon"
                :icon-color="item.iconColor"
                @click="go(item.route)"
              />
            </n-grid-item>
          </n-grid>
        </n-card>

        <n-card title="最近审计动态" :bordered="false" size="small" class="shadow-sm rounded-16px">
          <template #header-extra>
            <a class="text-primary cursor-pointer" @click="go('/audit')">查看全部</a>
          </template>
          <n-spin :show="loading">
            <n-list v-if="audits.length">
              <n-list-item v-for="item in audits.slice(0, 8)" :key="item.id">
                <n-thing>
                  <template #header>
                    <span class="text-14px">{{ formatAction(item.action) }}</span>
                    <n-tag size="tiny" class="ml-8px" :bordered="false">{{ item.performedBy }}</n-tag>
                  </template>
                  <template #description>
                    <span class="text-12px text-[#999]">
                      {{ item.eventTime }}
                      <template v-if="item.targetType">· {{ item.targetType }}</template>
                      <template v-if="detailOf(item)">· {{ detailOf(item) }}</template>
                    </span>
                  </template>
                </n-thing>
              </n-list-item>
            </n-list>
            <n-empty v-else description="暂无审计记录" size="small" />
          </n-spin>
        </n-card>
      </n-space>
    </n-grid-item>

    <n-grid-item span="0:24 640:24 1024:8">
      <n-space :vertical="true" :size="16">
        <n-card title="快捷入口" :bordered="false" size="small" class="shadow-sm rounded-16px">
          <n-grid :item-responsive="true" responsive="screen" cols="m:2 l:3" :x-gap="8" :y-gap="8">
            <n-grid-item v-for="item in shortcuts" :key="item.id">
              <shortcuts-card
                :label="item.label"
                :icon="item.icon"
                :icon-color="item.iconColor"
                @click="go(item.route)"
              />
            </n-grid-item>
          </n-grid>
        </n-card>

        <n-card title="运行摘要" :bordered="false" size="small" class="shadow-sm rounded-16px">
          <n-descriptions :column="1" label-placement="left" size="small">
            <n-descriptions-item label="传感器总数">{{ sensorTotal }}</n-descriptions-item>
            <n-descriptions-item label="粒子 / 生物 / 模拟">
              {{ category.particle }} / {{ category.biocapt }} / {{ category.analog }}
            </n-descriptions-item>
            <n-descriptions-item label="已调度采样">{{ scheduled }}</n-descriptions-item>
            <n-descriptions-item label="报表文档">{{ reportCount }}</n-descriptions-item>
          </n-descriptions>
          <n-divider />
          <n-space>
            <n-button type="primary" secondary size="small" @click="go('/sampling/realtime')">实时信号</n-button>
            <n-button type="warning" secondary size="small" @click="go('/alarms')">报警管理</n-button>
            <n-button secondary size="small" @click="go('/rt-trend')">实时趋势</n-button>
          </n-space>
        </n-card>
      </n-space>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import type { AuditTrailEntry } from '@/types';
import { ShortcutsCard, TechnologyCard } from './components';

defineOptions({ name: 'DashboardWorkbenchMain' });

defineProps<{
  loading?: boolean;
  audits: AuditTrailEntry[];
  sensorTotal: number;
  category: { particle: number; biocapt: number; analog: number };
  scheduled: number;
  reportCount: number;
}>();

const router = useRouter();

function go(path: string) {
  router.push(path);
}

function formatAction(action: string) {
  const map: Record<string, string> = {
    'auth.login': '登录系统',
    'auth.logout': '退出登录',
    'page.open': '打开页面',
    'sensor.deviceConfig.save': '保存设备配置',
    'report.export': '导出报表',
    'userEvent.custom': '用户事件'
  };
  return map[action] || action;
}

function detailOf(item: AuditTrailEntry) {
  const d = item.detail;
  if (!d) return item.reason || '';
  if (typeof d.title === 'string' && d.title) return d.title;
  if (typeof d.path === 'string') return d.path;
  if (typeof d.message === 'string') return d.message;
  return item.reason || '';
}

const modules = [
  {
    id: 1,
    name: '配方管理',
    description: 'Recipes Editor：配方生命周期与采样笔配置',
    icon: 'mdi:file-document-edit-outline',
    iconColor: '#2080f0',
    route: '/recipe/list'
  },
  {
    id: 2,
    name: '采样管理',
    description: '调度、中止与自定义字段，含实时信号看板',
    icon: 'mdi:flask-outline',
    iconColor: '#18a058',
    route: '/sampling/list'
  },
  {
    id: 3,
    name: '传感器',
    description: '粒子 / 生物 / 模拟量点位与限值配置',
    icon: 'mdi:access-point',
    iconColor: '#f0a020',
    route: '/sensors/particle'
  },
  {
    id: 4,
    name: '报警中心',
    description: '报警确认、通信与流量异常处理',
    icon: 'mdi:bell-alert-outline',
    iconColor: '#d03050',
    route: '/alarms'
  },
  {
    id: 5,
    name: '数据报表',
    description: '生成审计/数据/趋势报表并导出',
    icon: 'mdi:file-chart-outline',
    iconColor: '#8a2be2',
    route: '/reports/generator'
  },
  {
    id: 6,
    name: '审计轨迹',
    description: '登录、配置、导出等操作可追溯',
    icon: 'mdi:history',
    iconColor: '#36ad6a',
    route: '/audit'
  }
];

const shortcuts = [
  { id: 0, label: '实时信号', icon: 'mdi:monitor-dashboard', iconColor: '#409eff', route: '/sampling/realtime' },
  { id: 1, label: '实时趋势', icon: 'mdi:chart-timeline-variant', iconColor: '#18a058', route: '/rt-trend' },
  { id: 2, label: '运行逻辑', icon: 'mdi:sitemap', iconColor: '#f0a020', route: '/runtime-logic' },
  { id: 3, label: '厂房结构', icon: 'mdi:office-building', iconColor: '#7238d1', route: '/system/facility' },
  { id: 4, label: '统计分析', icon: 'mdi:chart-bell-curve', iconColor: '#d03050', route: '/reports/sda' },
  { id: 5, label: '系统运维', icon: 'mdi:cog-outline', iconColor: '#666', route: '/system/backup' }
];
</script>
