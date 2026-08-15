/** 用户相关模块 */
declare namespace Auth {
  /**
   * 用户角色类型(前端静态路由用角色类型进行路由权限的控制)
   * - super: 超级管理员(该权限具有所有路由数据)
   * - admin: 管理员
   * - user: 用户
   * - custom: 自定义角色
   */
  type RoleType = keyof typeof import('@/enum').EnumUserRole;

  /** 用户信息 */
  interface UserInfo {
    /** 用户id */
    userId: string;
    /** 用户名 */
    userName: string;
    /** 用户角色类型 */
    userRole: RoleType;
  }
}

declare namespace UserManagement {
  interface User extends ApiUserManagement.User {
    /** 序号 */
    index: number;
    /** 表格的key（id） */
    key: string;
  }

  /**
   * 用户性别
   * - 0: 女
   * - 1: 男
   */
  type GenderKey = NonNullable<User['gender']>;

  /**
   * 用户状态
   * - 1: 启用
   * - 2: 禁用
   * - 3: 冻结
   * - 4: 软删除
   */
  type UserStatusKey = NonNullable<User['userStatus']>;
}

declare namespace RoleManagement {
  interface Role extends ApiRoleManagement.Role {
    index: number;
    key: number;
  }
}

/**
 * Pharmaceutical Net Pro 业务类型入口说明
 * 具体 interface 定义在 src/types/（可 import）：
 *   import type { PharmaUser, Recipe, Sensor } from '@/types';
 *   import { mapPharmaRoleToAuth } from '@/types';
 *
 * 角色映射：
 *   Auth.RoleType.super  ↔ PharmaUserGroupRole.Administrator
 *   Auth.RoleType.admin  ↔ PharmaUserGroupRole.Supervisor
 *   Auth.RoleType.user   ↔ PharmaUserGroupRole.User | PowerUser | Nobody
 */
