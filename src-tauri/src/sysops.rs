//! 模块 5：用户权限、电子签名与审计
//! 模块 6：系统运维与数据库初始化

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::db;
use crate::server::{err, ok, ApiError, ApiResponse, AppState};

#[derive(Deserialize)]
struct AuditQuery {
  #[serde(rename = "dateFrom")]
  date_from: Option<String>,
  #[serde(rename = "dateTo")]
  date_to: Option<String>,
  action: Option<String>,
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
  #[serde(rename = "targetType")]
  target_type: Option<String>,
  limit: Option<i64>,
}

#[derive(Deserialize, Serialize)]
struct PasswordPolicyBody {
  #[serde(rename = "expireDays")]
  expire_days: i32,
  #[serde(rename = "minLength")]
  min_length: i32,
  #[serde(rename = "rememberOldCount")]
  remember_old_count: i32,
  #[serde(rename = "autoLogoffSeconds")]
  auto_logoff_seconds: i32,
  #[serde(rename = "electronicSignatureEnabled")]
  electronic_signature_enabled: bool,
}

#[derive(Deserialize)]
struct SignatureBody {
  #[serde(rename = "userName")]
  user_name: String,
  password: String,
  action: String,
  #[serde(rename = "targetType")]
  target_type: String,
  #[serde(rename = "targetId")]
  target_id: Option<String>,
  reason: Option<String>,
}

#[derive(Deserialize)]
struct UserEventBody {
  message: String,
  reason: Option<String>,
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct BackupConfigBody {
  enabled: bool,
  #[serde(rename = "dailyAt")]
  daily_at: String,
  #[serde(rename = "targetPath")]
  target_path: String,
  #[serde(rename = "retainDays")]
  retain_days: i32,
}

#[derive(Deserialize)]
struct BackupRunBody {
  #[serde(rename = "createdBy")]
  created_by: Option<String>,
}

#[derive(Deserialize)]
struct RestoreBody {
  #[serde(rename = "backupFilePath")]
  backup_file_path: String,
  #[serde(rename = "facilityPro")]
  facility_pro: Option<bool>,
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
}

#[derive(Deserialize)]
struct SystemCommandBody {
  action: String,
  #[serde(rename = "signedBy")]
  signed_by: Option<String>,
  reason: Option<String>,
}

#[derive(Deserialize)]
struct LanguageBody {
  locale: String,
  #[serde(rename = "displayName")]
  display_name: Option<String>,
}

#[derive(Deserialize)]
struct ResetBufferBody {
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
  reason: Option<String>,
}

#[derive(Deserialize)]
struct SetupInitBody {
  #[serde(rename = "performedBy")]
  performed_by: Option<String>,
  mode: Option<String>,
}

#[derive(Deserialize)]
struct SetupWizardBody {
  mode: Option<String>,
  step: Option<i32>,
  #[serde(rename = "dbConnection")]
  db_connection: Option<DbConnBody>,
  #[serde(rename = "restoreFile")]
  restore_file: Option<String>,
  completed: Option<bool>,
}

#[derive(Deserialize)]
struct DbConnBody {
  host: Option<String>,
  port: Option<i32>,
  database: Option<String>,
  user: Option<String>,
}

#[derive(Deserialize)]
struct CalibrationBody {
  #[serde(rename = "samplerId")]
  sampler_id: String,
  parameters: Value,
  #[serde(rename = "updatedBy")]
  updated_by: Option<String>,
}

pub fn routes() -> Router<AppState> {
  Router::new()
    // module 5
    .route(
      "/api/password-policy",
      get(get_password_policy).put(update_password_policy),
    )
    .route("/api/electronic-signature", post(electronic_signature))
    .route(
      "/api/audit-trail",
      get(list_audit_trail).post(crate::audit::create_client_audit),
    )
    .route("/api/user-events", post(create_user_event))
    .route("/api/group-functions", get(list_group_functions))
    .route("/api/pharma-users", get(list_pharma_users))
    // module 6
    .route(
      "/api/system/backup-config",
      get(get_backup_config).put(update_backup_config),
    )
    .route("/api/system/backup", post(run_backup))
    .route("/api/system/backup/jobs", get(list_backup_jobs))
    .route("/api/system/restore", post(run_restore))
    .route("/api/system/command", post(system_command))
    .route(
      "/api/system/language",
      get(get_language).put(update_language),
    )
    .route("/api/system/reset-buffer", post(reset_buffer))
    .route("/api/system/facility/tree", get(facility_tree))
    .route("/api/system/facility", get(list_facility).post(create_facility))
    .route(
      "/api/system/facility/:id",
      get(get_facility).put(update_facility).delete(delete_facility),
    )
    .route("/api/setup/status", get(setup_status))
    .route("/api/setup/init-db", post(setup_init_db))
    .route("/api/setup/wizard", get(get_wizard).put(update_wizard))
    .route(
      "/api/setup/calibration",
      get(list_calibration).put(upsert_calibration),
    )
    .route("/api/setup/calibration/:sampler_id", get(get_calibration))
}

async fn insert_audit(
  pool: &crate::db::DbPool,
  action: &str,
  target_type: &str,
  target_id: Option<&str>,
  performed_by: &str,
  role_code: &str,
  reason: Option<&str>,
  detail: Option<Value>,
) -> i64 {
  crate::audit::write_audit(
    pool,
    action,
    target_type,
    target_id,
    performed_by,
    role_code,
    reason,
    detail,
  )
  .await
}

async fn get_password_policy(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i32, i32, i32, i32, i8)> = sqlx::query_as(
    "SELECT expire_days, min_length, remember_old_count, auto_logoff_seconds, electronic_signature_enabled
     FROM ph_password_policy WHERE id = 1",
  )
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let (expire_days, min_length, remember_old_count, auto_logoff_seconds, esign) =
    row.unwrap_or((90, 6, 3, 1800, 1));

