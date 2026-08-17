use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub type DbPool = Pool<Sqlite>;

const SCHEMA_SQL: &str = include_str!("../../database/sqlite/field-schema.sql");
const SEED_SQL: &str = include_str!("../../database/sqlite/field-seed.sql");

pub struct DbConfig {
  pub driver: String,
  pub sqlite_path: PathBuf,
  // MySQL 字段保留，便于以后切回
  #[allow(dead_code)]
  pub host: String,
  #[allow(dead_code)]
  pub port: u16,
  #[allow(dead_code)]
  pub database: String,
  #[allow(dead_code)]
  pub user: String,
  #[allow(dead_code)]
  pub password: String,
}

impl DbConfig {
  pub fn from_env() -> Self {
    Self {
      driver: std::env::var("DB_DRIVER").unwrap_or_else(|_| "sqlite".to_string()),
      sqlite_path: PathBuf::from(
        std::env::var("DB_SQLITE_PATH").unwrap_or_else(|_| "field-test.db".to_string()),
      ),
      host: std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
      port: std::env::var("DB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3306),
      database: std::env::var("DB_NAME").unwrap_or_else(|_| "soybean_admin".to_string()),
      user: std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string()),
      password: std::env::var("DB_PASSWORD").unwrap_or_default(),
    }
  }

  /// 安装包/现场：把相对路径落到可写的应用数据目录
  pub fn resolve_for_runtime(mut self, data_dir: &Path) -> Self {
    if self.sqlite_path.is_relative() {
      self.sqlite_path = data_dir.join(&self.sqlite_path);
    }
    self
  }
}

pub fn load_env() {
  // 1) 开发：仓库根 .env
  let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
  let connection_env = manifest.join("../database/connection.env");
  if connection_env.exists() {
    let _ = dotenvy::from_path(&connection_env);
  }
  let _ = dotenvy::from_path_override(manifest.join("../.env"));

  // 2) 安装包：exe 同目录 .env（现场可放配置）
  if let Ok(exe) = std::env::current_exe() {
    if let Some(dir) = exe.parent() {
      let _ = dotenvy::from_path_override(dir.join(".env"));
    }
  }

  // 3) 当前工作目录
  let _ = dotenvy::from_path_override(PathBuf::from(".env"));
}

async fn exec_sql_script(pool: &DbPool, raw: &str) -> Result<(), sqlx::Error> {
  for stmt in split_statements(raw) {
    sqlx::query(&stmt).execute(pool).await?;
  }
  Ok(())
}

fn split_statements(sql: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut buf = String::new();
  for line in sql.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with("--") {
      continue;
    }
    buf.push_str(line);
    buf.push('\n');
    if trimmed.ends_with(';') {
      let s = buf.trim().trim_end_matches(';').trim().to_string();
      if !s.is_empty() {
        out.push(s);
      }
      buf.clear();
    }
  }
  let s = buf.trim().trim_end_matches(';').trim().to_string();
  if !s.is_empty() {
    out.push(s);
  }
  out
}

async fn bootstrap_schema_and_seed(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  crate::syslog::info("db", "schema", "应用内置 SQLite schema");
  exec_sql_script(pool, SCHEMA_SQL).await?;

  let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sys_user")
    .fetch_one(pool)
    .await?;

  if user_count.0 == 0 {
    crate::syslog::info("db", "seed", "空库，写入内置种子数据");
    exec_sql_script(pool, SEED_SQL).await?;
  } else {
    crate::syslog::info(
      "db",
      "seed",
      &format!("已有用户 {} 个，跳过种子", user_count.0),
    );
  }

  let _ = sqlx::query(
    "INSERT OR IGNORE INTO sys_menu
     (id, parent_id, route_name, path, component, title, icon, order_num, hide, requires_auth, menu_type, status)
     VALUES (186, 180, 'system_license', '/system/license', 'self', '授权管理', 'mdi:license', 6, 0, 1, 2, 1)",
  )
  .execute(pool)
  .await;

  let _ = sqlx::query(
    "INSERT OR IGNORE INTO sys_menu
     (id, parent_id, route_name, path, component, title, icon, order_num, hide, requires_auth, menu_type, status)
     VALUES (187, 180, 'system_device-poll', '/system/device-poll', 'self', '设备轮询联调', 'mdi:lan-connect', 7, 0, 1, 2, 1)",
  )
  .execute(pool)
  .await;

  let _ = sqlx::query(
    "INSERT OR IGNORE INTO sys_role_menu (role_id, menu_id)
     SELECT r.id, m.id FROM sys_role r CROSS JOIN sys_menu m
     WHERE r.role_code IN ('super','admin') AND m.route_name IN ('system_license','system_device-poll')",
  )
  .execute(pool)
  .await;

  Ok(())
}

pub async fn init(config: &DbConfig) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
  if config.driver.to_lowercase() != "sqlite" {
    return Err(format!(
      "当前仅启用 SQLite 现场模式（DB_DRIVER=sqlite），收到: {}",
      config.driver
    )
    .into());
  }

  if let Some(parent) = config.sqlite_path.parent() {
    std::fs::create_dir_all(parent)?;
  }

  // Windows 路径在 sqlite URL 里需要用正斜杠更稳妥
  let path_str = config.sqlite_path.display().to_string().replace('\\', "/");
  let url = format!("sqlite:{path_str}?mode=rwc");
  crate::syslog::info_detail(
    "db",
    "connect",
    "连接 SQLite",
    serde_json::json!({
      "path": config.sqlite_path.display().to_string(),
      "url": url,
    }),
  );

  let options = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await
    .map_err(|e| {
      crate::syslog::error("db", "connect", &format!("SQLite 连接失败: {e}"));
      e
    })?;

  sqlx::query("PRAGMA foreign_keys = ON")
    .execute(&pool)
    .await?;
  sqlx::query("SELECT 1").execute(&pool).await?;

  crate::syslog::info(
    "db",
    "connect",
    &format!("SQLite 已连接: {}", config.sqlite_path.display()),
  );

  bootstrap_schema_and_seed(&pool).await?;
  Ok(pool)
}

pub fn md5_hex(input: &str) -> String {
  format!("{:x}", md5::compute(input.as_bytes()))
}
