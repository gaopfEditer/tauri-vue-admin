-- =============================================================================
-- Pharmaceutical Net Pro 种子数据 + 菜单注册
-- 先执行 pharma-schema.sql，再执行本文件
--   mysql -u root -p123456 soybean_admin < database/mysql/pharma-seed.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `ph_sensor_limit_history`;
DELETE FROM `ph_sensor_limit`;
DELETE FROM `ph_sampling_task`;
DELETE FROM `ph_sampling_custom_field`;
DELETE FROM `ph_recipe`;
DELETE FROM `ph_sensor`;
DELETE FROM `ph_sensor_group`;

INSERT INTO `ph_sensor_group` (`id`, `name`, `description`, `tags`) VALUES
(1, 'Cleanroom A', 'A区粒子与生物采样点', JSON_ARRAY('area-a')),
(2, 'Cleanroom B', 'B区模拟量监测点', JSON_ARRAY('area-b'));

INSERT INTO `ph_sensor`
(`id`, `category`, `custom_id`, `description`, `group_id`, `channel`, `acquire_mode`, `runtime_state`, `power_on`, `com_alarm`, `flow_calc_alarm`, `last_value`, `unit`)
VALUES
(1, 'particle', 'PC-01', 'ISO Class 5 Particle Counter', 1, 'EBI-CH1', 'Continuous', 'Idle', 0, 0, 0, JSON_OBJECT('0.5um', 12, '5.0um', 1), 'counts'),
(2, 'particle', 'PC-02', 'ISO Class 7 Particle Counter', 1, 'EBI-CH2', 'Volume', 'Idle', 0, 0, 0, JSON_OBJECT('0.5um', 80, '5.0um', 4), 'counts'),
(3, 'biocapt',  'BC-01', 'BioCapt Microbial Sampler',    1, 'B0-Remote', 'Time', 'Idle', 0, 0, 0, JSON_OBJECT('cfu', 0), 'cfu'),
(4, 'analog',   'AI-T1', 'Room Temperature',             2, 'AI-1', NULL, 'Idle', 1, 0, 0, JSON_OBJECT('value', 22.5), 'C'),
(5, 'analog',   'AI-RH1','Relative Humidity',            2, 'AI-2', NULL, 'Idle', 1, 0, 0, JSON_OBJECT('value', 45.0), '%RH'),
(6, 'analog',   'AI-DP1','Differential Pressure',        2, 'AI-3', NULL, 'Idle', 1, 0, 0, JSON_OBJECT('value', 15.2), 'Pa');

INSERT INTO `ph_sensor_limit` (`id`, `sensor_id`, `data_type`, `warning_limit`, `alarm_limit`, `changed_by`) VALUES
(1, 1, '0.5um', 100, 352, 'Admin'),
(2, 1, '5.0um', 10,  29,  'Admin'),
(3, 2, '0.5um', 500, 3520,'Admin'),
(4, 4, 'value', 18,  30,  'Admin'),
(5, 5, 'value', 30,  65,  'Admin'),
(6, 6, 'value', 10,  25,  'Admin');

INSERT INTO `ph_recipe` (`id`, `name`, `description`, `sensor_group_ids`, `pens`, `created_by`, `status`) VALUES
(1, 'Daily Monitoring', '日常环境监测配方',
 JSON_ARRAY(1, 2),
 JSON_ARRAY(
   JSON_OBJECT('id','1','sensorId','1','dataType','0.5um','color','#2080f0','enabled',true,'label','PC-01 0.5um'),
   JSON_OBJECT('id','2','sensorId','4','dataType','value','color','#18a058','enabled',true,'label','Temp')
 ),
 'Admin', 'active'),
(2, 'At Rest Check', '静态检测配方',
 JSON_ARRAY(1),
 JSON_ARRAY(
   JSON_OBJECT('id','1','sensorId','1','dataType','0.5um','color','#f0a020','enabled',true,'label','PC-01'),
   JSON_OBJECT('id','2','sensorId','3','dataType','cfu','color','#d03050','enabled',true,'label','BC-01')
 ),
 'Admin', 'active');

INSERT INTO `ph_sampling_custom_field` (`field_key`, `display_name`, `enabled`, `editable`, `default_entries`, `sort_order`) VALUES
('batchNumber', 'Batch Number', 1, 1, JSON_ARRAY('B001','B002','B003'), 1),
('operator',    'Operator',     1, 1, NULL, 2),
('area',        'Area',         1, 1, JSON_ARRAY('A','B','C'), 3);

INSERT INTO `ph_sampling_task`
(`id`, `recipe_id`, `recipe_name`, `scheduled_at`, `sampling_mode`, `custom_fields`, `input_notes`, `status`, `created_by`)
VALUES
(1, 1, 'Daily Monitoring', DATE_ADD(NOW(), INTERVAL 1 HOUR), 'Operational',
 JSON_OBJECT('batchNumber','B001','operator','Admin','area','A'), '首次调度示例', 'scheduled', 'Admin');

-- 菜单：配方 / 采样 / 传感器 / 系统传感器编辑与限值
DELETE FROM `sys_role_menu` WHERE `menu_id` BETWEEN 100 AND 120;
DELETE FROM `sys_menu` WHERE `id` BETWEEN 100 AND 120;

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
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

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 1, `id` FROM `sys_menu` WHERE `id` BETWEEN 100 AND 125;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 2, `id` FROM `sys_menu` WHERE `id` BETWEEN 100 AND 125;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(3, 100), (3, 101),
(3, 110), (3, 111),
(3, 120), (3, 121), (3, 122), (3, 123);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'sensor_groups' AS item, COUNT(*) AS total FROM `ph_sensor_group`
UNION ALL SELECT 'sensors', COUNT(*) FROM `ph_sensor`
UNION ALL SELECT 'recipes', COUNT(*) FROM `ph_recipe`
UNION ALL SELECT 'samplings', COUNT(*) FROM `ph_sampling_task`;
