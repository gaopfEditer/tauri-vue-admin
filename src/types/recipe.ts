/**
 * 模块 1：配方与采样管理
 * 手册：Ch.4 Recipes / Ch.5 Sampling
 */

import type { EntityId, EnableStatus, IsoDateTime } from './common';

/**
 * 采样运行模式（Sampling Mode）
 * 手册示例：Operational / At Rest / Normal Operation / Custom
 */
export type SamplingMode = 'Operational' | 'AtRest' | 'NormalOperation' | 'Custom';

/**
 * 传感器采集模式（Volume / Time / Continuous）
 * 手册 Ch.6 Particle Sensors Values
 */
export type SensorAcquireMode = 'Volume' | 'Time' | 'Continuous';

/** 配方状态 */
export type RecipeStatus = Extract<EnableStatus, 'active' | 'deleted'>;

/** 趋势笔 / 通道显示配置（Recipes & Real Time Trend） */
export interface RecipePen {
  id: EntityId;
  sensorId: EntityId;
  /** 数据类型，如粒子通道、流量、温度等 */
  dataType: string;
  color: string;
  enabled: boolean;
  label?: string;
}

/** 配方（Recipe） */
export interface Recipe {
  id: EntityId;
  name: string;
  description?: string;
  /** 关联传感器组 */
  sensorGroupIds: EntityId[];
  pens: RecipePen[];
  createdBy: string;
  updatedAt: IsoDateTime;
  createdAt?: IsoDateTime;
  status: RecipeStatus;
}

/** 创建/更新配方 */
export interface RecipeUpsert {
  name: string;
  description?: string;
  sensorGroupIds: Array<EntityId | number>;
  pens: Omit<RecipePen, 'id'>[] | RecipePen[] | unknown;
}

/** 采样任务状态 */
export type SamplingTaskStatus = 'scheduled' | 'running' | 'aborted' | 'finished' | 'failed';

/** 已调度/进行中的采样任务 */
export interface SamplingTask {
  id: EntityId;
  recipeId: EntityId;
  recipeName: string;
  /** 计划开始时间 */
  scheduledAt: IsoDateTime;
  samplingMode: SamplingMode;
  /** 自定义字段键值（Batch Number 等） */
  customFields: Record<string, string>;
  inputNotes?: string;
  status: SamplingTaskStatus;
  startedAt?: IsoDateTime;
  finishedAt?: IsoDateTime;
  createdBy: string;
  abortReason?: string;
}

/** 调度/编辑采样 */
export interface SamplingSchedulePayload {
  recipeId: EntityId;
  scheduledAt: IsoDateTime;
  samplingMode: SamplingMode;
  customFields?: Record<string, string>;
  inputNotes?: string;
}

/** Sampling Editor 自定义字段定义（Admin Configuration） */
export interface SamplingCustomFieldDef {
  /** 内部字段名 */
  key: string;
  /** 显示名，如 Batch Number */
  displayName: string;
  /** 是否在 Sampling editor 显示 */
  enabled: boolean;
  /** 是否允许用户编辑 */
  editable: boolean;
  /** 下拉默认选项（逗号分隔配置解析后） */
  defaultEntries?: string[];
  sortOrder?: number;
}
