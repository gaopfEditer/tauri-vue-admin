import { getToken, getUserInfo } from '@/utils/auth';

const AUDIT_API_BASE = import.meta.env.VITE_DESKTOP_API === 'Y' ? 'http://127.0.0.1:8080' : '';

export interface ReportAuditPayload {
  action: string;
  targetType?: string;
  targetId?: string;
  reason?: string;
  detail?: Record<string, unknown>;
}

/**
 * 主动上报审计事件（使用 fetch，避免走 axios 拦截器造成循环）。
 * 失败静默，不影响主流程。
 */
export async function reportAudit(payload: ReportAuditPayload): Promise<void> {
  if (!payload.action?.trim()) return;

  const user = getUserInfo();
  const token = getToken();
  const body = {
    action: payload.action,
    targetType: payload.targetType ?? 'client',
    targetId: payload.targetId,
    performedBy: user.userName || 'anonymous',
    roleCode: user.userRole || 'user',
    reason: payload.reason,
    detail: payload.detail ?? {}
  };

  try {
    await fetch(`${AUDIT_API_BASE}/api/audit-trail`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: token || '',
        'X-Audit-User': user.userName || 'anonymous',
        'X-Audit-Role': user.userRole || 'user'
      },
      body: JSON.stringify(body)
    });
  } catch {
    // ignore
  }
}

/** 打开页面 */
export function reportPageOpen(path: string, title?: string, routeName?: string) {
  // 登录页、重定向页不记
  if (!path || path === '/login' || path.startsWith('/login/')) return;
  if (!getToken()) return;

  void reportAudit({
    action: 'page.open',
    targetType: 'page',
    targetId: path,
    detail: {
      path,
      title: title || '',
      routeName: routeName || ''
    }
  });
}

/** 登出 */
export function reportLogout() {
  void reportAudit({
    action: 'auth.logout',
    targetType: 'auth',
    detail: { at: new Date().toISOString() }
  });
}

/** 导出类操作（浏览器侧下载） */
export function reportExport(kind: string, targetId?: string, detail?: Record<string, unknown>) {
  void reportAudit({
    action: `export.${kind}`,
    targetType: 'export',
    targetId,
    detail
  });
}
