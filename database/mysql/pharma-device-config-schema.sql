-- =============================================================================
-- 粒子设备配置 + 多工况阈值 + 实时信号支撑
-- mysql -u root -p soybean_admin < database/mysql/pharma-device-config-schema.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `ph_sensor_state_threshold`;
DROP TABLE IF EXISTS `ph_sensor_device_config`;

CREATE TABLE `ph_sensor_device_config` (
  `sensor_id`            BIGINT       NOT NULL COMMENT 'ph_sensor.id',
  `facility_node_id`     BIGINT                DEFAULT NULL COMMENT '挂载房间/区域节点',
  `device_name`          VARCHAR(128) NOT NULL DEFAULT '' COMMENT '设备名称',
  `device_code`          VARCHAR(64)  NOT NULL DEFAULT '' COMMENT '设备编号',
  `instrument_type`      VARCHAR(64)           DEFAULT NULL COMMENT '仪表类型',
  `update_interval_sec`  INT          NOT NULL DEFAULT 60,
  `slave_address`        VARCHAR(64)           DEFAULT NULL,
  `plc_ip`               VARCHAR(64)           DEFAULT NULL,
  `data_unit`            VARCHAR(32)           DEFAULT NULL,
  `data_type`            VARCHAR(32)  NOT NULL DEFAULT 'ulong',
  `data_length`          INT          NOT NULL DEFAULT 4,
  `protocol`             VARCHAR(32)  NOT NULL DEFAULT 'ModbusTCP',
  `decimal_places`       INT          NOT NULL DEFAULT 0,
  `cleanroom_class`      VARCHAR(32)           DEFAULT NULL,
  `serial_number`        VARCHAR(64)           DEFAULT NULL,
  `calibration_date`     DATE                  DEFAULT NULL,
  `operating_mode`       VARCHAR(64)  NOT NULL DEFAULT 'Operational',
  `production_state`     VARCHAR(64)  NOT NULL DEFAULT 'production' COMMENT '当前工况',
  `flow_rate`            DOUBLE                DEFAULT NULL COMMENT '实时流量 L/min',
  `updated_at`           DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`sensor_id`),
  KEY `idx_facility` (`facility_node_id`),
  CONSTRAINT `fk_device_cfg_sensor` FOREIGN KEY (`sensor_id`) REFERENCES `ph_sensor` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_device_cfg_facility` FOREIGN KEY (`facility_node_id`) REFERENCES `ph_facility_node` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_sensor_state_threshold` (
  `id`            BIGINT       NOT NULL AUTO_INCREMENT,
  `sensor_id`     BIGINT       NOT NULL,
  `state_group`   VARCHAR(64)  NOT NULL COMMENT 'production/disinfection/static/post_vent/shutdown',
  `metric_key`    VARCHAR(64)  NOT NULL DEFAULT '0.5um',
  `alarm_enable`  VARCHAR(32)  NOT NULL DEFAULT 'unlimited' COMMENT 'unlimited|enabled|disabled',
  `warn_high`     DOUBLE                DEFAULT NULL,
  `warn_low`      DOUBLE                DEFAULT NULL,
  `alarm_high`    DOUBLE                DEFAULT NULL,
  `alarm_low`     DOUBLE                DEFAULT NULL,
  `updated_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_sensor_state_metric` (`sensor_id`, `state_group`, `metric_key`),
  CONSTRAINT `fk_thr_sensor` FOREIGN KEY (`sensor_id`) REFERENCES `ph_sensor` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

SET FOREIGN_KEY_CHECKS = 1;
