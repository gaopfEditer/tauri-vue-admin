import type { NavigationGuardNext, RouteLocationNormalized } from 'vue-router';
import { routeName } from '@/router';
import { fetchLicenseStatus } from '@/service';
import { useAuthStore } from '@/store';
import { exeStrategyActions, getToken } from '@/utils';
import { createDynamicRouteGuard } from './dynamic';

/** 处理路由页面的权限 */
export async function createPermissionGuard(
  to: RouteLocationNormalized,
  from: RouteLocationNormalized,
  next: NavigationGuardNext
) {
  // 授权锁定页允许直达；其余页面校验 License
  const isLicensePage = to.name === routeName('license');
  if (!isLicensePage) {
    try {
      const { data } = await fetchLicenseStatus();
      if (data && !data.valid) {
        next({ name: routeName('license'), replace: true });
        return;
      }
    } catch {
      // 后端未就绪时不阻断
    }
  }

  // 动态路由
  const permission = await createDynamicRouteGuard(to, from, next);
  if (!permission) return;

  // 外链路由, 从新标签打开，返回上一个路由
  if (to.meta.href) {
    window.open(to.meta.href);
    next({ path: from.fullPath, replace: true, query: from.query });
    return;
  }

  const auth = useAuthStore();
  const isLogin = Boolean(getToken());
  const permissions = to.meta.permissions || [];
  const needLogin = Boolean(to.meta?.requiresAuth) || Boolean(permissions.length);
  const hasPermission = !permissions.length || permissions.includes(auth.userInfo.userRole);

  const actions: Common.StrategyAction[] = [
    [
      isLogin && to.name === routeName('login'),
      () => {
        next({ name: routeName('root') });
      }
    ],
    [
      !needLogin,
      () => {
        next();
      }
    ],
    [
      !isLogin && needLogin,
      () => {
        const redirect = to.fullPath;
        next({ name: routeName('login'), query: { redirect } });
      }
    ],
    [
      isLogin && needLogin && hasPermission,
      () => {
        next();
      }
    ],
    [
      isLogin && needLogin && !hasPermission,
      () => {
        next({ name: routeName('no-permission') });
      }
    ]
  ];

  exeStrategyActions(actions);
}
