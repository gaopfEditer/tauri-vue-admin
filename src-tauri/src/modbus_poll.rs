//! Modbus TCP 主站轮询 + 现场联调日志

use axum::{
  extract::{Query, State},
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;

use crate::server::{err, ok, ApiError, ApiResponse, AppState};

const MAX_LOGS: usize = 500;
const DEFAULT_PORT: u16 = 502;
const CONNECT_TIMEOUT_MS: u64 = 3000;
const IO_TIMEOUT_MS: u64 = 3000;

static POLL_ENABLED: AtomicBool = AtomicBool::new(true);
static POLL_CYCLE: AtomicU64 = AtomicU64::new(0);

/// 简易全局日志环（进程内）
fn log_store() -> &'static Mutex<VecDeque<PollLogEntry>> {
  use std::sync::OnceLock;
  static STORE: OnceLock<Mutex<VecDeque<PollLogEntry>>> = OnceLock::new();
  STORE.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_LOGS)))
}

fn tx_counter() -> &'static AtomicU64 {
  use std::sync::OnceLock;
  static TX: OnceLock<AtomicU64> = OnceLock::new();
  TX.get_or_init(|| AtomicU64::new(1))
}

#[derive(Clone, Serialize)]
pub struct PollLogEntry {
  pub time: String,
  pub level: String,
  #[serde(rename = "sensorId")]
  pub sensor_id: Option<i64>,
  #[serde(rename = "deviceCode")]
  pub device_code: Option<String>,
  pub host: Option<String>,
  pub action: String,
  pub message: String,
  pub detail: Option<Value>,
  #[serde(rename = "elapsedMs")]
  pub elapsed_ms: Option<u64>,
}

fn push_log(entry: PollLogEntry) {
  // 统一写入系统 Debug 日志环 + 文件
  crate::syslog::modbus_log(
    &entry.level,
    &entry.action,
    &entry.message,
    entry.sensor_id,
    entry.device_code.as_deref(),
    entry.host.as_deref(),
    entry.detail.clone(),
    entry.elapsed_ms,
  );
  if let Ok(mut q) = log_store().lock() {
    if q.len() >= MAX_LOGS {
      q.pop_front();
    }
    q.push_back(entry);
  }
}

fn poll_log_path() -> Result<PathBuf, ()> {
  let base = std::env::var("MODBUS_POLL_LOG_DIR").unwrap_or_else(|_| "logs".into());
  Ok(PathBuf::from(base).join("modbus-poll.log"))
}

