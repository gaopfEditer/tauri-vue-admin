/**
 * 模块 3：报警、实时趋势与运行逻辑
 * 手册：Ch.7 Alarms / Ch.9 Real Time Trend / Ch.10 Run Time Logic
 */

import type { EntityId, IsoDateTime } from './common';
import type { RecipePen } from './recipe';

/** 报警严重级别 */
export type AlarmSeverity = 'warning' | 'alarm' | 'communication' | 'flow';

/** 报警事件 */
export interface AlarmEvent {
  id: EntityId;
  sensorId: EntityId;
  sensorName: string;
  severity: AlarmSeverity;
  message: string;
  raisedAt: IsoDateTime;
  acknowledged: boolean;
  acknowledgedBy?: string;
  acknowledgedAt?: IsoDateTime;
  samplingId?: EntityId;
  dataType?: string;
  value?: number;
  limitValue?: number;
}

/** 确认报警 */
export interface AlarmAckPayload {
  alarmIds?: EntityId[];
  /** true = Ack All */
  ackAll?: boolean;
  reason?: string;
}

/** 实时趋势会话 */
export interface RealTimeTrendSession {
  id: EntityId;
  recipeId: EntityId;
  recipeName?: string;
  running: boolean;
  pens: RecipePen[];
  startedAt?: IsoDateTime;
  stoppedAt?: IsoDateTime;
  startedBy?: string;
}

/** 实时趋势数据点（WS/轮询推送） */
export interface RealTimeTrendPoint {
  sensorId: EntityId;
  dataType: string;
  timestamp: IsoDateTime;
  value: number;
}

/** Run Time Logic Tag 文件夹（Available Tags pane） */
export type RuntimeTagFolder =
  | 'AnalogInput'
  | 'BioCapt'
  | 'DigitalInput'
  | 'DigitalOutput'
  | 'VirtualDX'
  | 'IOModule'
  | 'ParticleCounter'
  | 'PumpsGroup'
  | 'PneumaticModule'
  | 'System';

export type RuntimeTagDataType = 'bool' | 'number' | 'string';

/** 可用 Tag */
export interface RuntimeTag {
  id: EntityId;
  folder: RuntimeTagFolder;
  name: string;
  dataType: RuntimeTagDataType;
  value?: boolean | number | string;
  path?: string;
}

/** 规则类型（手册 Rule Type，保留字符串扩展） */
export type RuntimeRuleType = 'condition' | 'towerLight' | 'samplingOnInput' | 'child' | string;

export type RuntimeConditionOperator = 'eq' | 'ne' | 'gt' | 'lt' | 'gte' | 'lte';

export interface RuntimeCondition {
  tagId: EntityId;
  operator: RuntimeConditionOperator;
  value: boolean | number | string;
}

export type RuntimeActionType = 'setDO' | 'startSampling' | 'stopSampling' | 'towerLight' | 'raiseAlarm';

export interface RuntimeAction {
  type: RuntimeActionType;
  targetId: EntityId;
  value?: boolean | number | string;
}

/** 运行逻辑规则（含父子规则、灯塔、Sampling on Input） */
export interface RuntimeRule {
  id: EntityId;
  name: string;
  ruleType: RuntimeRuleType;
  parentId?: EntityId;
  conditions: RuntimeCondition[];
  actions: RuntimeAction[];
  enabled: boolean;
  /** Sampling on Input 关联配方 */
  recipeId?: EntityId;
  updatedAt?: IsoDateTime;
}

/** 测试输出激活 */
export interface RuntimeOutputTestPayload {
  tagId: EntityId;
  value: boolean | number | string;
  durationMs?: number;
}
