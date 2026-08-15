-- =============================================================================
-- 三级厂区空间组织：厂房/园区 → 区域/洁净区 → 房间 Room
-- mysql -u root -p soybean_admin < database/mysql/pharma-facility-schema.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `ph_facility_node`;

CREATE TABLE `ph_facility_node` (
  `id`          BIGINT       NOT NULL AUTO_INCREMENT,
  `parent_id`   BIGINT                DEFAULT NULL COMMENT '父节点，一级为 NULL',
  `level`       TINYINT      NOT NULL COMMENT '1厂房/园区 2区域/洁净区 3房间',
  `code`        VARCHAR(64)  NOT NULL DEFAULT '' COMMENT '编码',
  `name`        VARCHAR(128) NOT NULL COMMENT '名称',
  `description` VARCHAR(255)          DEFAULT NULL,
  `sort_order`  INT          NOT NULL DEFAULT 0,
  `status`      TINYINT      NOT NULL DEFAULT 1 COMMENT '1启用 0停用',
  `created_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_parent` (`parent_id`),
  KEY `idx_level` (`level`),
  CONSTRAINT `fk_facility_parent` FOREIGN KEY (`parent_id`) REFERENCES `ph_facility_node` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='厂区三级空间组织';

SET FOREIGN_KEY_CHECKS = 1;