  Ok(ok(json!({
    "expireDays": expire_days,
    "minLength": min_length,
    "rememberOldCount": remember_old_count,
    "autoLogoffSeconds": auto_logoff_seconds,
    "electronicSignatureEnabled": esign == 1
  })))
}

async fn update_password_policy(
  State(state): State<AppState>,
  Json(body): Json<PasswordPolicyBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  if body.min_length < 4 || body.min_length > 64 {
    return Err(err(10000, "最小密码长度无效"));
  }
  sqlx::query(
    "INSERT INTO ph_password_policy
     (id, expire_days, min_length, remember_old_count, auto_logoff_seconds, electronic_signature_enabled)
     VALUES (1, ?, ?, ?, ?, ?)
     ON CONFLICT(id) DO UPDATE SET
       expire_days = excluded.expire_days,
       min_length = excluded.min_length,
       remember_old_count = excluded.remember_old_count,
       auto_logoff_seconds = excluded.auto_logoff_seconds,
       electronic_signature_enabled = excluded.electronic_signature_enabled",
  )
  .bind(body.expire_days)
  .bind(body.min_length)
  .bind(body.remember_old_count)
  .bind(body.auto_logoff_seconds)
  .bind(if body.electronic_signature_enabled { 1 } else { 0 })
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  insert_audit(
    &state.pool,
    "passwordPolicy.update",
    "password-policy",
    Some("1"),
    "Admin",
    "super",
    None,
    Some(serde_json::to_value(&body).unwrap_or(json!({}))),
  )
  .await;

  Ok(ok(true))
}

