# Portunex 重建蓝图（RECONSTRUCTION_BLUEPRINT）

> 用途:照着这份蓝图,把 197 个模块骨架逐个填成真实实现,重建出一个功能等价的 Portunex。
> 依据:对原 `portunex-server`(去符号 Rust 二进制)的路径痕迹、字符串、路由、DB schema、
> 配置、前端产物的逆向。**内部算法不可见,标 TODO,需你(或 Codex)按契约实现。**

---

## 0. 现状(Phase 1 已完成)

**可运行核心**(`crates/portunex-gateway`):
- `src/config.rs` — 完整 `PORTUNEX__*` 配置层级 + 默认值。
- `src/main.rs` — 配置 → 连 Postgres → `sqlx::migrate!` → 首启播种 admin(随机密码打印日志,argon2id)→ 起服务。
- `src/state.rs` — `AppState`(pool + settings;待接 redis/clickhouse/upstream/scheduler)。
- `src/server/mod.rs` — CORS(`*`)+ `/health` `/ready`(与原实例逐字节一致)+ 全部业务端点接线(现返回 501)。
- `crates/portunex-gateway/migrations/0001_init.sql` — **真实 33 表 schema**(pg_dump 恢复)。

**其余 196 个模块** = 带头注释的骨架文件,按下方职责填。

### 怎么编译 / 跑
```bash
# 需要 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd ~/portunex-rebuild
export PORTUNEX__DB__URL='postgres://portunex:PASS@localhost:5432/portunex'
export PORTUNEX__SERVER__PORT=8080
# 可直接复用现成 compose 栈里的 postgres(~/portunex-stack)
cargo run -p portunex-gateway
# 首启日志会打印 admin@portunex.local 的随机密码;curl :8080/health /ready 验证
```
> 第一次 `cargo build` 大概率有编译错误需要修(我无法在本机编译验证)——按报错迭代即可,核心结构是对的。

---

## 1. Workspace 架构(7 crate)