fn now_str() -> String {
  Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn log_info(
  sensor_id: Option<i64>,
  device_code: Option<&str>,
  host: Option<&str>,
  action: &str,
  message: &str,
  detail: Option<Value>,
  elapsed_ms: Option<u64>,
) {
  push_log(PollLogEntry {
    time: now_str(),
    level: "INFO".into(),
    sensor_id,
    device_code: device_code.map(|s| s.to_string()),
    host: host.map(|s| s.to_string()),
    action: action.into(),
    message: message.into(),
    detail,
    elapsed_ms,
  });
}

fn log_warn(
  sensor_id: Option<i64>,
  device_code: Option<&str>,
  host: Option<&str>,
  action: &str,
  message: &str,
  detail: Option<Value>,
  elapsed_ms: Option<u64>,
) {
  push_log(PollLogEntry {
    time: now_str(),
    level: "WARN".into(),
    sensor_id,
    device_code: device_code.map(|s| s.to_string()),
    host: host.map(|s| s.to_string()),
    action: action.into(),
    message: message.into(),
    detail,
    elapsed_ms,
  });
}

fn log_error(
  sensor_id: Option<i64>,
  device_code: Option<&str>,
  host: Option<&str>,
  action: &str,
  message: &str,
  detail: Option<Value>,
  elapsed_ms: Option<u64>,
) {
  push_log(PollLogEntry {
    time: now_str(),
    level: "ERROR".into(),
    sensor_id,
    device_code: device_code.map(|s| s.to_string()),
    host: host.map(|s| s.to_string()),
    action: action.into(),
    message: message.into(),
    detail,
    elapsed_ms,
  });
}

fn looks_like_host(s: &str) -> bool {
  let t = s.trim();
  if t.is_empty() {
    return false;
  }
  // 排除占位 PLC1/PLC2
  if t.eq_ignore_ascii_case("PLC1") || t.eq_ignore_ascii_case("PLC2") {
    return false;
  }
  t.contains('.') || t.contains(':') || t.chars().all(|c| c.is_ascii_digit())
}

fn parse_endpoint(plc_ip: &str) -> Option<(String, u16)> {
  let t = plc_ip.trim();
  if !looks_like_host(t) {
    return None;
  }
  if let Some((h, p)) = t.rsplit_once(':') {
    if let Ok(port) = p.parse::<u16>() {
      if !h.is_empty() && !h.contains(':') {
        return Some((h.to_string(), port));
      }
    }
  }
  Some((t.to_string(), DEFAULT_PORT))
}

fn parse_unit_id(slave: &Option<String>) -> u8 {
  slave
    .as_ref()
    .and_then(|s| s.trim().parse::<u16>().ok())
    .filter(|n| (1..=247).contains(n))
    .map(|n| n as u8)
    .unwrap_or(1)
}

/// 极简 Modbus TCP 客户端（读保持/输入、写单寄存器）
struct ModbusTcp {
  stream: TcpStream,
  unit_id: u8,
}

impl ModbusTcp {
  async fn connect(host: &str, port: u16, unit_id: u8) -> Result<Self, String> {
    let addr: SocketAddr = format!("{host}:{port}")
      .parse()
      .map_err(|e| format!("地址无效 {host}:{port}: {e}"))?;
    let stream = timeout(
      Duration::from_millis(CONNECT_TIMEOUT_MS),
      TcpStream::connect(addr),
    )
    .await
    .map_err(|_| format!("连接超时 {host}:{port}"))?
    .map_err(|e| format!("连接失败 {host}:{port}: {e}"))?;
    let _ = stream.set_nodelay(true);
    Ok(Self { stream, unit_id })
  }

  async fn transaction(&mut self, pdu: &[u8]) -> Result<Vec<u8>, String> {
    let tid = tx_counter().fetch_add(1, Ordering::Relaxed) as u16;
    let len = (pdu.len() + 1) as u16;
    let mut req = Vec::with_capacity(7 + pdu.len());
    req.extend_from_slice(&tid.to_be_bytes());
    req.extend_from_slice(&0u16.to_be_bytes()); // protocol
    req.extend_from_slice(&len.to_be_bytes());
    req.push(self.unit_id);
    req.extend_from_slice(pdu);

    timeout(
      Duration::from_millis(IO_TIMEOUT_MS),
      self.stream.write_all(&req),
    )
    .await
    .map_err(|_| "写请求超时".to_string())?
    .map_err(|e| format!("写请求失败: {e}"))?;

    let mut header = [0u8; 7];
    timeout(
      Duration::from_millis(IO_TIMEOUT_MS),
      self.stream.read_exact(&mut header),
    )
    .await
    .map_err(|_| "读响应头超时".to_string())?
    .map_err(|e| format!("读响应头失败: {e}"))?;

    let resp_len = u16::from_be_bytes([header[4], header[5]]) as usize;
    if resp_len < 2 {
      return Err("响应长度异常".into());
    }
    let pdu_len = resp_len - 1; // exclude unit id already in header[6]
    let mut pdu_buf = vec![0u8; pdu_len];
    timeout(
      Duration::from_millis(IO_TIMEOUT_MS),
      self.stream.read_exact(&mut pdu_buf),
    )
    .await
    .map_err(|_| "读响应体超时".to_string())?
    .map_err(|e| format!("读响应体失败: {e}"))?;

    if pdu_buf.first().map(|b| b & 0x80) == Some(0x80) {
      let code = pdu_buf.get(1).copied().unwrap_or(0);
      return Err(format!("Modbus 异常码 0x{code:02X}"));
    }
    Ok(pdu_buf)
  }

  async fn read_holding(&mut self, address: u16, qty: u16) -> Result<Vec<u16>, String> {
    let mut pdu = vec![0x03];
    pdu.extend_from_slice(&address.to_be_bytes());
    pdu.extend_from_slice(&qty.to_be_bytes());
    let resp = self.transaction(&pdu).await?;
    if resp.len() < 2 || resp[0] != 0x03 {
      return Err("保持寄存器响应无效".into());
    }
    let bc = resp[1] as usize;
    if resp.len() < 2 + bc {
      return Err("保持寄存器数据不完整".into());
    }
    let mut out = Vec::new();
    let mut i = 2;
    while i + 1 < 2 + bc {
      out.push(u16::from_be_bytes([resp[i], resp[i + 1]]));
      i += 2;
    }
    Ok(out)
  }

  async fn read_input(&mut self, address: u16, qty: u16) -> Result<Vec<u16>, String> {
    let mut pdu = vec![0x04];
    pdu.extend_from_slice(&address.to_be_bytes());
    pdu.extend_from_slice(&qty.to_be_bytes());
    let resp = self.transaction(&pdu).await?;
    if resp.len() < 2 || resp[0] != 0x04 {
      return Err("输入寄存器响应无效".into());
    }
    let bc = resp[1] as usize;
    if resp.len() < 2 + bc {
      return Err("输入寄存器数据不完整".into());
    }
    let mut out = Vec::new();
    let mut i = 2;
    while i + 1 < 2 + bc {
      out.push(u16::from_be_bytes([resp[i], resp[i + 1]]));
      i += 2;
    }
    Ok(out)
  }

  async fn write_single(&mut self, address: u16, value: u16) -> Result<(), String> {
    let mut pdu = vec![0x06];
    pdu.extend_from_slice(&address.to_be_bytes());
    pdu.extend_from_slice(&value.to_be_bytes());
    let resp = self.transaction(&pdu).await?;
    if resp.len() < 5 || resp[0] != 0x06 {
      return Err("写单寄存器响应无效".into());
    }
    Ok(())
  }
}

fn u32_from_regs(high: u16, low: u16) -> u32 {
  ((high as u32) << 16) | (low as u32)
}

#[derive(FromRow)]
#[allow(dead_code)]
struct PollTarget {
  sensor_id: i64,
  custom_id: String,
  device_code: Option<String>,
  device_name: Option<String>,
  plc_ip: Option<String>,
  slave_address: Option<String>,
  protocol: String,
  update_interval_sec: i32,
  power_on: i8,
}

async fn load_targets(pool: &crate::db::DbPool) -> Vec<PollTarget> {
  sqlx::query_as(
    "SELECT s.id AS sensor_id, s.custom_id, c.device_code, c.device_name, c.plc_ip, c.slave_address,
            c.protocol, c.update_interval_sec, s.power_on
     FROM ph_sensor s
     INNER JOIN ph_sensor_device_config c ON c.sensor_id = s.id
     WHERE s.status = 1 AND s.category = 'particle'
       AND UPPER(c.protocol) LIKE 'MODBUSTCP%'",
  )
  .fetch_all(pool)
  .await
  .unwrap_or_default()
}

async fn poll_one(pool: &crate::db::DbPool, t: &PollTarget) -> bool {
  let code = t
    .device_code
    .clone()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| t.custom_id.clone());
  let Some(plc) = t.plc_ip.as_ref() else {
    log_warn(
      Some(t.sensor_id),
      Some(&code),
      None,
      "skip",
      "未配置 plcIp/设备 IP，跳过轮询",
      None,
      None,
    );
    return false;
  };
  let Some((host, port)) = parse_endpoint(plc) else {
    log_warn(
      Some(t.sensor_id),
      Some(&code),
      Some(plc),
      "skip",
      "plcIp 不是有效 IP/主机名（占位如 PLC1 不会轮询）",
      None,
      None,
    );
    return false;
  };

  let unit = parse_unit_id(&t.slave_address);
  let started = Instant::now();
  let endpoint = format!("{host}:{port}");

  log_info(
    Some(t.sensor_id),
    Some(&code),
    Some(&endpoint),
    "connect",
    &format!("开始连接 unit={unit}"),
    None,
    None,
  );

  let mut client = match ModbusTcp::connect(&host, port, unit).await {
    Ok(c) => c,
    Err(e) => {
      let ms = started.elapsed().as_millis() as u64;
      log_error(
        Some(t.sensor_id),
        Some(&code),
        Some(&endpoint),
        "connect",
        &e,
        None,
        Some(ms),
      );
      let _ = sqlx::query(
        "UPDATE ph_sensor SET com_alarm = 1, runtime_state = 'Offline', updated_at = datetime('now') WHERE id = ?",
      )
      .bind(t.sensor_id)
      .execute(pool)
      .await;
      return false;
    }
  };

  // ApexRp: 40003 Device Status → holding addr 2；40023 Flow → addr 22
  // 30010/30011 ch1、30012/30013 ch2（输入寄存器，addr = n-30001）
  let status_regs = match client.read_holding(2, 1).await {
    Ok(v) => v,
    Err(e) => {
      let ms = started.elapsed().as_millis() as u64;
      log_error(
        Some(t.sensor_id),
        Some(&code),
        Some(&endpoint),
        "read_status",
        &e,
        None,
        Some(ms),
      );
      let _ = sqlx::query(
        "UPDATE ph_sensor SET com_alarm = 1, runtime_state = 'Offline', updated_at = datetime('now') WHERE id = ?",
      )
      .bind(t.sensor_id)
      .execute(pool)
      .await;
      return false;
    }
  };
  let status_word = status_regs.first().copied().unwrap_or(0);
  let flow_regs = client.read_holding(22, 1).await.unwrap_or_default();
  let flow_raw = flow_regs.first().copied().unwrap_or(0);
  let flow_val = (flow_raw as f64) / 100.0;

  let particle = client.read_input(9, 4).await.unwrap_or_default();
  let ch1 = if particle.len() >= 2 {
    u32_from_regs(particle[0], particle[1])
  } else {
    0
  };
  let ch2 = if particle.len() >= 4 {
    u32_from_regs(particle[2], particle[3])
  } else {
    0
  };

  let running = status_word & 0x01 != 0;
  let sampling = status_word & 0x02 != 0;
  let flow_alarm = status_word & (1 << 12) != 0;
  let runtime = if sampling {
    "Sampling"
  } else if running {
    "Idle"
  } else if t.power_on == 1 {
    "Idle"
  } else {
    "Idle"
  };

  let last_value = json!({
    "0.5um": ch1,
    "5.0um": ch2,
    "statusWord": status_word,
    "flow": flow_val,
    "flowRaw": flow_raw
  });

  let _ = sqlx::query(
    "UPDATE ph_sensor
     SET com_alarm = 0,
         flow_calc_alarm = ?,
         runtime_state = ?,
         last_value = ?,
         power_on = CASE WHEN ? THEN 1 ELSE power_on END,
         updated_at = datetime('now')
     WHERE id = ?",
  )
  .bind(if flow_alarm { 1 } else { 0 })
  .bind(runtime)
  .bind(&last_value)
  .bind(running || sampling)
  .bind(t.sensor_id)
  .execute(pool)
  .await;

  let _ = sqlx::query("UPDATE ph_sensor_device_config SET flow_rate = ?, updated_at = datetime('now') WHERE sensor_id = ?")
    .bind(flow_val)
    .bind(t.sensor_id)
    .execute(pool)
    .await;

  let ms = started.elapsed().as_millis() as u64;
  log_info(
    Some(t.sensor_id),
    Some(&code),
    Some(&endpoint),
    "poll_ok",
    &format!("status=0x{status_word:04X} flow={flow_val} ch1={ch1} ch2={ch2}"),
    Some(json!({
      "unitId": unit,
      "statusWord": status_word,
      "running": running,
      "sampling": sampling,
      "flowAlarm": flow_alarm,
      "lastValue": last_value
    })),
    Some(ms),
  );
  true
}