async fn electronic_signature(
  State(state): State<AppState>,
  Json(body): Json<SignatureBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if body.user_name.is_empty() || body.password.is_empty() || body.action.is_empty() {
    return Err(err(10000, "参数校验失败！"));
  }

  let policy: Option<(i8,)> =
    sqlx::query_as("SELECT electronic_signature_enabled FROM ph_password_policy WHERE id = 1")
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  if policy.map(|p| p.0).unwrap_or(1) == 0 {
    return Err(err(10000, "电子签名未启用"));
  }

  let password_hash = db::md5_hex(&body.password);
  let user: Option<(i64, String, Option<String>)> = sqlx::query_as(
    "SELECT u.id, u.user_name, r.role_code
     FROM sys_user u
     LEFT JOIN sys_user_role ur ON ur.user_id = u.id
     LEFT JOIN sys_role r ON r.id = ur.role_id
     WHERE u.user_name = ? AND u.password = ? AND u.status = 1 AND u.user_status = '1'
     LIMIT 1",
  )
  .bind(&body.user_name)
  .bind(&password_hash)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let (user_id, user_name, role_code) = user.ok_or(err(10000, "用户名或密码错误"))?;
  let role_code = role_code.unwrap_or_else(|| "user".to_string());

  let profile: Option<(String,)> =
    sqlx::query_as("SELECT pharma_role FROM ph_user_profile WHERE user_id = ?")
      .bind(user_id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  let pharma_role = profile
    .map(|p| p.0)
    .unwrap_or_else(|| match role_code.as_str() {
      "super" => "Administrator".to_string(),
      "admin" => "Supervisor".to_string(),
      _ => "User".to_string(),
    });

  let signature_id = insert_audit(
    &state.pool,
    &body.action,
    &body.target_type,
    body.target_id.as_deref(),
    &user_name,
    &role_code,
    body.reason.as_deref(),
    Some(json!({
      "signature": true,
      "pharmaRole": pharma_role,
      "userId": user_id
    })),
  )
  .await;

  Ok(ok(json!({
    "signatureId": signature_id.to_string(),
    "performedBy": user_name,
    "role": pharma_role,
    "signedAt": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
  })))
}

async fn list_audit_trail(
  State(state): State<AppState>,
  Query(q): Query<AuditQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let limit = q.limit.unwrap_or(500).clamp(1, 2000);
  let rows: Vec<(
    i64,
    chrono::NaiveDateTime,
    String,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
    Option<Value>,
  )> = sqlx::query_as(
    "SELECT id, event_time, action, target_type, target_id, performed_by, role_code, reason, detail_json
     FROM ph_audit_trail
     WHERE (? IS NULL OR event_time >= ?)
       AND (? IS NULL OR event_time <= ?)
       AND (? IS NULL OR action LIKE '%' || ? || '%')
       AND (? IS NULL OR performed_by LIKE '%' || ? || '%')
       AND (? IS NULL OR target_type LIKE '%' || ? || '%')
     ORDER BY event_time DESC
     LIMIT ?",
  )
  .bind(&q.date_from)
  .bind(&q.date_from)
  .bind(&q.date_to)
  .bind(&q.date_to)
  .bind(&q.action)
  .bind(&q.action)
  .bind(&q.performed_by)
  .bind(&q.performed_by)
  .bind(&q.target_type)
  .bind(&q.target_type)
  .bind(limit)
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(
        |(id, event_time, action, target_type, target_id, performed_by, role_code, reason, detail)| {
          let role = match role_code.as_str() {
            "super" => "Administrator",
            "admin" => "Supervisor",
            "emergency" => "Emergency",
            "nobody" => "Nobody",
            "power_user" => "PowerUser",
            _ => "User",
          };
          json!({
            "id": id.to_string(),
            "eventTime": event_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            "action": action,
            "targetType": target_type,
            "targetId": target_id,
            "performedBy": performed_by,
            "role": role,
            "reason": reason,
            "detail": detail
          })
        },
      )
      .collect(),
  ))
}

async fn create_user_event(
  State(state): State<AppState>,
  Json(body): Json<UserEventBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if body.message.trim().is_empty() {
    return Err(err(10000, "事件内容不能为空"));
  }
  let by = body
    .performed_by
    .unwrap_or_else(|| "system".to_string());
  let id = insert_audit(
    &state.pool,
    "userEvent.custom",
    "user-event",
    None,
    &by,
    "user",
    body.reason.as_deref(),
    Some(json!({ "message": body.message })),
  )
  .await;
  Ok(ok(json!({ "id": id.to_string() })))
}

async fn list_group_functions() -> Json<ApiResponse<Value>> {
  ok(json!({
    "Administrator": [
      "RecipeView","RecipeCreate","RecipeModify","RecipeDelete",
      "SamplingView","SamplingCreate","SamplingEdit","SamplingAbort","SamplingDelete","SamplingCustomFields",
      "SensorsView","SensorsSwitch","AlarmView","AlarmAcknowledge",
      "SamplingReportView","SamplingReportPrint","ReportGeneratorView","ReportGeneratorPrint","ReportGeneratorExport",
      "SystemView","SystemConfigure","LimitsView","LimitsConfigure","UserManagement","BackupRestore","RTTrend","RunTimeLogic"
    ],
    "Supervisor": [
      "RecipeView","RecipeCreate","RecipeModify",
      "SamplingView","SamplingCreate","SamplingEdit","SamplingAbort",
      "SensorsView","SensorsSwitch","AlarmView","AlarmAcknowledge",
      "SamplingReportView","SamplingReportPrint","ReportGeneratorView","ReportGeneratorPrint",
      "SystemView","LimitsView","LimitsConfigure","RTTrend"
    ],
    "PowerUser": [
      "RecipeView","SamplingView","SamplingCreate","SensorsView","AlarmView","AlarmAcknowledge",
      "SamplingReportView","ReportGeneratorView","LimitsView","RTTrend"
    ],
    "User": [
      "RecipeView","SamplingView","SensorsView","AlarmView","SamplingReportView","LimitsView"
    ],
    "Emergency": [
      "RecipeView","RecipeCreate","RecipeModify","RecipeDelete",
      "SamplingView","SamplingCreate","SamplingEdit","SamplingAbort","SamplingDelete",
      "SensorsView","SensorsSwitch","AlarmView","AlarmAcknowledge",
      "SystemView","SystemConfigure","UserManagement","BackupRestore","RTTrend","RunTimeLogic"
    ],
    "Nobody": ["RecipeView","SamplingView","SensorsView","AlarmView","LimitsView"]
  }))
}

