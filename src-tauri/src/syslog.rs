//! 统一运行日志（系统 / 数据库 / Modbus），供 Debug 面板拉取

use axum::{extract::Query, routing::get, Json, Router};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::server::{ok, ApiResponse, AppState};

const MAX_LOGS: usize = 800;

#[derive(Clone, Serialize)]
pub struct LogEntry {
  pub time: String,
  pub level: String,
  pub source: String,
  pub action: String,
  pub message: String,
  #[serde(rename = "sensorId", skip_serializing_if = "Option::is_none")]
  pub sensor_id: Option<i64>,
  #[serde(rename = "deviceCode", skip_serializing_if = "Option::is_none")]
  pub device_code: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub host: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub detail: Option<Value>,
  #[serde(rename = "elapsedMs", skip_serializing_if = "Option::is_none")]
  pub elapsed_ms: Option<u64>,
}

fn store() -> &'static Mutex<VecDeque<LogEntry>> {
  static STORE: OnceLock<Mutex<VecDeque<LogEntry>>> = OnceLock::new();
  STORE.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_LOGS)))
}

fn now_str() -> String {
  Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn log_dir() -> PathBuf {
  PathBuf::from(std::env::var("MODBUS_POLL_LOG_DIR").unwrap_or_else(|_| "logs".into()))
}

fn append_file(filename: &str, line: &str) {
  let path = log_dir().join(filename);
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Ok(mut f) = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
  {
    let _ = f.write_all(line.as_bytes());
  }
}

pub fn push(
  level: &str,
  source: &str,
  action: &str,
  message: &str,
  sensor_id: Option<i64>,
  device_code: Option<&str>,
  host: Option<&str>,
  detail: Option<Value>,
  elapsed_ms: Option<u64>,
) {
  let entry = LogEntry {
    time: now_str(),
    level: level.to_uppercase(),
    source: source.into(),
    action: action.into(),
    message: message.into(),
    sensor_id,
    device_code: device_code.map(|s| s.to_string()),
    host: host.map(|s| s.to_string()),
    detail,
    elapsed_ms,
  };

  println!(
    "[{}][{}][{}] {} {}",
    entry.level, entry.source, entry.time, entry.action, entry.message
  );

  let file = if source == "modbus" {
    "modbus-poll.log"
  } else {
    "app.log"
  };
  append_file(
    file,
    &format!(
      "{} | {:5} | {:8} | {} | {} | sid={:?} | {:?}\n",
      entry.time, entry.level, entry.source, entry.action, entry.message, entry.sensor_id, entry.detail
    ),
  );

  if let Ok(mut q) = store().lock() {
    if q.len() >= MAX_LOGS {
      q.pop_front();
    }
    q.push_back(entry);
  }
}

pub fn info(source: &str, action: &str, message: &str) {
  push("INFO", source, action, message, None, None, None, None, None);
}

pub fn warn(source: &str, action: &str, message: &str) {
  push("WARN", source, action, message, None, None, None, None, None);
}

pub fn error(source: &str, action: &str, message: &str) {
  push("ERROR", source, action, message, None, None, None, None, None);
}

pub fn info_detail(source: &str, action: &str, message: &str, detail: Value) {
  push(
    "INFO",
    source,
    action,
    message,
    None,
    None,
    None,
    Some(detail),
    None,
  );
}

pub fn modbus_log(
  level: &str,
  action: &str,
  message: &str,
  sensor_id: Option<i64>,
  device_code: Option<&str>,
  host: Option<&str>,
  detail: Option<Value>,
  elapsed_ms: Option<u64>,
) {
  push(
    level,
    "modbus",
    action,
    message,
    sensor_id,
    device_code,
    host,
    detail,
    elapsed_ms,
  );
}

#[derive(Deserialize)]
struct LogsQuery {
  limit: Option<usize>,
  level: Option<String>,
  source: Option<String>,
}

pub fn routes() -> Router<AppState> {
  Router::new().route("/api/debug/logs", get(get_logs))
}

async fn get_logs(Query(q): Query<LogsQuery>) -> Json<ApiResponse<Vec<LogEntry>>> {
  let limit = q.limit.unwrap_or(200).clamp(1, MAX_LOGS);
  let level = q.level.map(|s| s.to_uppercase());
  let source = q.source.map(|s| s.to_lowercase());
  let list = store()
    .lock()
    .map(|q| {
      q.iter()
        .rev()
        .filter(|e| level.as_ref().map(|l| e.level == *l).unwrap_or(true))
        .filter(|e| source.as_ref().map(|s| e.source == *s).unwrap_or(true))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  ok(list)
}

pub fn snapshot_status() -> Value {
  json!({
    "logCount": store().lock().map(|q| q.len()).unwrap_or(0),
    "logDir": log_dir().display().to_string(),
    "files": ["app.log", "modbus-poll.log"]
  })
}
