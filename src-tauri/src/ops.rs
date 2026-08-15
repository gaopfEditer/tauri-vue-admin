use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  routing::{get, post, put},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::server::{err, ok, ApiError, ApiResponse, AppState};

#[derive(Deserialize)]
struct AckBody {
  #[serde(rename = "acknowledgedBy")]
  acknowledged_by: Option<String>,
  reason: Option<String>,
}

#[derive(Deserialize)]
struct RtStartBody {
  #[serde(rename = "recipeId")]
  recipe_id: i64,
  #[serde(rename = "startedBy")]
  started_by: Option<String>,
  pens: Option<Value>,
}

#[derive(Deserialize)]
struct RtStopBody {
  #[serde(rename = "sessionId")]
  session_id: Option<i64>,
}

#[derive(Deserialize)]
struct RuleUpsertBody {
  name: String,
  #[serde(rename = "ruleType")]
  rule_type: Option<String>,
  #[serde(rename = "parentId")]
  parent_id: Option<i64>,
  conditions: Value,
  actions: Value,
  enabled: Option<bool>,
  #[serde(rename = "recipeId")]
  recipe_id: Option<i64>,
}

#[derive(Deserialize)]
struct TestOutputBody {
  #[serde(rename = "tagId")]
  tag_id: i64,
  value: Value,
}

#[derive(Deserialize, Serialize)]
struct ReportGenerateBody {
  kind: String,
  mode: String,
  #[serde(rename = "dateFrom")]
  date_from: String,
  #[serde(rename = "dateTo")]
  date_to: String,
  interval: Option<String>,
  #[serde(rename = "groupIds")]
  group_ids: Option<Vec<Value>>,
  #[serde(rename = "recipeId")]
  recipe_id: Option<Value>,
  #[serde(rename = "samplingIds")]
  sampling_ids: Option<Vec<Value>>,
  #[serde(rename = "sensorIds")]
  sensor_ids: Option<Vec<Value>>,
  #[serde(rename = "trendCalc")]
  trend_calc: Option<Value>,
  #[serde(rename = "generatedBy")]
  generated_by: Option<String>,
}

#[derive(Deserialize)]
struct TagCatalogQuery {
  tab: Option<String>,
}

#[derive(Deserialize)]
struct TagCatalogUpsert {
  tab: String,
  name: String,
  color: Option<String>,
  #[serde(rename = "linkedIds")]
  linked_ids: Option<Value>,
  enabled: Option<bool>,
}

#[derive(Deserialize)]
struct SdaUpsertBody {
  #[serde(rename = "configName")]
  config_name: Option<String>,
  #[serde(rename = "csvFiles")]
  csv_files: Option<Value>,
  #[serde(rename = "csvContent")]
  csv_content: Option<String>,
  #[serde(rename = "displayConfig")]
  display_config: Option<Value>,
  #[serde(rename = "virtualPens")]
  virtual_pens: Option<Value>,
  #[serde(rename = "markersEnabled")]
  markers_enabled: Option<bool>,
}

#[derive(Deserialize)]
struct ExportQuery {
  format: Option<String>,
}

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/api/alarms", get(list_alarms))
    .route("/api/alarms/:id/ack", post(ack_alarm))
    .route("/api/alarms/ack-all", post(ack_all_alarms))
    .route("/api/rt-trend/start", post(rt_start))
    .route("/api/rt-trend/stop", post(rt_stop))
    .route("/api/rt-trend/current", get(rt_current))
    .route("/api/rt-trend/points", get(rt_points))
    .route("/api/runtime-tags", get(list_runtime_tags))
    .route("/api/runtime-rules", get(list_runtime_rules).post(create_runtime_rule))
    .route(
      "/api/runtime-rules/:id",
      put(update_runtime_rule).delete(delete_runtime_rule),
    )
    .route("/api/runtime-rules/:id/test-output", post(test_output))
    .route("/api/reports/generate", post(generate_report))
    .route("/api/reports", get(list_reports))
    .route("/api/reports/:id", get(get_report))
    .route("/api/reports/:id/export", get(export_report))
    .route("/api/tags-catalog", get(list_tag_catalog).post(create_tag_catalog))
    .route("/api/tags-catalog/:id", put(update_tag_catalog).delete(delete_tag_catalog))
    .route("/api/sda/sessions", get(list_sda).post(create_sda))
    .route("/api/sda/sessions/:id", get(get_sda).put(update_sda))
    .route("/api/sda/analyze", post(analyze_sda))
}

