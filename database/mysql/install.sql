-- =============================================================================
-- 一键安装入口 (仅命令行)
-- =============================================================================
-- 推荐在项目根目录执行:
--   ./database/mysql/setup-local.sh
--
-- 或手动三步:
--   mysql -u root -p -e "CREATE DATABASE IF NOT EXISTS soybean_admin DEFAULT CHARSET utf8mb4 COLLATE utf8mb4_unicode_ci;"
--   mysql -u root -p soybean_admin < database/mysql/schema.sql
--   mysql -u root -p soybean_admin < database/mysql/init-data.sql
-- =============================================================================

CREATE DATABASE IF NOT EXISTS `soybean_admin`
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;
