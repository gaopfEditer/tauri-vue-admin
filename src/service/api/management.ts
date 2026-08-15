import { adapter } from '@/utils';
import { managementRequest, mockRequest } from '../request';
import { adapterOfFetchUserList } from './management.adapter';

/** 获取用户列表 */
export const fetchUserList = async () => {
  const data = await mockRequest.post<ApiUserManagement.User[] | null>('/getAllUserList');
  return adapter(adapterOfFetchUserList, data);
};

/** 获取角色列表 */
export const fetchRoleList = async () => {
  const data = await managementRequest.get<ApiRoleManagement.Role[] | null>('/api/management/role/list');
  return adapter(
    (list: ApiRoleManagement.Role[] | null): RoleManagement.Role[] =>
      (list ?? []).map((item, index) => ({
        ...item,
        index: index + 1,
        key: item.id
      })),
    data
  );
};

/** 新增用户 */
export const createUser = (body: ApiUserManagement.UserUpsert) => {
  return managementRequest.post<{ id: number }>('/api/management/user', body);
};

/** 更新用户 */
export const updateUser = (id: string, body: ApiUserManagement.UserUpsert) => {
  return managementRequest.put<boolean>(`/api/management/user/${id}`, body);
};

/** 删除用户（软删除） */
export const deleteUser = (id: string) => {
  return managementRequest.delete<boolean>(`/api/management/user/${id}`, {});
};
