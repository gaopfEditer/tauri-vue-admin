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

use std::time::Duration;

fn main() {
  db::load_env();
  syslog::info("system", "boot", "Tauri 应用启动");

  tauri::Builder::default()
    .setup(|_app| {
      let config = db::DbConfig::from_env();
      syslog::info_detail(
        "system",
        "boot",
        "准备连接数据库",
        serde_json::json!({
          "driver": config.driver,
          "sqlitePath": config.sqlite_path.display().to_string(),
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
