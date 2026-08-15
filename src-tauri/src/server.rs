use axum::{
  extract::{Path, State},
  http::{HeaderMap, StatusCode},
  middleware,
  routing::{get, post, put},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use tower_http::cors::{Any, CorsLayer};

use crate::audit;
use crate::db::{self, DbPool};
use crate::license;

const REFRESH_TOKEN_CODE: i64 = 66666;

#[derive(Clone)]
pub struct AppState {
  pub pool: DbPool,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
  pub code: i64,
  pub message: &'static str,
  pub data: T,
}

#[derive(Serialize)]
pub struct ApiError {
  pub code: i64,
  pub message: &'static str,
  pub data: Value,
}

#[derive(Deserialize)]
struct LoginBody {
  #[serde(rename = "userName")]
  user_name: Option<String>,
  password: Option<String>,
}

#[derive(Deserialize)]
struct UserRoutesBody {
  #[serde(rename = "userId")]
  user_id: Option<String>,
}

#[derive(Deserialize)]
struct RefreshBody {
  #[serde(rename = "refreshToken")]
  refresh_token: Option<String>,
}

#[derive(Serialize)]
struct TokenData {
  token: String,
  #[serde(rename = "refreshToken")]
  refresh_token: String,
}

#[derive(Serialize)]
struct UserInfoData {
  #[serde(rename = "userId")]
  user_id: String,
  #[serde(rename = "userName")]
  user_name: String,
  #[serde(rename = "userRole")]
  user_role: String,
}

#[derive(Serialize)]
struct UserRoutesData {
  routes: Vec<Value>,
  home: String,
}

#[derive(Serialize, FromRow)]
struct UserRow {
  id: i64,
  user_name: String,
  nick_name: Option<String>,
  age: Option<i64>,
  gender: Option<String>,
  phone: Option<String>,
  email: Option<String>,
  user_status: String,
}

#[derive(FromRow)]
struct MenuRow {
  id: i64,
  parent_id: i64,
  route_name: String,
  path: String,
  component: Option<String>,
  redirect: Option<String>,
  title: String,
  icon: Option<String>,
  local_icon: Option<String>,
  order_num: i64,
  hide: i64,
  requires_auth: i64,
  href: Option<String>,
  single_layout: Option<String>,
  permissions: Option<String>,
}

pub async fn start(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let state = AppState { pool: pool.clone() };

  crate::syslog::info("system", "api", "启动桌面 API :8080");

  // 后台 Modbus TCP 轮询（现场联调日志写入 logs/modbus-poll.log）
  crate::modbus_poll::spawn_poller(pool.clone());

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/mock/getSmsCode", post(get_sms_code))
    .route("/mock/login", post(login))
    .route("/mock/getUserInfo", get(get_user_info))
    .route("/mock/getUserRoutes", post(get_user_routes))
    .route("/mock/updateToken", post(update_token))
    .route("/mock/getAllUserList", post(get_all_user_list))
    .route("/api/management/user/list", get(list_users))
    .route("/api/management/user", post(create_user))
    .route("/api/management/user/:id", put(update_user).delete(delete_user))
    .route("/api/management/role/list", get(list_roles))
    .route("/api/management/menu/tree", get(list_menu_tree))
    .route("/api/management/permission/list", get(list_permissions))
    .merge(crate::pharma::routes())
    .merge(crate::ops::routes())
    .merge(crate::sysops::routes())
    .merge(crate::license::routes())
    .merge(crate::modbus_poll::routes())
    .merge(crate::syslog::routes())
    .layer(middleware::from_fn_with_state(
      state.clone(),
      audit::http_audit_middleware,
    ))
    .layer(middleware::from_fn_with_state(
      state.clone(),
      license::license_guard_middleware,
    ))
    .layer(cors)
    .with_state(state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
  println!("desktop api server listening on http://127.0.0.1:8080");
  axum::serve(listener, app).await?;
  Ok(())
}

pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
  Json(ApiResponse {
    code: 200,
    message: "ok",
    data,
  })
}

pub fn err(code: i64, message: &'static str) -> (StatusCode, Json<ApiError>) {
  (
    StatusCode::OK,
    Json(ApiError {
      code,
      message,
      data: Value::Null,
    }),
  )
}

fn to_api_user_id(db_id: i64) -> String {
  db_id.to_string()
}

fn from_api_user_id(user_id: &str) -> Option<i64> {
  user_id.parse::<i64>().ok()
}

async fn get_sms_code() -> Json<ApiResponse<bool>> {
  ok(true)
}

async fn login(
  State(state): State<AppState>,
  Json(body): Json<LoginBody>,
) -> Result<Json<ApiResponse<TokenData>>, (StatusCode, Json<ApiError>)> {
  let user_name = body.user_name.unwrap_or_default();
  let password = body.password.unwrap_or_default();
  if user_name.is_empty() || password.is_empty() {
    return Err(err(10000, "参数校验失败！"));
  }

  let password_hash = db::md5_hex(&password);
  let user: Option<UserRow> = sqlx::query_as(
    "SELECT id, user_name, nick_name, age, gender, phone, email, user_status
     FROM sys_user WHERE user_name = ? AND password = ? AND status = 1",
  )
  .bind(&user_name)
  .bind(&password_hash)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let user = match user {
    Some(u) => u,
    None => {
      audit::write_audit(
        &state.pool,
        "auth.login.failed",
        "auth",
        None,
        &user_name,
        "nobody",
        Some("用户名或密码错误"),
        Some(json!({ "userName": user_name })),
      )
      .await;
      return Err(err(1000, "用户名或密码错误！"));
    }
  };

  // 授权无效则禁止登录
  license::ensure_license_or_err(&state.pool).await?;

  let token = format!("__TOKEN_{}__", uuid::Uuid::new_v4());
  let refresh_token = format!("__REFRESH_TOKEN_{}__", uuid::Uuid::new_v4());

  sqlx::query("DELETE FROM sys_user_token WHERE user_id = ?")
    .bind(user.id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  sqlx::query("INSERT INTO sys_user_token (user_id, token, refresh_token) VALUES (?, ?, ?)")
    .bind(user.id)
    .bind(&token)
    .bind(&refresh_token)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  let role_code: String = sqlx::query_scalar(
    "SELECT COALESCE(r.role_code, 'user')
     FROM sys_user_role ur
     LEFT JOIN sys_role r ON r.id = ur.role_id
     WHERE ur.user_id = ?
     LIMIT 1",
  )
  .bind(user.id)
  .fetch_optional(&state.pool)
  .await
  .ok()
  .flatten()
  .unwrap_or_else(|| "user".to_string());

  audit::write_audit(
    &state.pool,
    "auth.login",
    "auth",
    Some(&user.id.to_string()),
    &user.user_name,
    &role_code,
    None,
    Some(json!({ "userId": user.id, "userName": user.user_name })),
  )
  .await;

  Ok(ok(TokenData { token, refresh_token }))
}

async fn get_user_info(
  State(state): State<AppState>,
  headers: HeaderMap,
) -> Result<Json<ApiResponse<UserInfoData>>, (StatusCode, Json<ApiError>)> {
  let token = extract_token(&headers);
  if token.is_empty() {
    return Err(err(REFRESH_TOKEN_CODE, "用户已失效或不存在！"));
  }

  let row: Option<(i64, String, String)> = sqlx::query_as(
    "SELECT u.id, u.user_name, r.role_code
     FROM sys_user_token t
     INNER JOIN sys_user u ON u.id = t.user_id
     INNER JOIN sys_user_role ur ON ur.user_id = u.id
     INNER JOIN sys_role r ON r.id = ur.role_id
     WHERE t.token = ? AND u.status = 1
     LIMIT 1",
  )
  .bind(&token)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let (id, user_name, role_code) = row.ok_or(err(REFRESH_TOKEN_CODE, "用户信息异常！"))?;

  Ok(ok(UserInfoData {
    user_id: to_api_user_id(id),
    user_name,
    user_role: role_code,
  }))
}

async fn update_token(
  State(state): State<AppState>,
  Json(body): Json<RefreshBody>,
) -> Result<Json<ApiResponse<TokenData>>, (StatusCode, Json<ApiError>)> {
  let refresh_token = body.refresh_token.unwrap_or_default();
  let row: Option<(i64, String)> = sqlx::query_as(
    "SELECT user_id, token FROM sys_user_token WHERE refresh_token = ?",
  )
  .bind(&refresh_token)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let (user_id, old_token) = row.ok_or(err(3000, "用户已失效或不存在！"))?;
  let token = format!("__TOKEN_{}__", uuid::Uuid::new_v4());
  let new_refresh = format!("__REFRESH_TOKEN_{}__", uuid::Uuid::new_v4());

  sqlx::query("UPDATE sys_user_token SET token = ?, refresh_token = ? WHERE user_id = ?")
    .bind(&token)
    .bind(&new_refresh)
    .bind(user_id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  let _ = old_token;
  Ok(ok(TokenData {
    token,
    refresh_token: new_refresh,
  }))
}

async fn get_user_routes(
  State(state): State<AppState>,
  Json(body): Json<UserRoutesBody>,
) -> Result<Json<ApiResponse<UserRoutesData>>, (StatusCode, Json<ApiError>)> {
  let user_id = body.user_id.unwrap_or_default();
  let db_user_id = from_api_user_id(&user_id).ok_or(err(10000, "参数校验失败！"))?;

  let home: Option<String> = sqlx::query_scalar(
    "SELECT r.home_route FROM sys_user_role ur
     INNER JOIN sys_role r ON r.id = ur.role_id
     WHERE ur.user_id = ? LIMIT 1",
  )
  .bind(db_user_id)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let menus: Vec<MenuRow> = sqlx::query_as(
    "SELECT m.id, m.parent_id, m.route_name, m.path, m.component, m.redirect, m.title,
            m.icon, m.local_icon, m.order_num, m.hide, m.requires_auth, m.href,
            m.single_layout, CAST(m.permissions AS CHAR) AS permissions
     FROM sys_menu m
     INNER JOIN sys_role_menu rm ON rm.menu_id = m.id
     INNER JOIN sys_user_role ur ON ur.role_id = rm.role_id
     WHERE ur.user_id = ? AND m.status = 1
     ORDER BY m.order_num ASC, m.id ASC",
  )
  .bind(db_user_id)
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let routes = build_route_tree(&menus, 0);
  Ok(ok(UserRoutesData {
    routes,
    home: home.unwrap_or_else(|| "dashboard_analysis".to_string()),
  }))
}

async fn get_all_user_list(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let users: Vec<UserRow> = sqlx::query_as(
    "SELECT id, user_name, nick_name, age, gender, phone, email, user_status
     FROM sys_user WHERE status = 1 ORDER BY id ASC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let list = users
    .into_iter()
    .map(|u| {
      json!({
        "id": u.id.to_string(),
        "userName": u.nick_name.clone().or(Some(u.user_name.clone())),
        "age": u.age,
        "gender": u.gender,
        "phone": u.phone.unwrap_or_default(),
        "email": u.email,
        "userStatus": u.user_status
      })
    })
    .collect();

  Ok(ok(list))
}

async fn list_users(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  get_all_user_list(State(state)).await
}

#[derive(Deserialize)]
struct UserUpsertBody {
  #[serde(rename = "userName")]
  user_name: Option<String>,
  age: Option<i64>,
  gender: Option<String>,
  phone: Option<String>,
  email: Option<String>,
  #[serde(rename = "userStatus")]
  user_status: Option<String>,
  password: Option<String>,
  #[serde(rename = "roleId")]
  role_id: Option<i64>,
}

async fn create_user(
  State(state): State<AppState>,
  Json(body): Json<UserUpsertBody>,
) -> Result<Json<ApiResponse<Value>>, (StatusCode, Json<ApiError>)> {
  let user_name = body.user_name.unwrap_or_default();
  if user_name.is_empty() {
    return Err(err(10000, "参数校验失败！"));
  }
  let password_raw = body
    .password
    .filter(|p| !p.is_empty())
    .ok_or(err(10000, "请设置登录密码！"))?;
  let password = db::md5_hex(&password_raw);
  let result = sqlx::query(
    "INSERT INTO sys_user (user_name, password, nick_name, age, gender, phone, email, user_status)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(&user_name)
  .bind(&password)
  .bind(&user_name)
  .bind(body.age)
  .bind(body.gender)
  .bind(body.phone)
  .bind(body.email)
  .bind(body.user_status.unwrap_or_else(|| "1".to_string()))
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  let user_id = result.last_insert_rowid() as i64;
  let role_id = body.role_id.unwrap_or(3);
  sqlx::query("INSERT INTO sys_user_role (user_id, role_id) VALUES (?, ?)")
    .bind(user_id)
    .bind(role_id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(json!({ "id": user_id })))
}

async fn update_user(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<UserUpsertBody>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query(
    "UPDATE sys_user SET nick_name = COALESCE(?, nick_name), age = COALESCE(?, age),
     gender = COALESCE(?, gender), phone = COALESCE(?, phone), email = COALESCE(?, email),
     user_status = COALESCE(?, user_status), updated_at = datetime('now') WHERE id = ?",
  )
  .bind(body.user_name)
  .bind(body.age)
  .bind(body.gender)
  .bind(body.phone)
  .bind(body.email)
  .bind(body.user_status)
  .bind(id)
  .execute(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  if let Some(password) = body.password.filter(|p| !p.is_empty()) {
    let password_hash = db::md5_hex(&password);
    sqlx::query("UPDATE sys_user SET password = ?, updated_at = datetime('now') WHERE id = ?")
      .bind(password_hash)
      .bind(id)
      .execute(&state.pool)
      .await
      .map_err(|_| err(5000, "服务器错误"))?;
  }

  Ok(ok(true))
}

async fn delete_user(
  State(state): State<AppState>,
  Path(id): Path<i64>,
) -> Result<Json<ApiResponse<bool>>, (StatusCode, Json<ApiError>)> {
  sqlx::query("UPDATE sys_user SET user_status = '4', status = 0, updated_at = datetime('now') WHERE id = ?")
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| err(5000, "服务器错误"))?;
  Ok(ok(true))
}

async fn list_roles(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, String, String, Option<String>)> = sqlx::query_as(
    "SELECT id, role_code, role_name, home_route, description FROM sys_role WHERE status = 1 ORDER BY sort ASC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, role_code, role_name, home_route, description)| {
        json!({
          "id": id,
          "roleCode": role_code,
          "roleName": role_name,
          "homeRoute": home_route,
          "description": description
        })
      })
      .collect(),
  ))
}

async fn list_menu_tree(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let menus: Vec<MenuRow> = sqlx::query_as(
    "SELECT id, parent_id, route_name, path, component, redirect, title, icon, local_icon,
            order_num, hide, requires_auth, href, single_layout,
            CAST(permissions AS CHAR) AS permissions
     FROM sys_menu WHERE status = 1 ORDER BY order_num ASC, id ASC",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(build_route_tree(&menus, 0)))
}

async fn list_permissions(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Value>>>, (StatusCode, Json<ApiError>)> {
  let rows: Vec<(i64, String, String, Option<i64>, Option<String>, Option<String>)> = sqlx::query_as(
    "SELECT id, permission_code, permission_name, menu_id, api_path, http_method FROM sys_permission WHERE status = 1",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| err(5000, "服务器错误"))?;

  Ok(ok(
    rows.into_iter()
      .map(|(id, code, name, menu_id, api_path, method)| {
        json!({
          "id": id,
          "permissionCode": code,
          "permissionName": name,
          "menuId": menu_id,
          "apiPath": api_path,
          "httpMethod": method
        })
      })
      .collect(),
  ))
}

fn extract_token(headers: &HeaderMap) -> String {
  headers
    .get("authorization")
    .or_else(|| headers.get("Authorization"))
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_string()
}

fn build_route_tree(menus: &[MenuRow], parent_id: i64) -> Vec<Value> {
  menus
    .iter()
    .filter(|m| m.parent_id == parent_id)
    .map(|menu| menu_to_route(menu, menus))
    .collect()
}

fn menu_to_route(menu: &MenuRow, all: &[MenuRow]) -> Value {
  let children: Vec<Value> = build_route_tree(all, menu.id);
  let mut meta = json!({
    "title": menu.title,
    "requiresAuth": menu.requires_auth == 1
  });
  if let Some(icon) = &menu.icon {
    meta["icon"] = json!(icon);
  }
  if let Some(local_icon) = &menu.local_icon {
    meta["localIcon"] = json!(local_icon);
  }
  if menu.hide == 1 {
    meta["hide"] = json!(true);
  }
  if menu.order_num > 0 {
    meta["order"] = json!(menu.order_num);
  }
  if let Some(href) = &menu.href {
    meta["href"] = json!(href);
  }
  if let Some(single_layout) = &menu.single_layout {
    meta["singleLayout"] = json!(single_layout);
  }
  if let Some(permissions) = &menu.permissions {
    if let Ok(list) = serde_json::from_str::<Vec<String>>(permissions) {
      meta["permissions"] = json!(list);
    }
  }

  let mut route = json!({
    "name": menu.route_name,
    "path": menu.path,
    "meta": meta
  });
  if let Some(component) = &menu.component {
    route["component"] = json!(component);
  }
  if let Some(redirect) = &menu.redirect {
    route["redirect"] = json!(redirect);
  }
  if !children.is_empty() {
    route["children"] = json!(children);
  }
  route
}
