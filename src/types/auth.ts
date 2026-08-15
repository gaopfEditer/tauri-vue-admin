/**
 * 模块 5：用户权限、电子签名与审计
 * 手册：Ch.1 Electronic Signature / Ch.11 User Management
 *
 * 兼容说明：
 * - 现有前端路由权限仍使用 Auth.RoleType（super | admin | user）
 * - 手册业务权限使用 PharmaUserGroupRole（Administrator | Supervisor | …）
 * - 通过 mapAuthRoleToPharma / mapPharmaRoleToAuth 双向映射
 */

import type { EntityId, IsoDate, IsoDateTime } from './common';

/**
 * 手册 Default User Groups（Table 11-2）
 * - Nobody: 启动默认游客，只读
 * - Emergency: 灾难恢复本地账号，权限≈Administrator，但不能再创建 Emergency
 */
export type PharmaUserGroupRole = 'Nobody' | 'User' | 'PowerUser' | 'Supervisor' | 'Administrator' | 'Emergency';

/**
 * 与现有 Auth.RoleType / EnumUserRole 对齐的编码
 * - super  → Administrator
 * - admin  → Supervisor
 * - user   → User
 */
export type CompatibleAuthRole = Auth.RoleType;

/** 手册 Table 11-2 功能点（按钮级鉴权） */
export type PharmaGroupFunction =
  | 'RecipeView'
  | 'RecipeCreate'
  | 'RecipeModify'
  | 'RecipeDelete'
  | 'SamplingView'
  | 'SamplingCreate'
  | 'SamplingEdit'
  | 'SamplingAbort'
  | 'SamplingDelete'
  | 'SamplingCustomFields'
  | 'SensorsView'
  | 'SensorsSwitch'
  | 'AlarmView'
  | 'AlarmAcknowledge'
  | 'SamplingReportView'
  | 'SamplingReportPrint'
  | 'ReportGeneratorView'
  | 'ReportGeneratorPrint'
  | 'ReportGeneratorExport'
  | 'SystemView'
  | 'SystemConfigure'
  | 'LimitsView'
  | 'LimitsConfigure'
  | 'UserManagement'
  | 'BackupRestore'
  | 'RTTrend'
  | 'RunTimeLogic';

/**
 * Pharmaceutical Net Pro 用户实体
 * 字段同时覆盖手册 User Management 与现有 ApiUserManagement.User
 */
export interface PharmaUser {
  /** 数据库主键（兼容 UserManagement.User.id） */
  id: EntityId;
  /** 登录用户名（兼容 userName） */
  userName: string;
  /** 手册 User ID（展示/审计用） */
  userIdCode: string;
  /** 手册用户组角色 */
  pharmaRole: PharmaUserGroupRole;
  /**
   * 现有路由权限角色（兼容 Auth.UserInfo.userRole）
   * 由 pharmaRole 映射而来，便于沿用 soybean 动态路由
   */
  userRole: CompatibleAuthRole;
  /** 账号过期日 */
  expiryDate?: IsoDate | null;
  /** 下次登录必须改密 */
  mustChangePassword: boolean;
  /** 是否启用 */
  enabled: boolean;
  /** 灾难恢复本地 Emergency 账号 */
  isLocalEmergency?: boolean;
  /** 兼容现有用户管理页字段 */
  age?: number | null;
  gender?: '0' | '1' | null;
  phone?: string;
  email?: string | null;
  /** 兼容 userStatus: 1启用 2禁用 3冻结 4软删除 */
  userStatus?: '1' | '2' | '3' | '4' | null;
  createdAt?: IsoDateTime;
  updatedAt?: IsoDateTime;
}

/** 创建/更新用户（兼容 ApiUserManagement.UserUpsert） */
export interface PharmaUserUpsert {
  userName: string;
  userIdCode?: string;
  password?: string;
  pharmaRole: PharmaUserGroupRole;
  /** 若未传，后端按 pharmaRole 映射 */
  userRole?: CompatibleAuthRole;
  roleId?: number;
  expiryDate?: IsoDate | null;
  mustChangePassword?: boolean;
  enabled?: boolean;
  age?: number | null;
  gender?: '0' | '1' | null;
  phone?: string;
  email?: string | null;
  userStatus?: '1' | '2' | '3' | '4' | null;
}

