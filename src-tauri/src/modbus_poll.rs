//! Modbus TCP / RTU(RS-485) 主站轮询 + 现场联调日志

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
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::server::{err, ok, ApiError, ApiResponse, AppState};

const MAX_LOGS: usize = 500;
const DEFAULT_PORT: u16 = 502;
const CONNECT_TIMEOUT_MS: u64 = 3000;
const IO_TIMEOUT_MS: u64 = 3000;
const RTU_IO_TIMEOUT_MS: u64 = 2000;
const DEFAULT_RTU_BAUD: u32 = 19200;
const DEFAULT_RTU_INPUT_COUNT: u16 = 16;

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

fn modbus_crc16(data: &[u8]) -> u16 {
  let mut crc = 0xFFFFu16;
  for &b in data {
    crc ^= u16::from(b);
    for _ in 0..8 {
      if crc & 0x0001 != 0 {
        crc = (crc >> 1) ^ 0xA001;
      } else {
        crc >>= 1;
      }
    }
  }
  crc
}

fn parse_serial_endpoint(raw: &str) -> Option<(String, u32)> {
  let t = raw.trim();
  if t.is_empty() {
    return None;
  }
  // 排除 TCP 占位
  if t.eq_ignore_ascii_case("PLC1") || t.eq_ignore_ascii_case("PLC2") {
    return None;
  }
  let default_baud = std::env::var("MODBUS_RTU_BAUD")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(DEFAULT_RTU_BAUD);

  // COM3@19200 或 COM3:19200（后者若像 IP 则不走这里）
  if let Some((port, baud_s)) = t.rsplit_once('@') {
    let baud = baud_s.parse().unwrap_or(default_baud);
    let port = port.trim();
    if !port.is_empty() {
      return Some((port.to_string(), baud));
    }
  }
  // 纯串口名：COM3 / /dev/ttyUSB0
  if t.to_ascii_uppercase().starts_with("COM") || t.starts_with("/dev/") {
    return Some((t.to_string(), default_baud));
  }
  None
}

fn normalize_serial_port(port: &str) -> String {
  // Windows COM10+ 需要 \\.\COMx；COM3 也可统一加前缀更稳
  let u = port.to_ascii_uppercase();
  if u.starts_with("COM") && !port.starts_with(r"\\.\") {
    format!(r"\\.\{port}")
  } else {
    port.to_string()
  }
}

fn regs_to_f32_abcd(hi: u16, lo: u16) -> f32 {
  let raw = ((hi as u32) << 16) | (lo as u32);
  f32::from_bits(raw)
}

fn regs_to_f32_cdab(hi: u16, lo: u16) -> f32 {
  regs_to_f32_abcd(lo, hi)
}

/// Modbus RTU（RS-485 / USB 串口）客户端：功能码 04 等
struct ModbusRtu {
  stream: SerialStream,
  unit_id: u8,
}

impl ModbusRtu {
  async fn open(port: &str, baud: u32, unit_id: u8) -> Result<Self, String> {
    let path = normalize_serial_port(port);
    let stream = tokio_serial::new(&path, baud)
      .data_bits(tokio_serial::DataBits::Eight)
      .parity(tokio_serial::Parity::None)
      .stop_bits(tokio_serial::StopBits::One)
      .timeout(Duration::from_millis(RTU_IO_TIMEOUT_MS))
      .open_native_async()
      .map_err(|e| format!("打开串口 {port}@{baud} 失败: {e}"))?;
    Ok(Self { stream, unit_id })
  }

  async fn transaction(&mut self, pdu: &[u8]) -> Result<Vec<u8>, String> {
    let mut frame = Vec::with_capacity(pdu.len() + 3);
    frame.push(self.unit_id);
    frame.extend_from_slice(pdu);
    let crc = modbus_crc16(&frame);
    frame.push((crc & 0xFF) as u8);
    frame.push((crc >> 8) as u8);

    // 发送前短暂静默，利于 485 半双工切换
    tokio::time::sleep(Duration::from_millis(20)).await;

    timeout(
      Duration::from_millis(RTU_IO_TIMEOUT_MS),
      self.stream.write_all(&frame),
    )
    .await
    .map_err(|_| "RTU 写请求超时".to_string())?
    .map_err(|e| format!("RTU 写请求失败: {e}"))?;
    let _ = self.stream.flush().await;

    let data = self.read_frame().await?;
    let n = data.len();
    if n < 5 {
      return Err(format!("RTU 响应过短 ({n} bytes)"));
    }
    let body = &data[..n - 2];
    let got = u16::from(data[n - 2]) | (u16::from(data[n - 1]) << 8);
    let expect = modbus_crc16(body);
    if got != expect {
      return Err(format!("RTU CRC 错误 expect=0x{expect:04X} got=0x{got:04X}"));
    }
    if body[0] != self.unit_id {
      return Err(format!("RTU 从站地址不匹配: {}", body[0]));
    }
    let pdu_resp = &body[1..];
    if pdu_resp.first().map(|b| b & 0x80) == Some(0x80) {
      let code = pdu_resp.get(1).copied().unwrap_or(0);
      return Err(format!("Modbus 异常码 0x{code:02X}"));
    }
    Ok(pdu_resp.to_vec())
  }

