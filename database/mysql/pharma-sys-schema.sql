-- =============================================================================
-- 模块 5+6：权限/电子签名/审计扩展 + 系统运维
-- mysql -u root -p soybean_admin < database/mysql/pharma-sys-schema.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `ph_sampler_calibration`;
DROP TABLE IF EXISTS `ph_setup_state`;
DROP TABLE IF EXISTS `ph_backup_job`;
DROP TABLE IF EXISTS `ph_backup_config`;
DROP TABLE IF EXISTS `ph_system_setting`;
DROP TABLE IF EXISTS `ph_password_history`;
DROP TABLE IF EXISTS `ph_password_policy`;
DROP TABLE IF EXISTS `ph_user_profile`;

CREATE TABLE `ph_user_profile` (
  `user_id`               BIGINT       NOT NULL COMMENT 'sys_user.id',
  `user_id_code`          VARCHAR(64)  NOT NULL DEFAULT '' COMMENT '手册 User ID',
  `pharma_role`           VARCHAR(32)  NOT NULL DEFAULT 'User',
  `expiry_date`           DATE                  DEFAULT NULL,
  `must_change_password`  TINYINT      NOT NULL DEFAULT 0,
  `is_local_emergency`    TINYINT      NOT NULL DEFAULT 0,
  `password_changed_at`   DATETIME              DEFAULT NULL,
  `updated_at`            DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`user_id`),
  CONSTRAINT `fk_ph_user_profile_user` FOREIGN KEY (`user_id`) REFERENCES `sys_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_password_policy` (
  `id`                              TINYINT      NOT NULL DEFAULT 1,
  `expire_days`                     INT          NOT NULL DEFAULT 90,
  `min_length`                      INT          NOT NULL DEFAULT 6,
  `remember_old_count`              INT          NOT NULL DEFAULT 3,
  `auto_logoff_seconds`             INT          NOT NULL DEFAULT 1800,
  `electronic_signature_enabled`    TINYINT      NOT NULL DEFAULT 1,
  `updated_at`                      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_password_history` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `user_id`     BIGINT       NOT NULL,
  `password`    VARCHAR(128) NOT NULL,
  `created_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_user` (`user_id`),
  CONSTRAINT `fk_pwd_hist_user` FOREIGN KEY (`user_id`) REFERENCES `sys_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_system_setting` (
  `setting_key`   VARCHAR(64)  NOT NULL,
  `setting_value` JSON         NOT NULL,
  `updated_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`setting_key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_backup_config` (
  `id`           TINYINT      NOT NULL DEFAULT 1,
  `enabled`      TINYINT      NOT NULL DEFAULT 0,
  `daily_at`     VARCHAR(8)   NOT NULL DEFAULT '02:00',
  `target_path`  VARCHAR(512) NOT NULL DEFAULT './backups',
  `retain_days`  INT          NOT NULL DEFAULT 30,
  `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_backup_job` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `job_type`    VARCHAR(16)  NOT NULL DEFAULT 'manual',
  `status`      VARCHAR(16)  NOT NULL DEFAULT 'pending',
  `file_path`   VARCHAR(512)          DEFAULT NULL,
  `message`     VARCHAR(512)          DEFAULT NULL,
  `created_by`  VARCHAR(64)  NOT NULL DEFAULT 'system',
  `started_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `finished_at` DATETIME              DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_started` (`started_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_setup_state` (
  `id`             TINYINT      NOT NULL DEFAULT 1,
  `mode`           VARCHAR(64)           DEFAULT NULL,
  `step`           INT          NOT NULL DEFAULT 0,
  `db_host`        VARCHAR(128)          DEFAULT NULL,
  `db_port`        INT                   DEFAULT NULL,
  `db_name`        VARCHAR(128)          DEFAULT NULL,
  `db_user`        VARCHAR(128)          DEFAULT NULL,
  `initialized`    TINYINT      NOT NULL DEFAULT 0,
  `completed`      TINYINT      NOT NULL DEFAULT 0,
  `restore_file`   VARCHAR(512)          DEFAULT NULL,
  `updated_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `ph_sampler_calibration` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `sampler_id`  VARCHAR(64)  NOT NULL,
  `parameters`  JSON         NOT NULL,
  `updated_by`  VARCHAR(64)  NOT NULL DEFAULT 'system',
  `updated_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_sampler` (`sampler_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

SET FOREIGN_KEY_CHECKS = 1;
