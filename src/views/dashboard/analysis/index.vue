<template>
  <n-space :vertical="true" :size="16">
    <data-card
      :sensors="onlineSensors.length"
      :unacked-alarms="unackedAlarms.length"
      :running-samplings="runningSamplings.length"
      :reports="reports.length"
    />
    <top-chart
      :loading="loading"
      :unacked-alarms="unackedAlarms.length"
      :online-sensors="onlineSensors.length"
      :trend-days="alarmTrend7d.days"
      :trend-counts="alarmTrend7d.counts"
      :category="sensorByCategory"
    />
    <bottom-part :alarms="alarms" :samplings="samplings" />
  </n-space>
</template>

<script lang="ts" setup>
import { useDashboardOverview } from '../useDashboardOverview';
import { BottomPart, DataCard, TopChart } from './components';

const {
  loading,
  alarms,
  samplings,
  reports,
  onlineSensors,
  unackedAlarms,
  runningSamplings,
  sensorByCategory,
  alarmTrend7d
} = useDashboardOverview();
</script>
