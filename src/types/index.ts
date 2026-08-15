/**
 * Pharmaceutical Net Pro — 业务类型统一出口
 *
 * 用法：
 *   import type { Recipe, PharmaUser, Sensor } from '@/types';
 *   import { mapPharmaRoleToAuth } from '@/types';
 *
 * 与现有全局声明兼容：
 *   - Auth.RoleType / Auth.UserInfo（src/typings/business.d.ts）
 *   - ApiUserManagement.User / ApiRoleManagement.Role（src/typings/api.d.ts）
 */

export * from './common';
export * from './auth';
export * from './recipe';
export * from './sensor';
export * from './alarm';
export * from './report';
export * from './system';
