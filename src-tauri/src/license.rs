//! 时间锁 License + 设备 MAC 绑定

use axum::{
  extract::{Request, State},
  http::{Method, StatusCode},
  middleware::Next,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chrono::{Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::db;
use crate::server::{err, ok, ApiError, ApiResponse, AppState};

const LICENSE_BIZ_CODE: i64 = 40301;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
  pub v: i32,
  #[serde(rename = "licenseId")]
  pub license_id: String,
  pub customer: String,
  #[serde(rename = "validFrom")]
  pub valid_from: String,
  #[serde(rename = "validUntil")]
  pub valid_until: String,
  #[serde(rename = "deviceLock")]
  pub device_lock: bool,
}

#[derive(FromRow)]
struct LicenseRow {
  id: i64,
  license_id: String,
  customer_name: String,
  valid_from: NaiveDate,
  valid_until: NaiveDate,
  device_lock: i64,
  bound_mac: Option<String>,
  #[allow(dead_code)]
  license_code: String,
  max_observed_at: Option<NaiveDateTime>,
  #[allow(dead_code)]
  activated_at: Option<NaiveDateTime>,
  #[allow(dead_code)]
  status: String,
}

#[derive(Deserialize)]
pub struct ActivateBody {
  #[serde(rename = "licenseCode")]
  license_code: String,
}

#[derive(Deserialize)]
pub struct IssueBody {
  customer: String,
  #[serde(rename = "validFrom")]
  valid_from: String,
  #[serde(rename = "validUntil")]
  valid_until: String,
  #[serde(rename = "deviceLock")]
  device_lock: Option<bool>,
  #[serde(rename = "masterKey")]
  master_key: String,
}

fn license_secret() -> String {
  std::env::var("LICENSE_SECRET").unwrap_or_else(|_| "pnpro-dev-license-secret".to_string())
}

fn license_master_key() -> String {
  std::env::var("LICENSE_MASTER_KEY").unwrap_or_else(|_| "pnpro-master-change-me".to_string())
}

/// 本机 MAC（取第一块非空网卡）
pub fn local_mac_address() -> String {
  if let Ok(Some(ma)) = mac_address::get_mac_address() {
    return ma.to_string().to_lowercase();
  }
  "00:00:00:00:00:00".to_string()
}

fn sign_payload(payload_b64: &str) -> String {
  db::md5_hex(&format!("{}|{}", payload_b64, license_secret()))
}

pub fn encode_license(payload: &LicensePayload) -> Result<String, String> {
  let json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
  let b64 = B64.encode(json.as_bytes());
  let sig = sign_payload(&b64);
  Ok(format!("PN1.{b64}.{sig}"))
}

pub fn decode_license(code: &str) -> Result<LicensePayload, String> {
  let parts: Vec<&str> = code.trim().split('.').collect();
  if parts.len() != 3 || parts[0] != "PN1" {
    return Err("授权码格式无效".into());
  }
  let b64 = parts[1];
  let sig = parts[2];
  if sign_payload(b64) != sig {
    return Err("授权码签名校验失败".into());
  }
  let bytes = B64.decode(b64).map_err(|_| "授权码内容损坏".to_string())?;
  let text = String::from_utf8(bytes).map_err(|_| "授权码内容损坏".to_string())?;
  serde_json::from_str(&text).map_err(|_| "授权码解析失败".to_string())
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
  NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| format!("日期格式应为 YYYY-MM-DD: {s}"))
}

#[derive(Clone, Serialize)]
pub struct LicenseStatus {
  pub valid: bool,
  pub reason: String,
  #[serde(rename = "reasonCode")]
  pub reason_code: String,
  pub customer: Option<String>,
  #[serde(rename = "validFrom")]
  pub valid_from: Option<String>,
  #[serde(rename = "validUntil")]
  pub valid_until: Option<String>,
  #[serde(rename = "deviceLock")]
  pub device_lock: bool,
  #[serde(rename = "boundMac")]
  pub bound_mac: Option<String>,
  #[serde(rename = "currentMac")]
  pub current_mac: String,
  #[serde(rename = "daysLeft")]
  pub days_left: Option<i64>,
  #[serde(rename = "licenseId")]
  pub license_id: Option<String>,
}

