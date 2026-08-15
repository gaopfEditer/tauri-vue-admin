# Pharmaceutical Net Pro — 业务功能开发大纲

> 依据：`md/PHARMACEUTICALNET PRO MANUAL(1)(1)/PHARMACEUTICALNET PRO MANUAL(1)(1).md`  
> 产品：Pharmaceutical Net Pro（洁净区粒子/生物采样监控系统）  
> 目标：按业务边界拆为 **6 个可独立交付的开发模块**，供桌面端（Tauri + Vue）落地。

---

## 模块总览与手册映射

| # | 开发模块 | 对应手册章节 | 优先级建议 |
|---|----------|--------------|------------|
| 1 | 配方与采样管理 | Ch.4 Recipes / Ch.5 Sampling | P0 |
| 2 | 设备与传感器管理 | Ch.6 Sensors / Ch.12 Sensors Editor & Limits | P0 |
| 3 | 报警、实时趋势与运行逻辑 | Ch.7 Alarms / Ch.9 Real Time Trend / Ch.10 Run Time Logic | P1 |
| 4 | 数据报表与统计分析 | Ch.8 Reports / Appendix A SDA | P1 |
| 5 | 用户权限、电子签名与审计 | Ch.1 Electronic Signature / Ch.11 User Management | P0 |
| 6 | 系统运维与数据库初始化 | Ch.12 System Tasks / Ch.13 Aventino Setup | P2 |

**跨模块公共能力（不单独成模块，但所有模块依赖）：**

- 登录态 / 角色权限校验（Nobody、Administrator、Supervisor、Power User、User、Emergency）
- **电子签名（Electronic Signature）**：关键配置变更、采样启停、报警确认、用户增删改均需签名并写入 Audit Trail
- 主界面 Header：Logo、Logged user、Group、登录/登出、快捷入口按钮行

---

## 模块 1：配方与采样管理

### 1.1 模块职责

管理采样配方（Recipe）生命周期，以及基于配方的采样任务调度、中止、编辑与自定义字段配置。

### 1.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 配方编辑器 | `/recipe/list` | Recipes Editor：配方列表 + 新建/编辑/删除 |
| 配方详情/编辑 | `/recipe/edit/:id` | 配方参数、关联传感器组、笔（Pens）配置 |
| 采样编辑器 | `/sampling/list` | Sampling Editor：已调度采样表 |
| 采样调度弹窗 | 组件 `SamplingScheduleModal` | 选配方、模式、时间、自定义字段、备注 |
| 自定义字段配置 | `/sampling/custom-fields` | Admin：Display Name / Enabled / Editable / DefaultEntries |

### 1.3 数据模型（Types）

```ts
/** 采样模式 */
type SamplingMode = 'Operational' | 'AtRest' | 'NormalOperation' | 'Custom';

/** 传感器采集模式（配方内点位） */
type SensorAcquireMode = 'Volume' | 'Time' | 'Continuous';

interface Recipe {
  id: string;
  name: string;
  description?: string;
  sensorGroupIds: string[];
  pens: RecipePen[];          // 趋势笔/通道显示配置
  createdBy: string;
  updatedAt: string;
  status: 'active' | 'deleted';
}

interface RecipePen {
  id: string;
  sensorId: string;
  dataType: string;           // 如粒子通道、流量等
  color: string;
  enabled: boolean;
}

interface SamplingTask {
  id: string;
  recipeId: string;
  recipeName: string;
  scheduledAt: string;        // 计划开始时间
  samplingMode: SamplingMode;
  customFields: Record<string, string>;
  inputNotes?: string;
  status: 'scheduled' | 'running' | 'aborted' | 'finished' | 'failed';
  startedAt?: string;
  finishedAt?: string;
  createdBy: string;
}

interface SamplingCustomFieldDef {
  key: string;                // 内部字段名
  displayName: string;        // 如 Batch Number
  enabled: boolean;
  editable: boolean;
  defaultEntries?: string[];  // 逗号分隔下拉选项
}
```

### 1.4 核心业务逻辑