/// 写命令寄存器 40002（addr=1）：11=Start，12=Stop
pub async fn send_power_command(
  pool: &crate::db::DbPool,
  sensor_id: i64,
  action: &str,
) -> Result<(), String> {
  let row: Option<(Option<String>, Option<String>, String)> = sqlx::query_as(
    "SELECT c.plc_ip, c.slave_address, c.protocol
     FROM ph_sensor_device_config c WHERE c.sensor_id = ?",
  )
  .bind(sensor_id)
  .fetch_optional(pool)
  .await
  .map_err(|e| e.to_string())?;

  let Some((plc_ip, slave, protocol)) = row else {
    return Err("无设备配置".into());
  };
  if !protocol.to_uppercase().contains("MODBUSTCP") {
    return Err(format!("协议 {protocol} 暂不支持现场写命令"));
  }
  let plc = plc_ip.ok_or("未配置设备 IP")?;
  let (host, port) = parse_endpoint(&plc).ok_or("设备 IP 无效")?;
  let unit = parse_unit_id(&slave);
  let cmd: u16 = match action {
    "on" => 11,
    "off" => 12,
    "cleaning" => 12, // 先停采；Cleaning 由软件态表示
    _ => return Err("无效动作".into()),
  };

  let endpoint = format!("{host}:{port}");
  let started = Instant::now();
  let mut client = ModbusTcp::connect(&host, port, unit).await?;
  client.write_single(1, cmd).await?; // 40002
  log_info(
    Some(sensor_id),
    None,
    Some(&endpoint),
    "command",
    &format!("写 40002={cmd} ({action})"),
    Some(json!({ "command": cmd, "action": action })),
    Some(started.elapsed().as_millis() as u64),
  );
  Ok(())
}

