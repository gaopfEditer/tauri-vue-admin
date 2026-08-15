import type {
  AlarmEvent,
  RealTimeTrendPoint,
  RealTimeTrendSession,
  ReportDocument,
  ReportKind,
  ReportQuery,
  ReportSelectMode,
  RuntimeOutputTestPayload,
  RuntimeRule,
  RuntimeTag,
  SdaSession,
  TagCatalogItem
} from '@/types';
import { managementRequest } from '../request';

/** 报警 */
export function fetchAlarmList() {
  return managementRequest.get<AlarmEvent[]>('/api/alarms');
}

export function ackAlarm(id: string, body?: { acknowledgedBy?: string; reason?: string }) {
  return managementRequest.post<boolean>(`/api/alarms/${id}/ack`, body ?? {});
}

export function ackAllAlarms(body?: { acknowledgedBy?: string; reason?: string }) {
  return managementRequest.post<boolean>('/api/alarms/ack-all', body ?? {});
}

/** 实时趋势 */
export function startRtTrend(body: { recipeId: string | number; startedBy?: string; pens?: unknown }) {
  return managementRequest.post<{ id: string; recipeId: string; running: boolean }>('/api/rt-trend/start', {
    ...body,
    recipeId: Number(body.recipeId)
  });
}

export function stopRtTrend(body: { sessionId: string | number }) {
  return managementRequest.post<boolean>('/api/rt-trend/stop', {
    sessionId: Number(body.sessionId)
  });
}

export function fetchRtTrendCurrent() {
  return managementRequest.get<RealTimeTrendSession | null>('/api/rt-trend/current');
}

export function fetchRtTrendPoints() {
  return managementRequest.get<RealTimeTrendPoint[]>('/api/rt-trend/points');
}

/** 运行逻辑 */
export function fetchRuntimeTags() {
  return managementRequest.get<RuntimeTag[]>('/api/runtime-tags');
}

export function fetchRuntimeRules() {
  return managementRequest.get<RuntimeRule[]>('/api/runtime-rules');
}

export function createRuntimeRule(body: Omit<RuntimeRule, 'id' | 'updatedAt'> & { enabled?: boolean }) {
  return managementRequest.post<{ id: string }>('/api/runtime-rules', body);
}

export function updateRuntimeRule(id: string, body: Partial<RuntimeRule>) {
  return managementRequest.put<boolean>(`/api/runtime-rules/${id}`, body);
}

export function deleteRuntimeRule(id: string) {
  return managementRequest.delete<boolean>(`/api/runtime-rules/${id}`, {});
}

export function testRuntimeOutput(ruleId: string, body: RuntimeOutputTestPayload) {
  return managementRequest.post<boolean>(`/api/runtime-rules/${ruleId}/test-output`, {
    tagId: Number(body.tagId),
    value: body.value,
    durationMs: body.durationMs
  });
}

/** 报表 */
export function generateReport(
  body: ReportQuery & { generatedBy?: string; mode?: ReportSelectMode; kind: ReportKind }
) {
  return managementRequest.post<ReportDocument>('/api/reports/generate', body);
}

export function fetchReportList() {
  return managementRequest.get<
    Array<{ id: string; kind: string; mode: string; generatedAt: string; generatedBy: string }>
  >('/api/reports');
}

export function fetchReport(id: string) {
  return managementRequest.get<ReportDocument>(`/api/reports/${id}`);
}

export function exportReport(id: string, format: 'csv' | 'pdf' = 'csv') {
  return managementRequest.get<{ content: string; format: string }>(`/api/reports/${id}/export?format=${format}`);
}

/** Tags Catalog */
export function fetchTagCatalog(tab?: string) {
  const query = tab ? `?tab=${tab}` : '';
  return managementRequest.get<TagCatalogItem[]>(`/api/tags-catalog${query}`);
}

export function createTagCatalog(body: Omit<TagCatalogItem, 'id'>) {
  return managementRequest.post<{ id: string }>('/api/tags-catalog', body);
}

export function updateTagCatalog(id: string, body: Partial<TagCatalogItem>) {
  return managementRequest.put<boolean>(`/api/tags-catalog/${id}`, body);
}

export function deleteTagCatalog(id: string) {
  return managementRequest.delete<boolean>(`/api/tags-catalog/${id}`, {});
}

/** SDA */
export function fetchSdaSessions() {
  return managementRequest.get<SdaSession[]>('/api/sda/sessions');
}

export function fetchSdaSession(id: string) {
  return managementRequest.get<SdaSession>(`/api/sda/sessions/${id}`);
}

export function createSdaSession(body: Partial<SdaSession> & { configName?: string; csvContent?: string }) {
  return managementRequest.post<{ id: string }>('/api/sda/sessions', body);
}

export function updateSdaSession(id: string, body: Partial<SdaSession> & { csvContent?: string }) {
  return managementRequest.put<boolean>(`/api/sda/sessions/${id}`, body);
}

export function analyzeSda(body: { sessionId?: string; csvContent?: string }) {
  return managementRequest.post<{
    columns: string[];
    points: Array<Record<string, string | number>>;
    stats: Record<string, number>;
  }>('/api/sda/analyze', body);
}