  async fn read_frame(&mut self) -> Result<Vec<u8>, String> {
    let mut acc = Vec::new();
    let deadline = Instant::now() + Duration::from_millis(RTU_IO_TIMEOUT_MS);
    while Instant::now() < deadline {
      let mut tmp = [0u8; 64];
      match timeout(Duration::from_millis(80), self.stream.read(&mut tmp)).await {
        Ok(Ok(0)) => {
          if !acc.is_empty() {
            break;
          }
        }
        Ok(Ok(n)) => {
          acc.extend_from_slice(&tmp[..n]);
          if acc.len() >= 5 {
            if acc.get(1).map(|b| b & 0x80) == Some(0x80) {
              // 异常帧：addr+fc+ex+crc = 5
              if acc.len() >= 5 {
                return Ok(acc[..5].to_vec());
              }
            } else if acc.get(1) == Some(&0x04) && acc.len() >= 3 {
              let bc = acc[2] as usize;
              let expect = 3 + bc + 2;
              if acc.len() >= expect {
                return Ok(acc[..expect].to_vec());
              }
            }
          }
        }
        Ok(Err(e)) => return Err(format!("RTU 读响应失败: {e}")),
        Err(_) => {
          // 字节间隙超时：若已有数据则结束
          if !acc.is_empty() {
            break;
          }
        }
      }
    }
    if acc.is_empty() {
      Err("RTU 读响应超时".into())
    } else {
      Ok(acc)
    }
  }

  async fn read_input(&mut self, address: u16, qty: u16) -> Result<Vec<u16>, String> {    let mut pdu = vec![0x04];
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
       AND (
         UPPER(c.protocol) LIKE 'MODBUSTCP%'
         OR UPPER(c.protocol) LIKE 'MODBUSRTU%'
       )",
  )
  .fetch_all(pool)
  .await
  .unwrap_or_default()
}

fn device_code_of(t: &PollTarget) -> String {
  t.device_code
    .clone()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| t.custom_id.clone())
}

async fn poll_one(pool: &crate::db::DbPool, t: &PollTarget) -> bool {
  let proto = t.protocol.to_uppercase();
  if proto.contains("MODBUSRTU") {
    poll_one_rtu(pool, t).await
  } else {
    poll_one_tcp(pool, t).await
  }
}

/// RS-485 / USB-串口：FC04 Input Register（与 Modscan32 验证参数一致）
async fn poll_one_rtu(pool: &crate::db::DbPool, t: &PollTarget) -> bool {
  let code = device_code_of(t);
  let Some(raw_port) = t.plc_ip.as_ref() else {
    log_warn(
      Some(t.sensor_id),
      Some(&code),
      None,
      "skip",
      "ModbusRTU 未配置串口（请在「PLC IP」填 COM3 或 COM3@19200）",
      None,
      None,
    );
    return false;
  };
  let Some((port, baud)) = parse_serial_endpoint(raw_port) else {
    log_warn(
      Some(t.sensor_id),
      Some(&code),
      Some(raw_port),
      "skip",
      "串口名无效（示例：COM3 / COM3@19200）",
      None,
      None,
    );
    return false;
  };

  let unit = parse_unit_id(&t.slave_address);
  let count = std::env::var("MODBUS_RTU_INPUT_COUNT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(DEFAULT_RTU_INPUT_COUNT)
    .clamp(10, 20);
  let started = Instant::now();
  let endpoint = format!("{port}@{baud}");

  log_info(
    Some(t.sensor_id),
    Some(&code),
    Some(&endpoint),
    "connect",
    &format!("RTU 打开串口 unit={unit} FC04 addr=0 count={count}"),
    None,
    None,
  );

  let mut client = match ModbusRtu::open(&port, baud, unit).await {
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

  let regs = match client.read_input(0, count).await {
    Ok(v) => v,
    Err(e) => {
      let ms = started.elapsed().as_millis() as u64;
      log_error(
        Some(t.sensor_id),
        Some(&code),
        Some(&endpoint),
        "read_input",
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

  let f_abcd_01 = if regs.len() >= 2 {
    Some(regs_to_f32_abcd(regs[0], regs[1]))
  } else {
    None
  };
  let f_cdab_01 = if regs.len() >= 2 {
    Some(regs_to_f32_cdab(regs[0], regs[1]))
  } else {
    None
  };
  let f_abcd_23 = if regs.len() >= 4 {
    Some(regs_to_f32_abcd(regs[2], regs[3]))
  } else {
    None
  };

  let last_value = json!({
    "protocol": "ModbusRTU",
    "port": port,
    "baud": baud,
    "unitId": unit,
    "raw": regs,
    "floatABCD": { "reg0_1": f_abcd_01, "reg2_3": f_abcd_23 },
    "floatCDAB": { "reg0_1": f_cdab_01 },
    // 卡片优先展示：原始前两字 + ABCD 浮点（现场按手册选字序）
    "0.5um": f_abcd_01,
    "5.0um": f_abcd_23,
  });

  let _ = sqlx::query(
    "UPDATE ph_sensor
     SET com_alarm = 0,
         runtime_state = 'Idle',
         last_value = ?,
         updated_at = datetime('now')
     WHERE id = ?",
  )
  .bind(&last_value)
  .bind(t.sensor_id)
  .execute(pool)
  .await;

  let ms = started.elapsed().as_millis() as u64;
  let raw_preview: Vec<String> = regs
    .iter()
    .enumerate()
    .take(8)
    .map(|(i, v)| format!("[{i}]={v}"))
    .collect();
  log_info(
    Some(t.sensor_id),
    Some(&code),
    Some(&endpoint),
    "poll_ok",
    &format!(
      "RTU FC04 ok raw={} abcd01={:?} cdab01={:?}",
      raw_preview.join(" "),
      f_abcd_01,
      f_cdab_01
    ),
    Some(json!({ "lastValue": last_value })),
    Some(ms),
  );
  true
}

async fn poll_one_tcp(pool: &crate::db::DbPool, t: &PollTarget) -> bool {
  let code = device_code_of(t);
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
    return Err(err(4040, "未找到可轮询的粒子设备配置（需 ModbusTCP+IP 或 ModbusRTU+COM）"));
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
