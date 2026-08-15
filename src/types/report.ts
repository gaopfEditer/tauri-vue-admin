/**
 * 模块 4：数据报表与统计分析
 * 手册：Ch.8 Reports / Appendix A Statistical Data Analyzer
 */

import type { DateTimeRange, EntityId, IsoDateTime } from './common';
import type { RecipePen } from './recipe';

/** 报告种类 */
export type ReportKind = 'audit' | 'data' | 'trend' | 'sampling';

/** 选择模式：按组 / 按采样 */
export type ReportSelectMode = 'byGroup' | 'bySampling';

/** 导出格式 */
export type ReportExportFormat = 'pdf' | 'csv';

/** Trend 聚合计算参数 */
export interface TrendCalcParams {
  aggregation: 'avg' | 'min' | 'max' | 'sum';
  bucketSize: string;
}

/** 报表生成查询 */
export interface ReportQuery extends Partial<DateTimeRange> {
  kind: ReportKind;
  mode: ReportSelectMode;
  dateFrom: IsoDateTime;
  dateTo: IsoDateTime;
  interval?: string;
  groupIds?: EntityId[];
  recipeId?: EntityId;
  samplingIds?: EntityId[];
  sensorIds?: EntityId[];
  dataTypeGroupIds?: EntityId[];
  trendCalc?: TrendCalcParams;
}

export interface ReportHeader {
  title: string;
  facility?: string;
  dateRange: string;
  filtersSummary: string;
}

export interface ReportBodySection {
  title: string;
  columns: string[];
  rows: Array<Record<string, string | number | null>>;
}

export interface ReportFooter {
  pageNote?: string;
  signedInfo?: string;
}

/** 已生成的报告文档 */
export interface ReportDocument {
  id: EntityId;
  query: ReportQuery;
  header: ReportHeader;
  body: ReportBodySection[];
  summary?: Record<string, number | string>;
  footer: ReportFooter;
  generatedAt: IsoDateTime;
  generatedBy: string;
}

/** Tags Catalog 页签 */
export type TagCatalogTab =
  | 'sensorType'
  | 'sensorsAlarms'
  | 'sensors'
  | 'dataTypes'
  | 'sensorGroups'
  | 'dataTypeGroups'
  | 'recipes'
  | 'tagsControl';

/** Tags Catalog 条目（报表可选标签 / 笔颜色等） */
export interface TagCatalogItem {
  id: EntityId;
  tab: TagCatalogTab;
  name: string;
  color?: string;
  linkedIds?: EntityId[];
  enabled?: boolean;
}

/** History Automation 配置 */
export interface HistoryAutomationConfig {
  enabled: boolean;
  /** 完成后自动导出路径 */
  exportPath: string;
  formats: ReportExportFormat[];
  checkIntervalSeconds: number;
}

/** Statistical Data Analyzer 显示配置 */
export interface SdaDisplayConfig {
  xAxis?: { min?: number; max?: number; label?: string };
  yAxis?: { min?: number; max?: number; label?: string };
  zoom?: { x?: [number, number]; y?: [number, number] };
  interpolation?: 'none' | 'linear' | 'step';
  showMarkers?: boolean;
  [key: string]: unknown;
}

/** SDA 会话 */
export interface SdaSession {
  id: EntityId;
  csvFiles: string[];
  displayConfig: SdaDisplayConfig;
  virtualPens: RecipePen[];
  markersEnabled: boolean;
  configName?: string;
}