1. **配方 CRUD**：仅 Administrator 可建/改/删；保存前强制电子签名；删除二次确认。
2. **调度采样**：选配方 → 填自定义字段/备注 → 选 Sampling Mode → 电子签名 → 写入 `SamplingTask` 并进入调度队列。
3. **中止采样**：对 `running/scheduled` 任务 Abort → 电子签名 → 状态改 `aborted`。
4. **编辑已调度任务**：可改时间、日期、Sampling Mode；保存需电子签名。
5. **自定义字段**：Admin 配置后，下次打开 Sampling editor 动态渲染表单。
6. **权限**：User/Power User 可调度与查看；删除配方仅 Admin。

### 1.5 关键 API（建议）

- `GET/POST/PUT/DELETE /api/recipes`
- `GET/POST/PUT/DELETE /api/samplings`
- `POST /api/samplings/:id/abort`
- `GET/PUT /api/samplings/custom-fields`

---

## 模块 2：设备与传感器管理

### 2.1 模块职责

监控与配置三类采样点：粒子计数器（Particle）、生物采样（BioCapt）、模拟量输入（Analog）；维护点位元数据与报警/预警限值。

### 2.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 粒子传感器总览 | `/sensors/particle` | Particle counter Window / 点位状态卡片 |
| 粒子点位详情 | `/sensors/particle/:id` | Custom ID、Description、状态、流量、通道值、ON/OFF |
| 生物传感器总览 | `/sensors/biocapt` | Biocapt Window |
| 生物点位详情 | `/sensors/biocapt/:id` | Sampling/Pause、Flow/Com Alarm、ON/OFF |
| 模拟量输入 | `/sensors/analog` | Analog Inputs：Value / Alarm box |
| 传感器编辑器 | `/system/sensors-editor` | Sensors Editor：改点位描述/Custom ID |
| 限值编辑器 | `/system/limits` | Limits Editor（粒子/预警/报警） |
| 限值历史 | `/system/limits/history` | Limits Editor History |
| 模拟量限值 | `/system/limits/analog` | Analog Inputs Limits |

### 2.3 数据模型（Types）

```ts
type SensorCategory = 'particle' | 'biocapt' | 'analog';

type SensorRuntimeState =
  | 'Idle'
  | 'Sampling'
  | 'Cleaning'   // 粒子：ON 但不采集/不报警
  | 'Pause'
  | 'Offline'
  | 'Fault';

interface Sensor {
  id: string;
  category: SensorCategory;
  customId: string;
  description: string;
  groupId?: string;
  channel?: string;           // 如 EBI Channel / B0 Remote
  acquireMode?: SensorAcquireMode;
  runtimeState: SensorRuntimeState;
  powerOn: boolean;
  comAlarm: boolean;
  flowCalcAlarm?: boolean;    // biocapt / particle 真空故障
  lastValue?: number | Record<string, number>; // 通道值
  updatedAt: string;
}

interface SensorLimit {
  id: string;
  sensorId: string;
  dataType: string;
  warningLimit?: number;
  alarmLimit?: number;
  effectiveFrom: string;
  changedBy: string;
}

interface SensorLimitHistory {
  id: string;
  limitId: string;
  before: Partial<SensorLimit>;
  after: Partial<SensorLimit>;
  signedBy: string;
  signedAt: string;
  reason?: string;
}

interface SensorGroup {
  id: string;
  name: string;
  sensorIds: string[];
  tags?: string[];
}
```

### 2.4 核心业务逻辑

1. **实时状态刷新**：轮询或 WebSocket 推送 `runtimeState`、通道值、Com/Flow Alarm。
2. **手动 Switch ON/OFF**：需电子签名；粒子支持 Cleaning 模式。
3. **Sensors Editor**：Operator 只读；Installer/Admin 改 Custom ID / Description，保存需签名。
4. **Limits Editor**：按传感器类别配置 warning/alarm；变更写入 History；模拟量走独立限值页。
5. **分组**：Sensor Groups 供报表「按组」选择与主界面区域图使用。

### 2.5 关键 API（建议）

- `GET /api/sensors?category=`
- `GET /api/sensors/:id`
- `POST /api/sensors/:id/power`（on/off/cleaning）
- `PUT /api/sensors/:id/meta`
- `GET/PUT /api/sensor-limits`
- `GET /api/sensor-limits/history`
- `GET/POST/PUT /api/sensor-groups`