| crate | 职责 | 文件数 |
|---|---|---|
| **portunex-core** | 统一中间表示(IR)+ Claude/OpenAI/Gemini 三家格式**互转** translators + 模型 | 10 |
| **portunex-db** | 33 表的 entities + repositories(数据访问层,sqlx) | 34 |
| **portunex-upstream** | 上游厂商调用:anthropic/openai/gemini/antigravity 的 client + oauth + WS 池 + node_proxy | 27 |
| **portunex-clickhouse** | 请求日志写入 ClickHouse(events/bodies/request_logs) | 4 |
| **portunex-redis** | 缓存层(auth/balance/provider/sticky 的 TTL 缓存) | 2 |
| **portunex-logging** | 日志设施 | 1 |
| **portunex-gateway** | 主程序:handlers / middleware / routes/admin/* / services / websocket | 119 |

> 完整文件清单见 `MODULE_TREE.txt`(197 行)。

---

## 2. 关键模块职责(按填充优先级)

### portunex-db(先做——其它都依赖它)
- `src/entities/*.rs` — 每张表一个 struct(`#[derive(sqlx::FromRow)]`),字段照 `migrations/0001_init.sql`。
- `src/repositories/*.rs` — 每个实体的 CRUD + 业务查询。关键:`user`、`api_key`、`auth_session`、
  `provider` / `provider_credential`、`model_alias` / `*_pricing`、`order` / `payment_channel`、
  `redemption_code`、`subscription_plan` / `user_subscription`、`sticky_routing`、`oidc_*`、`kv_store`。

### portunex-core(格式互转——网关的核心价值)
- `translators/{claude_to_openai, openai_to_claude, gemini_to_claude, gemini_to_chatcompletions}.rs`
  —— 请求/响应/流式事件在三家格式间转换(含 tool calls、cache_control、SSE)。
- `ir/*` + `models/openai/responses.rs` —— 内部统一表示 + OpenAI Responses API 模型。

### portunex-upstream(上游接入)
- `{anthropic,openai,gemini,antigravity}/oauth.rs` —— 各家 OAuth 换/刷 token(Claude:platform.claude.com;
  OpenAI:auth.openai.com;Gemini:Google cloud-platform;Antigravity:googleapis)。
- `{anthropic,openai}/ws/{pool,connection,transport}.rs` —— WebSocket 上游**连接池**
  (`OpenAI-Beta: responses_websockets=…`,idle/preamble/lease 管理)。
- `node_proxy/client.rs` —— 经 node-proxy 做 TLS 指纹伪装的出站。
- `gemini/{service_account,cloudcode}.rs`、`sse/mod.rs`、`errors/mod.rs`。

### portunex-gateway
- `handlers/{auth,user,api_key,provider,points,usage,session}.rs` —— 用户面业务。
- `middleware/{auth,oidc_auth,request_log}.rs` —— Bearer(`portunex_token`)鉴权、OIDC 鉴权、请求日志。
- `routes/{claude,openai_compat,gemini,messages,responses,models,oidc,auth,api_keys}.rs` —— 客户端 API 路由。
- `routes/admin/*.rs` —— 后台 CRUD(users/providers/api_keys/orders/payment_channels/
  subscription_plans/redemption_codes/model_aliases/*_pricing/request_logs/sessions/stats/window_configs/points/oidc_clients)。
- `services/*` —— **调度器**(sticky + load-aware spill + P2C + flood sentinel + headroom;见配置项)、
  provider 缓存、failover、OIDC 签发、计费。
- `websocket/*` —— 面向客户端的 WS + responses 缓存。

---

## 3. API 契约(客户端可见端点)

| 端点 | 方法 | 说明 |
|---|---|---|
| `/health` `/ready` | GET | 探活(已实现) |
| `/v1/models` | GET | OpenAI 兼容模型列表(源自 model_aliases) |
| `/v1/chat/completions` `/v1/responses` `/responses` | POST | OpenAI 兼容 |
| `/messages` `/messages/count_tokens` | POST | Anthropic 兼容 |
| `/v1beta/*` | POST | Gemini |
| `/auth/*` | POST | 邮箱密码 / magic link / LinuxDO OAuth 登录 |
| `/api-keys` `/subscriptions` `/usage` `/points/stats` | GET/POST | 用户面 |
| `/oauth/{claude,openai,gemini,antigravity}/{start,refresh,import}` | POST | 上游账号接入 |
| `/.well-known/openid-configuration` `/oidc/{jwks,authorize,token,userinfo,revoke}` | GET/POST | **自建 OIDC 提供方** |
| `/admin/*` | GET/POST/PUT/DELETE | 管理后台(鉴权:role=admin) |

鉴权:客户端用 `Authorization: Bearer <api_key>`(前端 token 键 `portunex_token`)。

---

## 4. 配置(PORTUNEX__* — 见 config.rs)
数据库 / 路由调度(sticky_ttl、load_aware_spill、p2c_choices、flood_sentinel、headroom_reserve)/
upstream 超时 / anthropic(downgrade_1h_cache_to_5m 转售计费开关、WS)/ openai(WS)/ redis / clickhouse /
oidc / user_oauth(linuxdo)/ mail / captcha / media(local|s3)。

---

## 5. 前端(单独重建)
原前端是 React-Router v7/Remix SPA(Vite + Tailwind + three.js),**无 source map**,只能从产物看结构。
页面/路由(从 assets 文件名还原):`_dashboard.{providers,users,api-keys,orders,payment-channels,
subscription-plans,redemption-codes,model-aliases,model-alias-pricing,provider-type-pricing,
request-logs,logs,usage/stats,recharge,redeem,checkout,my-orders,subscriptions,window-configs,
oidc-clients,pricing}`、`auth.{magic,oauth.callback}`、`oidc.authorize`。
建议:新起一个 React-Router v7 项目,按上面页面清单 + 第 3 节 API 契约重写(比逆向压缩 JS 更快更干净)。

---

## 6. 建议构建顺序
1. **portunex-db**:entities + repositories(照 schema)——地基。
2. **portunex-gateway 用户面**:auth(注册/登录/argon2/session)、api_keys、middleware/auth。
3. **portunex-core translators** + **portunex-upstream** 一家(先 Claude)→ 打通 `/messages` 转发。
4. 扩展 OpenAI/Gemini/Antigravity 上游 + `/v1/*`。
5. **调度器 services**(sticky/spill/p2c…)+ provider 后台。
6. 计费(points/orders/subscriptions/redemption)+ 支付渠道。
7. OIDC 提供方。8. ClickHouse 日志 + Redis 缓存。9. 前端。

---

## 已完成阶段(实跑验证,均已提交推送)

- **P1 核心**:配置 / 迁移(33表)/ 首启播种 admin / `/health` `/ready`(还原)。
- **P2 数据层**:32 实体(schema 精确生成)+ repositories。
- **P3 认证**:`/auth/register` `/auth/login`(argon2 + 会话 token)+ Bearer/Admin 提取器。
- **P4 用户面**:`/me`、API Key 增删查(`/api-keys*`)。
- **P5 计费+后台**:兑换码→积分(`/redeem`)、`/admin/*` CRUD(users/providers/model-aliases/codes)、RBAC、`/v1/models` 真实数据。
- **P6 订阅**:`/subscriptions`(积分购买/到期)、`/points/stats`、`/usage`、套餐管理。

### 剩余(需你的凭据或属大件,未做)
- **上游转发**(P4 蓝图第 3-5 步):`/messages` `/v1/chat/completions` 等 + Claude/OpenAI/Gemini/Antigravity 的 OAuth + translators + 调度器。**要真验证必须用你的真实订阅/OAuth,会消耗额度**,我这边无法测。
- OIDC 提供方签发、ClickHouse/Redis 接入、前端。
- 已知 TODO:subscribe 的扣分+建订阅应包进单事务(当前分两步)。
