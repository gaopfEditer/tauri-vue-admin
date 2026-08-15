-- =============================================================================
-- 模块 5+6 种子数据 + 菜单
-- mysql -u root -p soybean_admin < database/mysql/pharma-sys-seed.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `ph_sampler_calibration`;
DELETE FROM `ph_setup_state`;
DELETE FROM `ph_backup_job`;
DELETE FROM `ph_backup_config`;
DELETE FROM `ph_system_setting`;
DELETE FROM `ph_password_history`;
DELETE FROM `ph_password_policy`;
DELETE FROM `ph_user_profile`;

INSERT INTO `ph_password_policy`
(`id`, `expire_days`, `min_length`, `remember_old_count`, `auto_logoff_seconds`, `electronic_signature_enabled`)
VALUES (1, 90, 6, 3, 1800, 1);

INSERT INTO `ph_backup_config` (`id`, `enabled`, `daily_at`, `target_path`, `retain_days`)
VALUES (1, 1, '02:00', './backups', 30);

INSERT INTO `ph_setup_state`
(`id`, `mode`, `step`, `db_host`, `db_port`, `db_name`, `db_user`, `initialized`, `completed`)
VALUES (1, 'modifyConfiguration', 0, '127.0.0.1', 3306, 'soybean_admin', 'root', 1, 1);

INSERT INTO `ph_system_setting` (`setting_key`, `setting_value`) VALUES
('language', JSON_OBJECT('locale', 'zh-CN', 'displayName', '简体中文')),
('buffer', JSON_OBJECT('lastResetAt', NULL, 'resetCount', 0));

-- 为现有用户补档案
INSERT INTO `ph_user_profile` (`user_id`, `user_id_code`, `pharma_role`, `must_change_password`, `is_local_emergency`, `password_changed_at`)
SELECT u.id,
       CONCAT('UID-', u.id),
       CASE r.role_code
         WHEN 'super' THEN 'Administrator'
         WHEN 'admin' THEN 'Supervisor'
         ELSE 'User'
       END,
       0, 0, NOW()
FROM `sys_user` u
LEFT JOIN `sys_user_role` ur ON ur.user_id = u.id
LEFT JOIN `sys_role` r ON r.id = ur.role_id
WHERE u.status = 1
ON DUPLICATE KEY UPDATE
  `pharma_role` = VALUES(`pharma_role`),
  `user_id_code` = VALUES(`user_id_code`);

INSERT INTO `ph_sampler_calibration` (`sampler_id`, `parameters`, `updated_by`) VALUES
('SAMPLER-01', JSON_OBJECT('flowRate', 28.3, 'volume', 1000, 'offset', 0), 'Admin');

-- 菜单 170-199
DELETE FROM `sys_role_menu` WHERE `menu_id` BETWEEN 170 AND 199;
DELETE FROM `sys_menu` WHERE `id` BETWEEN 170 AND 199;

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
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

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 1, `id` FROM `sys_menu` WHERE `id` BETWEEN 170 AND 189;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 2, `id` FROM `sys_menu` WHERE `id` BETWEEN 170 AND 189;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(3, 171), (3, 180), (3, 181), (3, 183), (3, 185);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'policy' AS item, COUNT(*) AS total FROM `ph_password_policy`
UNION ALL SELECT 'profiles', COUNT(*) FROM `ph_user_profile`
UNION ALL SELECT 'backup_cfg', COUNT(*) FROM `ph_backup_config`
UNION ALL SELECT 'menus', COUNT(*) FROM `sys_menu` WHERE id BETWEEN 170 AND 189;