---

## 模块 3：报警、实时趋势与运行逻辑

### 3.1 模块职责

集中处理报警确认、实时趋势曲线，以及 DI/DO/虚拟点位的运行时规则（含灯塔 Tower Lights、Sampling on Input）。

### 3.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 报警中心 | `/alarms` | Alarms Window：列表、Ack / Ack All |
| 实时趋势 | `/rt-trend` | Real Time Trend：启停、配方笔编辑 |
| 运行逻辑编辑器 | `/runtime-logic` | Run Time Logic Editor |
| 规则编辑 | 组件 `RuleEditor` | Rule Type、父子规则 |
| Sampling on Input | 组件 `SamplingOnInputEditor` | 输入触发采样 |
| 灯塔规则 | 组件 `TowerLightRules` | Tower Lights 规则与输出测试 |

### 3.3 数据模型（Types）

```ts
type AlarmSeverity = 'warning' | 'alarm' | 'communication' | 'flow';

interface AlarmEvent {
  id: string;
  sensorId: string;
  sensorName: string;
  severity: AlarmSeverity;
  message: string;
  raisedAt: string;
  acknowledged: boolean;
  acknowledgedBy?: string;
  acknowledgedAt?: string;
  samplingId?: string;
}

interface RealTimeTrendSession {
  id: string;
  recipeId: string;
  running: boolean;
  pens: RecipePen[];
  startedAt?: string;
  stoppedAt?: string;
}

/** 运行逻辑可用 Tag 分类 */
type RuntimeTagFolder =
  | 'AnalogInput'
  | 'BioCapt'
  | 'DigitalInput'
  | 'DigitalOutput'
  | 'VirtualDX'
  | 'IOModule'
  | 'ParticleCounter'
  | 'PumpsGroup'
  | 'PneumaticModule'
  | 'System';

interface RuntimeTag {
  id: string;
  folder: RuntimeTagFolder;
  name: string;
  dataType: 'bool' | 'number' | 'string';
  value?: boolean | number | string;
}

interface RuntimeRule {
  id: string;
  name: string;
  ruleType: string;           // 手册 Rule Type
  parentId?: string;          // 子规则
  conditions: RuntimeCondition[];
  actions: RuntimeAction[];
  enabled: boolean;
}

interface RuntimeCondition {
  tagId: string;
  operator: 'eq' | 'ne' | 'gt' | 'lt' | 'gte' | 'lte';
  value: boolean | number | string;
}

interface RuntimeAction {
  type: 'setDO' | 'startSampling' | 'stopSampling' | 'towerLight' | 'raiseAlarm';
  targetId: string;
  value?: boolean | number | string;
}
```

### 3.4 核心业务逻辑

1. **报警确认**：单条 Ack / 全部 Ack All → 电子签名 → 更新 `acknowledged*` 字段并记审计。
2. **实时趋势**：基于 Recipe Pens 订阅实时点；Start/Stop 需签名（Stop 明确要求签名）。
3. **编辑 Pens**：会话内调整颜色/启用；可回写到 Recipe（需权限）。
4. **运行逻辑**：从 Tag 树拖拽/选择 → Rule Editor 配置条件动作 → 保存启用。
5. **Sampling on Input**：输入边沿触发调度采样（与模块 1 联动）。
6. **Tower Lights**：按规则驱动灯塔输出；支持 Testing Output Activation。

### 3.5 关键 API（建议）

- `GET /api/alarms` / `POST /api/alarms/:id/ack` / `POST /api/alarms/ack-all`
- `POST /api/rt-trend/start|stop` / `GET /api/rt-trend/stream`（WS）
- `GET/PUT /api/runtime-rules` / `POST /api/runtime-rules/:id/test-output`
- `GET /api/runtime-tags`

---

## 模块 4：数据报表与统计分析

### 4.1 模块职责

从数据库提取采样/审计/趋势数据，生成可打印、可导出的报告；并支持 Statistical Data Analyzer（CSV 分析）。

