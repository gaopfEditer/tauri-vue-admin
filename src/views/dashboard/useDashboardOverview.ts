import { computed, onMounted, ref } from 'vue';
import {
  fetchAlarmList,
  fetchAuditTrail,
  fetchRecipeList,
  fetchReportList,
  fetchSamplingList,
  fetchSensorList
} from '@/service';
import type { AlarmEvent, AuditTrailEntry, Recipe, SamplingTask, Sensor } from '@/types';

export function useDashboardOverview() {
  const loading = ref(false);
  const sensors = ref<Sensor[]>([]);
  const alarms = ref<AlarmEvent[]>([]);
  const samplings = ref<SamplingTask[]>([]);
  const recipes = ref<Recipe[]>([]);
  const reports = ref<Array<{ id: string }>>([]);
  const audits = ref<AuditTrailEntry[]>([]);

  const activeSensors = computed(() => sensors.value.filter(s => !s.deleted && s.status !== 'deleted'));
  const onlineSensors = computed(() => activeSensors.value.filter(s => s.powerOn && s.runtimeState !== 'Offline'));
  const unackedAlarms = computed(() => alarms.value.filter(a => !a.acknowledged));
  const runningSamplings = computed(() => samplings.value.filter(s => s.status === 'running'));
  const scheduledSamplings = computed(() => samplings.value.filter(s => s.status === 'scheduled'));

  const sensorByCategory = computed(() => {
    const map = { particle: 0, biocapt: 0, analog: 0 };
    for (const s of activeSensors.value) {
      if (s.category in map) map[s.category as keyof typeof map] += 1;
    }
    return map;
  });

  const alarmBySeverity = computed(() => {
    const map = { warning: 0, alarm: 0, communication: 0, flow: 0 };
    for (const a of unackedAlarms.value) {
      if (a.severity in map) map[a.severity as keyof typeof map] += 1;
      else map.alarm += 1;
    }
    return map;
  });

  /** 近 7 天每日报警数（用于折线） */
  const alarmTrend7d = computed(() => {
    const days: string[] = [];
    const counts: number[] = [];
    const now = new Date();
    for (let i = 6; i >= 0; i -= 1) {
      const d = new Date(now);
      d.setDate(now.getDate() - i);
      const key = `${d.getMonth() + 1}/${d.getDate()}`;
      days.push(key);
      counts.push(0);
    }
    const start = new Date(now);
    start.setHours(0, 0, 0, 0);
    start.setDate(start.getDate() - 6);

    for (const a of alarms.value) {
      const t = new Date(a.raisedAt.replace('T', ' '));
      if (Number.isNaN(t.getTime()) || t < start) continue;
      const idx = Math.floor((t.getTime() - start.getTime()) / 86400000);
      if (idx >= 0 && idx < 7) counts[idx] += 1;
    }
    return { days, counts };
  });

  async function load() {
    loading.value = true;
    try {
      const [s, a, samp, r, rep, aud] = await Promise.all([
        fetchSensorList(),
        fetchAlarmList(),
        fetchSamplingList(),
        fetchRecipeList(),
        fetchReportList(),
        fetchAuditTrail({ limit: 30 })
      ]);
      if (s.data) sensors.value = s.data;
      if (a.data) alarms.value = a.data;
      if (samp.data) samplings.value = samp.data;
      if (r.data) recipes.value = r.data;
      if (rep.data) reports.value = rep.data;
      if (aud.data) audits.value = aud.data;
    } finally {
      loading.value = false;
    }
  }

  onMounted(load);

  return {
    loading,
    sensors,
    alarms,
    samplings,
    recipes,
    reports,
    audits,
    activeSensors,
    onlineSensors,
    unackedAlarms,
    runningSamplings,
    scheduledSamplings,
    sensorByCategory,
    alarmBySeverity,
    alarmTrend7d,
    load
  };
}