pub fn spawn_poller(pool: crate::db::DbPool) {
  let enabled = std::env::var("MODBUS_POLL_ENABLED").unwrap_or_else(|_| "Y".into());
  if enabled != "Y" && enabled != "1" && !enabled.eq_ignore_ascii_case("true") {
    POLL_ENABLED.store(false, Ordering::SeqCst);
    log_warn(
      None,
      None,
      None,
      "startup",
      "MODBUS_POLL_ENABLED 未开启，跳过后台轮询",
      None,
      None,
    );
    return;
  }
  POLL_ENABLED.store(true, Ordering::SeqCst);
  let default_interval: u64 = std::env::var("MODBUS_POLL_INTERVAL_SEC")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(5)
    .max(1);

  log_info(
    None,
    None,
    None,
    "startup",
    &format!("Modbus 轮询已启动，默认周期 {default_interval}s，日志目录见 MODBUS_POLL_LOG_DIR/logs"),
    Some(json!({ "logFile": poll_log_path().ok() })),
    None,
  );

  tokio::spawn(async move {
    // 防止重叠周期
    let tick_lock = Arc::new(AsyncMutex::new(()));
    loop {
      if !POLL_ENABLED.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(2)).await;
        continue;
      }
      let _guard = tick_lock.lock().await;
      let cycle = POLL_CYCLE.fetch_add(1, Ordering::SeqCst) + 1;
      let targets = load_targets(&pool).await;
      log_info(
        None,
        None,
        None,
        "cycle",
        &format!("第 {cycle} 轮，目标 {} 台", targets.len()),
        None,
        None,
      );
      let mut ok_n = 0;
      for t in &targets {
        if poll_one(&pool, t).await {
          ok_n += 1;
        }
        // 小间隔避免打爆设备
        tokio::time::sleep(Duration::from_millis(200)).await;
      }
      log_info(
        None,
        None,
        None,
        "cycle_done",
        &format!("第 {cycle} 轮完成：成功 {ok_n}/{}", targets.len()),
        None,
        None,
      );
      drop(_guard);
      tokio::time::sleep(Duration::from_secs(default_interval)).await;
    }
  });
}

