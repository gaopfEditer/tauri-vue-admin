use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  routing::{get, post, put},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::db::DbPool;
use crate::server::{err, ok, ApiError, ApiResponse, AppState};

#[derive(Deserialize)]
pub struct CategoryQuery {
  pub category: Option<String>,
  /// includeDeleted=1 时返回已软删除设备
  #[serde(rename = "includeDeleted")]
  pub include_deleted: Option<String>,
  /// onlyDeleted=1 时仅返回已删除
  #[serde(rename = "onlyDeleted")]
  pub only_deleted: Option<String>,
}

#[derive(Serialize, FromRow)]
struct SensorRow {
  id: i64,
  category: String,
  custom_id: String,
  description: String,
  group_id: Option<i64>,
  channel: Option<String>,
  acquire_mode: Option<String>,
  runtime_state: String,
  power_on: i8,
  com_alarm: i8,
  flow_calc_alarm: i8,
  last_value: Option<Value>,
  unit: Option<String>,
  status: i8,
  updated_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
struct SensorMetaBody {
  #[serde(rename = "customId")]
  custom_id: Option<String>,
  description: Option<String>,
  #[serde(rename = "groupId")]
  group_id: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct SensorPowerBody {
  action: String,
}

#[derive(Deserialize)]
struct BatchPowerBody {
  action: String,
  #[serde(rename = "roomIds")]
  room_ids: Option<Vec<Value>>,
  #[serde(rename = "sensorIds")]
  sensor_ids: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct SensorCreateBody {
  category: String,
  #[serde(rename = "customId")]
  custom_id: String,
  description: Option<String>,
  #[serde(rename = "groupId")]
  group_id: Option<Value>,
  channel: Option<String>,
  unit: Option<String>,
  #[serde(rename = "acquireMode")]
  acquire_mode: Option<String>,
}

#[derive(Serialize, FromRow)]
struct GroupRow {
  id: i64,
  name: String,
  description: Option<String>,
  tags: Option<Value>,
}

#[derive(Serialize, FromRow)]
struct LimitRow {
  id: i64,
  sensor_id: i64,
  data_type: String,
  warning_limit: Option<f64>,
  alarm_limit: Option<f64>,
  effective_from: chrono::NaiveDateTime,
  changed_by: String,
}

#[derive(Deserialize)]
struct LimitUpsertBody {
  #[serde(rename = "sensorId")]
  sensor_id: serde_json::Value,
  #[serde(rename = "dataType")]
  data_type: String,
  #[serde(rename = "warningLimit")]
  warning_limit: Option<f64>,
  #[serde(rename = "alarmLimit")]
  alarm_limit: Option<f64>,
  #[serde(rename = "changedBy")]
  changed_by: Option<String>,
  reason: Option<String>,
}

fn parse_i64(value: &Value) -> Option<i64> {
  match value {
    Value::Number(n) => n.as_i64(),
    Value::String(s) => s.parse().ok(),
    _ => None,
  }
}

#[derive(Serialize, FromRow)]
struct RecipeRow {
  id: i64,
  name: String,
  description: Option<String>,
  sensor_group_ids: Value,
  pens: Value,
  created_by: String,
  status: String,
  updated_at: chrono::NaiveDateTime,
  created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
struct RecipeUpsertBody {
  name: String,
  description: Option<String>,
  #[serde(rename = "sensorGroupIds")]
  sensor_group_ids: Vec<i64>,
  pens: Value,
  #[serde(rename = "createdBy")]
  created_by: Option<String>,
}

#[derive(Serialize, FromRow)]
struct SamplingRow {
  id: i64,
  recipe_id: i64,
  recipe_name: String,
  scheduled_at: chrono::NaiveDateTime,
  sampling_mode: String,
  custom_fields: Option<Value>,
  input_notes: Option<String>,
  status: String,
  started_at: Option<chrono::NaiveDateTime>,
  finished_at: Option<chrono::NaiveDateTime>,
  abort_reason: Option<String>,
  created_by: String,
}

#[derive(Deserialize)]
struct SamplingUpsertBody {
  #[serde(rename = "recipeId")]
  recipe_id: i64,
  #[serde(rename = "scheduledAt")]
  scheduled_at: String,
  #[serde(rename = "samplingMode")]
  sampling_mode: Option<String>,
  #[serde(rename = "customFields")]
  custom_fields: Option<Value>,
  #[serde(rename = "inputNotes")]
  input_notes: Option<String>,
  #[serde(rename = "createdBy")]
  created_by: Option<String>,
}

#[derive(Deserialize)]
struct AbortBody {
  reason: Option<String>,
}

#[derive(Serialize, FromRow)]
struct CustomFieldRow {
  id: i64,
  field_key: String,
  display_name: String,
  enabled: i8,
  editable: i8,
  default_entries: Option<Value>,
  sort_order: i32,
}

#[derive(Deserialize)]
struct CustomFieldBody {
  key: String,
  #[serde(rename = "displayName")]
  display_name: String,
  enabled: Option<bool>,
  editable: Option<bool>,
  #[serde(rename = "defaultEntries")]
  default_entries: Option<Value>,
  #[serde(rename = "sortOrder")]
  sort_order: Option<i32>,
}

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/api/recipes", get(list_recipes).post(create_recipe))
    .route("/api/recipes/:id", get(get_recipe).put(update_recipe).delete(delete_recipe))
    .route("/api/samplings", get(list_samplings).post(create_sampling))
    .route("/api/samplings/:id", put(update_sampling).delete(delete_sampling))
    .route("/api/samplings/:id/abort", post(abort_sampling))
    .route(
      "/api/samplings/custom-fields",
      get(list_custom_fields).put(replace_custom_fields),
    )
    .route("/api/sensors", get(list_sensors).post(create_sensor))
    .route("/api/sensors/:id", get(get_sensor).delete(delete_sensor))
    .route("/api/sensors/:id/power", post(sensor_power))
    .route("/api/sensors/:id/meta", put(update_sensor_meta))
    .route("/api/sensors/:id/restore", post(restore_sensor))
    .route(
      "/api/sensors/:id/device-config",
      get(get_device_config).put(upsert_device_config),
    )
    .route("/api/realtime/rooms", get(realtime_rooms))
    .route("/api/realtime/signals", get(realtime_signals))
    .route("/api/realtime/batch-power", post(batch_power_by_rooms))
    .route("/api/sensor-groups", get(list_sensor_groups))
    .route("/api/sensor-limits", get(list_limits).put(upsert_limit))
    .route("/api/sensor-limits/history", get(list_limit_history))
}

fn sensor_json(s: SensorRow) -> Value {
  json!({
    "id": s.id.to_string(),
    "category": s.category,
    "customId": s.custom_id,
    "description": s.description,
    "groupId": s.group_id.map(|v| v.to_string()),
    "channel": s.channel,
    "acquireMode": s.acquire_mode,
    "runtimeState": s.runtime_state,
    "powerOn": s.power_on == 1,
    "comAlarm": s.com_alarm == 1,
    "flowCalcAlarm": s.flow_calc_alarm == 1,
    "lastValue": s.last_value,
    "unit": s.unit,
    "status": if s.status == 1 { "active" } else { "deleted" },
    "deleted": s.status == 0,
    "updatedAt": s.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string()
  })
}

fn recipe_json(r: RecipeRow) -> Value {
  json!({
    "id": r.id.to_string(),
    "name": r.name,
    "description": r.description,
    "sensorGroupIds": r.sensor_group_ids,
    "pens": r.pens,
    "createdBy": r.created_by,
    "status": r.status,
    "updatedAt": r.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    "createdAt": r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string()
  })
}

fn sampling_json(s: SamplingRow) -> Value {
  json!({
    "id": s.id.to_string(),
    "recipeId": s.recipe_id.to_string(),
    "recipeName": s.recipe_name,
    "scheduledAt": s.scheduled_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    "samplingMode": s.sampling_mode,
    "customFields": s.custom_fields.unwrap_or(json!({})),
    "inputNotes": s.input_notes,
    "status": s.status,
    "startedAt": s.started_at.map(|v| v.format("%Y-%m-%dT%H:%M:%S").to_string()),
    "finishedAt": s.finished_at.map(|v| v.format("%Y-%m-%dT%H:%M:%S").to_string()),
    "abortReason": s.abort_reason,
    "createdBy": s.created_by
  })
}

async fn list_sensors(
  State(state): State<AppState>,
  Query(q): Query<CategoryQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let only_deleted = q.only_deleted.as_deref() == Some("1") || q.only_deleted.as_deref() == Some("true");
  let include_deleted =
    only_deleted || q.include_deleted.as_deref() == Some("1") || q.include_deleted.as_deref() == Some("true");

  let status_sql = if only_deleted {
    "status = 0"
  } else if include_deleted {
    "status IN (0, 1)"
  } else {
    "status = 1"
  };

  let rows: Vec<SensorRow> = if let Some(cat) = q.category.filter(|c| !c.is_empty()) {
    let sql = format!(
      "SELECT id, category, custom_id, description, group_id, channel, acquire_mode, runtime_state,
              power_on, com_alarm, flow_calc_alarm, last_value, unit, status, updated_at
       FROM ph_sensor WHERE {status_sql} AND category = ? ORDER BY status DESC, id"
    );
    sqlx::query_as(&sql)
      .bind(cat)
      .fetch_all(&state.pool)
      .await
  } else {
    let sql = format!(
      "SELECT id, category, custom_id, description, group_id, channel, acquire_mode, runtime_state,
              power_on, com_alarm, flow_calc_alarm, last_value, unit, status, updated_at
       FROM ph_sensor WHERE {status_sql} ORDER BY status DESC, id"
    );
    sqlx::query_as(&sql).fetch_all(&state.pool).await
  }
  .map_err(|e| {
    eprintln!("list_sensors error: {e}");
    err(5000, "服务器错误")
  })?;

  Ok(ok(rows.into_iter().map(sensor_json).collect()))
}

async fn get_sensor(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<SensorRow> = sqlx::query_as(
    "SELECT id, category, custom_id, description, group_id, channel, acquire_mode, runtime_state,
            power_on, com_alarm, flow_calc_alarm, last_value, unit, status, updated_at
     FROM ph_sensor WHERE id = ?",
  )
  .bind(id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let row = row.ok_or(err(4040, "传感器不存在"))?;
  Ok(ok(sensor_json(row)))
}

async fn update_sensor_meta(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<SensorMetaBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let group_id = body.group_id.as_ref().and_then(parse_i64);
  sqlx::query(
    "UPDATE ph_sensor SET
       custom_id = COALESCE(?, custom_id),
       description = COALESCE(?, description),
       group_id = ?,
       updated_at = datetime('now')
     WHERE id = ? AND status = 1",
  )
  .bind(body.custom_id)
  .bind(body.description)
  .bind(group_id)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn create_sensor(
  State(state): State<AppState>,
  Json(body): Json<SensorCreateBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let category = body.category.trim().to_string();
  if !matches!(category.as_str(), "particle" | "biocapt" | "analog") {
    return Err(err(10000, "category 必须为 particle/biocapt/analog"));
  }
  let custom_id = body.custom_id.trim().to_string();
  if custom_id.is_empty() {
    return Err(err(10000, "设备编号/Custom ID 不能为空"));
  }

  let exists: Option<(i64,)> =
    sqlx::query_as("SELECT id FROM ph_sensor WHERE custom_id = ? AND status = 1 LIMIT 1")
      .bind(&custom_id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  if exists.is_some() {
    return Err(err(10000, "设备编号已存在"));
  }

  // 若存在同名已删除记录，提示先恢复
  let archived: Option<(i64,)> =
    sqlx::query_as("SELECT id FROM ph_sensor WHERE custom_id = ? AND status = 0 LIMIT 1")
      .bind(&custom_id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  if archived.is_some() {
    return Err(err(10000, "该编号存在于已删除设备，请先恢复或换一个编号"));
  }

  let group_id = body.group_id.as_ref().and_then(parse_i64);
  let description = body
    .description
    .unwrap_or_else(|| custom_id.clone())
    .trim()
    .to_string();

  let result = sqlx::query(
    "INSERT INTO ph_sensor
     (category, custom_id, description, group_id, channel, acquire_mode, runtime_state, power_on, unit, status)
     VALUES (?, ?, ?, ?, ?, ?, 'Idle', 0, ?, 1)",
  )
  .bind(&category)
  .bind(&custom_id)
  .bind(description)
  .bind(group_id)
  .bind(body.channel)
  .bind(body.acquire_mode)
  .bind(body.unit)
  .execute(&state.pool)
  .await
  .map_err(|e| {
    eprintln!("create_sensor error: {e}");
    err(5000, "服务器错误")
  })?;

  let id = result.last_insert_rowid() as i64;

  // 粒子设备默认补一份基础配置，方便后续编辑
  if category == "particle" {
    let _ = sqlx::query(
      "INSERT INTO ph_sensor_device_config
       (sensor_id, device_name, device_code, instrument_type, update_interval_sec,
        data_unit, data_type, data_length, protocol, decimal_places, operating_mode, production_state)
       VALUES (?, ?, ?, '尘埃粒子0.5', 60, 'pt/3', 'ulong', 4, 'ModbusTCP', 0, 'Operational', 'production')",
    )
    .bind(id)
    .bind(&custom_id)
    .bind(&custom_id)
    .execute(&state.pool)
    .await;
  }

  Ok(ok(json!({ "id": id.to_string() })))
}

async fn delete_sensor(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let result = sqlx::query(
    "UPDATE ph_sensor SET status = 0, power_on = 0, runtime_state = 'Offline', updated_at = datetime('now')
     WHERE id = ? AND status = 1",
  )
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  if result.rows_affected() == 0 {
    return Err(err(4040, "传感器不存在或已删除"));
  }
  Ok(ok(true))
}

async fn restore_sensor(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(String,)> =
    sqlx::query_as("SELECT custom_id FROM ph_sensor WHERE id = ? AND status = 0")
      .bind(id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  let (custom_id,) = row.ok_or(err(4040, "待恢复设备不存在"))?;

  let conflict: Option<(i64,)> =
    sqlx::query_as("SELECT id FROM ph_sensor WHERE custom_id = ? AND status = 1 AND id <> ? LIMIT 1")
      .bind(&custom_id)
      .bind(id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  if conflict.is_some() {
    return Err(err(10000, "已有同编号在用设备，无法恢复"));
  }

  let result = sqlx::query(
    "UPDATE ph_sensor SET status = 1, runtime_state = 'Idle', updated_at = datetime('now')
     WHERE id = ? AND status = 0",
  )
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  if result.rows_affected() == 0 {
    return Err(err(4040, "恢复失败"));
  }
  Ok(ok(true))
}

async fn sensor_power(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<SensorPowerBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let (power_on, runtime_state) = match body.action.as_str() {
    "on" => (1, "Idle"),
    "off" => (0, "Idle"),
    "cleaning" => (1, "Cleaning"),
    _ => return Err(err(10000, "无效的电源动作")),
  };

  sqlx::query(
    "UPDATE ph_sensor SET power_on = ?, runtime_state = ?, updated_at = datetime('now')
     WHERE id = ? AND status = 1",
  )
  .bind(power_on)
  .bind(runtime_state)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  // 尝试下发 ApexRp 命令（失败只记日志，不阻断 UI）
  if let Err(e) = crate::modbus_poll::send_power_command(&state.pool, id, &body.action).await {
    eprintln!("modbus power command skipped/failed: {e}");
  }

  Ok(ok(true))
}

fn parse_id_list(values: &Option<Vec<Value>>) -> Vec<i64> {
  values
    .as_ref()
    .map(|list| {
      list
        .iter()
        .filter_map(|v| match v {
          Value::Number(n) => n.as_i64(),
          Value::String(s) => s.parse().ok(),
          _ => None,
        })
        .collect()
    })
    .unwrap_or_default()
}

async fn batch_power_by_rooms(
  State(state): State<AppState>,
  Json(body): Json<BatchPowerBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let (power_on, runtime_state) = match body.action.as_str() {
    "on" => (1, "Idle"),
    "off" => (0, "Idle"),
    "cleaning" => (1, "Cleaning"),
    _ => return Err(err(10000, "无效的电源动作")),
  };

  let room_ids = parse_id_list(&body.room_ids);
  let sensor_ids = parse_id_list(&body.sensor_ids);
  if room_ids.is_empty() && sensor_ids.is_empty() {
    return Err(err(10000, "请选择房间或设备"));
  }

  let mut affected = 0i64;
  if !sensor_ids.is_empty() {
    for id in &sensor_ids {
      let r = sqlx::query(
        "UPDATE ph_sensor SET power_on = ?, runtime_state = ?, updated_at = datetime('now')
         WHERE id = ? AND status = 1",
      )
      .bind(power_on)
      .bind(runtime_state)
      .bind(id)
      .execute(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
      affected += r.rows_affected() as i64;
    }
  }

  if !room_ids.is_empty() {
    // MySQL IN clause via loop to avoid dynamic SQL complexity
    for room_id in &room_ids {
      let r = sqlx::query(
        "UPDATE ph_sensor
         SET power_on = ?, runtime_state = ?, updated_at = datetime('now')
         WHERE status = 1 AND id IN (
           SELECT sensor_id FROM ph_sensor_device_config WHERE facility_node_id = ?
         )",
      )
      .bind(power_on)
      .bind(runtime_state)
      .bind(room_id)
      .execute(&state.pool)
      .await
      .map_err(|e| {
        eprintln!("batch_power_by_rooms error: {e}");
        err(5000, "服务器错误")
      })?;
      affected += r.rows_affected() as i64;
    }
  }

  Ok(ok(json!({ "affected": affected, "action": body.action })))
}

async fn list_sensor_groups(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let groups: Vec<GroupRow> =
    sqlx::query_as("SELECT id, name, description, tags FROM ph_sensor_group WHERE status = 1 ORDER BY id")
      .fetch_all(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;

  let mut result = Vec::new();
  for g in groups {
    let sensor_ids: Vec<(i64,)> =
      sqlx::query_as("SELECT id FROM ph_sensor WHERE group_id = ? AND status = 1")
        .bind(g.id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| err(5000, "服务器错误"))?;
    result.push(json!({
      "id": g.id.to_string(),
      "name": g.name,
      "description": g.description,
      "tags": g.tags.unwrap_or(json!([])),
      "sensorIds": sensor_ids.into_iter().map(|(id,)| id.to_string()).collect::<Vec<_>>()
    }));
  }
  Ok(ok(result))
}

async fn list_limits(
  State(state): State<AppState>,
  Query(q): Query<CategoryQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<LimitRow> = if let Some(cat) = q.category.filter(|c| !c.is_empty()) {
    sqlx::query_as(
      "SELECT l.id, l.sensor_id, l.data_type,
              CAST(l.warning_limit AS DOUBLE) AS warning_limit,
              CAST(l.alarm_limit AS DOUBLE) AS alarm_limit,
              l.effective_from, l.changed_by
       FROM ph_sensor_limit l
       INNER JOIN ph_sensor s ON s.id = l.sensor_id
       WHERE s.category = ?
       ORDER BY l.id",
    )
    .bind(cat)
    .fetch_all(&state.pool)
    .await
  } else {
    sqlx::query_as(
      "SELECT id, sensor_id, data_type,
              CAST(warning_limit AS DOUBLE) AS warning_limit,
              CAST(alarm_limit AS DOUBLE) AS alarm_limit,
              effective_from, changed_by
       FROM ph_sensor_limit ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
  }
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|l| {
        json!({
          "id": l.id.to_string(),
          "sensorId": l.sensor_id.to_string(),
          "dataType": l.data_type,
          "warningLimit": l.warning_limit,
          "alarmLimit": l.alarm_limit,
          "effectiveFrom": l.effective_from.format("%Y-%m-%dT%H:%M:%S").to_string(),
          "changedBy": l.changed_by
        })
      })
      .collect(),
  ))
}

async fn upsert_limit(
  State(state): State<AppState>,
  Json(body): Json<LimitUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let sensor_id = parse_i64(&body.sensor_id).ok_or(err(10000, "sensorId 无效"))?;
  let existing: Option<LimitRow> = sqlx::query_as(
    "SELECT id, sensor_id, data_type,
            CAST(warning_limit AS DOUBLE) AS warning_limit,
            CAST(alarm_limit AS DOUBLE) AS alarm_limit,
            effective_from, changed_by
     FROM ph_sensor_limit WHERE sensor_id = ? AND data_type = ?",
  )
  .bind(sensor_id)
  .bind(&body.data_type)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let changed_by = body.changed_by.unwrap_or_else(|| "system".to_string());

  if let Some(old) = existing {
    let before = json!({
      "warningLimit": old.warning_limit,
      "alarmLimit": old.alarm_limit
    });
    sqlx::query(
      "UPDATE ph_sensor_limit SET warning_limit = ?, alarm_limit = ?, changed_by = ?,
       effective_from = datetime('now'), updated_at = datetime('now') WHERE id = ?",
    )
    .bind(body.warning_limit)
    .bind(body.alarm_limit)
    .bind(&changed_by)
    .bind(old.id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

    let after = json!({
      "warningLimit": body.warning_limit,
      "alarmLimit": body.alarm_limit
    });
    sqlx::query(
      "INSERT INTO ph_sensor_limit_history (limit_id, before_json, after_json, signed_by, reason)
       VALUES (?, ?, ?, ?, ?)",
    )
    .bind(old.id)
    .bind(before)
    .bind(after)
    .bind(&changed_by)
    .bind(body.reason)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

    Ok(ok(json!({ "id": old.id })))
  } else {
    let result = sqlx::query(
      "INSERT INTO ph_sensor_limit (sensor_id, data_type, warning_limit, alarm_limit, changed_by)
       VALUES (?, ?, ?, ?, ?)",
    )
    .bind(sensor_id)
    .bind(&body.data_type)
    .bind(body.warning_limit)
    .bind(body.alarm_limit)
    .bind(&changed_by)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
    Ok(ok(json!({ "id": result.last_insert_rowid() })))
  }
}

async fn list_limit_history(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, i64, Option<Value>, Option<Value>, String, chrono::NaiveDateTime, Option<String>)> =
    sqlx::query_as(
      "SELECT id, limit_id, before_json, after_json, signed_by, signed_at, reason
       FROM ph_sensor_limit_history ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, limit_id, before, after, signed_by, signed_at, reason)| {
        json!({
          "id": id.to_string(),
          "limitId": limit_id.to_string(),
          "before": before,
          "after": after,
          "signedBy": signed_by,
          "signedAt": signed_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
          "reason": reason
        })
      })
      .collect(),
  ))
}

async fn list_recipes(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<RecipeRow> = sqlx::query_as(
    "SELECT id, name, description, sensor_group_ids, pens, created_by, status, updated_at, created_at
     FROM ph_recipe WHERE status = 'active' ORDER BY id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(rows.into_iter().map(recipe_json).collect()))
}

async fn get_recipe(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<RecipeRow> = sqlx::query_as(
    "SELECT id, name, description, sensor_group_ids, pens, created_by, status, updated_at, created_at
     FROM ph_recipe WHERE id = ?",
  )
  .bind(id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  let row = row.ok_or(err(4040, "配方不存在"))?;
  Ok(ok(recipe_json(row)))
}

async fn create_recipe(
  State(state): State<AppState>,
  Json(body): Json<RecipeUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  if body.name.trim().is_empty() {
    return Err(err(10000, "配方名称不能为空"));
  }
  let group_ids = json!(body.sensor_group_ids);
  let result = sqlx::query(
    "INSERT INTO ph_recipe (name, description, sensor_group_ids, pens, created_by, status)
     VALUES (?, ?, ?, ?, ?, 'active')",
  )
  .bind(body.name.trim())
  .bind(body.description)
  .bind(group_ids)
  .bind(body.pens)
  .bind(body.created_by.unwrap_or_else(|| "system".to_string()))
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_recipe(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<RecipeUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let group_ids = json!(body.sensor_group_ids);
  sqlx::query(
    "UPDATE ph_recipe SET name = ?, description = ?, sensor_group_ids = ?, pens = ?, updated_at = datetime('now')
     WHERE id = ? AND status = 'active'",
  )
  .bind(body.name.trim())
  .bind(body.description)
  .bind(group_ids)
  .bind(body.pens)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn delete_recipe(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("UPDATE ph_recipe SET status = 'deleted', updated_at = datetime('now') WHERE id = ?")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn list_samplings(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<SamplingRow> = sqlx::query_as(
    "SELECT id, recipe_id, recipe_name, scheduled_at, sampling_mode, custom_fields, input_notes,
            status, started_at, finished_at, abort_reason, created_by
     FROM ph_sampling_task ORDER BY scheduled_at DESC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(rows.into_iter().map(sampling_json).collect()))
}

async fn create_sampling(
  State(state): State<AppState>,
  Json(body): Json<SamplingUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let recipe: Option<(String,)> = sqlx::query_as("SELECT name FROM ph_recipe WHERE id = ? AND status = 'active'")
    .bind(body.recipe_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  let (recipe_name,) = recipe.ok_or(err(4040, "配方不存在"))?;

  let result = sqlx::query(
    "INSERT INTO ph_sampling_task
     (recipe_id, recipe_name, scheduled_at, sampling_mode, custom_fields, input_notes, status, created_by)
     VALUES (?, ?, ?, ?, ?, ?, 'scheduled', ?)",
  )
  .bind(body.recipe_id)
  .bind(recipe_name)
  .bind(&body.scheduled_at)
  .bind(body.sampling_mode.unwrap_or_else(|| "Operational".to_string()))
  .bind(body.custom_fields.unwrap_or(json!({})))
  .bind(body.input_notes)
  .bind(body.created_by.unwrap_or_else(|| "system".to_string()))
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_sampling(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<SamplingUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "UPDATE ph_sampling_task SET scheduled_at = ?, sampling_mode = COALESCE(?, sampling_mode),
     custom_fields = COALESCE(?, custom_fields), input_notes = ?, updated_at = datetime('now')
     WHERE id = ? AND status IN ('scheduled')",
  )
  .bind(&body.scheduled_at)
  .bind(body.sampling_mode)
  .bind(body.custom_fields)
  .bind(body.input_notes)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn delete_sampling(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("DELETE FROM ph_sampling_task WHERE id = ? AND status = 'scheduled'")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn abort_sampling(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<AbortBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let result = sqlx::query(
    "UPDATE ph_sampling_task SET status = 'aborted', abort_reason = ?, finished_at = datetime('now'), updated_at = datetime('now')
     WHERE id = ? AND status IN ('scheduled', 'running')",
  )
  .bind(body.reason)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  if result.rows_affected() == 0 {
    return Err(err(10000, "当前状态不可中止"));
  }
  Ok(ok(true))
}

async fn list_custom_fields(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<CustomFieldRow> = sqlx::query_as(
    "SELECT id, field_key, display_name, enabled, editable, default_entries, sort_order
     FROM ph_sampling_custom_field ORDER BY sort_order, id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|f| {
        json!({
          "key": f.field_key,
          "displayName": f.display_name,
          "enabled": f.enabled == 1,
          "editable": f.editable == 1,
          "defaultEntries": f.default_entries.unwrap_or(json!([])),
          "sortOrder": f.sort_order
        })
      })
      .collect(),
  ))
}

async fn replace_custom_fields(
  State(state): State<AppState>,
  Json(body): Json<Vec<CustomFieldBody>>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let mut tx = state.pool.begin().await.map_err(|_| err(5000, "服务器错误"))?;
  sqlx::query("DELETE FROM ph_sampling_custom_field")
    .execute(&mut *tx)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  for (idx, field) in body.into_iter().enumerate() {
    sqlx::query(
      "INSERT INTO ph_sampling_custom_field
       (field_key, display_name, enabled, editable, default_entries, sort_order)
       VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(field.key)
    .bind(field.display_name)
    .bind(if field.enabled.unwrap_or(true) { 1 } else { 0 })
    .bind(if field.editable.unwrap_or(true) { 1 } else { 0 })
    .bind(field.default_entries.unwrap_or(json!([])))
    .bind(field.sort_order.unwrap_or(idx as i32))
    .execute(&mut *tx)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  }

  tx.commit().await.map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}


#[derive(Deserialize)]
struct RoomQuery {
  #[serde(rename = "roomId")]
  room_id: Option<String>,
}

#[derive(Deserialize)]
struct DeviceConfigBody {
  #[serde(rename = "facilityNodeId")]
  facility_node_id: Option<Value>,
  #[serde(rename = "deviceName")]
  device_name: Option<String>,
  #[serde(rename = "deviceCode")]
  device_code: Option<String>,
  #[serde(rename = "instrumentType")]
  instrument_type: Option<String>,
  #[serde(rename = "updateIntervalSec")]
  update_interval_sec: Option<i32>,
  #[serde(rename = "slaveAddress")]
  slave_address: Option<String>,
  #[serde(rename = "plcIp")]
  plc_ip: Option<String>,
  #[serde(rename = "dataUnit")]
  data_unit: Option<String>,
  #[serde(rename = "dataType")]
  data_type: Option<String>,
  #[serde(rename = "dataLength")]
  data_length: Option<i32>,
  protocol: Option<String>,
  #[serde(rename = "decimalPlaces")]
  decimal_places: Option<i32>,
  #[serde(rename = "cleanroomClass")]
  cleanroom_class: Option<String>,
  #[serde(rename = "serialNumber")]
  serial_number: Option<String>,
  #[serde(rename = "calibrationDate")]
  calibration_date: Option<String>,
  #[serde(rename = "operatingMode")]
  operating_mode: Option<String>,
  #[serde(rename = "productionState")]
  production_state: Option<String>,
  #[serde(rename = "flowRate")]
  flow_rate: Option<f64>,
  thresholds: Option<Vec<ThresholdBody>>,
}

#[derive(Deserialize)]
struct ThresholdBody {
  #[serde(rename = "stateGroup")]
  state_group: String,
  #[serde(rename = "metricKey")]
  metric_key: Option<String>,
  #[serde(rename = "alarmEnable")]
  alarm_enable: Option<String>,
  #[serde(rename = "warnHigh")]
  warn_high: Option<f64>,
  #[serde(rename = "warnLow")]
  warn_low: Option<f64>,
  #[serde(rename = "alarmHigh")]
  alarm_high: Option<f64>,
  #[serde(rename = "alarmLow")]
  alarm_low: Option<f64>,
}

#[derive(FromRow)]
struct DeviceConfigRow {
  facility_node_id: Option<i64>,
  device_name: String,
  device_code: String,
  instrument_type: Option<String>,
  update_interval_sec: i32,
  slave_address: Option<String>,
  plc_ip: Option<String>,
  data_unit: Option<String>,
  data_type: String,
  data_length: i32,
  protocol: String,
  decimal_places: i32,
  cleanroom_class: Option<String>,
  serial_number: Option<String>,
  calibration_date: Option<chrono::NaiveDate>,
  operating_mode: String,
  production_state: String,
  flow_rate: Option<f64>,
}

#[derive(FromRow)]
struct ThresholdRow {
  state_group: String,
  metric_key: String,
  alarm_enable: String,
  warn_high: Option<f64>,
  warn_low: Option<f64>,
  alarm_high: Option<f64>,
  alarm_low: Option<f64>,
}

#[derive(FromRow)]
struct RealtimeSignalRow {
  id: i64,
  custom_id: String,
  runtime_state: String,
  power_on: i8,
  com_alarm: i8,
  flow_calc_alarm: i8,
  last_value: Option<Value>,
  unit: Option<String>,
  device_name: String,
  device_code: String,
  instrument_type: Option<String>,
  decimal_places: i32,
  cleanroom_class: Option<String>,
  serial_number: Option<String>,
  calibration_date: Option<chrono::NaiveDate>,
  operating_mode: String,
  production_state: String,
  flow_rate: Option<f64>,
  slave_address: Option<String>,
  room_code: Option<String>,
  room_name: Option<String>,
}

fn parse_opt_id(value: &Option<Value>) -> Option<i64> {
  match value {
    Some(Value::Number(n)) => n.as_i64(),
    Some(Value::String(s)) if !s.is_empty() => s.parse().ok(),
    _ => None,
  }
}

fn eval_metric_status(
  value: f64,
  enable: &str,
  warn_high: Option<f64>,
  warn_low: Option<f64>,
  alarm_high: Option<f64>,
  alarm_low: Option<f64>,
) -> &'static str {
  if enable == "unlimited" || enable == "disabled" {
    return "normal";
  }
  if alarm_high.map(|h| value > h).unwrap_or(false) || alarm_low.map(|l| value < l).unwrap_or(false) {
    return "alarm";
  }
  if warn_high.map(|h| value > h).unwrap_or(false) || warn_low.map(|l| value < l).unwrap_or(false) {
    return "warning";
  }
  "normal"
}

async fn get_device_config(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM ph_sensor WHERE id = ? AND status = 1")
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  if exists.is_none() {
    return Err(err(4040, "传感器不存在"));
  }

  let cfg: Option<DeviceConfigRow> = sqlx::query_as(
    "SELECT facility_node_id, device_name, device_code, instrument_type, update_interval_sec,
            slave_address, plc_ip, data_unit, data_type, data_length, protocol, decimal_places,
            cleanroom_class, serial_number, calibration_date, operating_mode, production_state, flow_rate
     FROM ph_sensor_device_config WHERE sensor_id = ?",
  )
  .bind(id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|e| {
    eprintln!("get_device_config error: {e}");
    err(5000, "服务器错误")
  })?;

  let thresholds: Vec<ThresholdRow> = sqlx::query_as(
    "SELECT state_group, metric_key, alarm_enable, warn_high, warn_low, alarm_high, alarm_low
     FROM ph_sensor_state_threshold WHERE sensor_id = ? ORDER BY state_group, metric_key",
  )
  .bind(id)
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let thr_json: Vec<Value> = thresholds
    .into_iter()
    .map(|t| {
      json!({
        "stateGroup": t.state_group,
        "metricKey": t.metric_key,
        "alarmEnable": t.alarm_enable,
        "warnHigh": t.warn_high,
        "warnLow": t.warn_low,
        "alarmHigh": t.alarm_high,
        "alarmLow": t.alarm_low
      })
    })
    .collect();

  if let Some(c) = cfg {
    Ok(ok(json!({
      "sensorId": id.to_string(),
      "facilityNodeId": c.facility_node_id.map(|v| v.to_string()),
      "deviceName": c.device_name,
      "deviceCode": c.device_code,
      "instrumentType": c.instrument_type,
      "updateIntervalSec": c.update_interval_sec,
      "slaveAddress": c.slave_address,
      "plcIp": c.plc_ip,
      "dataUnit": c.data_unit,
      "dataType": c.data_type,
      "dataLength": c.data_length,
      "protocol": c.protocol,
      "decimalPlaces": c.decimal_places,
      "cleanroomClass": c.cleanroom_class,
      "serialNumber": c.serial_number,
      "calibrationDate": c.calibration_date.map(|d| d.format("%Y-%m-%d").to_string()),
      "operatingMode": c.operating_mode,
      "productionState": c.production_state,
      "flowRate": c.flow_rate,
      "thresholds": thr_json
    })))
  } else {
    Ok(ok(json!({
      "sensorId": id.to_string(),
      "facilityNodeId": null,
      "deviceName": "",
      "deviceCode": "",
      "instrumentType": "尘埃粒子0.5",
      "updateIntervalSec": 60,
      "slaveAddress": "",
      "plcIp": "PLC1",
      "dataUnit": "pt/3",
      "dataType": "ulong",
      "dataLength": 4,
      "protocol": "ModbusTCP",
      "decimalPlaces": 0,
      "cleanroomClass": "Class A",
      "serialNumber": "",
      "calibrationDate": null,
      "operatingMode": "Operational",
      "productionState": "production",
      "flowRate": null,
      "thresholds": thr_json
    })))
  }
}

async fn upsert_device_config(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<DeviceConfigBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM ph_sensor WHERE id = ? AND status = 1")
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  if exists.is_none() {
    return Err(err(4040, "传感器不存在"));
  }

  let facility_node_id = parse_opt_id(&body.facility_node_id);
  let device_name = body.device_name.clone().unwrap_or_default();
  let device_code = body.device_code.clone().unwrap_or_default();

  sqlx::query(
    "INSERT INTO ph_sensor_device_config
     (sensor_id, facility_node_id, device_name, device_code, instrument_type, update_interval_sec,
      slave_address, plc_ip, data_unit, data_type, data_length, protocol, decimal_places,
      cleanroom_class, serial_number, calibration_date, operating_mode, production_state, flow_rate)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
     ON CONFLICT(sensor_id) DO UPDATE SET
       facility_node_id = excluded.facility_node_id,
       device_name = excluded.device_name,
       device_code = excluded.device_code,
       instrument_type = excluded.instrument_type,
       update_interval_sec = excluded.update_interval_sec,
       slave_address = excluded.slave_address,
       plc_ip = excluded.plc_ip,
       data_unit = excluded.data_unit,
       data_type = excluded.data_type,
       data_length = excluded.data_length,
       protocol = excluded.protocol,
       decimal_places = excluded.decimal_places,
       cleanroom_class = excluded.cleanroom_class,
       serial_number = excluded.serial_number,
       calibration_date = excluded.calibration_date,
       operating_mode = excluded.operating_mode,
       production_state = excluded.production_state,
       flow_rate = excluded.flow_rate",
  )
  .bind(id)
  .bind(facility_node_id)
  .bind(&device_name)
  .bind(&device_code)
  .bind(body.instrument_type)
  .bind(body.update_interval_sec.unwrap_or(60))
  .bind(body.slave_address)
  .bind(body.plc_ip)
  .bind(body.data_unit.clone())
  .bind(body.data_type.unwrap_or_else(|| "ulong".into()))
  .bind(body.data_length.unwrap_or(4))
  .bind(body.protocol.unwrap_or_else(|| "ModbusTCP".into()))
  .bind(body.decimal_places.unwrap_or(0))
  .bind(body.cleanroom_class)
  .bind(body.serial_number)
  .bind(body.calibration_date)
  .bind(body.operating_mode.unwrap_or_else(|| "Operational".into()))
  .bind(body.production_state.unwrap_or_else(|| "production".into()))
  .bind(body.flow_rate)
  .execute(&state.pool)
  .await
  .map_err(|e| {
    eprintln!("upsert_device_config error: {e}");
    err(5000, "服务器错误")
  })?;

  if !device_name.is_empty() || body.data_unit.is_some() {
    let _ = sqlx::query(
      "UPDATE ph_sensor SET
         custom_id = COALESCE(NULLIF(?, ''), custom_id),
         description = COALESCE(NULLIF(?, ''), description),
         unit = COALESCE(?, unit),
         updated_at = datetime('now')
       WHERE id = ?",
    )
    .bind(&device_code)
    .bind(&device_name)
    .bind(body.data_unit)
    .bind(id)
    .execute(&state.pool)
    .await;
  }

  if let Some(thresholds) = body.thresholds {
    for t in thresholds {
      let metric = t.metric_key.unwrap_or_else(|| "0.5um".into());
      sqlx::query(
        "INSERT INTO ph_sensor_state_threshold
         (sensor_id, state_group, metric_key, alarm_enable, warn_high, warn_low, alarm_high, alarm_low)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(sensor_id, state_group, metric_key) DO UPDATE SET
           alarm_enable = excluded.alarm_enable,
           warn_high = excluded.warn_high,
           warn_low = excluded.warn_low,
           alarm_high = excluded.alarm_high,
           alarm_low = excluded.alarm_low",
      )
      .bind(id)
      .bind(t.state_group)
      .bind(metric)
      .bind(t.alarm_enable.unwrap_or_else(|| "unlimited".into()))
      .bind(t.warn_high)
      .bind(t.warn_low)
      .bind(t.alarm_high)
      .bind(t.alarm_low)
      .execute(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
    }
  }

  Ok(ok(true))
}

async fn realtime_rooms(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rooms: Vec<(i64, String, String, Option<String>)> = sqlx::query_as(
    "SELECT id, code, name, description FROM ph_facility_node WHERE level = 3 AND status = 1 ORDER BY sort_order, id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let mut result = Vec::new();
  for (room_id, code, name, description) in rooms {
    let cards = load_room_signals(&state.pool, room_id).await.unwrap_or_default();
    let mut alarm_level = "normal";
    for card in &cards {
      match card.get("cardStatus").and_then(|v| v.as_str()) {
        Some("alarm") => alarm_level = "alarm",
        Some("warning") if alarm_level != "alarm" => alarm_level = "warning",
        _ => {}
      }
    }
    result.push(json!({
      "id": room_id.to_string(),
      "code": code,
      "name": name,
      "description": description,
      "deviceCount": cards.len(),
      "alarmLevel": alarm_level
    }));
  }
  Ok(ok(result))
}

async fn load_room_signals(
  pool: &crate::db::DbPool,
  room_id: i64,
) -> Result<Vec<Value>, sqlx::Error> {
  let rows: Vec<RealtimeSignalRow> = sqlx::query_as(
    "SELECT s.id, s.custom_id, s.runtime_state, s.power_on, s.com_alarm, s.flow_calc_alarm, s.last_value, s.unit,
            c.device_name, c.device_code, c.instrument_type, c.decimal_places,
            c.cleanroom_class, c.serial_number, c.calibration_date, c.operating_mode, c.production_state, c.flow_rate,
            c.slave_address, f.code AS room_code, f.name AS room_name
     FROM ph_sensor s
     INNER JOIN ph_sensor_device_config c ON c.sensor_id = s.id
     LEFT JOIN ph_facility_node f ON f.id = c.facility_node_id
     WHERE s.status = 1 AND c.facility_node_id = ?
     ORDER BY s.id",
  )
  .bind(room_id)
  .fetch_all(pool)
  .await?;

  let mut cards = Vec::new();
  for row in rows {
    let thr_rows: Vec<ThresholdRow> = sqlx::query_as(
      "SELECT state_group, metric_key, alarm_enable, warn_high, warn_low, alarm_high, alarm_low
       FROM ph_sensor_state_threshold
       WHERE sensor_id = ? AND state_group = ?",
    )
    .bind(row.id)
    .bind(&row.production_state)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut metrics = Vec::new();
    let mut card_status = "normal";
    if let Some(Value::Object(map)) = &row.last_value {
      for (metric, val) in map {
        if let Some(num) = val.as_f64() {
          let thr = thr_rows.iter().find(|t| &t.metric_key == metric);
          let (enable, wh, wl, ah, al) = if let Some(t) = thr {
            (t.alarm_enable.as_str(), t.warn_high, t.warn_low, t.alarm_high, t.alarm_low)
          } else {
            ("unlimited", None, None, None, None)
          };
          let st = eval_metric_status(num, enable, wh, wl, ah, al);
          if st == "alarm" {
            card_status = "alarm";
          } else if st == "warning" && card_status != "alarm" {
            card_status = "warning";
          }
          let display = if row.decimal_places <= 0 {
            format!("{:.0}", num)
          } else {
            format!("{:.prec$}", num, prec = row.decimal_places as usize)
          };
          metrics.push(json!({
            "key": metric,
            "label": format!("Part >= {}", metric),
            "value": num,
            "displayValue": display,
            "status": st,
            "statusLabel": match st {
              "warning" => "预警",
              "alarm" => "报警",
              _ => "正常"
            }
          }));
        }
      }
    }

    let device_code = if row.device_code.is_empty() {
      row.custom_id.clone()
    } else {
      row.device_code.clone()
    };

    cards.push(json!({
      "sensorId": row.id.to_string(),
      "deviceCode": device_code,
      "deviceName": row.device_name,
      "roomCode": row.room_code,
      "roomName": row.room_name,
      "cardStatus": card_status,
      "metrics": metrics,
      "flowRate": row.flow_rate,
      "unit": row.unit,
      "cleanroomClass": row.cleanroom_class,
      "serialNumber": row.serial_number,
      "calibrationDate": row.calibration_date.map(|d| d.format("%m/%d/%Y").to_string()),
      "operatingMode": row.operating_mode,
      "productionState": row.production_state,
      "instrumentType": row.instrument_type,
      "slaveAddress": row.slave_address,
      "powerOn": row.power_on == 1,
      "runtimeState": row.runtime_state,
      "comAlarm": row.com_alarm == 1,
      "flowCalcAlarm": row.flow_calc_alarm == 1,
      "decimalPlaces": row.decimal_places
    }));
  }
  Ok(cards)
}

async fn realtime_signals(
  State(state): State<AppState>,
  Query(q): Query<RoomQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let room_id = q
    .room_id
    .as_ref()
    .and_then(|s| s.parse::<i64>().ok())
    .ok_or(err(10000, "请指定 roomId"))?;
  let cards = load_room_signals(&state.pool, room_id).await.map_err(|e| {
    eprintln!("realtime_signals error: {e}");
    err(5000, "服务器错误")
  })?;
  Ok(ok(cards))
}

#[allow(dead_code)]
fn _pool_type_check(_: &DbPool) {}
