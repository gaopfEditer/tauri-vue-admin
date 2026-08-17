#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audit;
mod db;
mod license;
mod modbus_poll;
mod ops;
mod pharma;
mod server;
mod syslog;
mod sysops;

use std::path::PathBuf;
use std::time::Duration;

fn resolve_data_dir(app: &tauri::App) -> PathBuf {
  // Tauri 1.x：app_dir ≈ Roaming 应用数据目录
  if let Some(dir) = app.path_resolver().app_dir() {
    return dir;
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(parent) = exe.parent() {
      return parent.join("data");
    }
  }
  PathBuf::from("data")
}

fn main() {
  db::load_env();
  // 安装包无 .env 时，默认旁路授权，避免现场无法登录（正式交付可在 exe 旁放 .env 覆盖）
  if std::env::var("LICENSE_BYPASS").is_err() {
    std::env::set_var("LICENSE_BYPASS", "Y");
  }
  if std::env::var("DB_DRIVER").is_err() {
    std::env::set_var("DB_DRIVER", "sqlite");
  }

  syslog::info("system", "boot", "Tauri 应用启动");

  tauri::Builder::default()
    .setup(|app| {
      let data_dir = resolve_data_dir(app);
      let _ = std::fs::create_dir_all(&data_dir);
      let log_dir = data_dir.join("logs");
      let _ = std::fs::create_dir_all(&log_dir);
      if std::env::var("MODBUS_POLL_LOG_DIR").is_err() {
        std::env::set_var("MODBUS_POLL_LOG_DIR", log_dir.display().to_string());
      }

      let config = db::DbConfig::from_env().resolve_for_runtime(&data_dir);
      syslog::info_detail(
        "system",
        "boot",
        "准备连接数据库",
        serde_json::json!({
          "driver": config.driver,
          "sqlitePath": config.sqlite_path.display().to_string(),
          "dataDir": data_dir.display().to_string(),
        }),
      );

      tauri::async_runtime::spawn(async move {
        let pool = loop {
          match db::init(&config).await {
            Ok(pool) => {
              syslog::info("db", "ready", "数据库就绪");
              break pool;
            }
            Err(error) => {
              syslog::error(
                "db",
                "retry",
                &format!("数据库连接失败，3 秒后重试: {error}"),
              );
              tokio::time::sleep(Duration::from_secs(3)).await;
            }
          }
        };

        if let Err(error) = server::start(pool).await {
          syslog::error("system", "api", &format!("桌面 API 服务异常: {error}"));
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
