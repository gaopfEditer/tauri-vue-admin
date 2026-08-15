/**
 * 模块 6：系统运维与数据库初始化
 * 手册：Ch.12 System Tasks / Ch.13 Aventino Setup Options
 */

import type { EntityId, IsoDateTime, TimeOfDay } from './common';

/** 日备配置 */
export interface BackupConfig {
  enabled: boolean;
  dailyAt: TimeOfDay;
  targetPath: string;
  retainDays: number;
}

export type BackupJobType = 'manual' | 'daily';
export type BackupJobStatus = 'pending' | 'running' | 'success' | 'failed';

/** 备份任务 */
export interface BackupJob {
  id: EntityId;
  type: BackupJobType;
  status: BackupJobStatus;
  filePath?: string;
  startedAt: IsoDateTime;
  finishedAt?: IsoDateTime;
  message?: string;
  createdBy?: string;
}

/** 恢复请求 */
export interface RestorePayload {
  backupFilePath: string;
  /** FacilityPro 专用恢复 */
  facilityPro?: boolean;
}

/** Aventino Setup 向导模式 */
export type SetupWizardMode =
  | 'newInstallation'
  | 'upgrade'
  | 'restoreFromBackup'
  | 'modifyConfiguration'
  | 'restoreFacilityPro';

/** 数据库连接（与 .env DB_* 对齐） */
export interface DbConnectionConfig {
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
}

/** 采样器校准配置（安装向导子步） */
export interface SamplerCalibrationConfig {
  samplerId: EntityId;
  parameters: Record<string, number | string>;
}

/** 安装向导状态 */
export interface SetupWizardState {
  mode: SetupWizardMode;
  step: number;
  dbConnection: DbConnectionConfig;
  restoreFile?: string;
  calibration?: SamplerCalibrationConfig;
  completed?: boolean;
}

/** 系统控制命令 */
export type SystemCommandAction = 'closeApp' | 'restart' | 'shutdown' | 'shutdownDisconnectedClient' | 'resetBuffer';

export interface SystemCommandPayload {
  action: SystemCommandAction;
}

/** 显示语言 */
export interface DisplayLanguageConfig {
  locale: string;
  /** 如 en-US / zh-CN / it-IT */
  displayName: string;
}

/** 厂区组织层级：1厂房/园区 2区域/洁净区 3房间 */
export type FacilityLevel = 1 | 2 | 3;

export interface FacilityNode {
  id: string;
  parentId?: string | null;
  level: FacilityLevel;
  levelLabel?: string;
  code: string;
  name: string;
  description?: string | null;
  sortOrder: number;
  status: boolean;
  children?: FacilityNode[];
}

export interface FacilityUpsert {
  parentId?: string | null;
  level: FacilityLevel;
  code?: string;
  name: string;
  description?: string | null;
  sortOrder?: number;
  status?: boolean;
}

/** 软件授权（时间锁 + 设备 MAC 锁） */
export interface LicenseStatus {
  valid: boolean;
  reason: string;
  reasonCode: string;
  customer?: string | null;
  validFrom?: string | null;
  validUntil?: string | null;
  deviceLock: boolean;
  boundMac?: string | null;
  currentMac: string;
  daysLeft?: number | null;
  licenseId?: string | null;
}

export interface LicenseIssuePayload {
  customer: string;
  validFrom: string;
  validUntil: string;
  deviceLock?: boolean;
  masterKey: string;
}