async fn list_pharma_users(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    Option<chrono::NaiveDate>,
    Option<i8>,
    Option<i8>,
  )> = sqlx::query_as(
    "SELECT u.id, u.user_name, u.nick_name, u.user_status, r.role_code,
            p.user_id_code, p.expiry_date, p.must_change_password, p.is_local_emergency
     FROM sys_user u
     LEFT JOIN sys_user_role ur ON ur.user_id = u.id
     LEFT JOIN sys_role r ON r.id = ur.role_id
     LEFT JOIN ph_user_profile p ON p.user_id = u.id
     WHERE u.status = 1 AND u.user_status <> '4'
     ORDER BY u.id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(
        |(
          id,
          user_name,
          nick_name,
          user_status,
          role_code,
          user_id_code,
          expiry_date,
          must_change,
          emergency,
        )| {
          let role_code = role_code.unwrap_or_else(|| "user".to_string());
          let pharma_role = match role_code.as_str() {
            "super" => "Administrator",
            "admin" => "Supervisor",
            "emergency" => "Emergency",
            _ => "User",
          };
          json!({
            "id": id.to_string(),
            "userName": user_name,
            "userIdCode": user_id_code.unwrap_or_else(|| format!("UID-{id}")),
            "nickName": nick_name,
            "pharmaRole": pharma_role,
            "userRole": role_code,
            "expiryDate": expiry_date.map(|d| d.format("%Y-%m-%d").to_string()),
            "mustChangePassword": must_change.unwrap_or(0) == 1,
            "enabled": user_status == "1",
            "isLocalEmergency": emergency.unwrap_or(0) == 1,
            "userStatus": user_status
          })
        },
      )
      .collect(),
  ))
}

async fn get_backup_config(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i8, String, String, i32)> = sqlx::query_as(
    "SELECT enabled, daily_at, target_path, retain_days FROM ph_backup_config WHERE id = 1",
  )
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  let (enabled, daily_at, target_path, retain_days) =
    row.unwrap_or((0, "02:00".into(), "./backups".into(), 30));
  Ok(ok(json!({
    "enabled": enabled == 1,
    "dailyAt": daily_at,
    "targetPath": target_path,
    "retainDays": retain_days
  })))
}

async fn update_backup_config(
  State(state): State<AppState>,
  Json(body): Json<BackupConfigBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "INSERT INTO ph_backup_config (id, enabled, daily_at, target_path, retain_days)
     VALUES (1, ?, ?, ?, ?)
     ON CONFLICT(id) DO UPDATE SET
       enabled = excluded.enabled,
       daily_at = excluded.daily_at,
       target_path = excluded.target_path,
       retain_days = excluded.retain_days",
  )
  .bind(if body.enabled { 1 } else { 0 })
  .bind(&body.daily_at)
  .bind(&body.target_path)
  .bind(body.retain_days)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn list_backup_jobs(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(
    i64,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    chrono::NaiveDateTime,
    Option<chrono::NaiveDateTime>,
  )> = sqlx::query_as(
    "SELECT id, job_type, status, file_path, message, created_by, started_at, finished_at
     FROM ph_backup_job ORDER BY id DESC LIMIT 50",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(
        |(id, job_type, status, file_path, message, created_by, started_at, finished_at)| {
          json!({
            "id": id.to_string(),
            "type": job_type,
            "status": status,
            "filePath": file_path,
            "message": message,
            "createdBy": created_by,
            "startedAt": started_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            "finishedAt": finished_at.map(|v| v.format("%Y-%m-%dT%H:%M:%S").to_string())
          })
        },
      )
      .collect(),
  ))
}

