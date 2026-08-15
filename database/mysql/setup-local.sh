#!/usr/bin/env bash
# 本地 MySQL 一键建库 + 建表 + 种子数据 (不用 Docker)
#
# 用法 (在项目根目录):
#   ./database/mysql/setup-local.sh
#   ./database/mysql/setup-local.sh -u root -p你的密码
#
# 环境变量 (可选):
#   MYSQL_HOST=127.0.0.1  MYSQL_PORT=3306  MYSQL_USER=root  MYSQL_PWD=xxx

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_USER="${MYSQL_USER:-root}"

mysql_cmd=(mysql -h "$MYSQL_HOST" -P "$MYSQL_PORT" -u "$MYSQL_USER")

if [[ -n "${MYSQL_PWD:-}" ]]; then
  export MYSQL_PWD
fi

if [[ $# -gt 0 ]]; then
  mysql_cmd+=("$@")
fi

echo ">> 创建数据库 soybean_admin ..."
"${mysql_cmd[@]}" < "$ROOT_DIR/database/mysql/install.sql"

echo ">> 建表 ..."
"${mysql_cmd[@]}" soybean_admin < "$ROOT_DIR/database/mysql/schema.sql"

echo ">> 写入种子数据 ..."
"${mysql_cmd[@]}" soybean_admin < "$ROOT_DIR/database/mysql/init-data.sql"

echo ">> 完成。可登录账号: Soybean/soybean123  Super/super123  Admin/admin123  User01/user01123"