pub async fn evaluate_license(pool: &crate::db::DbPool) -> LicenseStatus {
  let current_mac = local_mac_address();

  // 开发旁路：.env 中 LICENSE_BYPASS=Y
  if std::env::var("LICENSE_BYPASS").unwrap_or_default() == "Y" {
    return LicenseStatus {
      valid: true,
      reason: "bypass".into(),
      reason_code: "bypass".into(),
      customer: Some("DEV-BYPASS".into()),
      valid_from: None,
      valid_until: None,
      device_lock: false,
      bound_mac: None,
      current_mac,
      days_left: Some(9999),
      license_id: Some("bypass".into()),
    };
  }

  let now = Local::now().naive_local();
  let today = now.date();

  let row: Option<LicenseRow> = sqlx::query_as(
    "SELECT id, license_id, customer_name, valid_from, valid_until, device_lock, bound_mac,
            license_code, max_observed_at, activated_at, status
     FROM ph_license
     WHERE status = 'active'
     ORDER BY id DESC
     LIMIT 1",
  )
  .fetch_optional(pool)
  .await
  .ok()
  .flatten();

  let Some(lic) = row else {
    return LicenseStatus {
      valid: false,
      reason: "未激活授权，请导入 License".into(),
      reason_code: "missing".into(),
      customer: None,
      valid_from: None,
      valid_until: None,
      device_lock: true,
      bound_mac: None,
      current_mac,
      days_left: None,
      license_id: None,
    };
  };

  // 时钟回拨检测：当前时间早于已观测最大时间超过 1 天
  if let Some(max_obs) = lic.max_observed_at {
    if now + chrono::Duration::days(1) < max_obs {
      return LicenseStatus {
        valid: false,
        reason: "检测到系统时间被回拨，授权已锁定".into(),
        reason_code: "clock_rollback".into(),
        customer: Some(lic.customer_name),
        valid_from: Some(lic.valid_from.format("%Y-%m-%d").to_string()),
        valid_until: Some(lic.valid_until.format("%Y-%m-%d").to_string()),
        device_lock: lic.device_lock == 1,
        bound_mac: lic.bound_mac.clone(),
        current_mac,
        days_left: None,
        license_id: Some(lic.license_id),
      };
    }
  }

  // 更新最大观测时间
  let _ = sqlx::query(
    "UPDATE ph_license SET max_observed_at = CASE
       WHEN max_observed_at IS NULL OR max_observed_at < ? THEN ?
       ELSE max_observed_at END WHERE id = ?",
  )
  .bind(now)
  .bind(now)
  .bind(lic.id)
  .execute(pool)
  .await;

  if today < lic.valid_from {
    return LicenseStatus {
      valid: false,
      reason: format!("授权尚未生效（{} 起）", lic.valid_from),
      reason_code: "not_started".into(),
      customer: Some(lic.customer_name.clone()),
      valid_from: Some(lic.valid_from.format("%Y-%m-%d").to_string()),
      valid_until: Some(lic.valid_until.format("%Y-%m-%d").to_string()),
      device_lock: lic.device_lock == 1,
      bound_mac: lic.bound_mac.clone(),
      current_mac,
      days_left: None,
      license_id: Some(lic.license_id),
    };
  }

  if today > lic.valid_until {
    return LicenseStatus {
      valid: false,
      reason: format!("授权已于 {} 到期", lic.valid_until),
      reason_code: "expired".into(),
      customer: Some(lic.customer_name.clone()),
      valid_from: Some(lic.valid_from.format("%Y-%m-%d").to_string()),
      valid_until: Some(lic.valid_until.format("%Y-%m-%d").to_string()),
      device_lock: lic.device_lock == 1,
      bound_mac: lic.bound_mac.clone(),
      current_mac,
      days_left: Some(0),
      license_id: Some(lic.license_id),
    };
  }

  if lic.device_lock == 1 {
    match &lic.bound_mac {
      None => {
        // 首次运行自动绑定
        let _ = sqlx::query(
          "UPDATE ph_license SET bound_mac = ?, activated_at = COALESCE(activated_at, ?) WHERE id = ?",
        )
        .bind(&current_mac)
        .bind(now)
        .bind(lic.id)
        .execute(pool)
        .await;
      }
      Some(bound) if bound.to_lowercase() != current_mac => {
        return LicenseStatus {
          valid: false,
          reason: format!("设备未授权（已绑定 {bound}，本机 {current_mac}）"),
          reason_code: "mac_mismatch".into(),
          customer: Some(lic.customer_name.clone()),
          valid_from: Some(lic.valid_from.format("%Y-%m-%d").to_string()),
          valid_until: Some(lic.valid_until.format("%Y-%m-%d").to_string()),
          device_lock: true,
          bound_mac: Some(bound.clone()),
          current_mac,
          days_left: Some((lic.valid_until - today).num_days()),
          license_id: Some(lic.license_id),
        };
      }
      _ => {}
    }
  }

  let days_left = (lic.valid_until - today).num_days();
  LicenseStatus {
    valid: true,
    reason: "ok".into(),
    reason_code: "ok".into(),
    customer: Some(lic.customer_name),
    valid_from: Some(lic.valid_from.format("%Y-%m-%d").to_string()),
    valid_until: Some(lic.valid_until.format("%Y-%m-%d").to_string()),
    device_lock: lic.device_lock == 1,
    bound_mac: lic.bound_mac.or(Some(current_mac.clone())),
    current_mac,
    days_left: Some(days_left),
    license_id: Some(lic.license_id),
  }
}