async fn run_backup(
  State(state): State<AppState>,
  Json(body): Json<BackupRunBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.created_by.unwrap_or_else(|| "system".to_string());
  let cfg: Option<(String,)> =
    sqlx::query_as("SELECT target_path FROM ph_backup_config WHERE id = 1")
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  let target = cfg.map(|c| c.0).unwrap_or_else(|| "./backups".to_string());

  let insert = sqlx::query(
    "INSERT INTO ph_backup_job (job_type, status, created_by, started_at)
     VALUES ('manual', 'running', ?, datetime('now'))",
  )
  .bind(&by)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  let job_id = insert.last_insert_rowid();

  let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
  let file_name = format!("pharma_backup_{stamp}.json");
  let mut path = PathBuf::from(&target);
  let _ = fs::create_dir_all(&path);
  path.push(&file_name);

  let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_user")
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
  let recipes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ph_recipe")
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
  let sensors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ph_sensor")
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
  let audits: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ph_audit_trail")
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

  let payload = json!({
    "version": "1.0",
    "createdAt": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    "createdBy": by,
    "summary": {
      "users": users,
      "recipes": recipes,
      "sensors": sensors,
      "auditTrail": audits
    }
  });

  let (status, message, file_path) = match fs::write(&path, payload.to_string()) {
    Ok(_) => ("success", "备份完成".to_string(), Some(path.display().to_string())),
    Err(e) => (
      "failed",
      format!("写入失败: {e}"),
      Some(path.display().to_string()),
    ),
  };

  sqlx::query(
    "UPDATE ph_backup_job SET status = ?, file_path = ?, message = ?, finished_at = datetime('now') WHERE id = ?",
  )
  .bind(status)
  .bind(&file_path)
  .bind(&message)
  .bind(job_id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  insert_audit(
    &state.pool,
    "system.backup",
    "backup",
    Some(&job_id.to_string()),
    &by,
    "super",
    None,
    Some(json!({ "filePath": file_path, "status": status })),
  )
  .await;

  if status == "failed" {
    return Err(err(5000, "备份失败"));
  }

  Ok(ok(json!({
    "id": job_id.to_string(),
    "type": "manual",
    "status": status,
    "filePath": file_path,
    "message": message,
    "createdBy": by
  })))
}

async fn run_restore(
  State(state): State<AppState>,
  Json(body): Json<RestoreBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.performed_by.unwrap_or_else(|| "system".to_string());
  if body.backup_file_path.trim().is_empty() {
    return Err(err(10000, "请指定备份文件路径"));
  }

  let content = fs::read_to_string(&body.backup_file_path)
    .map_err(|_| err(10000, "无法读取备份文件"))?;
  let parsed: Value = serde_json::from_str(&content).unwrap_or(json!({}));

  sqlx::query(
    "UPDATE ph_setup_state SET mode = ?, restore_file = ?, updated_at = datetime('now') WHERE id = 1",
  )
  .bind(if body.facility_pro.unwrap_or(false) {
    "restoreFacilityPro"
  } else {
    "restoreFromBackup"
  })
  .bind(&body.backup_file_path)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let id = insert_audit(
    &state.pool,
    "system.restore",
    "backup",
    None,
    &by,
    "super",
    None,
    Some(json!({
      "file": body.backup_file_path,
      "facilityPro": body.facility_pro.unwrap_or(false),
      "summary": parsed.get("summary").cloned().unwrap_or(json!({}))
    })),
  )
  .await;

  Ok(ok(json!({
    "ok": true,
    "auditId": id.to_string(),
    "summary": parsed.get("summary").cloned().unwrap_or(json!({}))
  })))
}

async fn system_command(
  State(state): State<AppState>,
  Json(body): Json<SystemCommandBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let allowed = [
    "closeApp",
    "restart",
    "shutdown",
    "shutdownDisconnectedClient",
    "resetBuffer",
  ];
  if !allowed.contains(&body.action.as_str()) {
    return Err(err(10000, "不支持的系统命令"));
  }
  let by = body.signed_by.unwrap_or_else(|| "system".to_string());
  let id = insert_audit(
    &state.pool,
    "system.command",
    "system",
    Some(&body.action),
    &by,
    "super",
    body.reason.as_deref(),
    Some(json!({ "action": body.action })),
  )
  .await;

  Ok(ok(json!({
    "accepted": true,
    "action": body.action,
    "auditId": id.to_string(),
    "note": "桌面端已记录命令；真实 OS 关机/重启需客户端确认执行"
  })))
}

async fn get_language(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(Value,)> =
    sqlx::query_as("SELECT setting_value FROM ph_system_setting WHERE setting_key = 'language'")
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(
    row.map(|r| r.0)
      .unwrap_or(json!({ "locale": "zh-CN", "displayName": "简体中文" })),
  ))
}

