import type { Router } from 'vue-router';
import { useTitle } from '@vueuse/core';
import { reportPageOpen } from '@/utils';
import { createPermissionGuard } from './permission';

/**
 * 路由守卫函数
 * @param router - 路由实例
 */
export function createRouterGuard(router: Router) {
  router.beforeEach(async (to, from, next) => {
    // 开始 loadingBar
    window.$loadingBar?.start();
    // 页面跳转权限处理
    await createPermissionGuard(to, from, next);
  });
  router.afterEach(to => {
    // 设置document title
    useTitle(to.meta.title);
    // 结束 loadingBar
    window.$loadingBar?.finish();
    // 审计：打开页面
    const title = typeof to.meta.title === 'string' ? to.meta.title : undefined;
    reportPageOpen(to.fullPath || to.path, title, String(to.name || ''));
  });
}
