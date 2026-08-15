-- =============================================================================
-- Soybean Admin 初始化数据
-- =============================================================================
-- 请先执行 schema.sql 建表，并在客户端选中 soybean_admin 库后再执行本文件
--
-- 命令行:
--   mysql -u root -p soybean_admin < database/mysql/init-data.sql
--
-- 测试账号 (密码 MD5 存储，仅开发用):
--   Soybean / soybean123
--   Super   / super123
--   Admin   / admin123
--   User01  / user01123
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `sys_user_token`;
DELETE FROM `sys_role_permission`;
DELETE FROM `sys_role_menu`;
DELETE FROM `sys_user_role`;
DELETE FROM `sys_permission`;
DELETE FROM `sys_menu`;
DELETE FROM `sys_user`;
DELETE FROM `sys_role`;

ALTER TABLE `sys_role` AUTO_INCREMENT = 1;
ALTER TABLE `sys_user` AUTO_INCREMENT = 1;
ALTER TABLE `sys_user_role` AUTO_INCREMENT = 1;
ALTER TABLE `sys_menu` AUTO_INCREMENT = 1;
ALTER TABLE `sys_permission` AUTO_INCREMENT = 1;
ALTER TABLE `sys_role_menu` AUTO_INCREMENT = 1;
ALTER TABLE `sys_role_permission` AUTO_INCREMENT = 1;
ALTER TABLE `sys_user_token` AUTO_INCREMENT = 1;

INSERT INTO `sys_role` (`id`, `role_code`, `role_name`, `description`, `home_route`, `status`, `sort`) VALUES
(1, 'super', '超级管理员', '拥有全部菜单与权限', 'dashboard_analysis', 1, 1),
(2, 'admin', '管理员', '拥有除 super 专属演示外的菜单', 'dashboard_analysis', 1, 2),
(3, 'user',  '普通用户', '仅基础业务菜单', 'dashboard_analysis', 1, 3);

INSERT INTO `sys_user` (`id`, `user_name`, `password`, `nick_name`, `age`, `gender`, `phone`, `email`, `user_status`, `status`) VALUES
(1, 'Soybean', '7326f9139204e1359eaf7b1df363c791', 'Soybean', 28, '1', '13800000001', 'soybean@example.com', '1', 1),
(2, 'Super',   'f35364bc808b079853de5a1e343e7159', 'Super',   30, '1', '13800000002', 'super@example.com',   '1', 1),
(3, 'Admin',   '0192023a7bbd73250516f069df18b500', 'Admin',   26, '1', '13800000003', 'admin@example.com',   '1', 1),
(4, 'User01',  'd3de66ecd25f8b5c93a480f9543e0783', 'User01',  24, '0', '13800000004', 'user01@example.com',  '1', 1),
(5, '张三',     '7326f9139204e1359eaf7b1df363c791', '张三',     32, '1', '13900001111', 'zhangsan@example.com', '1', 1),
(6, '李四',     '7326f9139204e1359eaf7b1df363c791', '李四',     29, '0', '13900002222', 'lisi@example.com',     '2', 1),
(7, '王五',     '7326f9139204e1359eaf7b1df363c791', '王五',     35, '1', '13900003333', NULL,                   '1', 1);

