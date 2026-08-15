/**
 * 模块 2：设备与传感器管理
 * 手册：Ch.6 Sensors / Ch.12 Sensors Editor & Limits Editor
 */

import type { EntityId, IsoDateTime } from './common';
import type { SensorAcquireMode } from './recipe';

/** 传感器大类 */
export type SensorCategory = 'particle' | 'biocapt' | 'analog';

/**
 * 运行时状态
 * - Cleaning: 粒子计数器 ON 但不采集、不报警
 * - Pause: 生物采样暂停
 */
export type SensorRuntimeState = 'Idle' | 'Sampling' | 'Cleaning' | 'Pause' | 'Offline' | 'Fault';

/** 电源/运行控制 */
export type SensorPowerAction = 'on' | 'off' | 'cleaning';

/** 传感器 / 采样点 */
export interface Sensor {
  id: EntityId;
  category: SensorCategory;
  /** Sensors Editor 用户自定义 ID */
  customId: string;
  description: string;
  groupId?: EntityId;
  /** 通道标识，如 EBI Channel / B0 Remote */
  channel?: string;
  acquireMode?: SensorAcquireMode;
  runtimeState: SensorRuntimeState;
  powerOn: boolean;
  /** 通信报警 */
  comAlarm: boolean;
  /** 流量/真空报警（particle / biocapt） */
  flowCalcAlarm?: boolean;
  /** 最新值：单通道 number 或多通道 map */
  lastValue?: number | Record<string, number>;
  unit?: string;
  /** active | deleted */
  status?: 'active' | 'deleted' | string;
  deleted?: boolean;
  updatedAt: IsoDateTime;
}

/** 新建设备 */
export interface SensorCreatePayload {
  category: SensorCategory;
  customId: string;
  description?: string;
  groupId?: EntityId | null;
  channel?: string;
  unit?: string;
  acquireMode?: SensorAcquireMode;
}

/** Sensors Editor 可改元数据 */
export interface SensorMetaUpdate {
  customId?: string;
  description?: string;
  groupId?: EntityId | null;
}

/** 电源切换请求 */
export interface SensorPowerPayload {
  action: SensorPowerAction;
}

/** 报警/预警限值 */
export interface SensorLimit {
  id: EntityId;
  sensorId: EntityId;
  dataType: string;
  warningLimit?: number | null;
  alarmLimit?: number | null;
  effectiveFrom: IsoDateTime;
  changedBy: string;
}

/** 限值变更历史（Limits Editor History） */
export interface SensorLimitHistory {
  id: EntityId;
  limitId: EntityId;
  before: Partial<SensorLimit>;
  after: Partial<SensorLimit>;
  signedBy: string;
  signedAt: IsoDateTime;
  reason?: string;
}

/** 传感器组（报表 Select by Group / Tags Catalog） */
export interface SensorGroup {
  id: EntityId;
  name: string;
  sensorIds: EntityId[];
  tags?: string[];
  description?: string;
}

/** 数据类型分组（报表 Data Type Groups） */
export interface DataTypeGroup {
  id: EntityId;
  name: string;
  dataTypes: string[];
  tags?: string[];
}

/** 生产工况分组（多工况阈值） */
export type ProductionStateGroup = 'production' | 'disinfection' | 'static' | 'post_vent' | 'shutdown';

export type AlarmEnableMode = 'unlimited' | 'enabled' | 'disabled';

export interface SensorStateThreshold {
  stateGroup: ProductionStateGroup | string;
  metricKey: string;
  alarmEnable: AlarmEnableMode | string;
  warnHigh?: number | null;
  warnLow?: number | null;
  alarmHigh?: number | null;
  alarmLow?: number | null;
}

/** 粒子设备完整配置（编辑页） */
export interface SensorDeviceConfig {
  sensorId: EntityId;
  facilityNodeId?: EntityId | null;
  deviceName: string;
  deviceCode: string;
  instrumentType?: string | null;
  updateIntervalSec: number;
  slaveAddress?: string | null;
  plcIp?: string | null;
  dataUnit?: string | null;
  dataType: string;
  dataLength: number;
  protocol: string;
  decimalPlaces: number;
  cleanroomClass?: string | null;
  serialNumber?: string | null;
  calibrationDate?: string | null;
  operatingMode?: string | null;
  productionState?: ProductionStateGroup | string;
  flowRate?: number | null;
  thresholds: SensorStateThreshold[];
}

export interface RealtimeRoomNav {
  id: EntityId;
  code: string;
  name: string;
  description?: string | null;
  deviceCount: number;
  alarmLevel: 'normal' | 'warning' | 'alarm' | string;
}

export interface RealtimeMetric {
  key: string;
  label: string;
  value: number;
  displayValue: string;
  status: 'normal' | 'warning' | 'alarm' | string;
  statusLabel: string;
}

export interface RealtimeSignalCard {
  sensorId: EntityId;
  deviceCode: string;
  deviceName: string;
  roomCode?: string | null;
  roomName?: string | null;
  cardStatus: 'normal' | 'warning' | 'alarm' | string;
  metrics: RealtimeMetric[];
  flowRate?: number | null;
  unit?: string | null;
  cleanroomClass?: string | null;
  serialNumber?: string | null;
  calibrationDate?: string | null;
  operatingMode?: string | null;
  productionState?: string | null;
  instrumentType?: string | null;
  slaveAddress?: string | null;
  powerOn: boolean;
  runtimeState: string;
  comAlarm: boolean;
  flowCalcAlarm: boolean;
  decimalPlaces: number;
}
