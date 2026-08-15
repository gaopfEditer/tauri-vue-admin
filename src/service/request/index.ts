import { getServiceEnvConfig } from '~/.env-config';
import { createRequest } from './request';

const { url, urlPattern, secondUrl, secondUrlPattern } = getServiceEnvConfig(import.meta.env);

const isHttpProxy = import.meta.env.VITE_HTTP_PROXY === 'Y';
const isDesktopApi = import.meta.env.VITE_DESKTOP_API === 'Y';
const desktopApiBaseURL = 'http://127.0.0.1:8080/mock';

export const request = createRequest({ baseURL: isHttpProxy ? urlPattern : url });

export const secondRequest = createRequest({ baseURL: isHttpProxy ? secondUrlPattern : secondUrl });

export const mockRequest = createRequest({
  baseURL: isDesktopApi ? desktopApiBaseURL : '/mock'
});

/** 管理模块 API（用户/角色/菜单/权限 CRUD） */
export const managementRequest = createRequest({
  baseURL: isDesktopApi ? 'http://127.0.0.1:8080' : ''
});
