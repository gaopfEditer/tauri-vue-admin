-- =============================================================================
-- Soybean Admin RBAC 建表脚本 (MySQL 8.0+)
-- =============================================================================
-- 【Navicat / DataGrip / Workbench 用法】
--   1. 先手动创建数据库 soybean_admin (utf8mb4 / utf8mb4_unicode_ci)
--   2. 在客户端里「选中 soybean_admin 库」
--   3. 打开本文件，全选执行
--
-- 【命令行用法】
--   mysql -u root -p soybean_admin < database/mysql/schema.sql
--
-- 【一键建库+建表+种子数据】
--   mysql -u root -p < database/mysql/install.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DROP TABLE IF EXISTS `sys_user_token`;
DROP TABLE IF EXISTS `sys_role_permission`;
DROP TABLE IF EXISTS `sys_role_menu`;
DROP TABLE IF EXISTS `sys_user_role`;
DROP TABLE IF EXISTS `sys_permission`;
DROP TABLE IF EXISTS `sys_menu`;
DROP TABLE IF EXISTS `sys_user`;
DROP TABLE IF EXISTS `sys_role`;

CREATE TABLE `sys_role` (
  `id`            BIGINT       NOT NULL AUTO_INCREMENT COMMENT '角色ID',
  `role_code`     VARCHAR(32)  NOT NULL COMMENT '角色编码: super/admin/user',
  `role_name`     VARCHAR(64)  NOT NULL COMMENT '角色名称',
  `description`   VARCHAR(255)          DEFAULT NULL COMMENT '角色描述',
  `home_route`    VARCHAR(128) NOT NULL DEFAULT 'dashboard_analysis' COMMENT '登录后首页路由 name',
  `status`        TINYINT      NOT NULL DEFAULT 1 COMMENT '状态: 1启用 0禁用',
  `sort`          INT          NOT NULL DEFAULT 0 COMMENT '排序',
  `created_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_role_code` (`role_code`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='系统角色';

CREATE TABLE `sys_user` (
  `id`            BIGINT       NOT NULL AUTO_INCREMENT COMMENT '用户ID',
  `user_name`     VARCHAR(64)  NOT NULL COMMENT '登录用户名',
  `password`      VARCHAR(128) NOT NULL COMMENT '密码(MD5/BCrypt，由后端加密)',
  `nick_name`     VARCHAR(64)           DEFAULT NULL COMMENT '昵称/姓名',
  `age`           INT                   DEFAULT NULL COMMENT '年龄',
  `gender`        CHAR(1)               DEFAULT NULL COMMENT '性别: 0女 1男',
  `phone`         VARCHAR(20)           DEFAULT NULL COMMENT '手机号',
  `email`         VARCHAR(128)          DEFAULT NULL COMMENT '邮箱',
  `user_status`   CHAR(1)      NOT NULL DEFAULT '1' COMMENT '用户状态: 1启用 2禁用 3冻结 4软删除',
  `status`        TINYINT      NOT NULL DEFAULT 1 COMMENT '账号状态: 1正常 0停用',
  `created_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_user_name` (`user_name`),
  KEY `idx_phone` (`phone`),
  KEY `idx_user_status` (`user_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='系统用户';

CREATE TABLE `sys_user_role` (
  `id`         BIGINT   NOT NULL AUTO_INCREMENT,
  `user_id`    BIGINT   NOT NULL COMMENT '用户ID',
  `role_id`    BIGINT   NOT NULL COMMENT '角色ID',
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_user_role` (`user_id`, `role_id`),
  CONSTRAINT `fk_user_role_user` FOREIGN KEY (`user_id`) REFERENCES `sys_user` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_user_role_role` FOREIGN KEY (`role_id`) REFERENCES `sys_role` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='用户角色关联';

CREATE TABLE `sys_menu` (
  `id`             BIGINT       NOT NULL AUTO_INCREMENT COMMENT '菜单ID',
  `parent_id`      BIGINT       NOT NULL DEFAULT 0 COMMENT '父菜单ID, 0为顶级',
  `route_name`     VARCHAR(128) NOT NULL COMMENT '路由 name, 如 dashboard_analysis',
  `path`           VARCHAR(255) NOT NULL COMMENT '路由 path',
  `component`      VARCHAR(32)           DEFAULT NULL COMMENT '布局组件: basic/blank/multi/self',
  `redirect`       VARCHAR(255)          DEFAULT NULL COMMENT '重定向地址',
  `title`          VARCHAR(128) NOT NULL COMMENT '菜单标题',
  `icon`           VARCHAR(128)          DEFAULT NULL COMMENT '图标',
  `local_icon`     VARCHAR(128)          DEFAULT NULL COMMENT '本地图标',
  `order_num`      INT          NOT NULL DEFAULT 0 COMMENT '排序',
  `hide`           TINYINT      NOT NULL DEFAULT 0 COMMENT '是否在菜单隐藏',
  `requires_auth`  TINYINT      NOT NULL DEFAULT 1 COMMENT '是否需要登录',
  `href`           VARCHAR(512)          DEFAULT NULL COMMENT '外链地址',
  `single_layout`  VARCHAR(32)           DEFAULT NULL COMMENT 'singleLayout: basic/blank',
  `permissions`    JSON                  DEFAULT NULL COMMENT '路由权限标识 JSON 数组',
  `menu_type`      TINYINT      NOT NULL DEFAULT 1 COMMENT '1目录 2菜单 3按钮',
  `status`         TINYINT      NOT NULL DEFAULT 1 COMMENT '1启用 0禁用',
  `created_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_route_name` (`route_name`),
  KEY `idx_parent_id` (`parent_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='系统菜单/路由';

CREATE TABLE `sys_permission` (
  `id`               BIGINT       NOT NULL AUTO_INCREMENT COMMENT '权限ID',
  `permission_code`  VARCHAR(128) NOT NULL COMMENT '权限编码',
  `permission_name`  VARCHAR(128) NOT NULL COMMENT '权限名称',
  `menu_id`          BIGINT                DEFAULT NULL COMMENT '所属菜单ID',
  `api_path`         VARCHAR(255)          DEFAULT NULL COMMENT '接口路径',
  `http_method`      VARCHAR(16)           DEFAULT NULL COMMENT 'HTTP 方法',
  `description`      VARCHAR(255)          DEFAULT NULL COMMENT '描述',
  `status`           TINYINT      NOT NULL DEFAULT 1 COMMENT '1启用 0禁用',
  `created_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at`       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_permission_code` (`permission_code`),
  KEY `idx_menu_id` (`menu_id`),
  CONSTRAINT `fk_permission_menu` FOREIGN KEY (`menu_id`) REFERENCES `sys_menu` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='系统权限';

CREATE TABLE `sys_role_menu` (
  `id`         BIGINT   NOT NULL AUTO_INCREMENT,
  `role_id`    BIGINT   NOT NULL COMMENT '角色ID',
  `menu_id`    BIGINT   NOT NULL COMMENT '菜单ID',
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_role_menu` (`role_id`, `menu_id`),
  CONSTRAINT `fk_role_menu_role` FOREIGN KEY (`role_id`) REFERENCES `sys_role` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_role_menu_menu` FOREIGN KEY (`menu_id`) REFERENCES `sys_menu` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='角色菜单关联';

CREATE TABLE `sys_role_permission` (
  `id`              BIGINT   NOT NULL AUTO_INCREMENT,
  `role_id`         BIGINT   NOT NULL COMMENT '角色ID',
  `permission_id`   BIGINT   NOT NULL COMMENT '权限ID',
  `created_at`      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_role_permission` (`role_id`, `permission_id`),
  CONSTRAINT `fk_role_perm_role` FOREIGN KEY (`role_id`) REFERENCES `sys_role` (`id`) ON DELETE CASCADE,
  CONSTRAINT `fk_role_perm_permission` FOREIGN KEY (`permission_id`) REFERENCES `sys_permission` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='角色权限关联';

CREATE TABLE `sys_user_token` (
  `id`             BIGINT       NOT NULL AUTO_INCREMENT,
  `user_id`        BIGINT       NOT NULL COMMENT '用户ID',
  `token`          VARCHAR(255) NOT NULL COMMENT '访问令牌',
  `refresh_token`  VARCHAR(255) NOT NULL COMMENT '刷新令牌',
  `created_at`     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_token` (`token`),
  UNIQUE KEY `uk_refresh_token` (`refresh_token`),
  KEY `idx_user_id` (`user_id`),
  CONSTRAINT `fk_user_token_user` FOREIGN KEY (`user_id`) REFERENCES `sys_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='用户登录令牌';

SET FOREIGN_KEY_CHECKS = 1;
