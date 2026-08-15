-- =============================================================================
-- 模块 3+4 种子数据 + 菜单
-- mysql -u root -p soybean_admin < database/mysql/pharma-ops-seed.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `ph_sda_session`;
DELETE FROM `ph_report_document`;
DELETE FROM `ph_tag_catalog`;
DELETE FROM `ph_audit_trail`;
DELETE FROM `ph_rt_trend_session`;
DELETE FROM `ph_runtime_rule`;
DELETE FROM `ph_runtime_tag`;
DELETE FROM `ph_alarm_event`;

INSERT INTO `ph_alarm_event`
(`id`, `sensor_id`, `sensor_name`, `severity`, `message`, `data_type`, `value`, `limit_value`, `acknowledged`)
VALUES
(1, 1, 'PC-01', 'warning', '0.5um particle count approaching warning limit', '0.5um', 95, 100, 0),
(2, 1, 'PC-01', 'alarm', '5.0um exceeded alarm limit', '5.0um', 35, 29, 0),
(3, 3, 'BC-01', 'flow', 'Flow Calc Alarm on BioCapt vacuum line', NULL, NULL, NULL, 0),
(4, 4, 'AI-T1', 'communication', 'Analog sensor communication lost briefly', 'value', NULL, NULL, 1),
(5, 6, 'AI-DP1', 'warning', 'Differential pressure near warning', 'value', 11.2, 10, 0);

INSERT INTO `ph_runtime_tag` (`folder`, `name`, `data_type`, `value_json`, `path`) VALUES
('ParticleCounter', 'PC-01.Power', 'bool', 'true', 'Particle/PC-01/Power'),
('ParticleCounter', 'PC-01.0.5um', 'number', '12', 'Particle/PC-01/0.5um'),
('BioCapt', 'BC-01.Sampling', 'bool', 'false', 'BioCapt/BC-01/Sampling'),
('AnalogInput', 'AI-T1.Value', 'number', '22.5', 'Analog/AI-T1'),
('DigitalInput', 'DI-Door', 'bool', 'false', 'DI/Door'),
('DigitalOutput', 'DO-Tower-Red', 'bool', 'false', 'DO/Tower/Red'),
('DigitalOutput', 'DO-Tower-Green', 'bool', 'true', 'DO/Tower/Green'),
('System', 'System.Ready', 'bool', 'true', 'System/Ready');

INSERT INTO `ph_runtime_rule`
(`id`, `name`, `rule_type`, `parent_id`, `conditions`, `actions`, `enabled`, `recipe_id`)
VALUES
(1, 'Door Open Stops Sampling', 'condition', NULL,
 JSON_ARRAY(JSON_OBJECT('tagId','5','operator','eq','value', true)),
 JSON_ARRAY(JSON_OBJECT('type','stopSampling','targetId','0','value', true)),
 1, NULL),
(2, 'Tower Red on Alarm', 'towerLight', NULL,
 JSON_ARRAY(JSON_OBJECT('tagId','2','operator','gt','value', 100)),
 JSON_ARRAY(JSON_OBJECT('type','towerLight','targetId','6','value', true)),
 1, NULL),
(3, 'Sampling on Door Close', 'samplingOnInput', NULL,
 JSON_ARRAY(JSON_OBJECT('tagId','5','operator','eq','value', false)),
 JSON_ARRAY(JSON_OBJECT('type','startSampling','targetId','1','value', true)),
 1, 1);

INSERT INTO `ph_audit_trail` (`action`, `target_type`, `target_id`, `performed_by`, `role_code`, `reason`, `detail_json`) VALUES
('alarm.ack', 'alarm', '4', 'Admin', 'admin', '通信恢复后确认', JSON_OBJECT('sensor','AI-T1')),
('sampling.schedule', 'sampling', '1', 'Admin', 'admin', '日常调度', NULL),
('recipe.save', 'recipe', '1', 'Admin', 'admin', '更新笔配置', NULL),
('sensor.power', 'sensor', '1', 'Soybean', 'super', '手动开启粒子计数器', NULL);