async fn update_language(
  State(state): State<AppState>,
  Json(body): Json<LanguageBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let value = json!({
    "locale": body.locale,
    "displayName": body.display_name.unwrap_or_else(|| body.locale.clone())
  });
  sqlx::query(
    "INSERT INTO ph_system_setting (setting_key, setting_value) VALUES ('language', ?)
     ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value",
  )
  .bind(&value)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn reset_buffer(
  State(state): State<AppState>,
  Json(body): Json<ResetBufferBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.performed_by.unwrap_or_else(|| "system".to_string());

  let _ = sqlx::query("UPDATE ph_rt_trend_session SET running = 0, stopped_at = datetime('now') WHERE running = 1")
    .execute(&state.pool)
    .await;
  let _ = sqlx::query("UPDATE ph_runtime_tag SET value_json = NULL")
    .execute(&state.pool)
    .await;

  let buffer = json!({
    "lastResetAt": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    "resetCount": 1
  });
  sqlx::query(
    "INSERT INTO ph_system_setting (setting_key, setting_value) VALUES ('buffer', ?)
     ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value",
  )
  .bind(buffer.to_string()) // JSON 文本写入 SQLite
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let id = insert_audit(
    &state.pool,
    "system.resetBuffer",
    "system",
    None,
    &by,
    "super",
    body.reason.as_deref(),
    None,
  )
  .await;

  Ok(ok(json!({ "ok": true, "auditId": id.to_string() })))
}

async fn setup_status(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(
    Option<String>,
    i32,
    Option<String>,
    Option<i32>,
    Option<String>,
    Option<String>,
    i8,
    i8,
    Option<String>,
  )> = sqlx::query_as(
    "SELECT mode, step, db_host, db_port, db_name, db_user, initialized, completed, restore_file
     FROM ph_setup_state WHERE id = 1",
  )
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let healthy = sqlx::query_scalar::<_, i64>("SELECT 1")
    .fetch_one(&state.pool)
    .await
    .is_ok();

  if let Some((mode, step, host, port, name, user, initialized, completed, restore_file)) = row {
    Ok(ok(json!({
      "mode": mode,
      "step": step,
      "dbConnection": {
        "host": host,
        "port": port,
        "database": name,
        "user": user
      },
      "initialized": initialized == 1,
      "completed": completed == 1,
      "restoreFile": restore_file,
      "dbHealthy": healthy
    })))
  } else {
    Ok(ok(json!({
      "initialized": false,
      "completed": false,
      "dbHealthy": healthy
    })))
  }
}

async fn setup_init_db(
  State(state): State<AppState>,
  Json(body): Json<SetupInitBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.performed_by.unwrap_or_else(|| "system".to_string());
  let mode = body.mode.unwrap_or_else(|| "newInstallation".to_string());

  sqlx::query(
    "INSERT INTO ph_setup_state (id, mode, step, initialized, completed, updated_at)
     VALUES (1, ?, 1, 1, 0, datetime('now'))
     ON CONFLICT(id) DO UPDATE SET mode = excluded.mode, step = 1, initialized = 1, updated_at = datetime('now')",
  )
  .bind(&mode)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let id = insert_audit(
    &state.pool,
    "setup.initDb",
    "setup",
    None,
    &by,
    "super",
    None,
    Some(json!({ "mode": mode })),
  )
  .await;

  Ok(ok(json!({
    "ok": true,
    "initialized": true,
    "auditId": id.to_string(),
    "note": "数据库已就绪（复用现有 MySQL schema/seed）"
  })))
}

async fn get_wizard(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  setup_status(State(state)).await
}

async fn update_wizard(
  State(state): State<AppState>,
  Json(body): Json<SetupWizardBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let db = body.db_connection;
  sqlx::query(
    "INSERT INTO ph_setup_state (id, mode, step, db_host, db_port, db_name, db_user, restore_file, completed, updated_at)
     VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
     ON CONFLICT(id) DO UPDATE SET
       mode = COALESCE(excluded.mode, mode),
       step = COALESCE(excluded.step, step),
       db_host = COALESCE(excluded.db_host, db_host),
       db_port = COALESCE(excluded.db_port, db_port),
       db_name = COALESCE(excluded.db_name, db_name),
       db_user = COALESCE(excluded.db_user, db_user),
       restore_file = COALESCE(excluded.restore_file, restore_file),
       completed = COALESCE(excluded.completed, completed),
       updated_at = datetime('now')",
  )
  .bind(body.mode)
  .bind(body.step)
  .bind(db.as_ref().and_then(|d| d.host.clone()))
  .bind(db.as_ref().and_then(|d| d.port))
  .bind(db.as_ref().and_then(|d| d.database.clone()))
  .bind(db.as_ref().and_then(|d| d.user.clone()))
  .bind(body.restore_file)
  .bind(body.completed.map(|v| if v { 1 } else { 0 }))
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn list_calibration(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, Value, String, chrono::NaiveDateTime)> = sqlx::query_as(
    "SELECT id, sampler_id, parameters, updated_by, updated_at FROM ph_sampler_calibration ORDER BY id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(
    rows.into_iter()
      .map(|(id, sampler_id, parameters, updated_by, updated_at)| {
        json!({
          "id": id.to_string(),
          "samplerId": sampler_id,
          "parameters": parameters,
          "updatedBy": updated_by,
          "updatedAt": updated_at.format("%Y-%m-%dT%H:%M:%S").to_string()
        })
      })
      .collect(),
  ))
}