### 4.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 报表生成器 | `/reports` | Report Window：Select by Group / by Sampling |
| Tags 目录配置 | `/reports/tags-catalog` | Sensor Type / Sensors / Groups / Recipes / Data Types… |
| 报表预览 | `/reports/preview/:id` | 缩放视图、打印、导出 PDF/CSV |
| 采样报告 | `/reports/sampling` | Sampling Report Window |
| 统计分析器 | `/sda` | Appendix A：加载 CSV、轴/缩放/插值/虚拟笔 |

### 4.3 数据模型（Types）

```ts
type ReportKind = 'audit' | 'data' | 'trend' | 'sampling';
type ReportSelectMode = 'byGroup' | 'bySampling';

interface ReportQuery {
  kind: ReportKind;
  mode: ReportSelectMode;
  dateFrom: string;
  dateTo: string;
  interval?: string;
  groupIds?: string[];
  recipeId?: string;
  samplingIds?: string[];
  sensorIds?: string[];
  dataTypeGroupIds?: string[];
  /** Trend 专用计算参数 */
  trendCalc?: {
    aggregation: 'avg' | 'min' | 'max' | 'sum';
    bucketSize: string;
  };
}

interface ReportDocument {
  id: string;
  query: ReportQuery;
  header: ReportHeader;
  body: ReportBodySection[];
  summary?: Record<string, number | string>;
  footer: ReportFooter;
  generatedAt: string;
  generatedBy: string;
}

interface ReportHeader {
  title: string;
  facility?: string;
  dateRange: string;
  filtersSummary: string;
}

interface ReportBodySection {
  title: string;
  columns: string[];
  rows: Array<Record<string, string | number | null>>;
}

interface ReportFooter {
  pageNote?: string;
  signedInfo?: string;
}

interface TagCatalogItem {
  id: string;
  tab:
    | 'sensorType'
    | 'sensorsAlarms'
    | 'sensors'
    | 'dataTypes'
    | 'sensorGroups'
    | 'dataTypeGroups'
    | 'recipes'
    | 'tagsControl';
  name: string;
  color?: string;
  linkedIds?: string[];
}

interface SdaSession {
  id: string;
  csvFiles: string[];
  displayConfig: Record<string, unknown>;
  virtualPens: RecipePen[];
  markersEnabled: boolean;
}
```

### 4.4 核心业务逻辑

1. **按组出报**：选时间范围 → 数据子集 → 模块组 → 报告类型 →（Trend 时填计算参数）→ Execute。
2. **按采样出报**：选时间 → 配方/采样/过滤 → 报告类型 → Execute。
3. **报告类型**：
   - **Audit Report**：操作审计（含电子签名 Performed by）
   - **Data Report**：采样数据 + Summary
   - **Trend Report**：趋势曲线参数与导出 PDF
   - **Sampling Report**：单次采样详情 + Sampling Alarms 段
4. **预览交互**：改视图、打印、导出。
5. **Tags Catalog（Admin）**：维护报表可选标签、笔颜色、History Automation。
6. **SDA**：加载 CSV → 轴/缩放/插值 → 虚拟笔 → 存/载配置 → 生成分析报告。
7. **自动化**：PMS Sampling Report Automation 周期检查已完成采样并导出到指定路径（后台任务）。

### 4.5 关键 API（建议）

- `POST /api/reports/generate` → `ReportDocument`
- `GET /api/reports/:id` / `GET /api/reports/:id/export?format=pdf|csv`
- `GET/PUT /api/tags-catalog`
- `POST /api/sda/load-csv` / `POST /api/sda/analyze`

---

## 模块 5：用户权限、电子签名与审计

### 5.1 模块职责

用户/用户组管理、密码策略、电子签名开关、关键操作审计轨迹；是合规与其它模块的安全底座。

### 5.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 登录 | `/login` | Log in window；默认 Nobody 只读游客态 |
| 用户管理 | `/management/user` | User Management window |
| 角色/用户组 | `/management/role` | 映射 Default User Groups 功能矩阵 |
| 密码策略 | `/management/password-policy` | 过期天数、最小长度、历史密码、自动登出 |
| 电子签名弹窗 | 全局组件 `ElectronicSignatureModal` | User / Password / Reason（可选） |
| 用户事件 | 主界面按钮触发 | User Event：自定义事件记入审计 |
| 审计查询入口 | 复用模块 4 Audit Report | 或 `/audit` 快捷列表 |