async fn insert_audit(
  pool: &crate::db::DbPool,
  action: &str,
  target_type: &str,
  target_id: Option<&str>,
  performed_by: &str,
  reason: Option<&str>,
  detail: Option<Value>,
) {
  let _ = sqlx::query(
    "INSERT INTO ph_audit_trail (action, target_type, target_id, performed_by, role_code, reason, detail_json)
     VALUES (?, ?, ?, ?, 'user', ?, ?)",
  )
  .bind(action)
  .bind(target_type)
  .bind(target_id)
  .bind(performed_by)
  .bind(reason)
  .bind(detail)
  .execute(pool)
  .await;
}

async fn list_alarms(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(
    i64,
    Option<i64>,
    String,
    String,
    String,
    Option<String>,
    Option<f64>,
    Option<f64>,
    Option<i64>,
    chrono::NaiveDateTime,
    i8,
    Option<String>,
    Option<chrono::NaiveDateTime>,
  )> = sqlx::query_as(
    "SELECT id, sensor_id, sensor_name, severity, message, data_type, value, limit_value, sampling_id,
            raised_at, acknowledged, acknowledged_by, acknowledged_at
     FROM ph_alarm_event ORDER BY acknowledged ASC, raised_at DESC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(
        |(
          id,
          sensor_id,
          sensor_name,
          severity,
          message,
          data_type,
          value,
          limit_value,
          sampling_id,
          raised_at,
          acknowledged,
          acknowledged_by,
          acknowledged_at,
        )| {
          json!({
            "id": id.to_string(),
            "sensorId": sensor_id.map(|v| v.to_string()),
            "sensorName": sensor_name,
            "severity": severity,
            "message": message,
            "dataType": data_type,
            "value": value,
            "limitValue": limit_value,
            "samplingId": sampling_id.map(|v| v.to_string()),
            "raisedAt": raised_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            "acknowledged": acknowledged == 1,
            "acknowledgedBy": acknowledged_by,
            "acknowledgedAt": acknowledged_at.map(|v| v.format("%Y-%m-%dT%H:%M:%S").to_string())
          })
        },
      )
      .collect(),
  ))
}

async fn ack_alarm(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<AckBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let by = body.acknowledged_by.unwrap_or_else(|| "system".to_string());
  let result = sqlx::query(
    "UPDATE ph_alarm_event SET acknowledged = 1, acknowledged_by = ?, acknowledged_at = datetime('now')
     WHERE id = ? AND acknowledged = 0",
  )
  .bind(&by)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  if result.rows_affected() == 0 {
    return Err(err(10000, "报警不存在或已确认"));
  }

  insert_audit(
    &state.pool,
    "alarm.ack",
    "alarm",
    Some(&id.to_string()),
    &by,
    body.reason.as_deref(),
    None,
  )
  .await;

  Ok(ok(true))
}

async fn ack_all_alarms(
  State(state): State<AppState>,
  Json(body): Json<AckBody>,
) -> Result<Json<ApiResponse<i64>>, (StatusCode, Json<ApiError>)> {
  let by = body.acknowledged_by.unwrap_or_else(|| "system".to_string());
  let result = sqlx::query(
    "UPDATE ph_alarm_event SET acknowledged = 1, acknowledged_by = ?, acknowledged_at = datetime('now')
     WHERE acknowledged = 0",
  )
  .bind(&by)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  insert_audit(
    &state.pool,
    "alarm.ackAll",
    "alarm",
    None,
    &by,
    body.reason.as_deref(),
    Some(json!({ "count": result.rows_affected() })),
  )
  .await;

  Ok(ok(result.rows_affected() as i64))
}

