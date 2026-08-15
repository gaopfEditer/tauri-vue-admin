-- =============================================================================
-- Pharmaceutical Net Pro 业务表（模块 1 配方采样 + 模块 2 设备传感器）
-- 在已选中 soybean_admin 库后执行:
--   mysql -u root -p soybean_admin < database/mysql/pharma-schema.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `ph_sensor_limit_history`;
DROP TABLE IF EXISTS `ph_sensor_limit`;
DROP TABLE IF EXISTS `ph_sampling_task`;
DROP TABLE IF EXISTS `ph_sampling_custom_field`;
DROP TABLE IF EXISTS `ph_recipe`;
DROP TABLE IF EXISTS `ph_sensor`;
DROP TABLE IF EXISTS `ph_sensor_group`;

CREATE TABLE `ph_sensor_group` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `name`        VARCHAR(128) NOT NULL,
  `description` VARCHAR(255)          DEFAULT NULL,
  `tags`        JSON                  DEFAULT NULL,
  `status`      TINYINT      NOT NULL DEFAULT 1,
  `created_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_group_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='传感器组';

CREATE TABLE `ph_sensor` (
  `id`               BIGINT       NOT NULL AUTO_INCREMENT,
  `category`         VARCHAR(32)  NOT NULL COMMENT 'particle|biocapt|analog',
  `custom_id`        VARCHAR(64)  NOT NULL,
  `description`      VARCHAR(255) NOT NULL DEFAULT '',
  `group_id`         BIGINT                DEFAULT NULL,
  `channel`          VARCHAR(64)           DEFAULT NULL,
  `acquire_mode`     VARCHAR(32)           DEFAULT NULL COMMENT 'Volume|Time|Continuous',
  `runtime_state`    VARCHAR(32)  NOT NULL DEFAULT 'Idle',
  `power_on`         TINYINT      NOT NULL DEFAULT 0,
  `com_alarm`        TINYINT      NOT NULL DEFAULT 0,
  `flow_calc_alarm`  TINYINT      NOT NULL DEFAULT 0,
  `last_value`       JSON                  DEFAULT NULL,
  `unit`             VARCHAR(32)           DEFAULT NULL,
  `status`           TINYINT      NOT NULL DEFAULT 1,
  `created_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_custom_id` (`custom_id`),
  KEY `idx_category` (`category`),
  KEY `idx_group_id` (`group_id`),
  CONSTRAINT `fk_sensor_group` FOREIGN KEY (`group_id`) REFERENCES `ph_sensor_group` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='传感器/采样点';

CREATE TABLE `ph_sensor_limit` (
  `id`             BIGINT        NOT NULL AUTO_INCREMENT,
  `sensor_id`      BIGINT        NOT NULL,
  `data_type`      VARCHAR(64)   NOT NULL,
  `warning_limit`  DECIMAL(18,4)          DEFAULT NULL,
  `alarm_limit`    DECIMAL(18,4)          DEFAULT NULL,
  `effective_from` DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `changed_by`     VARCHAR(64)   NOT NULL DEFAULT '',
  `created_at`     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_sensor_datatype` (`sensor_id`, `data_type`),
  CONSTRAINT `fk_limit_sensor` FOREIGN KEY (`sensor_id`) REFERENCES `ph_sensor` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='传感器限值';

CREATE TABLE `ph_sensor_limit_history` (
  `id`         BIGINT   NOT NULL AUTO_INCREMENT,
  `limit_id`   BIGINT   NOT NULL,
  `before_json` JSON             DEFAULT NULL,
  `after_json`  JSON             DEFAULT NULL,
  `signed_by`  VARCHAR(64) NOT NULL DEFAULT '',
  `signed_at`  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `reason`     VARCHAR(255)       DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_limit_id` (`limit_id`),
  CONSTRAINT `fk_limit_history` FOREIGN KEY (`limit_id`) REFERENCES `ph_sensor_limit` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='限值变更历史';

CREATE TABLE `ph_recipe` (
  `id`                BIGINT       NOT NULL AUTO_INCREMENT,
  `name`              VARCHAR(128) NOT NULL,
  `description`       VARCHAR(255)          DEFAULT NULL,
  `sensor_group_ids`  JSON         NOT NULL,
  `pens`              JSON         NOT NULL,
  `created_by`        VARCHAR(64)  NOT NULL DEFAULT '',
  `status`            VARCHAR(16)  NOT NULL DEFAULT 'active' COMMENT 'active|deleted',
  `created_at`        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_recipe_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='采样配方';

CREATE TABLE `ph_sampling_task` (
  `id`             BIGINT       NOT NULL AUTO_INCREMENT,
  `recipe_id`      BIGINT       NOT NULL,
  `recipe_name`    VARCHAR(128) NOT NULL,
  `scheduled_at`   DATETIME     NOT NULL,
  `sampling_mode`  VARCHAR(32)  NOT NULL DEFAULT 'Operational',
  `custom_fields`  JSON                  DEFAULT NULL,
  `input_notes`    VARCHAR(512)          DEFAULT NULL,
  `status`         VARCHAR(16)  NOT NULL DEFAULT 'scheduled',
  `started_at`     DATETIME              DEFAULT NULL,
  `finished_at`    DATETIME              DEFAULT NULL,
  `abort_reason`   VARCHAR(255)          DEFAULT NULL,
  `created_by`     VARCHAR(64)  NOT NULL DEFAULT '',
  `created_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_recipe_id` (`recipe_id`),
  KEY `idx_status` (`status`),
  CONSTRAINT `fk_sampling_recipe` FOREIGN KEY (`recipe_id`) REFERENCES `ph_recipe` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='采样任务';

CREATE TABLE `ph_sampling_custom_field` (
  `id`              BIGINT       NOT NULL AUTO_INCREMENT,
  `field_key`       VARCHAR(64)  NOT NULL,
  `display_name`    VARCHAR(128) NOT NULL,
  `enabled`         TINYINT      NOT NULL DEFAULT 1,
  `editable`        TINYINT      NOT NULL DEFAULT 1,
  `default_entries` JSON                  DEFAULT NULL,
  `sort_order`      INT          NOT NULL DEFAULT 0,
  `created_at`      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_field_key` (`field_key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='采样自定义字段定义';

SET FOREIGN_KEY_CHECKS = 1;
