-- =============================================================================
-- 粒子设备配置种子 + 实时信号菜单
-- mysql -u root -p soybean_admin < database/mysql/pharma-device-config-seed.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `ph_sensor_state_threshold`;
DELETE FROM `ph_sensor_device_config`;

INSERT INTO `ph_sensor_device_config`
(`sensor_id`, `facility_node_id`, `device_name`, `device_code`, `instrument_type`, `update_interval_sec`,
 `slave_address`, `plc_ip`, `data_unit`, `data_type`, `data_length`, `protocol`, `decimal_places`,
 `cleanroom_class`, `serial_number`, `calibration_date`, `operating_mode`, `production_state`, `flow_rate`)
VALUES
(1, 111, '粒1_0.5立方英尺', 'PC27', '尘埃粒子0.5', 60,
 '40001', 'PLC1', 'pt/3', 'ulong', 4, 'ModbusTCP', 0,
 'Class A', '247799', '2026-06-24', 'Operational', 'production', 25.0),
(2, 112, '粒2_0.5立方英尺', 'PC28', '尘埃粒子0.5', 60,
 '40002', 'PLC1', 'pt/3', 'ulong', 4, 'ModbusTCP', 0,
 'Class A', '247800', '2026-06-24', 'Operational', 'production', 24.5);

-- 多工况阈值（0.5um / 5.0um）
INSERT INTO `ph_sensor_state_threshold`
(`sensor_id`, `state_group`, `metric_key`, `alarm_enable`, `warn_high`, `warn_low`, `alarm_high`, `alarm_low`)
VALUES
-- PC-01 production
(1, 'production', '0.5um', 'enabled', 100, 0, 352, 0),
(1, 'production', '5.0um', 'enabled', 10, 0, 29, 0),
(1, 'disinfection', '0.5um', 'unlimited', NULL, NULL, NULL, NULL),
(1, 'disinfection', '5.0um', 'unlimited', NULL, NULL, NULL, NULL),
(1, 'static', '0.5um', 'enabled', 50, 0, 100, 0),
(1, 'static', '5.0um', 'enabled', 5, 0, 10, 0),
(1, 'post_vent', '0.5um', 'enabled', 80, 0, 200, 0),
(1, 'post_vent', '5.0um', 'enabled', 8, 0, 20, 0),
(1, 'shutdown', '0.5um', 'disabled', NULL, NULL, NULL, NULL),
(1, 'shutdown', '5.0um', 'disabled', NULL, NULL, NULL, NULL),
-- PC-02
(2, 'production', '0.5um', 'enabled', 100, 0, 352, 0),
(2, 'production', '5.0um', 'enabled', 10, 0, 29, 0),
(2, 'disinfection', '0.5um', 'unlimited', NULL, NULL, NULL, NULL),
(2, 'disinfection', '5.0um', 'unlimited', NULL, NULL, NULL, NULL),
(2, 'static', '0.5um', 'enabled', 50, 0, 100, 0),
(2, 'static', '5.0um', 'enabled', 5, 0, 10, 0),
(2, 'post_vent', '0.5um', 'enabled', 80, 0, 200, 0),
(2, 'post_vent', '5.0um', 'enabled', 8, 0, 20, 0),
(2, 'shutdown', '0.5um', 'disabled', NULL, NULL, NULL, NULL),
(2, 'shutdown', '5.0um', 'disabled', NULL, NULL, NULL, NULL);

-- 调高 PC-01 实时值以便演示告警（0.5um 超预警）
UPDATE `ph_sensor` SET `last_value` = JSON_OBJECT('0.5um', 1512, '5.0um', 2) WHERE `id` = 1;
UPDATE `ph_sensor` SET `last_value` = JSON_OBJECT('0.5um', 12, '5.0um', 1) WHERE `id` = 2;

-- 菜单：采样管理 - 实时信号
DELETE FROM `sys_role_menu` WHERE `menu_id` = 113;
DELETE FROM `sys_menu` WHERE `id` = 113;

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
VALUES
(113, 110, 'sampling_realtime', '/sampling/realtime', 'self', NULL, '实时信号', 'mdi:monitor-dashboard', NULL, 3, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(1, 113), (2, 113), (3, 113);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'device_cfg' AS item, COUNT(*) AS total FROM `ph_sensor_device_config`
UNION ALL SELECT 'thresholds', COUNT(*) FROM `ph_sensor_state_threshold`
UNION ALL SELECT 'menu113', COUNT(*) FROM `sys_menu` WHERE id = 113;