async fn get_calibration(
  State(state): State<AppState>,
  Path(sampler_id): Path<String>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i64, String, Value, String, chrono::NaiveDateTime)> = sqlx::query_as(
    "SELECT id, sampler_id, parameters, updated_by, updated_at FROM ph_sampler_calibration WHERE sampler_id = ?",
  )
  .bind(&sampler_id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  let (id, sampler_id, parameters, updated_by, updated_at) =
    row.ok_or(err(4040, "校准配置不存在"))?;
  Ok(ok(json!({
    "id": id.to_string(),
    "samplerId": sampler_id,
    "parameters": parameters,
    "updatedBy": updated_by,
    "updatedAt": updated_at.format("%Y-%m-%dT%H:%M:%S").to_string()
  })))
}

#[derive(Deserialize)]
struct FacilityQuery {
  #[serde(rename = "parentId")]
  parent_id: Option<String>,
  level: Option<i8>,
}

#[derive(Deserialize)]
struct FacilityUpsertBody {
  #[serde(rename = "parentId")]
  parent_id: Option<Value>,
  level: i8,
  code: Option<String>,
  name: String,
  description: Option<String>,
  #[serde(rename = "sortOrder")]
  sort_order: Option<i32>,
  status: Option<bool>,
}

fn parse_opt_i64(value: &Option<Value>) -> Option<i64> {
  match value {
    Some(Value::Number(n)) => n.as_i64(),
    Some(Value::String(s)) if !s.is_empty() => s.parse().ok(),
    _ => None,
  }
}

fn facility_json(
  id: i64,
  parent_id: Option<i64>,
  level: i8,
  code: String,
  name: String,
  description: Option<String>,
  sort_order: i32,
  status: i8,
) -> Value {
  json!({
    "id": id.to_string(),
    "parentId": parent_id.map(|v| v.to_string()),
    "level": level,
    "levelLabel": match level {
      1 => "厂房/园区",
      2 => "区域/洁净区",
      3 => "房间",
      _ => "未知"
    },
    "code": code,
    "name": name,
    "description": description,
    "sortOrder": sort_order,
    "status": status == 1
  })
}

async fn list_facility(
  State(state): State<AppState>,
  Query(q): Query<FacilityQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, Option<i64>, i8, String, String, Option<String>, i32, i8)> =
    if let Some(level) = q.level {
      sqlx::query_as(
        "SELECT id, parent_id, level, code, name, description, sort_order, status
         FROM ph_facility_node WHERE level = ? ORDER BY sort_order, id",
      )
      .bind(level)
      .fetch_all(&state.pool)
      .await
    } else if let Some(pid) = q.parent_id.filter(|s| !s.is_empty()) {
      if pid == "0" || pid.eq_ignore_ascii_case("null") {
        sqlx::query_as(
          "SELECT id, parent_id, level, code, name, description, sort_order, status
           FROM ph_facility_node WHERE parent_id IS NULL ORDER BY sort_order, id",
        )
        .fetch_all(&state.pool)
        .await
      } else {
        let parent = pid.parse::<i64>().map_err(|_| err(10000, "parentId 无效"))?;
        sqlx::query_as(
          "SELECT id, parent_id, level, code, name, description, sort_order, status
           FROM ph_facility_node WHERE parent_id = ? ORDER BY sort_order, id",
        )
        .bind(parent)
        .fetch_all(&state.pool)
        .await
      }
    } else {
      sqlx::query_as(
        "SELECT id, parent_id, level, code, name, description, sort_order, status
         FROM ph_facility_node ORDER BY level, sort_order, id",
      )
      .fetch_all(&state.pool)
      .await
    }
    .map_err(|e| {
      eprintln!("list_facility error: {e}");
      err(5000, "服务器错误")
    })?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, parent_id, level, code, name, description, sort_order, status)| {
        facility_json(id, parent_id, level, code, name, description, sort_order, status)
      })
      .collect(),
  ))
}

async fn facility_tree(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, Option<i64>, i8, String, String, Option<String>, i32, i8)> = sqlx::query_as(
    "SELECT id, parent_id, level, code, name, description, sort_order, status
     FROM ph_facility_node WHERE status = 1 ORDER BY level, sort_order, id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e| {
    eprintln!("facility_tree error: {e}");
    err(5000, "服务器错误")
  })?;

  let items: Vec<Value> = rows
    .into_iter()
    .map(|(id, parent_id, level, code, name, description, sort_order, status)| {
      facility_json(id, parent_id, level, code, name, description, sort_order, status)
    })
    .collect();

  let mut children_map: std::collections::HashMap<String, Vec<Value>> =
    std::collections::HashMap::new();
  for item in &items {
    let pid = item
      .get("parentId")
      .and_then(|v| v.as_str())
      .unwrap_or("")
      .to_string();
    children_map.entry(pid).or_default().push(item.clone());
  }

  fn attach(nodes: &mut Vec<Value>, map: &std::collections::HashMap<String, Vec<Value>>) {
    for node in nodes.iter_mut() {
      let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if let Some(mut kids) = map.get(&id).cloned() {
        attach(&mut kids, map);
        node
          .as_object_mut()
          .unwrap()
          .insert("children".to_string(), Value::Array(kids));
      } else {
        node
          .as_object_mut()
          .unwrap()
          .insert("children".to_string(), Value::Array(vec![]));
      }
    }
  }

  let mut roots = children_map.get("").cloned().unwrap_or_default();
  attach(&mut roots, &children_map);
  Ok(ok(roots))
}

