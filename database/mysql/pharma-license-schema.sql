-- =============================================================================
-- License：时间锁 + 设备 MAC 绑定
-- mysql -u root -p soybean_admin < database/mysql/pharma-license-schema.sql
-- =============================================================================

SET NAMES utf8mb4;

CREATE TABLE IF NOT EXISTS `ph_license` (
  `id`                BIGINT       NOT NULL AUTO_INCREMENT,
  `license_id`        VARCHAR(64)  NOT NULL COMMENT '授权码内唯一 ID',
  `customer_name`     VARCHAR(128) NOT NULL DEFAULT '',
  `valid_from`        DATE         NOT NULL,
  `valid_until`       DATE         NOT NULL,
  `device_lock`       TINYINT      NOT NULL DEFAULT 1 COMMENT '1=绑定本机 MAC',
  `bound_mac`         VARCHAR(64)           DEFAULT NULL COMMENT '首次激活写入',
  `license_code`      TEXT         NOT NULL,
  `max_observed_at`   DATETIME              DEFAULT NULL COMMENT '防回拨时钟',
  `activated_at`      DATETIME              DEFAULT NULL,
  `status`            VARCHAR(16)  NOT NULL DEFAULT 'active' COMMENT 'active|revoked',
  `created_at`        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`        DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_license_id` (`license_id`),
  KEY `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- 系统菜单：授权管理（parent 180 = system）
INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`,
 `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
SELECT 186, 180, 'system_license', '/system/license', 'self', NULL, '授权管理',
       'mdi:license', NULL, 6, 0, 1, NULL, NULL, NULL, 2, 1
FROM DUAL
WHERE NOT EXISTS (SELECT 1 FROM `sys_menu` WHERE `id` = 186 OR `route_name` = 'system_license');