async fn rt_start(
  State(state): State<AppState>,
  Json(body): Json<RtStartBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let recipe: Option<(String, Value)> =
    sqlx::query_as("SELECT name, pens FROM ph_recipe WHERE id = ? AND status = 'active'")
      .bind(body.recipe_id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  let (recipe_name, recipe_pens) = recipe.ok_or(err(4040, "配方不存在"))?;
  let pens = body.pens.unwrap_or(recipe_pens);
  let by = body.started_by.unwrap_or_else(|| "system".to_string());

  sqlx::query("UPDATE ph_rt_trend_session SET running = 0, stopped_at = datetime('now') WHERE running = 1")
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  let result = sqlx::query(
    "INSERT INTO ph_rt_trend_session (recipe_id, recipe_name, running, pens, started_at, started_by)
     VALUES (?, ?, 1, ?, datetime('now'), ?)",
  )
  .bind(body.recipe_id)
  .bind(&recipe_name)
  .bind(&pens)
  .bind(&by)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  insert_audit(
    &state.pool,
    "rtTrend.start",
    "rt-trend",
    Some(&result.last_insert_rowid().to_string()),
    &by,
    None,
    Some(json!({ "recipeId": body.recipe_id })),
  )
  .await;

  Ok(ok(json!({
    "id": result.last_insert_rowid().to_string(),
    "recipeId": body.recipe_id.to_string(),
    "recipeName": recipe_name,
    "running": true,
    "pens": pens,
    "startedBy": by
  })))
}

async fn rt_stop(
  State(state): State<AppState>,
  Json(body): Json<RtStopBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  let result = if let Some(id) = body.session_id {
    sqlx::query("UPDATE ph_rt_trend_session SET running = 0, stopped_at = datetime('now') WHERE id = ? AND running = 1")
      .bind(id)
      .execute(&state.pool)
      .await
  } else {
    sqlx::query("UPDATE ph_rt_trend_session SET running = 0, stopped_at = datetime('now') WHERE running = 1")
      .execute(&state.pool)
      .await
  }
  .map_err(|_| err(5000, "服务器错误"))?;

  if result.rows_affected() == 0 {
    return Err(err(10000, "没有运行中的趋势会话"));
  }

  insert_audit(&state.pool, "rtTrend.stop", "rt-trend", None, "system", None, None).await;
  Ok(ok(true))
}

async fn rt_current(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i64, i64, String, i8, Value, Option<chrono::NaiveDateTime>, Option<String>)> =
    sqlx::query_as(
      "SELECT id, recipe_id, recipe_name, running, pens, started_at, started_by
       FROM ph_rt_trend_session ORDER BY id DESC LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  if let Some((id, recipe_id, recipe_name, running, pens, started_at, started_by)) = row {
    Ok(ok(json!({
      "id": id.to_string(),
      "recipeId": recipe_id.to_string(),
      "recipeName": recipe_name,
      "running": running == 1,
      "pens": pens,
      "startedAt": started_at.map(|v| v.format("%Y-%m-%dT%H:%M:%S").to_string()),
      "startedBy": started_by
    })))
  } else {
    Ok(ok(Value::Null))
  }
}

async fn rt_points(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  // 简化：从传感器 last_value 生成实时点（替代 WebSocket）
  let rows: Vec<(i64, String, Option<Value>)> =
    sqlx::query_as("SELECT id, custom_id, last_value FROM ph_sensor WHERE status = 1 AND power_on = 1")
      .fetch_all(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;

  let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
  let mut points = Vec::new();
  for (id, _custom_id, last_value) in rows {
    if let Some(Value::Object(map)) = last_value {
      for (data_type, val) in map {
        if let Some(num) = val.as_f64() {
          points.push(json!({
            "sensorId": id.to_string(),
            "dataType": data_type,
            "timestamp": now,
            "value": num
          }));
        }
      }
    }
  }
  Ok(ok(points))
}

async fn list_runtime_tags(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, String, String, Option<Value>, Option<String>)> = sqlx::query_as(
    "SELECT id, folder, name, data_type, value_json, path FROM ph_runtime_tag WHERE status = 1 ORDER BY folder, id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, folder, name, data_type, value_json, path)| {
        json!({
          "id": id.to_string(),
          "folder": folder,
          "name": name,
          "dataType": data_type,
          "value": value_json,
          "path": path
        })
      })
      .collect(),
  ))
}

