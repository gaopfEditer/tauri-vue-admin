# 软件授权（时间锁 + 设备 MAC 锁）

## 能力

| 类型 | 行为 |
|------|------|
| **时间锁** | 仅在 `validFrom`～`validUntil` 内可用；到期后登录与业务 API 全部拒绝 |
| **设备锁** | `deviceLock=true` 时，首次激活绑定本机 MAC；换机后 `mac_mismatch` 锁定 |
| **防回拨** | 记录 `max_observed_at`，系统时间被大幅回拨则锁定 |

## 环境变量（`.env`）

```bash
LICENSE_BYPASS=Y                 # 开发旁路；现场必须为 N
LICENSE_SECRET=...               # 授权码签名密钥（厂商与交付一致）
LICENSE_MASTER_KEY=...           # 签发接口密钥
```

## 现场流程

1. 执行 `database/mysql/pharma-license-schema.sql`
2. 设置 `LICENSE_BYPASS=N`，更换上述密钥
3. 厂商在「系统运维 → 授权管理」用 MASTER KEY **签发**一段可用期的授权码（建议勾选设备锁）
4. 现场打开 `/license` 或授权管理页 **激活** → 绑定 MAC
5. 正常登录使用；到期或换机后进入锁定页，需新授权码

## API

- `GET /api/license/status`
- `POST /api/license/activate` `{ licenseCode }`
- `POST /api/license/issue` `{ customer, validFrom, validUntil, deviceLock, masterKey }`

未授权时除 `/api/license/*` 外接口返回业务码 `40301`。