INSERT INTO `ph_tag_catalog` (`tab`, `name`, `color`, `linked_ids`, `enabled`) VALUES
('sensorType', 'Particle', '#2080f0', JSON_ARRAY(), 1),
('sensorType', 'BioCapt', '#d03050', JSON_ARRAY(), 1),
('sensorType', 'Analog', '#18a058', JSON_ARRAY(), 1),
('sensors', 'PC-01', '#2080f0', JSON_ARRAY('1'), 1),
('sensors', 'BC-01', '#d03050', JSON_ARRAY('3'), 1),
('sensorGroups', 'Cleanroom A', '#f0a020', JSON_ARRAY('1'), 1),
('recipes', 'Daily Monitoring', '#8a2be2', JSON_ARRAY('1'), 1),
('dataTypes', '0.5um', '#2080f0', JSON_ARRAY(), 1),
('dataTypes', 'value', '#18a058', JSON_ARRAY(), 1);

INSERT INTO `ph_sda_session`
(`config_name`, `csv_files`, `csv_content`, `display_config`, `virtual_pens`, `markers_enabled`)
VALUES
('Demo Trend', JSON_ARRAY('demo.csv'),
 'timestamp,PC-01_0.5um,AI-T1\n2026-07-20 08:00:00,10,22.1\n2026-07-20 09:00:00,14,22.4\n2026-07-20 10:00:00,18,22.8\n2026-07-20 11:00:00,12,23.0\n',
 JSON_OBJECT('interpolation','linear','showMarkers', true),
 JSON_ARRAY(), 1);

-- 菜单 130-164
DELETE FROM `sys_role_menu` WHERE `menu_id` BETWEEN 130 AND 164;
DELETE FROM `sys_menu` WHERE `id` BETWEEN 130 AND 164;

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
VALUES
(130, 0,   'alarms',                 '/alarms',                 'self',  NULL, '报警中心',     'mdi:alarm-light-outline',     NULL, 6, 0, 1, NULL, 'basic', NULL, 2, 1),
(140, 0,   'rt-trend',               '/rt-trend',               'self',  NULL, '实时趋势',     'mdi:chart-timeline-variant',  NULL, 7, 0, 1, NULL, 'basic', NULL, 2, 1),
(150, 0,   'runtime-logic',          '/runtime-logic',          'self',  NULL, '运行逻辑',     'mdi:sitemap-outline',         NULL, 8, 0, 1, NULL, 'basic', NULL, 2, 1),
(160, 0,   'reports',                '/reports',                'basic', NULL, '数据报表',     'mdi:file-chart-outline',      NULL, 9, 0, 1, NULL, NULL, NULL, 1, 1),
(161, 160, 'reports_generator',      '/reports/generator',      'self',  NULL, '报表生成器',   'mdi:file-document-outline',   NULL, 1, 0, 1, NULL, NULL, NULL, 2, 1),
(162, 160, 'reports_sampling',       '/reports/sampling',       'self',  NULL, '采样报告',     'mdi:clipboard-text-outline',  NULL, 2, 0, 1, NULL, NULL, NULL, 2, 1),
(163, 160, 'reports_tags-catalog',   '/reports/tags-catalog',   'self',  NULL, 'Tags 目录',    'mdi:tag-multiple-outline',    NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1),
(164, 160, 'reports_sda',            '/reports/sda',            'self',  NULL, '统计分析器',   'mdi:chart-bell-curve',        NULL, 4, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 1, `id` FROM `sys_menu` WHERE `id` BETWEEN 130 AND 164;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`)
SELECT 2, `id` FROM `sys_menu` WHERE `id` BETWEEN 130 AND 164;

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(3, 130), (3, 140), (3, 160), (3, 161), (3, 162);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'alarms' AS item, COUNT(*) AS total FROM `ph_alarm_event`
UNION ALL SELECT 'runtime_tags', COUNT(*) FROM `ph_runtime_tag`
UNION ALL SELECT 'runtime_rules', COUNT(*) FROM `ph_runtime_rule`
UNION ALL SELECT 'audit', COUNT(*) FROM `ph_audit_trail`;