async fn list_runtime_rules(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(
    i64,
    String,
    String,
    Option<i64>,
    Value,
    Value,
    i8,
    Option<i64>,
    chrono::NaiveDateTime,
  )> = sqlx::query_as(
    "SELECT id, name, rule_type, parent_id, conditions, actions, enabled, recipe_id, updated_at
     FROM ph_runtime_rule ORDER BY id",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(
        |(id, name, rule_type, parent_id, conditions, actions, enabled, recipe_id, updated_at)| {
          json!({
            "id": id.to_string(),
            "name": name,
            "ruleType": rule_type,
            "parentId": parent_id.map(|v| v.to_string()),
            "conditions": conditions,
            "actions": actions,
            "enabled": enabled == 1,
            "recipeId": recipe_id.map(|v| v.to_string()),
            "updatedAt": updated_at.format("%Y-%m-%dT%H:%M:%S").to_string()
          })
        },
      )
      .collect(),
  ))
}

async fn create_runtime_rule(
  State(state): State<AppState>,
  Json(body): Json<RuleUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let result = sqlx::query(
    "INSERT INTO ph_runtime_rule (name, rule_type, parent_id, conditions, actions, enabled, recipe_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(body.name)
  .bind(body.rule_type.unwrap_or_else(|| "condition".to_string()))
  .bind(body.parent_id)
  .bind(body.conditions)
  .bind(body.actions)
  .bind(if body.enabled.unwrap_or(true) { 1 } else { 0 })
  .bind(body.recipe_id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_runtime_rule(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<RuleUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "UPDATE ph_runtime_rule SET name = ?, rule_type = COALESCE(?, rule_type), parent_id = ?,
     conditions = ?, actions = ?, enabled = ?, recipe_id = ?, updated_at = datetime('now') WHERE id = ?",
  )
  .bind(body.name)
  .bind(body.rule_type)
  .bind(body.parent_id)
  .bind(body.conditions)
  .bind(body.actions)
  .bind(if body.enabled.unwrap_or(true) { 1 } else { 0 })
  .bind(body.recipe_id)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn delete_runtime_rule(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("DELETE FROM ph_runtime_rule WHERE id = ?")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn test_output(
  State(state): State<AppState>,
  Path(_id): Path<i64>,
  Json(body): Json<TestOutputBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("UPDATE ph_runtime_tag SET value_json = ? WHERE id = ?")
    .bind(body.value)
    .bind(body.tag_id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn generate_report(
  State(state): State<AppState>,
  Json(body): Json<ReportGenerateBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let by = body.generated_by.clone().unwrap_or_else(|| "system".to_string());
  let (columns, rows, summary) = match body.kind.as_str() {
    "audit" => {
      let audits: Vec<(
        chrono::NaiveDateTime,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
      )> = sqlx::query_as(
        "SELECT event_time, action, target_type, target_id, performed_by, reason
         FROM ph_audit_trail
         WHERE event_time BETWEEN ? AND ?
         ORDER BY event_time DESC",
      )
      .bind(&body.date_from)
      .bind(&body.date_to)
      .fetch_all(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;

      let rows: Vec<Value> = audits
        .into_iter()
        .map(|(t, action, target_type, target_id, performed_by, reason)| {
          json!({
            "eventTime": t.format("%Y-%m-%d %H:%M:%S").to_string(),
            "action": action,
            "targetType": target_type,
            "targetId": target_id,
            "performedBy": performed_by,
            "reason": reason
          })
        })
        .collect();
      let count = rows.len();
      (
        vec![
          "eventTime".to_string(),
          "action".to_string(),
          "targetType".to_string(),
          "targetId".to_string(),
          "performedBy".to_string(),
          "reason".to_string(),
        ],
        rows,
        json!({ "totalEvents": count }),
      )
    }
    "sampling" => {
      let tasks: Vec<(i64, String, chrono::NaiveDateTime, String, String, String)> = sqlx::query_as(
        "SELECT id, recipe_name, scheduled_at, sampling_mode, status, created_by
         FROM ph_sampling_task
         WHERE scheduled_at BETWEEN ? AND ?
         ORDER BY scheduled_at DESC",
      )
      .bind(&body.date_from)
      .bind(&body.date_to)
      .fetch_all(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
      let rows: Vec<Value> = tasks
        .into_iter()
        .map(|(id, recipe_name, scheduled_at, mode, status, created_by)| {
          json!({
            "id": id.to_string(),
            "recipeName": recipe_name,
            "scheduledAt": scheduled_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            "samplingMode": mode,
            "status": status,
            "createdBy": created_by
          })
        })
        .collect();
      let count = rows.len();
      (
        vec![
          "id".to_string(),
          "recipeName".to_string(),
          "scheduledAt".to_string(),
          "samplingMode".to_string(),
          "status".to_string(),
          "createdBy".to_string(),
        ],
        rows,
        json!({ "totalSamplings": count }),
      )
    }
    "trend" | "data" => {
      let sensors: Vec<(i64, String, String, Option<Value>)> = sqlx::query_as(
        "SELECT id, custom_id, category, last_value FROM ph_sensor WHERE status = 1 ORDER BY id",
      )
      .fetch_all(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
      let rows: Vec<Value> = sensors
        .into_iter()
        .map(|(id, custom_id, category, last_value)| {
          json!({
            "sensorId": id.to_string(),
            "customId": custom_id,
            "category": category,
            "lastValue": last_value,
            "aggregation": body.trend_calc.clone().unwrap_or(json!({}))
          })
        })
        .collect();
      let count = rows.len();
      (
        vec![
          "sensorId".to_string(),
          "customId".to_string(),
          "category".to_string(),
          "lastValue".to_string(),
        ],
        rows,
        json!({ "sensorCount": count, "mode": body.mode }),
      )
    }
    _ => return Err(err(10000, "不支持的报告类型")),
  };

  let header = json!({
    "title": format!("{} Report", body.kind.to_uppercase()),
    "facility": "Pharmaceutical Net Pro",
    "dateRange": format!("{} ~ {}", body.date_from, body.date_to),
    "filtersSummary": format!("mode={}, interval={:?}", body.mode, body.interval)
  });
  let body_json = json!([{
    "title": "Report Body",
    "columns": columns,
    "rows": rows
  }]);
  let footer = json!({
    "pageNote": "Generated by Pharmaceutical Net Pro",
    "signedInfo": format!("Generated by {}", by)
  });
  let query_json = serde_json::to_value(&body).unwrap_or(json!({}));

  let result = sqlx::query(
    "INSERT INTO ph_report_document
     (kind, mode, query_json, header_json, body_json, summary_json, footer_json, generated_by)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(&body.kind)
  .bind(&body.mode)
  .bind(&query_json)
  .bind(&header)
  .bind(&body_json)
  .bind(&summary)
  .bind(&footer)
  .bind(&by)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let id = result.last_insert_rowid();
  Ok(ok(json!({
    "id": id.to_string(),
    "query": query_json,
    "header": header,
    "body": body_json,
    "summary": summary,
    "footer": footer,
    "generatedAt": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    "generatedBy": by
  })))
}

async fn list_reports(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, String, chrono::NaiveDateTime, String)> = sqlx::query_as(
    "SELECT id, kind, mode, generated_at, generated_by FROM ph_report_document ORDER BY id DESC LIMIT 100",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, kind, mode, generated_at, generated_by)| {
        json!({
          "id": id.to_string(),
          "kind": kind,
          "mode": mode,
          "generatedAt": generated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
          "generatedBy": generated_by
        })
      })
      .collect(),
  ))
}

async fn get_report(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i64, Value, Value, Value, Option<Value>, Value, chrono::NaiveDateTime, String)> =
    sqlx::query_as(
      "SELECT id, query_json, header_json, body_json, summary_json, footer_json, generated_at, generated_by
       FROM ph_report_document WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  let (id, query, header, body, summary, footer, generated_at, generated_by) =
    row.ok_or(err(4040, "报告不存在"))?;

  Ok(ok(json!({
    "id": id.to_string(),
    "query": query,
    "header": header,
    "body": body,
    "summary": summary,
    "footer": footer,
    "generatedAt": generated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    "generatedBy": generated_by
  })))
}

async fn export_report(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(Value, Value)> =
    sqlx::query_as("SELECT header_json, body_json FROM ph_report_document WHERE id = ?")
      .bind(id)
      .fetch_optional(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  let (header, body) = row.ok_or(err(4040, "报告不存在"))?;

  let format = q.format.unwrap_or_else(|| "csv".to_string());
  if format == "csv" {
    let mut csv = String::new();
    if let Some(sections) = body.as_array() {
      for section in sections {
        if let Some(cols) = section.get("columns").and_then(|v| v.as_array()) {
          let headers: Vec<String> = cols.iter().map(|c| c.as_str().unwrap_or("").to_string()).collect();
          csv.push_str(&headers.join(","));
          csv.push('\n');
          if let Some(rows) = section.get("rows").and_then(|v| v.as_array()) {
            for row in rows {
              let line: Vec<String> = headers
                .iter()
                .map(|h| {
                  row
                    .get(h)
                    .map(|v| match v {
                      Value::Null => "".to_string(),
                      Value::String(s) => s.clone(),
                      other => other.to_string(),
                    })
                    .unwrap_or_default()
                })
                .collect();
              csv.push_str(&line.join(","));
              csv.push('\n');
            }
          }
        }
      }
    }
    Ok(ok(json!({ "format": "csv", "content": csv, "header": header })))
  } else {
    Ok(ok(json!({
      "format": "pdf",
      "message": "PDF 导出占位：请使用前端打印或后续接入 PDF 引擎",
      "header": header,
      "body": body
    })))
  }
}

async fn list_tag_catalog(
  State(state): State<AppState>,
  Query(q): Query<TagCatalogQuery>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, String, Option<String>, Option<Value>, i8)> = if let Some(tab) =
    q.tab.filter(|t| !t.is_empty())
  {
    sqlx::query_as(
      "SELECT id, tab, name, color, linked_ids, enabled FROM ph_tag_catalog WHERE tab = ? ORDER BY id",
    )
    .bind(tab)
    .fetch_all(&state.pool)
    .await
  } else {
    sqlx::query_as("SELECT id, tab, name, color, linked_ids, enabled FROM ph_tag_catalog ORDER BY tab, id")
      .fetch_all(&state.pool)
      .await
  }
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, tab, name, color, linked_ids, enabled)| {
        json!({
          "id": id.to_string(),
          "tab": tab,
          "name": name,
          "color": color,
          "linkedIds": linked_ids.unwrap_or(json!([])),
          "enabled": enabled == 1
        })
      })
      .collect(),
  ))
}

async fn create_tag_catalog(
  State(state): State<AppState>,
  Json(body): Json<TagCatalogUpsert>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let result = sqlx::query(
    "INSERT INTO ph_tag_catalog (tab, name, color, linked_ids, enabled) VALUES (?, ?, ?, ?, ?)",
  )
  .bind(body.tab)
  .bind(body.name)
  .bind(body.color)
  .bind(body.linked_ids.unwrap_or(json!([])))
  .bind(if body.enabled.unwrap_or(true) { 1 } else { 0 })
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_tag_catalog(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<TagCatalogUpsert>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "UPDATE ph_tag_catalog SET tab = ?, name = ?, color = ?, linked_ids = ?, enabled = ? WHERE id = ?",
  )
  .bind(body.tab)
  .bind(body.name)
  .bind(body.color)
  .bind(body.linked_ids.unwrap_or(json!([])))
  .bind(if body.enabled.unwrap_or(true) { 1 } else { 0 })
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn delete_tag_catalog(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("DELETE FROM ph_tag_catalog WHERE id = ?")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn list_sda(
  State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, Option<String>, Value, Value, Value, i8)> = sqlx::query_as(
    "SELECT id, config_name, csv_files, display_config, virtual_pens, markers_enabled
     FROM ph_sda_session ORDER BY id DESC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, config_name, csv_files, display_config, virtual_pens, markers)| {
        json!({
          "id": id.to_string(),
          "configName": config_name,
          "csvFiles": csv_files,
          "displayConfig": display_config,
          "virtualPens": virtual_pens,
          "markersEnabled": markers == 1
        })
      })
      .collect(),
  ))
}