### 5.3 数据模型（Types）

```ts
type UserGroupRole =
  | 'Administrator'
  | 'Supervisor'
  | 'PowerUser'
  | 'User'
  | 'Emergency'
  | 'Nobody';

interface PharmaUser {
  id: string;
  userName: string;
  userIdCode: string;         // 手册 User ID
  role: UserGroupRole;
  expiryDate?: string;
  mustChangePassword: boolean;
  enabled: boolean;
  isLocalEmergency?: boolean; // 灾难恢复本地账号
}

interface PasswordPolicy {
  expireDays: number;
  minLength: number;
  rememberOldCount: number;
  autoLogoffSeconds: number;
  electronicSignatureEnabled: boolean;
}

interface ElectronicSignaturePayload {
  userName: string;
  password: string;
  action: string;             // 业务动作码，如 recipe.save
  targetType: string;
  targetId?: string;
  reason?: string;
}

interface AuditTrailEntry {
  id: string;
  eventTime: string;
  action: string;
  targetType: string;
  targetId?: string;
  performedBy: string;
  role: UserGroupRole;
  reason?: string;
  detail?: Record<string, unknown>;
}

/** 组功能矩阵（对应 Table 11-2，前端做按钮级权限） */
type GroupFunction =
  | 'Recipes'
  | 'Sampling'
  | 'SensorsView'
  | 'SensorsConfigure'
  | 'AlarmsAck'
  | 'Reports'
  | 'RTTrend'
  | 'RunTimeLogic'
  | 'UserManagement'
  | 'LimitsView'
  | 'LimitsConfigure'
  | 'BackupRestore'
  | 'SystemShutdown';
```

### 5.4 核心业务逻辑

1. **默认用户 Nobody**：启动即登录，仅浏览；正式操作需 Log out → Log in。
2. **用户 CRUD**：增/改/删均需电子签名；Emergency 权限≈Admin，但不能再创建 Emergency。
3. **密码策略**：过期强制改密；禁止复用最近 N 次密码；空闲自动登出。
4. **电子签名服务**：统一拦截关键写操作；校验账号密码 → 写 `AuditTrailEntry` → 放行业务。
5. **User Event**：手动记录自定义事件到审计。
6. **权限矩阵**：前端 `v-permission` + 后端二次校验（对齐 Table 11-2）。

### 5.5 关键 API（建议）

- `POST /api/auth/login|logout` / `GET /api/auth/me`
- `GET/POST/PUT/DELETE /api/users`
- `GET/PUT /api/password-policy`
- `POST /api/electronic-signature`
- `GET /api/audit-trail`
- `POST /api/user-events`

---

## 模块 6：系统运维与数据库初始化

### 6.1 模块职责

系统级运维：备份恢复、启停、语言、缓冲重置；以及 Aventino Setup 的安装/升级/库初始化/配置修改。

### 6.2 核心页面

| 页面 | 路由建议 | 说明 |
|------|----------|------|
| 系统管理 | `/system` | System management Window 入口 |
| 备份与恢复 | `/system/backup` | 日备配置、手动备份、拷贝备份/PDF |
| 系统控制 | `/system/control` | Close App / Restart / Shutdown / 断开客户端关机 |
| 语言设置 | `/system/language` | Change Display Language |
| 缓冲重置 | `/system/reset-buffer` | Reset the Buffer |
| Aventino 安装向导 | `/setup/wizard` | New Installation / Upgrade / Restore / Modify |
| 数据库初始化 | `/setup/db-init` | Database initialization |
| 数据库恢复 | `/setup/db-restore` | Restore from Backup / Restore FacilityPro |
| 采样器校准配置 | `/setup/sampler-calibration` | Sampler calibration configuration（安装向导子步） |

### 6.3 数据模型（Types）

