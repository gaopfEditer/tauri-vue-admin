-- Field-test SQLite bootstrap（现场临时库）
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sys_role (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role_code TEXT NOT NULL UNIQUE,
  role_name TEXT NOT NULL,
  description TEXT,
  home_route TEXT NOT NULL DEFAULT 'dashboard_analysis',
  status INTEGER NOT NULL DEFAULT 1,
  sort INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sys_user (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_name TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  nick_name TEXT,
  age INTEGER,
  gender TEXT,
  phone TEXT,
  email TEXT,
  user_status TEXT NOT NULL DEFAULT '1',
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sys_user_role (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  role_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(user_id, role_id)
);

CREATE TABLE IF NOT EXISTS sys_menu (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  parent_id INTEGER NOT NULL DEFAULT 0,
  route_name TEXT NOT NULL UNIQUE,
  path TEXT NOT NULL,
  component TEXT,
  redirect TEXT,
  title TEXT NOT NULL,
  icon TEXT,
  local_icon TEXT,
  order_num INTEGER NOT NULL DEFAULT 0,
  hide INTEGER NOT NULL DEFAULT 0,
  requires_auth INTEGER NOT NULL DEFAULT 1,
  href TEXT,
  single_layout TEXT,
  permissions TEXT,
  menu_type INTEGER NOT NULL DEFAULT 1,
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sys_permission (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  permission_code TEXT NOT NULL UNIQUE,
  permission_name TEXT NOT NULL,
  menu_id INTEGER,
  api_path TEXT,
  http_method TEXT,
  description TEXT,
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sys_role_menu (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role_id INTEGER NOT NULL,
  menu_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(role_id, menu_id)
);

CREATE TABLE IF NOT EXISTS sys_role_permission (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role_id INTEGER NOT NULL,
  permission_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS sys_user_token (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  token TEXT NOT NULL UNIQUE,
  refresh_token TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sensor_group (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  tags TEXT,
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sensor (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category TEXT NOT NULL,
  custom_id TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  group_id INTEGER,
  channel TEXT,
  acquire_mode TEXT,
  runtime_state TEXT NOT NULL DEFAULT 'Idle',
  power_on INTEGER NOT NULL DEFAULT 0,
  com_alarm INTEGER NOT NULL DEFAULT 0,
  flow_calc_alarm INTEGER NOT NULL DEFAULT 0,
  last_value TEXT,
  unit TEXT,
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sensor_device_config (
  sensor_id INTEGER PRIMARY KEY,
  facility_node_id INTEGER,
  device_name TEXT NOT NULL DEFAULT '',
  device_code TEXT NOT NULL DEFAULT '',
  instrument_type TEXT,
  update_interval_sec INTEGER NOT NULL DEFAULT 60,
  slave_address TEXT,
  plc_ip TEXT,
  data_unit TEXT,
  data_type TEXT NOT NULL DEFAULT 'ulong',
  data_length INTEGER NOT NULL DEFAULT 4,
  protocol TEXT NOT NULL DEFAULT 'ModbusTCP',
  decimal_places INTEGER NOT NULL DEFAULT 0,
  cleanroom_class TEXT,
  serial_number TEXT,
  calibration_date TEXT,
  operating_mode TEXT NOT NULL DEFAULT 'Operational',
  production_state TEXT NOT NULL DEFAULT 'production',
  flow_rate REAL,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sensor_state_threshold (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sensor_id INTEGER NOT NULL,
  state_group TEXT NOT NULL,
  metric_key TEXT NOT NULL DEFAULT '0.5um',
  alarm_enable TEXT NOT NULL DEFAULT 'unlimited',
  warn_high REAL,
  warn_low REAL,
  alarm_high REAL,
  alarm_low REAL,
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(sensor_id, state_group, metric_key)
);

CREATE TABLE IF NOT EXISTS ph_sensor_limit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sensor_id INTEGER NOT NULL,
  data_type TEXT NOT NULL,
  warning_limit REAL,
  alarm_limit REAL,
  effective_from TEXT NOT NULL DEFAULT (datetime('now')),
  changed_by TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(sensor_id, data_type)
);

CREATE TABLE IF NOT EXISTS ph_sensor_limit_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  limit_id INTEGER NOT NULL,
  before_json TEXT,
  after_json TEXT,
  signed_by TEXT,
  signed_at TEXT DEFAULT (datetime('now')),
  reason TEXT
);

CREATE TABLE IF NOT EXISTS ph_facility_node (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  parent_id INTEGER,
  level INTEGER NOT NULL,
  code TEXT,
  name TEXT NOT NULL,
  description TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  status INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_recipe (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  sensor_group_ids TEXT,
  pens TEXT,
  created_by TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sampling_task (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipe_id INTEGER NOT NULL,
  recipe_name TEXT,
  scheduled_at TEXT,
  sampling_mode TEXT,
  custom_fields TEXT,
  input_notes TEXT,
  status TEXT NOT NULL DEFAULT 'scheduled',
  started_at TEXT,
  finished_at TEXT,
  abort_reason TEXT,
  created_by TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sampling_custom_field (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  field_key TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  editable INTEGER NOT NULL DEFAULT 1,
  default_entries TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS ph_alarm_event (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sensor_id INTEGER,
  sensor_name TEXT,
  severity TEXT,
  message TEXT,
  raised_at TEXT NOT NULL DEFAULT (datetime('now')),
  acknowledged INTEGER NOT NULL DEFAULT 0,
  acknowledged_by TEXT,
  acknowledged_at TEXT,
  sampling_id INTEGER,
  data_type TEXT,
  value REAL,
  limit_value REAL
);

CREATE TABLE IF NOT EXISTS ph_rt_trend_session (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipe_id INTEGER,
  recipe_name TEXT,
  running INTEGER NOT NULL DEFAULT 0,
  pens TEXT,
  started_at TEXT,
  stopped_at TEXT,
  started_by TEXT
);

CREATE TABLE IF NOT EXISTS ph_runtime_tag (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  folder TEXT,
  name TEXT,
  data_type TEXT,
  value_json TEXT,
  path TEXT,
  status INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS ph_runtime_rule (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  rule_type TEXT,
  parent_id INTEGER,
  conditions TEXT,
  actions TEXT,
  enabled INTEGER NOT NULL DEFAULT 1,
  recipe_id INTEGER,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_report_document (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT,
  mode TEXT,
  query_json TEXT,
  header_json TEXT,
  body_json TEXT,
  summary_json TEXT,
  footer_json TEXT,
  generated_at TEXT NOT NULL DEFAULT (datetime('now')),
  generated_by TEXT
);

CREATE TABLE IF NOT EXISTS ph_tag_catalog (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tab TEXT,
  name TEXT,
  color TEXT,
  linked_ids TEXT,
  enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS ph_sda_session (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  config_name TEXT,
  csv_files TEXT,
  csv_content TEXT,
  display_config TEXT,
  virtual_pens TEXT,
  markers_enabled INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_audit_trail (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_time TEXT NOT NULL DEFAULT (datetime('now')),
  action TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id TEXT,
  performed_by TEXT NOT NULL,
  role_code TEXT NOT NULL DEFAULT 'user',
  reason TEXT,
  detail_json TEXT
);

CREATE TABLE IF NOT EXISTS ph_password_policy (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  expire_days INTEGER NOT NULL DEFAULT 90,
  min_length INTEGER NOT NULL DEFAULT 6,
  remember_old_count INTEGER NOT NULL DEFAULT 3,
  auto_logoff_seconds INTEGER NOT NULL DEFAULT 1800,
  electronic_signature_enabled INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_user_profile (
  user_id INTEGER PRIMARY KEY,
  user_id_code TEXT NOT NULL DEFAULT '',
  pharma_role TEXT NOT NULL DEFAULT 'User',
  expiry_date TEXT,
  must_change_password INTEGER NOT NULL DEFAULT 0,
  is_local_emergency INTEGER NOT NULL DEFAULT 0,
  password_changed_at TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_password_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  password TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_backup_config (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  enabled INTEGER NOT NULL DEFAULT 0,
  daily_at TEXT NOT NULL DEFAULT '02:00',
  target_path TEXT NOT NULL DEFAULT './backups',
  retain_days INTEGER NOT NULL DEFAULT 30,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_backup_job (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  job_type TEXT,
  status TEXT,
  file_path TEXT,
  started_at TEXT,
  finished_at TEXT,
  message TEXT,
  created_by TEXT
);

CREATE TABLE IF NOT EXISTS ph_system_setting (
  setting_key TEXT PRIMARY KEY,
  setting_value TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_setup_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  mode TEXT,
  step INTEGER NOT NULL DEFAULT 0,
  initialized INTEGER NOT NULL DEFAULT 0,
  db_host TEXT,
  db_port INTEGER,
  db_name TEXT,
  db_user TEXT,
  restore_file TEXT,
  completed INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_sampler_calibration (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sampler_id TEXT NOT NULL UNIQUE,
  parameters TEXT,
  updated_by TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ph_license (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  license_id TEXT NOT NULL UNIQUE,
  customer_name TEXT NOT NULL DEFAULT '',
  valid_from TEXT NOT NULL,
  valid_until TEXT NOT NULL,
  device_lock INTEGER NOT NULL DEFAULT 1,
  bound_mac TEXT,
  license_code TEXT NOT NULL,
  max_observed_at TEXT,
  activated_at TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