fn path_allowed_without_license(method: &Method, path: &str) -> bool {
  if path.starts_with("/api/license") {
    return true;
  }
  // 允许登录页相关只读，但登录成功仍校验（login handler 内检查）
  if method == Method::POST && path == "/mock/login" {
    return true; // login 内部会再判
  }
  if method == Method::POST && path == "/mock/updateToken" {
    return true;
  }
  if method == Method::GET && path == "/mock/getSmsCode" {
    return true;
  }
  false
}

pub async fn license_guard_middleware(
  State(state): State<AppState>,
  req: Request,
  next: Next,
) -> Response {
  let method = req.method().clone();
  let path = req.uri().path().to_string();

  if path_allowed_without_license(&method, &path) {
    return next.run(req).await;
  }

  let status = evaluate_license(&state.pool).await;
  if status.valid {
    return next.run(req).await;
  }

  // 返回业务错误（HTTP 200 + code），与现有前端约定一致
  (
    StatusCode::OK,
    Json(ApiError {
      code: LICENSE_BIZ_CODE,
      message: "软件授权无效或已过期",
      data: json!({
        "reason": status.reason,
        "reasonCode": status.reason_code,
        "validUntil": status.valid_until,
        "currentMac": status.current_mac,
        "boundMac": status.bound_mac
      }),
    }),
  )
    .into_response()
}

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/api/license/status", get(get_status))
    .route("/api/license/activate", post(activate))
    .route("/api/license/issue", post(issue))
    .route("/api/license/current", get(get_current))
}

async fn get_status(State(state): State<AppState>) -> Json<ApiResponse<LicenseStatus>> {
  ok(evaluate_license(&state.pool).await)
}

async fn get_current(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let status = evaluate_license(&state.pool).await;
  let row: Option<(String, Option<String>, Option<NaiveDateTime>, String)> = sqlx::query_as(
    "SELECT license_code, bound_mac, activated_at, status FROM ph_license
     WHERE status = 'active' ORDER BY id DESC LIMIT 1",
  )
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(json!({
    "status": status,
    "hasLicense": row.is_some(),
    "activatedAt": row.as_ref().and_then(|r| r.2.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())),
  })))
}

