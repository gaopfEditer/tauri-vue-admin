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
  UNIQUE(user_id, role_id),
  FOREIGN KEY(user_id) REFERENCES sys_user(id) ON DELETE CASCADE,
  FOREIGN KEY(role_id) REFERENCES sys_role(id) ON DELETE CASCADE
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
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY(menu_id) REFERENCES sys_menu(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS sys_role_menu (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role_id INTEGER NOT NULL,
  menu_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(role_id, menu_id),
  FOREIGN KEY(role_id) REFERENCES sys_role(id) ON DELETE CASCADE,
  FOREIGN KEY(menu_id) REFERENCES sys_menu(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sys_role_permission (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role_id INTEGER NOT NULL,
  permission_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(role_id, permission_id),
  FOREIGN KEY(role_id) REFERENCES sys_role(id) ON DELETE CASCADE,
  FOREIGN KEY(permission_id) REFERENCES sys_permission(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sys_user_token (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  token TEXT NOT NULL UNIQUE,
  refresh_token TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY(user_id) REFERENCES sys_user(id) ON DELETE CASCADE
);
