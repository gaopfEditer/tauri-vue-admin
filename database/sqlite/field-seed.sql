-- SQLite field seed (auto-built)
PRAGMA foreign_keys = OFF;
INSERT INTO sys_role (id, role_code, role_name, description, home_route, status, sort) VALUES
(1, 'super', '超级管理员', '拥有全部菜单与权限', 'dashboard_analysis', 1, 1),
(2, 'admin', '管理员', '拥有除 super 专属演示外的菜单', 'dashboard_analysis', 1, 2),
(3, 'user',  '普通用户', '仅基础业务菜单', 'dashboard_analysis', 1, 3);
INSERT INTO sys_user (id, user_name, password, nick_name, age, gender, phone, email, user_status, status) VALUES
(1, 'Soybean', '7326f9139204e1359eaf7b1df363c791', 'Soybean', 28, '1', '13800000001', 'soybean@example.com', '1', 1),
(2, 'Super',   'f35364bc808b079853de5a1e343e7159', 'Super',   30, '1', '13800000002', 'super@example.com',   '1', 1),
(3, 'Admin',   '0192023a7bbd73250516f069df18b500', 'Admin',   26, '1', '13800000003', 'admin@example.com',   '1', 1),
(4, 'User01',  'd3de66ecd25f8b5c93a480f9543e0783', 'User01',  24, '0', '13800000004', 'user01@example.com',  '1', 1),
(5, '张三',     '7326f9139204e1359eaf7b1df363c791', '张三',     32, '1', '13900001111', 'zhangsan@example.com', '1', 1),
(6, '李四',     '7326f9139204e1359eaf7b1df363c791', '李四',     29, '0', '13900002222', 'lisi@example.com',     '2', 1),
(7, '王五',     '7326f9139204e1359eaf7b1df363c791', '王五',     35, '1', '13900003333', NULL,                   '1', 1);
INSERT INTO sys_user_role (user_id, role_id) VALUES
(1, 1), (2, 1), (3, 2), (4, 3), (5, 2), (6, 3), (7, 3);
INSERT INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
(1,  0, 'dashboard',             '/dashboard',             'basic', NULL, '仪表盘',   'mdi:monitor-dashboard',               NULL,  1, 0, 1, NULL, NULL, NULL, 1, 1),
(2,  1, 'dashboard_analysis',    '/dashboard/analysis',    'self',  NULL, '分析页',   'icon-park-outline:analysis',          NULL,  1, 0, 1, NULL, NULL, NULL, 2, 1),
(3,  1, 'dashboard_workbench',   '/dashboard/workbench',   'self',  NULL, '工作台',   'icon-park-outline:workbench',         NULL,  2, 0, 1, NULL, NULL, NULL, 2, 1),
(90, 0, 'management',            '/management',            'basic', NULL, '系统管理', 'carbon:cloud-service-management',     NULL,  9, 0, 1, NULL, NULL, NULL, 1, 1),
(91, 90, 'management_auth',      '/management/auth',       'self',  NULL, '权限管理', 'ic:baseline-security',                NULL,  1, 0, 1, NULL, NULL, NULL, 2, 1),
(92, 90, 'management_role',      '/management/role',       'self',  NULL, '角色管理', 'carbon:user-role',                    NULL,  2, 0, 1, NULL, NULL, NULL, 2, 1),
(93, 90, 'management_user',      '/management/user',       'self',  NULL, '用户管理', 'ic:round-manage-accounts',            NULL,  3, 0, 1, NULL, NULL, NULL, 2, 1),
(94, 90, 'management_route',     '/management/route',      'self',  NULL, '路由管理', 'material-symbols:route',              NULL,  4, 0, 1, NULL, NULL, NULL, 2, 1);
INSERT INTO sys_permission (id, permission_code, permission_name, menu_id, api_path, http_method, description, status) VALUES
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
INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 1, id FROM sys_menu;
INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 2, id FROM sys_menu;
INSERT INTO sys_role_menu (role_id, menu_id) VALUES
(3, 1), (3, 2);
INSERT INTO sys_role_permission (role_id, permission_id)
SELECT 1, id FROM sys_permission;
INSERT INTO sys_role_permission (role_id, permission_id)
SELECT 2, id FROM sys_permission WHERE permission_code <> 'management:auth:assign';
INSERT INTO sys_role_permission (role_id, permission_id) VALUES
(3, 1);
-- from database/mysql/pharma-seed.sql
INSERT OR IGNORE INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
(100, 0,   'recipe',                    '/recipe',                    'basic', NULL, '配方管理',   'mdi:flask-outline',                 NULL, 3, 0, 1, NULL, NULL, NULL, 1, 1),
(101, 100, 'recipe_list',               '/recipe/list',               'self',  NULL, '配方编辑器', 'mdi:playlist-edit',                 NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(110, 0,   'sampling',                  '/sampling',                  'basic', NULL, '采样管理',   'mdi:timer-outline',                 NULL, 4, 0, 1, NULL, NULL, NULL, 1, 1),
(111, 110, 'sampling_list',             '/sampling/list',             'self',  NULL, '采样编辑器', 'mdi:calendar-clock',                NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(112, 110, 'sampling_custom-fields',    '/sampling/custom-fields',    'self',  NULL, '自定义字段', 'mdi:form-textbox',                  NULL, 2, 0, 1, NULL, NULL, NULL, 2, 1),
(120, 0,   'sensors',                   '/sensors',                   'basic', NULL, '设备传感器', 'mdi:access-point',                  NULL, 5, 0, 1, NULL, NULL, NULL, 1, 1),
(121, 120, 'sensors_particle',          '/sensors/particle',          'self',  NULL, '粒子传感器', 'mdi:circle-opacity',                NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(122, 120, 'sensors_biocapt',           '/sensors/biocapt',           'self',  NULL, '生物传感器', 'mdi:bacteria-outline',              NULL, 2, 0, 1, NULL, NULL, NULL, 2, 1),
(123, 120, 'sensors_analog',            '/sensors/analog',            'self',  NULL, '模拟量输入', 'mdi:sine-wave',                     NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1),
(124, 120, 'sensors_editor',            '/sensors/editor',            'self',  NULL, '传感器编辑', 'mdi:pencil-box-outline',            NULL, 4, 0, 1, NULL, NULL, NULL, 2, 1),
(125, 120, 'sensors_limits',            '/sensors/limits',            'self',  NULL, '限值编辑器', 'mdi:alert-octagon-outline',         NULL, 5, 0, 1, NULL, NULL, NULL, 2, 1);
-- from database/mysql/pharma-ops-seed.sql
INSERT OR IGNORE INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
(130, 0,   'alarms',                 '/alarms',                 'self',  NULL, '报警中心',     'mdi:alarm-light-outline',     NULL, 6, 0, 1, NULL, 'basic', NULL, 2, 1),
(140, 0,   'rt-trend',               '/rt-trend',               'self',  NULL, '实时趋势',     'mdi:chart-timeline-variant',  NULL, 7, 0, 1, NULL, 'basic', NULL, 2, 1),
(150, 0,   'runtime-logic',          '/runtime-logic',          'self',  NULL, '运行逻辑',     'mdi:sitemap-outline',         NULL, 8, 0, 1, NULL, 'basic', NULL, 2, 1),
(160, 0,   'reports',                '/reports',                'basic', NULL, '数据报表',     'mdi:file-chart-outline',      NULL, 9, 0, 1, NULL, NULL, NULL, 1, 1),
(161, 160, 'reports_generator',      '/reports/generator',      'self',  NULL, '报表生成器',   'mdi:file-document-outline',   NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(162, 160, 'reports_sampling',       '/reports/sampling',       'self',  NULL, '采样报告',     'mdi:clipboard-text-outline',  NULL, 2, 0, 1, NULL, NULL, NULL, 2, 1),
(163, 160, 'reports_tags-catalog',   '/reports/tags-catalog',   'self',  NULL, 'Tags 目录',    'mdi:tag-multiple-outline',    NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1),
(164, 160, 'reports_sda',            '/reports/sda',            'self',  NULL, '统计分析器',   'mdi:chart-bell-curve',        NULL, 4, 0, 1, NULL, NULL, NULL, 2, 1);
-- from database/mysql/pharma-sys-seed.sql
INSERT OR IGNORE INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
-- 模块 5 扩展
(170, 90,  'management_password-policy', '/management/password-policy', 'self', NULL, '密码策略',   'mdi:form-textbox-password', NULL, 5, 0, 1, NULL, NULL, NULL, 2, 1),
(171, 0,   'audit',                      '/audit',                      'self', NULL, '审计轨迹',   'mdi:clipboard-text-clock-outline', NULL, 10, 0, 1, NULL, 'basic', NULL, 2, 1),
-- 模块 6 系统运维
(180, 0,   'system',                     '/system',                     'basic', NULL, '系统运维',   'mdi:cog-outline', NULL, 11, 0, 1, NULL, NULL, NULL, 1, 1),
(181, 180, 'system_backup',              '/system/backup',              'self',  NULL, '备份与恢复', 'mdi:backup-restore', NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(182, 180, 'system_control',             '/system/control',             'self',  NULL, '系统控制',   'mdi:power', NULL, 2, 0, 1, NULL, NULL, NULL, 2, 1),
(183, 180, 'system_language',            '/system/language',            'self',  NULL, '语言设置',   'mdi:translate', NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1),
(184, 180, 'system_reset-buffer',        '/system/reset-buffer',        'self',  NULL, '缓冲重置',   'mdi:database-refresh-outline', NULL, 4, 0, 1, NULL, NULL, NULL, 2, 1),
(185, 180, 'system_facility',            '/system/facility',            'self',  NULL, '厂区组织',   'mdi:office-building-marker-outline', NULL, 5, 0, 1, NULL, NULL, NULL, 2, 1);
-- from database/mysql/pharma-device-config-seed.sql
INSERT OR IGNORE INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
(113, 110, 'sampling_realtime', '/sampling/realtime', 'self', NULL, '实时信号', 'mdi:monitor-dashboard', NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT OR IGNORE INTO sys_menu
(id, parent_id, route_name, path, component, redirect, title, icon, local_icon, order_num, hide, requires_auth, href, single_layout, permissions, menu_type, status)
VALUES
(186, 180, 'system_license', '/system/license', 'self', NULL, '授权管理', 'mdi:license', NULL, 6, 0, 1, NULL, NULL, NULL, 2, 1),
(187, 180, 'system_device-poll', '/system/device-poll', 'self', NULL, '设备轮询联调', 'mdi:lan-connect', NULL, 7, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT OR IGNORE INTO sys_role_menu (role_id, menu_id)
SELECT 1, id FROM sys_menu;
INSERT OR IGNORE INTO sys_role_menu (role_id, menu_id)
SELECT 2, id FROM sys_menu;

INSERT OR IGNORE INTO ph_password_policy (id) VALUES (1);
INSERT OR IGNORE INTO ph_backup_config (id) VALUES (1);
INSERT OR IGNORE INTO ph_setup_state (id, mode, step, initialized, completed) VALUES (1, 'fresh', 0, 0, 0);

PRAGMA foreign_keys = ON;