```ts
interface BackupConfig {
  enabled: boolean;
  dailyAt: string;            // HH:mm
  targetPath: string;
  retainDays: number;
}

interface BackupJob {
  id: string;
  type: 'manual' | 'daily';
  status: 'pending' | 'running' | 'success' | 'failed';
  filePath?: string;
  startedAt: string;
  finishedAt?: string;
  message?: string;
}

type SetupWizardMode =
  | 'newInstallation'
  | 'upgrade'
  | 'restoreFromBackup'
  | 'modifyConfiguration'
  | 'restoreFacilityPro';

interface SetupWizardState {
  mode: SetupWizardMode;
  step: number;
  dbConnection: {
    host: string;
    port: number;
    database: string;
    user: string;
    password: string;
  };
  restoreFile?: string;
  calibration?: SamplerCalibrationConfig;
}

interface SamplerCalibrationConfig {
  samplerId: string;
  parameters: Record<string, number | string>;
}

interface SystemCommand {
  action: 'closeApp' | 'restart' | 'shutdown' | 'shutdownDisconnectedClient' | 'resetBuffer';
  signedBy: string;
}
```

### 6.4 核心业务逻辑

1. **日备 / 手备**：Admin 配置路径与时间；手动 Backup 立即执行；支持拷贝备份文件与报表 PDF。
2. **恢复**：从备份包恢复库与配置；FacilityPro 专用恢复入口。
3. **新装向导**：建库 → 初始化 schema/种子数据 → 可选校准配置 → 完成。
4. **升级**：检测旧版本 → 迁移脚本 → 备份兜底。
5. **系统控制**：Restart/Shutdown 影响 Server+Clients；需 Admin + 电子签名。
6. **Reset Buffer**：清空运行缓冲（不影响历史库），需确认与签名。
7. **与当前项目衔接**：已有 MySQL `schema.sql` / `init-data.sql` / Tauri Axum 可作为本模块「Database initialization」落地基础。

### 6.5 关键 API（建议）

- `GET/PUT /api/system/backup-config`
- `POST /api/system/backup` / `POST /api/system/restore`
- `POST /api/system/command`
- `POST /api/setup/init-db` / `POST /api/setup/upgrade`
- `GET/PUT /api/setup/calibration`

---

## 推荐落地顺序（迭代）

```text
Sprint A (底座)
  └─ 模块 5 用户权限 + 电子签名 + 审计
  └─ 模块 6 数据库初始化（复用现有 MySQL/Tauri）

Sprint B (核心业务)
  └─ 模块 2 传感器监控与限值
  └─ 模块 1 配方 + 采样调度

Sprint C (运行与合规输出)
  └─ 模块 3 报警确认 + 实时趋势（运行逻辑可二期）
  └─ 模块 4 Audit/Data/Sampling 报表

Sprint D (增强)
  └─ 运行逻辑完整编辑器、SDA、Aventino 向导、自动导出任务
```

---

## 模块依赖关系

```text
                    ┌─────────────────────┐
                    │ 5. 权限/签名/审计    │
                    └──────────┬──────────┘
                               │ 签名 & 鉴权
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                      ▼
┌───────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ 1. 配方与采样  │◄───│ 2. 设备与传感器  │───►│ 3. 报警/趋势/逻辑 │
└───────┬───────┘    └────────┬────────┘    └────────┬────────┘
        │                     │                      │
        └──────────┬──────────┴──────────────────────┘
                   ▼
         ┌─────────────────┐         ┌─────────────────┐
         │ 4. 报表与统计    │◄────────│ 6. 系统运维/库   │
         └─────────────────┘  备份库  └─────────────────┘
```

---

## 与现有 tauri-vue-admin 的衔接说明

| 已有能力 | 可复用到 |
|----------|----------|
| 登录 / Token / 动态路由 / 用户角色 CRUD | 模块 5（需扩展角色组与电子签名） |
| MySQL schema + Axum `/api/management/*` | 模块 6 初始化；模块 5 用户表扩展 |
| 管理端页面骨架（user/role） | 模块 5 页面增强 |
| Vite Mock 关闭 + Desktop API | 全模块统一走 `127.0.0.1:8080` |

**下一步建议：** 选定 Sprint A，先出模块 5 的表结构变更（`sys_user` 扩展 + `sys_audit_trail` + `sys_password_policy`）与电子签名中间件，再挂接模块 1/2 写操作。