async fn activate(
  State(state): State<AppState>,
  Json(body): Json<ActivateBody>,
) -> Result<Json<ApiResponse<LicenseStatus>>, (StatusCode, Json<ApiError>)> {
  let code = body.license_code.trim();
  if code.is_empty() {
    return Err(err(10000, "请填写授权码"));
  }
  let payload = decode_license(code).map_err(|m| {
    (
      StatusCode::OK,
      Json(ApiError {
        code: 10000,
        message: "授权码无效",
        data: json!({ "detail": m }),
      }),
    )
  })?;

  let valid_from = parse_date(&payload.valid_from).map_err(|m| {
    (
      StatusCode::OK,
      Json(ApiError {
        code: 10000,
        message: "授权起始日期无效",
        data: json!({ "detail": m }),
      }),
    )
  })?;
  let valid_until = parse_date(&payload.valid_until).map_err(|m| {
    (
      StatusCode::OK,
      Json(ApiError {
        code: 10000,
        message: "授权结束日期无效",
        data: json!({ "detail": m }),
      }),
    )
  })?;
  if valid_until < valid_from {
    return Err(err(10000, "结束日期不能早于开始日期"));
  }

  let mac = local_mac_address();
  let now = Local::now().naive_local();

  // 吊销旧授权
  let _ = sqlx::query("UPDATE ph_license SET status = 'revoked' WHERE status = 'active'")
    .execute(&state.pool)
    .await;

  let bound = if payload.device_lock {
    Some(mac.clone())
  } else {
    None
  };

  sqlx::query(
    "INSERT INTO ph_license
     (license_id, customer_name, valid_from, valid_until, device_lock, bound_mac, license_code,
      max_observed_at, activated_at, status)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active')
     ON CONFLICT(license_id) DO UPDATE SET
       customer_name = excluded.customer_name,
       valid_from = excluded.valid_from,
       valid_until = excluded.valid_until,
       device_lock = excluded.device_lock,
       bound_mac = excluded.bound_mac,
       license_code = excluded.license_code,
       max_observed_at = excluded.max_observed_at,
       activated_at = excluded.activated_at,
       status = 'active'",
  )
  .bind(&payload.license_id)
  .bind(&payload.customer)
  .bind(valid_from)
  .bind(valid_until)
  .bind(if payload.device_lock { 1 } else { 0 })
  .bind(&bound)
  .bind(code)
  .bind(now)
  .bind(now)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "写入授权失败，请先执行 pharma-license-schema.sql"))?;

  let _ = crate::audit::write_audit(
    &state.pool,
    "license.activate",
    "license",
    Some(&payload.license_id),
    "system",
    "super",
    None,
    Some(json!({
      "customer": payload.customer,
      "validFrom": payload.valid_from,
      "validUntil": payload.valid_until,
      "deviceLock": payload.device_lock,
      "boundMac": bound
    })),
  )
  .await;

  Ok(ok(evaluate_license(&state.pool).await))
}

async fn issue(
  Json(body): Json<IssueBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if body.master_key != license_master_key() {
    return Err(err(10000, "签发密钥错误"));
  }
  let _ = parse_date(&body.valid_from).map_err(|_| err(10000, "起始日期无效"))?;
  let _ = parse_date(&body.valid_until).map_err(|_| err(10000, "结束日期无效"))?;

  let payload = LicensePayload {
    v: 1,
    license_id: uuid::Uuid::new_v4().to_string(),
    customer: body.customer.trim().to_string(),
    valid_from: body.valid_from.trim().to_string(),
    valid_until: body.valid_until.trim().to_string(),
    device_lock: body.device_lock.unwrap_or(true),
  };
  let code = encode_license(&payload).map_err(|_| err(5000, "生成授权码失败"))?;

  Ok(ok(json!({
    "licenseCode": code,
    "payload": payload,
    "hint": "将 licenseCode 交给现场激活；开启 deviceLock 后首次激活会绑定本机 MAC"
  })))
}

/// 登录前检查：未授权则拒绝登录
pub async fn ensure_license_or_err(
  pool: &crate::db::DbPool,
) -> Result<LicenseStatus, (StatusCode, Json<ApiError>)> {
  let status = evaluate_license(pool).await;
  if status.valid {
    return Ok(status);
  }
  Err((
    StatusCode::OK,
    Json(ApiError {
      code: LICENSE_BIZ_CODE,
      message: "软件授权无效或已过期，无法登录",
      data: json!({
        "reason": status.reason,
        "reasonCode": status.reason_code,
        "currentMac": status.current_mac,
        "boundMac": status.bound_mac,
        "validUntil": status.valid_until
      }),
    }),
  ))
}