async fn get_facility(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i64, Option<i64>, i8, String, String, Option<String>, i32, i8)> = sqlx::query_as(
    "SELECT id, parent_id, level, code, name, description, sort_order, status
     FROM ph_facility_node WHERE id = ?",
  )
  .bind(id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  let (id, parent_id, level, code, name, description, sort_order, status) =
    row.ok_or(err(4040, "节点不存在"))?;
  Ok(ok(facility_json(
    id,
    parent_id,
    level,
    code,
    name,
    description,
    sort_order,
    status,
  )))
}

async fn create_facility(
  State(state): State<AppState>,
  Json(body): Json<FacilityUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if !(1..=3).contains(&body.level) {
    return Err(err(10000, "level 必须为 1/2/3"));
  }
  if body.name.trim().is_empty() {
    return Err(err(10000, "名称不能为空"));
  }

  let parent_id = parse_opt_i64(&body.parent_id);
  if body.level == 1 {
    if parent_id.is_some() {
      return Err(err(10000, "一级节点不能有父节点"));
    }
  } else {
    let pid = parent_id.ok_or(err(10000, "二/三级节点必须指定父节点"))?;
    let parent: Option<(i8,)> = sqlx::query_as("SELECT level FROM ph_facility_node WHERE id = ?")
      .bind(pid)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
    let (parent_level,) = parent.ok_or(err(10000, "父节点不存在"))?;
    if parent_level + 1 != body.level {
      return Err(err(10000, "父子层级不匹配（应为 厂房→区域→房间）"));
    }
  }

  let result = sqlx::query(
    "INSERT INTO ph_facility_node (parent_id, level, code, name, description, sort_order, status)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(parent_id)
  .bind(body.level)
  .bind(body.code.unwrap_or_default())
  .bind(body.name.trim())
  .bind(body.description)
  .bind(body.sort_order.unwrap_or(0))
  .bind(if body.status.unwrap_or(true) { 1 } else { 0 })
  .execute(&state.pool)
  .await
  .map_err(|e| {
    eprintln!("create_facility error: {e}");
    err(5000, "服务器错误")
  })?;

  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_facility(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<FacilityUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  if body.name.trim().is_empty() {
    return Err(err(10000, "名称不能为空"));
  }
  let existing: Option<(i8,)> = sqlx::query_as("SELECT level FROM ph_facility_node WHERE id = ?")
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  if existing.is_none() {
    return Err(err(4040, "节点不存在"));
  }

  sqlx::query(
    "UPDATE ph_facility_node SET
       code = COALESCE(?, code),
       name = ?,
       description = ?,
       sort_order = COALESCE(?, sort_order),
       status = COALESCE(?, status),
       updated_at = datetime('now')
     WHERE id = ?",
  )
  .bind(body.code)
  .bind(body.name.trim())
  .bind(body.description)
  .bind(body.sort_order)
  .bind(body.status.map(|v| if v { 1 } else { 0 }))
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn delete_facility(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let child: Option<(i64,)> =
    sqlx::query_as("SELECT id FROM ph_facility_node WHERE parent_id = ? LIMIT 1")
      .bind(id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  if child.is_some() {
    return Err(err(10000, "请先删除子节点"));
  }
  let result = sqlx::query("DELETE FROM ph_facility_node WHERE id = ?")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  if result.rows_affected() == 0 {
    return Err(err(4040, "节点不存在"));
  }
  Ok(ok(true))
}

async fn upsert_calibration(
  State(state): State<AppState>,
  Json(body): Json<CalibrationBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.updated_by.unwrap_or_else(|| "system".to_string());
  sqlx::query(
    "INSERT INTO ph_sampler_calibration (sampler_id, parameters, updated_by)
     VALUES (?, ?, ?)
     ON CONFLICT(sampler_id) DO UPDATE SET parameters = excluded.parameters, updated_by = excluded.updated_by, updated_at = datetime('now')",
  )
  .bind(&body.sampler_id)
  .bind(&body.parameters)
  .bind(&by)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(json!({ "samplerId": body.sampler_id })))
}
