-- =============================================================================
-- 模块 3+4：报警/趋势/运行逻辑 + 报表/SDA
-- mysql -u root -p soybean_admin < database/mysql/pharma-ops-schema.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `ph_sda_session`;
DROP TABLE IF EXISTS `ph_report_document`;
DROP TABLE IF EXISTS `ph_tag_catalog`;
DROP TABLE IF EXISTS `ph_audit_trail`;
DROP TABLE IF EXISTS `ph_rt_trend_session`;
DROP TABLE IF EXISTS `ph_runtime_rule`;
DROP TABLE IF EXISTS `ph_runtime_tag`;
DROP TABLE IF EXISTS `ph_alarm_event`;

CREATE TABLE `ph_alarm_event` (
  `id`               BIGINT       NOT NULL AUTO_INCREMENT,
  `sensor_id`        BIGINT                DEFAULT NULL,
  `sensor_name`      VARCHAR(128) NOT NULL DEFAULT '',
  `severity`        VARCHAR(32)  NOT NULL COMMENT 'warning|alarm|communication|flow',
  `message`          VARCHAR(512) NOT NULL,
  `data_type`        VARCHAR(64)           DEFAULT NULL,
  `value`            DOUBLE                DEFAULT NULL,
  `limit_value`      DOUBLE                DEFAULT NULL,
  `sampling_id`      BIGINT                DEFAULT NULL,
  `raised_at`        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `acknowledged`     TINYINT      NOT NULL DEFAULT 0,
  `acknowledged_by`  VARCHAR(64)           DEFAULT NULL,
  `acknowledged_at`  DATETIME              DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_ack` (`acknowledged`),
  KEY `idx_raised` (`raised_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_runtime_tag` (
  `id`         BIGINT       NOT NULL AUTO_INCREMENT,
  `folder`     VARCHAR(64)  NOT NULL,
  `name`       VARCHAR(128) NOT NULL,
  `data_type`  VARCHAR(16)  NOT NULL DEFAULT 'bool',
  `value_json` JSON                  DEFAULT NULL,
  `path`       VARCHAR(255)          DEFAULT NULL,
  `status`     TINYINT      NOT NULL DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `idx_folder` (`folder`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_runtime_rule` (
  `id`           BIGINT       NOT NULL AUTO_INCREMENT,
  `name`         VARCHAR(128) NOT NULL,
  `rule_type`    VARCHAR(64)  NOT NULL DEFAULT 'condition',
  `parent_id`    BIGINT                DEFAULT NULL,
  `conditions`   JSON         NOT NULL,
  `actions`      JSON         NOT NULL,
  `enabled`      TINYINT      NOT NULL DEFAULT 1,
  `recipe_id`    BIGINT                DEFAULT NULL,
  `created_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_rt_trend_session` (
  `id`           BIGINT       NOT NULL AUTO_INCREMENT,
  `recipe_id`    BIGINT       NOT NULL,
  `recipe_name`  VARCHAR(128) NOT NULL DEFAULT '',
  `running`      TINYINT      NOT NULL DEFAULT 0,
  `pens`         JSON         NOT NULL,
  `started_at`   DATETIME              DEFAULT NULL,
  `stopped_at`   DATETIME              DEFAULT NULL,
  `started_by`   VARCHAR(64)           DEFAULT NULL,
  `created_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_audit_trail` (
  `id`            BIGINT       NOT NULL AUTO_INCREMENT,
  `event_time`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `action`        VARCHAR(128) NOT NULL,
  `target_type`   VARCHAR(64)  NOT NULL DEFAULT '',
  `target_id`     VARCHAR(64)           DEFAULT NULL,
  `performed_by`  VARCHAR(64)  NOT NULL DEFAULT '',
  `role_code`     VARCHAR(32)  NOT NULL DEFAULT 'user',
  `reason`        VARCHAR(255)          DEFAULT NULL,
  `detail_json`   JSON                  DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_event_time` (`event_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_tag_catalog` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `tab`         VARCHAR(64)  NOT NULL,
  `name`        VARCHAR(128) NOT NULL,
  `color`       VARCHAR(32)           DEFAULT NULL,
  `linked_ids`  JSON                  DEFAULT NULL,
  `enabled`     TINYINT      NOT NULL DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `idx_tab` (`tab`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_report_document` (
  `id`            BIGINT       NOT NULL AUTO_INCREMENT,
  `kind`          VARCHAR(32)  NOT NULL,
  `mode`          VARCHAR(32)  NOT NULL,
  `query_json`    JSON         NOT NULL,
  `header_json`   JSON         NOT NULL,
  `body_json`     JSON         NOT NULL,
  `summary_json`  JSON                  DEFAULT NULL,
  `footer_json`   JSON         NOT NULL,
  `generated_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `generated_by`  VARCHAR(64)  NOT NULL DEFAULT '',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_sda_session` (
  `id`               BIGINT       NOT NULL AUTO_INCREMENT,
  `config_name`      VARCHAR(128)          DEFAULT NULL,
  `csv_files`        JSON         NOT NULL,
  `csv_content`      MEDIUMTEXT            DEFAULT NULL,
  `display_config`   JSON         NOT NULL,
  `virtual_pens`     JSON         NOT NULL,
  `markers_enabled`  TINYINT      NOT NULL DEFAULT 0,
  `created_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

SET FOREIGN_KEY_CHECKS = 1;