INSERT INTO `sys_user_role` (`user_id`, `role_id`) VALUES
(1, 1), (2, 1), (3, 2), (4, 3), (5, 2), (6, 3), (7, 3);

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
VALUES
(1,  0, 'dashboard',             '/dashboard',             'basic', NULL, '仪表盘',   'mdi:monitor-dashboard',               NULL,  1, 0, 1, NULL, NULL, NULL, 1, 1),
(2,  1, 'dashboard_analysis',    '/dashboard/analysis',    'self',  NULL, '分析页',   'icon-park-outline:analysis',          NULL,  1, 0, 1, NULL, NULL, NULL, 2, 1),
(3,  1, 'dashboard_workbench',   '/dashboard/workbench',   'self',  NULL, '工作台',   'icon-park-outline:workbench',         NULL,  2, 0, 1, NULL, NULL, NULL, 2, 1),
(90, 0, 'management',            '/management',            'basic', NULL, '系统管理', 'carbon:cloud-service-management',     NULL,  9, 0, 1, NULL, NULL, NULL, 1, 1),
(91, 90, 'management_auth',      '/management/auth',       'self',  NULL, '权限管理', 'ic:baseline-security',                NULL,  1, 0, 1, NULL, NULL, NULL, 2, 1),
(92, 90, 'management_role',      '/management/role',       'self',  NULL, '角色管理', 'carbon:user-role',                    NULL,  2, 0, 1, NULL, NULL, NULL, 2, 1),
(93, 90, 'management_user',      '/management/user',       'self',  NULL, '用户管理', 'ic:round-manage-accounts',            NULL,  3, 0, 1, NULL, NULL, NULL, 2, 1),
(94, 90, 'management_route',     '/management/route',      'self',  NULL, '路由管理', 'material-symbols:route',              NULL,  4, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT INTO `sys_permission` (`id`, `permission_code`, `permission_name`, `menu_id`, `api_path`, `http_method`, `description`, `status`) VALUES
(1,  'management:user:list',   '用户列表', 93, '/api/management/user/list',   'GET',    '查看用户列表', 1),
(2,  'management:user:create', '新增用户', 93, '/api/management/user',        'POST',   '创建用户',     1),
(3,  'management:user:update', '编辑用户', 93, '/api/management/user/{id}',   'PUT',    '更新用户',     1),
(4,  'management:user:delete', '删除用户', 93, '/api/management/user/{id}',   'DELETE', '删除用户',     1),
(5,  'management:role:list',   '角色列表', 92, '/api/management/role/list',   'GET',    '查看角色列表', 1),
(6,  'management:role:create', '新增角色', 92, '/api/management/role',        'POST',   '创建角色',     1),
(7,  'management:role:update', '编辑角色', 92, '/api/management/role/{id}',   'PUT',    '更新角色',     1),
(8,  'management:role:delete', '删除角色', 92, '/api/management/role/{id}',   'DELETE', '删除角色',     1),
(9,  'management:menu:list',   '菜单列表', 94, '/api/management/menu/tree',   'GET',    '查看菜单树',   1),
(10, 'management:menu:create', '新增菜单', 94, '/api/management/menu',        'POST',   '创建菜单',     1),
(11, 'management:menu:update', '编辑菜单', 94, '/api/management/menu/{id}',   'PUT',    '更新菜单',     1),
(12, 'management:menu:delete', '删除菜单', 94, '/api/management/menu/{id}',   'DELETE', '删除菜单',     1),
(13, 'management:auth:list',   '权限列表', 91, '/api/management/permission/list', 'GET', '查看权限列表', 1),
(14, 'management:auth:assign', '分配权限', 91, '/api/management/role/assign-permission', 'POST', '为角色分配权限', 1);

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 1, `id` FROM `sys_menu`;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 2, `id` FROM `sys_menu`;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(3, 1), (3, 2);

INSERT INTO `sys_role_permission` (`role_id`, `permission_id`)
SELECT 1, `id` FROM `sys_permission`;

INSERT INTO `sys_role_permission` (`role_id`, `permission_id`)
SELECT 2, `id` FROM `sys_permission` WHERE `permission_code` <> 'management:auth:assign';

INSERT INTO `sys_role_permission` (`role_id`, `permission_id`) VALUES
(3, 1);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'roles' AS item, COUNT(*) AS total FROM `sys_role`
UNION ALL SELECT 'users', COUNT(*) FROM `sys_user`
UNION ALL SELECT 'menus', COUNT(*) FROM `sys_menu`
UNION ALL SELECT 'permissions', COUNT(*) FROM `sys_permission`;
