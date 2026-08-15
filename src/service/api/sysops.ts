import type {
  AuditTrailEntry,
  BackupConfig,
  BackupJob,
  DisplayLanguageConfig,
  ElectronicSignaturePayload,
  ElectronicSignatureResult,
  PasswordPolicy,
  PharmaUser,
  RestorePayload,
  SamplerCalibrationConfig,
  SetupWizardState,
  SystemCommandPayload,
  UserEventPayload
} from '@/types';
import { managementRequest } from '../request';

/** 密码策略 */
export function fetchPasswordPolicy() {
  return managementRequest.get<PasswordPolicy>('/api/password-policy');
}

export function updatePasswordPolicy(body: PasswordPolicy) {
  return managementRequest.put<boolean>('/api/password-policy', body);
}

/** 电子签名 */
export function submitElectronicSignature(body: ElectronicSignaturePayload) {
  return managementRequest.post<ElectronicSignatureResult>('/api/electronic-signature', body);
}

/** 审计轨迹 */
export function fetchAuditTrail(params?: {
  dateFrom?: string;
  dateTo?: string;
  action?: string;
  performedBy?: string;
  targetType?: string;
  limit?: number;
}) {
  const q = new URLSearchParams();
  if (params?.dateFrom) q.set('dateFrom', params.dateFrom);
  if (params?.dateTo) q.set('dateTo', params.dateTo);
  if (params?.action) q.set('action', params.action);
  if (params?.performedBy) q.set('performedBy', params.performedBy);
  if (params?.targetType) q.set('targetType', params.targetType);
  if (params?.limit) q.set('limit', String(params.limit));
  const qs = q.toString();
  return managementRequest.get<AuditTrailEntry[]>(`/api/audit-trail${qs ? `?${qs}` : ''}`);
}

/** 主动写入一条审计（一般用 utils/reportAudit；此处供业务调用） */
export function createAuditEvent(body: {
  action: string;
  targetType?: string;
  targetId?: string;
  performedBy?: string;
  roleCode?: string;
  reason?: string;
  detail?: Record<string, unknown>;
}) {
  return managementRequest.post<{ id: string }>('/api/audit-trail', body);
}

/** User Event */
export function createUserEvent(body: UserEventPayload & { performedBy?: string }) {
  return managementRequest.post<{ id: string }>('/api/user-events', body);
}

/** 组功能矩阵 */
export function fetchGroupFunctions() {
  return managementRequest.get<Record<string, string[]>>('/api/group-functions');
}

export function fetchPharmaUsers() {
  return managementRequest.get<PharmaUser[]>('/api/pharma-users');
}

/** 备份 */
export function fetchBackupConfig() {
  return managementRequest.get<BackupConfig>('/api/system/backup-config');
}

export function updateBackupConfig(body: BackupConfig) {
  return managementRequest.put<boolean>('/api/system/backup-config', body);
}

export function runBackup(createdBy?: string) {
  return managementRequest.post<BackupJob>('/api/system/backup', { createdBy });
}

export function fetchBackupJobs() {
  return managementRequest.get<BackupJob[]>('/api/system/backup/jobs');
}

export function runRestore(body: RestorePayload & { performedBy?: string }) {
  return managementRequest.post<{ ok: boolean; summary?: Record<string, number> }>('/api/system/restore', body);
}

/** 系统控制 */
export function runSystemCommand(body: SystemCommandPayload & { signedBy?: string; reason?: string }) {
  return managementRequest.post<{ accepted: boolean; action: string; note?: string }>('/api/system/command', body);
}

export function fetchDisplayLanguage() {
  return managementRequest.get<DisplayLanguageConfig>('/api/system/language');
}

export function updateDisplayLanguage(body: DisplayLanguageConfig) {
  return managementRequest.put<boolean>('/api/system/language', body);
}

export function resetSystemBuffer(body?: { performedBy?: string; reason?: string }) {
  return managementRequest.post<{ ok: boolean }>('/api/system/reset-buffer', body ?? {});
}

/** Setup */
export function fetchSetupStatus() {
  return managementRequest.get<SetupWizardState & { initialized?: boolean; dbHealthy?: boolean; restoreFile?: string }>(
    '/api/setup/status'
  );
}

