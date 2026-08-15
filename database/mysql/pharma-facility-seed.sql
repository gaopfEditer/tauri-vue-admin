-- =============================================================================
-- 厂区组织种子 + 菜单（系统运维子项）
-- mysql -u root -p soybean_admin < database/mysql/pharma-facility-seed.sql
-- =============================================================================

SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

DELETE FROM `ph_facility_node`;

-- 一级：厂房 / 园区
INSERT INTO `ph_facility_node` (`id`, `parent_id`, `level`, `code`, `name`, `description`, `sort_order`) VALUES
(1, NULL, 1, 'WS-01', '一车间', '主生产厂房', 1),
(2, NULL, 1, 'AREA-B', 'B区', 'B 园区', 2);

-- 二级：区域 / 洁净区
INSERT INTO `ph_facility_node` (`id`, `parent_id`, `level`, `code`, `name`, `description`, `sort_order`) VALUES
(11, 1, 2, 'ISO5', '百级洁净区', 'ISO Class 5', 1),
(12, 1, 2, 'PKG', '封装测试区', '包装与测试', 2),
(21, 2, 2, 'ISO7', '万级洁净区', 'ISO Class 7', 1);

-- 三级：房间 Room
INSERT INTO `ph_facility_node` (`id`, `parent_id`, `level`, `code`, `name`, `description`, `sort_order`) VALUES
(111, 11, 3, 'R-3012', 'Room 3012', '灌装间', 1),
(112, 11, 3, 'R-301A', 'Room 301A', '缓冲间', 2),
(121, 12, 3, 'R-2101', 'Room 2101', '封装间', 1),
(211, 21, 3, 'R-B101', 'Room B101', '称量间', 1);

ALTER TABLE `ph_facility_node` AUTO_INCREMENT = 1000;

-- 菜单：系统运维子项
DELETE FROM `sys_role_menu` WHERE `menu_id` = 185;
DELETE FROM `sys_menu` WHERE `id` = 185;

INSERT INTO `sys_menu`
(`id`, `parent_id`, `route_name`, `path`, `component`, `redirect`, `title`, `icon`, `local_icon`, `order_num`, `hide`, `requires_auth`, `href`, `single_layout`, `permissions`, `menu_type`, `status`)
VALUES
(185, 180, 'system_facility', '/system/facility', 'self', NULL, '厂区组织', 'mdi:office-building-marker-outline', NULL, 5, 0, 1, NULL, NULL, NULL, 2, 1);

INSERT INTO `sys_role_menu` (`role_id`, `menu_id`) VALUES
(1, 185), (2, 185), (3, 185);

SET FOREIGN_KEY_CHECKS = 1;

SELECT 'facility' AS item, COUNT(*) AS total FROM `ph_facility_node`
UNION ALL SELECT 'menu185', COUNT(*) FROM `sys_menu` WHERE id = 185;