#[derive(Deserialize)]
struct LogsQuery {
  limit: Option<usize>,
  level: Option<String>,
}

#[derive(Deserialize)]
struct TestBody {
  #[serde(rename = "sensorId")]
  sensor_id: i64,
}

#[derive(Deserialize)]
struct EnableBody {
  enabled: bool,
}

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/api/device-poll/logs", get(get_logs))
    .route("/api/device-poll/status", get(get_status))
    .route("/api/device-poll/test", post(test_one))
    .route("/api/device-poll/enabled", post(set_enabled).get(get_enabled))
}

async fn get_logs(Query(q): Query<LogsQuery>) -> Json<ApiResponse<Vec<PollLogEntry>>> {
  let limit = q.limit.unwrap_or(100).clamp(1, MAX_LOGS);
  let level = q.level.map(|s| s.to_uppercase());
  let list = log_store()
    .lock()
    .map(|q| {
      q.iter()
        .rev()
        .filter(|e| level.as_ref().map(|l| e.level == *l).unwrap_or(true))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  ok(list)
}

async fn get_status() -> Json<ApiResponse<Value>> {
  ok(json!({
    "enabled": POLL_ENABLED.load(Ordering::SeqCst),
    "cycle": POLL_CYCLE.load(Ordering::SeqCst),
    "logFile": poll_log_path().ok(),
    "logCount": log_store().lock().map(|q| q.len()).unwrap_or(0),
    "systemLog": crate::syslog::snapshot_status(),
  }))
}

async fn get_enabled() -> Json<ApiResponse<Value>> {
  ok(json!({ "enabled": POLL_ENABLED.load(Ordering::SeqCst) }))
}

async fn set_enabled(Json(body): Json<EnableBody>) -> Json<ApiResponse<Value>> {
  POLL_ENABLED.store(body.enabled, Ordering::SeqCst);
  log_info(
    None,
    None,
    None,
    "config",
    &format!("轮询开关 => {}", body.enabled),
    None,
    None,
  );
  ok(json!({ "enabled": body.enabled }))
}

async fn test_one(
  State(state): State<AppState>,
  Json(body): Json<TestBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let targets = load_targets(&state.pool).await;
  let Some(t) = targets.into_iter().find(|x| x.sensor_id == body.sensor_id) else {
    return Err(err(4040, "未找到可轮询的粒子设备配置（需 ModbusTCP + 有效 IP）"));
  };
  log_info(
    Some(t.sensor_id),
    t.device_code.as_deref(),
    t.plc_ip.as_deref(),
    "manual_test",
    "现场手动单次轮询",
    None,
    None,
  );
  let ok_flag = poll_one(&state.pool, &t).await;
  Ok(ok(json!({ "ok": ok_flag, "sensorId": body.sensor_id.to_string() })))
}