async fn get_sda(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let row: Option<(i64, Option<String>, Value, Option<String>, Value, Value, i8)> = sqlx::query_as(
    "SELECT id, config_name, csv_files, csv_content, display_config, virtual_pens, markers_enabled
     FROM ph_sda_session WHERE id = ?",
  )
  .bind(id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let (id, config_name, csv_files, csv_content, display_config, virtual_pens, markers) =
    row.ok_or(err(4040, "SDA 会话不存在"))?;

  Ok(ok(json!({
    "id": id.to_string(),
    "configName": config_name,
    "csvFiles": csv_files,
    "csvContent": csv_content,
    "displayConfig": display_config,
    "virtualPens": virtual_pens,
    "markersEnabled": markers == 1
  })))
}

async fn create_sda(
  State(state): State<AppState>,
  Json(body): Json<SdaUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let result = sqlx::query(
    "INSERT INTO ph_sda_session (config_name, csv_files, csv_content, display_config, virtual_pens, markers_enabled)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind(body.config_name)
  .bind(body.csv_files.unwrap_or(json!([])))
  .bind(body.csv_content)
  .bind(body.display_config.unwrap_or(json!({})))
  .bind(body.virtual_pens.unwrap_or(json!([])))
  .bind(if body.markers_enabled.unwrap_or(false) { 1 } else { 0 })
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(json!({ "id": result.last_insert_rowid().to_string() })))
}

async fn update_sda(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<SdaUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "UPDATE ph_sda_session SET
       config_name = COALESCE(?, config_name),
       csv_files = COALESCE(?, csv_files),
       csv_content = COALESCE(?, csv_content),
       display_config = COALESCE(?, display_config),
       virtual_pens = COALESCE(?, virtual_pens),
       markers_enabled = COALESCE(?, markers_enabled),
       updated_at = datetime('now')
     WHERE id = ?",
  )
  .bind(body.config_name)
  .bind(body.csv_files)
  .bind(body.csv_content)
  .bind(body.display_config)
  .bind(body.virtual_pens)
  .bind(body.markers_enabled.map(|v| if v { 1 } else { 0 }))
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn analyze_sda(
  State(state): State<AppState>,
  Json(body): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let mut content = body
    .get("csvContent")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();

  if content.is_empty() {
    if let Some(session_id) = body
      .get("sessionId")
      .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    {
      let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT csv_content FROM ph_sda_session WHERE id = ?")
          .bind(session_id)
          .fetch_optional(&state.pool)
          .await
          .map_err(|_| err(5000, "服务器错误"))?;
      content = row
        .and_then(|(c,)| c)
        .ok_or(err(4040, "SDA 会话不存在或无 CSV"))?;
    }
  }

  if content.trim().is_empty() {
    return Err(err(10000, "缺少 CSV 内容"));
  }

  let mut lines = content.lines().filter(|l| !l.trim().is_empty());
  let header = lines
    .next()
    .unwrap_or("timestamp,value")
    .split(',')
    .map(|s| s.trim().to_string())
    .collect::<Vec<_>>();

  let mut series: Vec<Value> = Vec::new();
  for line in lines {
    let cols: Vec<&str> = line.split(',').collect();
    let mut point = serde_json::Map::new();
    for (i, key) in header.iter().enumerate() {
      let raw = cols.get(i).copied().unwrap_or("");
      if let Ok(num) = raw.parse::<f64>() {
        point.insert(key.clone(), json!(num));
      } else {
        point.insert(key.clone(), json!(raw));
      }
    }
    series.push(Value::Object(point));
  }

  Ok(ok(json!({
    "columns": header,
    "points": series,
    "stats": {
      "pointCount": series.len()
    }
  })))
}