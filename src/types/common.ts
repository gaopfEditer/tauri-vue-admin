/**
 * 跨模块公共基础类型
 * Pharmaceutical Net Pro — 与手册及后端接口对齐的共享原语
 */

/** 后端实体主键（字符串化，兼容 BIGINT / UUID） */
export type EntityId = string;

/** ISO 8601 时间字符串，如 2026-07-20T14:30:00+08:00 */
export type IsoDateTime = string;

/** 日期字符串 YYYY-MM-DD */
export type IsoDate = string;

/** 时间字符串 HH:mm 或 HH:mm:ss */
export type TimeOfDay = string;

/** 通用启用状态 */
export type EnableStatus = 'active' | 'disabled' | 'deleted';

/** 通用操作结果（写接口） */
export interface MutationResult<T = boolean> {
  success: boolean;
  data?: T;
  message?: string;
}

/** 分页查询 */
export interface PageQuery {
  page: number;
  pageSize: number;
  keyword?: string;
}

/** 分页结果 */
export interface PageResult<T> {
  list: T[];
  total: number;
  page: number;
  pageSize: number;
}

/** 时间范围过滤 */
export interface DateTimeRange {
  dateFrom: IsoDateTime;
  dateTo: IsoDateTime;
}
