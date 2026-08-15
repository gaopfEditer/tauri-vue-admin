//! 统一审计写入与 HTTP 操作自动落库

use axum::{
  extract::{Request, State},
  http::{Method, StatusCode},
  middleware::Next,
  response::Response,
  Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::db::DbPool;

use crate::server::{err, ok, ApiError, ApiResponse, AppState};

pub async fn write_audit(
  pool: &DbPool,
  action: &str,
  target_type: &str,
  target_id: Option<&str>,
  performed_by: &str,
  role_code: &str,
  reason: Option<&str>,
  detail: Option<Value>,
) -> i64 {
  let result = sqlx::query(
    "INSERT INTO ph_audit_trail (action, target_type, target_id, performed_by, role_code, reason, detail_json)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(action)
  .bind(target_type)
  .bind(target_id)
  .bind(performed_by)
  .bind(role_code)
  .bind(reason)
  .bind(detail)
  .execute(pool)
  .await;

  result.map(|r| r.last_insert_rowid() as i64).unwrap_or(0)
}

fn header_str(req: &Request, name: &str) -> String {
  req
    .headers()
    .get(name)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .trim()
    .to_string()
}

fn business_already_audited(method: &Method, path: &str) -> bool {
  // 业务 handler 已写入更完整的审计，避免重复
  match (method.as_str(), path) {
    ("PUT", "/api/password-policy")
    | ("POST", "/api/electronic-signature")
    | ("POST", "/api/user-events")
    | ("POST", "/api/system/backup")
    | ("POST", "/api/system/restore")
    | ("POST", "/api/system/command")
    | ("POST", "/api/system/reset-buffer")
    | ("POST", "/api/setup/init-db")
    | ("POST", "/api/alarms/ack-all")
    | ("POST", "/api/rt-trend/start")
    | ("POST", "/api/rt-trend/stop") => true,
    ("POST", p) if p.starts_with("/api/alarms/") && p.ends_with("/ack") => true,
    _ => false,
  }
}

fn should_auto_audit(method: &Method, path: &str) -> bool {
  if path == "/api/audit-trail"
    || path == "/api/user-events"
    || path.starts_with("/api/license")
    || path.starts_with("/api/device-poll")
  {
    return false;
  }
  if business_already_audited(method, path) {
    return false;
  }
  // 高频轮询不记
  if matches!(
    path,
    "/api/rt-trend/points"
      | "/api/rt-trend/current"
      | "/api/realtime/signals"
      | "/api/realtime/rooms"
  ) || path.starts_with("/api/realtime/signals")
  {
    return false;
  }
  if method == Method::GET {
    return path.contains("/export") || path.ends_with("/download");
  }
  matches!(
    *method,
    Method::POST | Method::PUT | Method::PATCH | Method::DELETE
  )
}

fn map_action(method: &Method, path: &str) -> (String, String, Option<String>) {
  let method_l = method.as_str().to_lowercase();
  let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

  // /api/... or /mock/...
  let rest: Vec<&str> = if segments.first() == Some(&"api") || segments.first() == Some(&"mock") {
    segments[1..].to_vec()
  } else {
    segments
  };

  let target_type = rest.first().copied().unwrap_or("api").to_string();
  let mut target_id: Option<String> = None;
  for (i, seg) in rest.iter().enumerate() {
    if seg.chars().all(|c| c.is_ascii_digit()) {
      target_id = Some((*seg).to_string());
      let _ = i;
      break;
    }
  }

  let action = match (method_l.as_str(), rest.as_slice()) {
    ("post", ["login"]) => "auth.login".into(),
    ("post", ["sensors"]) => "sensor.create".into(),
    ("delete", ["sensors", id, ..]) => {
      target_id = Some((*id).to_string());
      "sensor.delete".into()
    }
    ("post", ["sensors", id, "restore"]) => {
      target_id = Some((*id).to_string());
      "sensor.restore".into()
    }
    ("put", ["sensors", id, "meta"]) => {
      target_id = Some((*id).to_string());
      "sensor.meta.update".into()
    }
    ("post", ["sensors", id, "power"]) => {
      target_id = Some((*id).to_string());
      "sensor.power".into()
    }
    ("put", ["sensors", id, "device-config"]) => {
      target_id = Some((*id).to_string());
      "sensor.deviceConfig.save".into()
    }
    ("put", ["sensor-limits"]) => "sensor.limit.save".into(),
    ("post", ["recipes"]) => "recipe.create".into(),
    ("put", ["recipes", id]) => {
      target_id = Some((*id).to_string());
      "recipe.update".into()
    }
    ("delete", ["recipes", id]) => {
      target_id = Some((*id).to_string());
      "recipe.delete".into()
    }
    ("post", ["samplings"]) => "sampling.create".into(),
    ("put", ["samplings", "custom-fields"]) => "sampling.customFields.save".into(),
    ("put", ["samplings", id]) => {
      target_id = Some((*id).to_string());
      "sampling.update".into()
    }
    ("delete", ["samplings", id]) => {
      target_id = Some((*id).to_string());
      "sampling.delete".into()
    }
    ("post", ["samplings", id, "abort"]) => {
      target_id = Some((*id).to_string());
      "sampling.abort".into()
    }
    ("post", ["realtime", "batch-power"]) => "realtime.batchPower".into(),
    ("post", ["alarms", id, "ack"]) => {
      target_id = Some((*id).to_string());
      "alarm.ack".into()
    }
    ("post", ["alarms", "ack-all"]) => "alarm.ackAll".into(),
    ("post", ["rt-trend", "start"]) => "rtTrend.start".into(),
    ("post", ["rt-trend", "stop"]) => "rtTrend.stop".into(),
    ("post", ["runtime-rules"]) => "runtimeRule.create".into(),
    ("put", ["runtime-rules", id]) => {
      target_id = Some((*id).to_string());
      "runtimeRule.update".into()
    }
    ("delete", ["runtime-rules", id]) => {
      target_id = Some((*id).to_string());
      "runtimeRule.delete".into()
    }
    ("post", ["reports", "generate"]) => "report.generate".into(),
    ("get", ["reports", id, "export"]) => {
      target_id = Some((*id).to_string());
      "report.export".into()
    }
    ("post", ["sda", "sessions"]) => "sda.session.create".into(),
    ("put", ["sda", "sessions", id]) => {
      target_id = Some((*id).to_string());
      "sda.session.update".into()
    }
    ("post", ["sda", "analyze"]) => "sda.analyze".into(),
    ("put", ["password-policy"]) => "passwordPolicy.update".into(),
    ("post", ["electronic-signature"]) => "esign.submit".into(),
    ("put", ["system", "backup-config"]) => "system.backupConfig.save".into(),
    ("post", ["system", "backup"]) => "system.backup.run".into(),
    ("post", ["system", "restore"]) => "system.restore".into(),
    ("post", ["system", "command"]) => "system.command".into(),
    ("put", ["system", "language"]) => "system.language.save".into(),
    ("post", ["system", "reset-buffer"]) => "system.resetBuffer".into(),
    ("post", ["system", "facility"]) => "facility.create".into(),
    ("put", ["system", "facility", id]) => {
      target_id = Some((*id).to_string());
      "facility.update".into()
    }
    ("delete", ["system", "facility", id]) => {
      target_id = Some((*id).to_string());
      "facility.delete".into()
    }
    ("post", ["setup", "init-db"]) => "setup.initDb".into(),
    ("put", ["setup", "wizard"]) => "setup.wizard.save".into(),
    ("put", ["setup", "calibration"]) => "setup.calibration.save".into(),
    ("post", ["management", "user"]) => "user.create".into(),
    ("put", ["management", "user", id]) => {
      target_id = Some((*id).to_string());
      "user.update".into()
    }
    ("delete", ["management", "user", id]) => {
      target_id = Some((*id).to_string());
      "user.delete".into()
    }
    _ => format!("api.{method_l}"),
  };

  (action, target_type, target_id)
}

/// 自动记录写操作 / 导出类 GET
pub async fn http_audit_middleware(
  State(state): State<AppState>,
  req: Request,
  next: Next,
) -> Response {
  let method = req.method().clone();
  let path = req.uri().path().to_string();
  let query = req.uri().query().map(|s| s.to_string());
  let performed_by = {
    let h = header_str(&req, "x-audit-user");
    if h.is_empty() {
      "anonymous".to_string()
    } else {
      h
    }
  };
  let role_code = {
    let h = header_str(&req, "x-audit-role");
    if h.is_empty() {
      "user".to_string()
    } else {
      h
    }
  };
  let do_audit = should_auto_audit(&method, &path);

  let response = next.run(req).await;

  if !do_audit {
    return response;
  }

  let status = response.status();
  // 业务接口多数返回 HTTP 200 + body.code，此处以 HTTP 成功为准
  if !(status.is_success() || status == StatusCode::OK) {
    return response;
  }

  let (action, target_type, target_id) = map_action(&method, &path);
  // 登录由 login handler 单独写更完整记录，中间件跳过
  if action == "auth.login" {
    return response;
  }

  let pool = state.pool.clone();
  let detail = json!({
    "method": method.as_str(),
    "path": path,
    "query": query,
    "httpStatus": status.as_u16(),
    "source": "http.middleware"
  });

  tokio::spawn(async move {
    write_audit(
      &pool,
      &action,
      &target_type,
      target_id.as_deref(),
      &performed_by,
      &role_code,
      None,
      Some(detail),
    )
    .await;
  });

  response
}

#[derive(Deserialize)]
pub struct ClientAuditBody {
  action: String,
  #[serde(rename = "targetType")]
  target_type: Option<String>,
  #[serde(rename = "targetId")]
  target_id: Option<String>,
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
  #[serde(rename = "roleCode")]
  role_code: Option<String>,
  reason: Option<String>,
  detail: Option<Value>,
}

/// 前端主动上报：打开页面、登出、本地导出等
pub async fn create_client_audit(
  State(state): State<AppState>,
  headers: axum::http::HeaderMap,
  Json(body): Json<ClientAuditBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if body.action.trim().is_empty() {
    return Err(err(10000, "action 不能为空"));
  }

  let from_header = headers
    .get("x-audit-user")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_string();
  let role_header = headers
    .get("x-audit-role")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_string();

  let performed_by = body
    .performed_by
    .filter(|s| !s.is_empty())
    .or_else(|| {
      if from_header.is_empty() {
        None
      } else {
        Some(from_header)
      }
    })
    .unwrap_or_else(|| "anonymous".to_string());

  let role_code = body
    .role_code
    .filter(|s| !s.is_empty())
    .or_else(|| {
      if role_header.is_empty() {
        None
      } else {
        Some(role_header)
      }
    })
    .unwrap_or_else(|| "user".to_string());

  let target_type = body
    .target_type
    .unwrap_or_else(|| "client".to_string());

  let mut detail = body.detail.unwrap_or_else(|| json!({}));
  if let Some(obj) = detail.as_object_mut() {
    obj.insert("source".into(), json!("client"));
  }

  let id = write_audit(
    &state.pool,
    &body.action,
    &target_type,
    body.target_id.as_deref(),
    &performed_by,
    &role_code,
    body.reason.as_deref(),
    Some(detail),
  )
  .await;

  Ok(ok(json!({ "id": id.to_string() })))
}