export function initDatabase(body?: { performedBy?: string; mode?: string }) {
  return managementRequest.post<{ ok: boolean; note?: string }>('/api/setup/init-db', body ?? {});
}

export function updateSetupWizard(body: Partial<SetupWizardState>) {
  return managementRequest.put<boolean>('/api/setup/wizard', body);
}

export function fetchCalibrations() {
  return managementRequest.get<
    Array<SamplerCalibrationConfig & { id?: string; updatedBy?: string; updatedAt?: string }>
  >('/api/setup/calibration');
}

export function upsertCalibration(body: SamplerCalibrationConfig & { updatedBy?: string }) {
  return managementRequest.put<{ samplerId: string }>('/api/setup/calibration', body);
}

/** 厂区三级组织 */
export function fetchFacilityTree() {
  return managementRequest.get<import('@/types').FacilityNode[]>('/api/system/facility/tree');
}

export function fetchFacilityList(params?: { parentId?: string; level?: number }) {
  const q = new URLSearchParams();
  if (params?.parentId != null) q.set('parentId', params.parentId);
  if (params?.level != null) q.set('level', String(params.level));
  const qs = q.toString();
  return managementRequest.get<import('@/types').FacilityNode[]>(`/api/system/facility${qs ? `?${qs}` : ''}`);
}

export function createFacility(body: import('@/types').FacilityUpsert) {
  return managementRequest.post<{ id: string }>('/api/system/facility', body);
}

export function updateFacility(id: string, body: import('@/types').FacilityUpsert) {
  return managementRequest.put<boolean>(`/api/system/facility/${id}`, body);
}

export function deleteFacility(id: string) {
  return managementRequest.delete<boolean>(`/api/system/facility/${id}`, {});
}

/** 软件授权 */
export function fetchLicenseStatus() {
  return managementRequest.get<import('@/types').LicenseStatus>('/api/license/status');
}

export function activateLicense(licenseCode: string) {
  return managementRequest.post<import('@/types').LicenseStatus>('/api/license/activate', {
    licenseCode
  });
}

export function issueLicense(body: import('@/types').LicenseIssuePayload) {
  return managementRequest.post<{
    licenseCode: string;
    payload: Record<string, unknown>;
    hint?: string;
  }>('/api/license/issue', body);
}

/** 统一 Debug 日志（系统 / 数据库 / Modbus） */
export function fetchDebugLogs(params?: { limit?: number; level?: string; source?: string }) {
  const q = new URLSearchParams();
  if (params?.limit) q.set('limit', String(params.limit));
  if (params?.level) q.set('level', params.level);
  if (params?.source) q.set('source', params.source);
  const qs = q.toString();
  return managementRequest.get<
    Array<{
      time: string;
      level: string;
      source: string;
      sensorId?: number;
      deviceCode?: string;
      host?: string;
      action: string;
      message: string;
      detail?: Record<string, unknown>;
      elapsedMs?: number;
    }>
  >(`/api/debug/logs${qs ? `?${qs}` : ''}`);
}

/** 设备 Modbus 轮询联调 */
export function fetchDevicePollLogs(params?: { limit?: number; level?: string }) {
  const q = new URLSearchParams();
  if (params?.limit) q.set('limit', String(params.limit));
  if (params?.level) q.set('level', params.level);
  const qs = q.toString();
  return managementRequest.get<
    Array<{
      time: string;
      level: string;
      sensorId?: number;
      deviceCode?: string;
      host?: string;
      action: string;
      message: string;
      detail?: Record<string, unknown>;
      elapsedMs?: number;
    }>
  >(`/api/device-poll/logs${qs ? `?${qs}` : ''}`);
}

export function fetchDevicePollStatus() {
  return managementRequest.get<{
    enabled: boolean;
    cycle: number;
    logFile?: string;
    logCount: number;
  }>('/api/device-poll/status');
}

export function setDevicePollEnabled(enabled: boolean) {
  return managementRequest.post<{ enabled: boolean }>('/api/device-poll/enabled', { enabled });
}

export function testDevicePoll(sensorId: number | string) {
  return managementRequest.post<{ ok: boolean; sensorId: string }>('/api/device-poll/test', {
    sensorId: Number(sensorId)
  });
}
