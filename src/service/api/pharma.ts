import type {
  Recipe,
  RecipeUpsert,
  SamplingTask,
  SamplingSchedulePayload,
  SamplingCustomFieldDef,
  Sensor,
  SensorCategory,
  SensorMetaUpdate,
  SensorPowerAction,
  SensorGroup,
  SensorLimit,
  SensorLimitHistory
} from '@/types';
import { managementRequest } from '../request';

/** 配方列表 */
export function fetchRecipeList() {
  return managementRequest.get<Recipe[]>('/api/recipes');
}

export function fetchRecipe(id: string) {
  return managementRequest.get<Recipe>(`/api/recipes/${id}`);
}

export function createRecipe(body: RecipeUpsert & { createdBy?: string }) {
  return managementRequest.post<{ id: string }>('/api/recipes', body);
}

export function updateRecipe(id: string, body: RecipeUpsert) {
  return managementRequest.put<boolean>(`/api/recipes/${id}`, body);
}

export function deleteRecipe(id: string) {
  return managementRequest.delete<boolean>(`/api/recipes/${id}`, {});
}

/** 采样任务 */
export function fetchSamplingList() {
  return managementRequest.get<SamplingTask[]>('/api/samplings');
}

export function createSampling(body: SamplingSchedulePayload & { createdBy?: string }) {
  return managementRequest.post<{ id: string }>('/api/samplings', body);
}

export function updateSampling(id: string, body: SamplingSchedulePayload) {
  return managementRequest.put<boolean>(`/api/samplings/${id}`, body);
}

export function deleteSampling(id: string) {
  return managementRequest.delete<boolean>(`/api/samplings/${id}`, {});
}

export function abortSampling(id: string, reason?: string) {
  return managementRequest.post<boolean>(`/api/samplings/${id}/abort`, { reason });
}

export function fetchSamplingCustomFields() {
  return managementRequest.get<SamplingCustomFieldDef[]>('/api/samplings/custom-fields');
}

export function saveSamplingCustomFields(fields: SamplingCustomFieldDef[]) {
  return managementRequest.put<boolean>('/api/samplings/custom-fields', fields);
}

/** 传感器 */
export function fetchSensorList(category?: SensorCategory, opts?: { includeDeleted?: boolean; onlyDeleted?: boolean }) {
  const q = new URLSearchParams();
  if (category) q.set('category', category);
  if (opts?.includeDeleted) q.set('includeDeleted', '1');
  if (opts?.onlyDeleted) q.set('onlyDeleted', '1');
  const qs = q.toString();
  return managementRequest.get<Sensor[]>(`/api/sensors${qs ? `?${qs}` : ''}`);
}

export function fetchSensor(id: string) {
  return managementRequest.get<Sensor>(`/api/sensors/${id}`);
}

export function createSensor(body: import('@/types').SensorCreatePayload) {
  return managementRequest.post<{ id: string }>('/api/sensors', body);
}

export function deleteSensor(id: string) {
  return managementRequest.delete<boolean>(`/api/sensors/${id}`, {});
}

export function restoreSensor(id: string) {
  return managementRequest.post<boolean>(`/api/sensors/${id}/restore`, {});
}

export function updateSensorMeta(id: string, body: SensorMetaUpdate) {
  return managementRequest.put<boolean>(`/api/sensors/${id}/meta`, body);
}

export function switchSensorPower(id: string, action: SensorPowerAction) {
  return managementRequest.post<boolean>(`/api/sensors/${id}/power`, { action });
}

export function fetchSensorGroups() {
  return managementRequest.get<SensorGroup[]>('/api/sensor-groups');
}

export function fetchSensorLimits(category?: SensorCategory) {
  const query = category ? `?category=${category}` : '';
  return managementRequest.get<SensorLimit[]>(`/api/sensor-limits${query}`);
}

export function upsertSensorLimit(
  body: Pick<SensorLimit, 'sensorId' | 'dataType' | 'warningLimit' | 'alarmLimit'> & {
    changedBy?: string;
    reason?: string;
  }
) {
  return managementRequest.put<{ id: string }>('/api/sensor-limits', body);
}

export function fetchSensorLimitHistory() {
  return managementRequest.get<SensorLimitHistory[]>('/api/sensor-limits/history');
}

export function fetchSensorDeviceConfig(id: string) {
  return managementRequest.get<import('@/types').SensorDeviceConfig>(`/api/sensors/${id}/device-config`);
}

export function saveSensorDeviceConfig(id: string, body: import('@/types').SensorDeviceConfig) {
  return managementRequest.put<boolean>(`/api/sensors/${id}/device-config`, body);
}

export function fetchRealtimeRooms() {
  return managementRequest.get<import('@/types').RealtimeRoomNav[]>('/api/realtime/rooms');
}

export function fetchRealtimeSignals(roomId: string) {
  return managementRequest.get<import('@/types').RealtimeSignalCard[]>(`/api/realtime/signals?roomId=${roomId}`);
}

export function batchRealtimePower(body: { action: SensorPowerAction; roomIds?: string[]; sensorIds?: string[] }) {
  return managementRequest.post<{ affected: number; action: string }>('/api/realtime/batch-power', body);
}