/** 密码与会话策略（User Management 配置区） */
export interface PasswordPolicy {
  /** 密码过期天数 */
  expireDays: number;
  /** 最小密码长度 */
  minLength: number;
  /** 记住不可复用的历史密码个数 */
  rememberOldCount: number;
  /** 空闲自动登出秒数 */
  autoLogoffSeconds: number;
  /** 是否启用电子签名 */
  electronicSignatureEnabled: boolean;
}

/** 电子签名动作码（写操作统一拦截） */
export type ElectronicSignatureAction =
  | 'recipe.save'
  | 'recipe.delete'
  | 'sampling.schedule'
  | 'sampling.abort'
  | 'sampling.update'
  | 'sampling.delete'
  | 'sensor.power'
  | 'sensor.meta'
  | 'limit.save'
  | 'alarm.ack'
  | 'alarm.ackAll'
  | 'rtTrend.start'
  | 'rtTrend.stop'
  | 'runtimeRule.save'
  | 'user.create'
  | 'user.update'
  | 'user.delete'
  | 'system.backup'
  | 'system.restore'
  | 'system.command'
  | 'userEvent.custom'
  | string;

/** 电子签名请求体 */
export interface ElectronicSignaturePayload {
  userName: string;
  password: string;
  action: ElectronicSignatureAction;
  targetType: string;
  targetId?: EntityId;
  reason?: string;
}

/** 电子签名校验结果 */
export interface ElectronicSignatureResult {
  signatureId: EntityId;
  performedBy: string;
  role: PharmaUserGroupRole;
  signedAt: IsoDateTime;
}

/** 审计轨迹（Audit Trail / Report） */
export interface AuditTrailEntry {
  id: EntityId;
  eventTime: IsoDateTime;
  action: ElectronicSignatureAction | string;
  targetType: string;
  targetId?: EntityId;
  performedBy: string;
  role: PharmaUserGroupRole;
  reason?: string;
  detail?: Record<string, unknown>;
}

/** 用户自定义事件（主界面 User Event） */
export interface UserEventPayload {
  message: string;
  reason?: string;
}

/** 兼容现有登录会话中的用户信息 */
export interface PharmaSessionUser extends Auth.UserInfo {
  pharmaRole: PharmaUserGroupRole;
  userIdCode?: string;
  mustChangePassword?: boolean;
}

/** Auth.RoleType → 手册角色 */
export function mapAuthRoleToPharma(role: CompatibleAuthRole): PharmaUserGroupRole {
  switch (role) {
    case 'super':
      return 'Administrator';
    case 'admin':
      return 'Supervisor';
    case 'user':
    default:
      return 'User';
  }
}

/** 手册角色 → Auth.RoleType（路由权限） */
export function mapPharmaRoleToAuth(role: PharmaUserGroupRole): CompatibleAuthRole {
  const map: Record<PharmaUserGroupRole, CompatibleAuthRole> = {
    Nobody: 'user',
    User: 'user',
    PowerUser: 'user',
    Supervisor: 'admin',
    Administrator: 'super',
    Emergency: 'super'
  };
  return map[role];
}

/** 将现有管理端用户行适配为 PharmaUser */
export function adaptUserManagementToPharma(
  user: ApiUserManagement.User,
  pharmaRole: PharmaUserGroupRole = 'User'
): PharmaUser {
  return {
    id: user.id,
    userName: user.userName ?? '',
    userIdCode: user.id,
    pharmaRole,
    userRole: mapPharmaRoleToAuth(pharmaRole),
    mustChangePassword: false,
    enabled: user.userStatus === '1' || user.userStatus == null,
    age: user.age,
    gender: user.gender,
    phone: user.phone,
    email: user.email,
    userStatus: user.userStatus
  };
}

/** roleCode（库表）与手册角色互转 */
export function roleCodeToPharma(roleCode: string): PharmaUserGroupRole {
  const map: Record<string, PharmaUserGroupRole> = {
    super: 'Administrator',
    admin: 'Supervisor',
    user: 'User',
    nobody: 'Nobody',
    power_user: 'PowerUser',
    poweruser: 'PowerUser',
    supervisor: 'Supervisor',
    administrator: 'Administrator',
    emergency: 'Emergency'
  };
  return map[roleCode.toLowerCase()] ?? 'User';
}

export function pharmaToRoleCode(role: PharmaUserGroupRole): string {
  const map: Record<PharmaUserGroupRole, string> = {
    Nobody: 'nobody',
    User: 'user',
    PowerUser: 'power_user',
    Supervisor: 'admin',
    Administrator: 'super',
    Emergency: 'emergency'
  };
  return map[role];
}
