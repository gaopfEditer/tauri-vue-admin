<template>
  <n-space :vertical="true" :size="16">
    <workbench-header
      :loading="loading"
      :online-sensors="onlineSensors.length"
      :unacked-alarms="unackedAlarms.length"
      :running-samplings="runningSamplings.length"
      :recipes="recipes.length"
    />
    <workbench-main
      :loading="loading"
      :audits="audits"
      :sensor-total="activeSensors.length"
      :category="sensorByCategory"
      :scheduled="scheduledSamplings.length"
      :report-count="reports.length"
    />
  </n-space>
</template>

<script lang="ts" setup>
import { useDashboardOverview } from '../useDashboardOverview';
import { WorkbenchHeader, WorkbenchMain } from './components';

const {
  loading,
  audits,
  recipes,
  reports,
  activeSensors,
  onlineSensors,
  unackedAlarms,
  runningSamplings,
  scheduledSamplings,
  sensorByCategory
} = useDashboardOverview();
</script>
