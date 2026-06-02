# 半中心化管理实施方案

**文档版本**: 1.0  
**最后更新**: 2026-06-02  
**分支**: `main`  
**状态**: 待实施

---

## 1. 概述

### 1.1 背景

Easytier 是一个去中心化 P2P VPN 工具。只要两台设备的 `network_name`（name）和 `network_secret`（secret）匹配，且有公网 IP，就能直接建立连接——无需经过任何中心服务器。

这带来了一个安全风险：在"云端 admin + 关键本地设备"的部署场景中，关键本地设备（如 10.0.210.253）拥有公网 IP，外部恶意设备可以在获知 network_name/secret 后直接连接到该设备，完全绕开 admin 的管控。

### 1.2 目标

在不改变 easytier 去中心化通信模型的前提下，实现 **"半中心化"IP 白名单管理**：

- Admin 统一配置白名单，通过 HTTP 端点暴露
- 关键本地设备部署 Agent，周期同步白名单并强制阻断白名单外的设备
- Admin 失联时，Agent 维持最后已知白名单，不降低安全性

### 1.3 架构概览

```
                      +------------------+
                      |   Admin (云端)    |
                      |   10.0.210.1     |
                      |                  |
                      | easytier-web     |
                      |   +-----------+  |
                      |   | SQLite    |  |  POST /api/v1/admin/ipwhitelist/...
                      |   | ip_whitelist| <--- Admin UI (Vue3)
                      |   +-----+-----+  |
                      |         |        |
                      |   whitelist.json |  GET /api/v1/public/whitelist.json
                      |   (export)       |  (无需认证)
                      +--------+---------+
                               |
                  30s HTTP GET | (虚拟网段内)
                               |
          +--------------------+--------------------+
          |                    |                    |
  +-------v--------+  +-------v--------+  +-------v--------+
  | Agent #1       |  | Agent #2       |  | Agent #3       |
  | 10.0.210.253   |  | 10.0.210.100   |  | 10.0.210.200   |
  |                |  |                |  |                |
  | sync-daemon    |  | sync-daemon    |  | sync-daemon    |
  |    |           |  |    |           |  |    |           |
  |    v           |  |    v           |  |    v           |
  | IP_WHITELIST_  |  | IP_WHITELIST_  |  | IP_WHITELIST_  |
  | FILE           |  | FILE           |  | FILE           |
  |    |           |  |    |           |  |    |           |
  |    v           |  |    v           |  |    v           |
  | easytier-core  |  | easytier-core  |  | easytier-core  |
  | (30s 轮询阻断) |  | (30s 轮询阻断) |  | (30s 轮询阻断) |
  +----------------+  +----------------+  +----------------+
          ^                    ^                    ^
          |   外部设备被阻断   |                    |
          +---- 白名单外 IP ----+--------------------+
```

---

## 2. 架构设计

### 2.1 关键数据流

```
[Admin UI] → SQLite ip_whitelist 表
       ↓ (30s, entrypoint.sh / 或直接查询)
[GET /api/v1/public/whitelist.json] → JSON [{ip, hostname}, ...]
       ↓ (30s, sync-daemon HTTP GET)
[IP_WHITELIST_FILE 磁盘 JSON]
       ↓ (30s, easytier-core 文件读取)
[load_ip_whitelist()] → Vec<WhitelistEntry>
       ↓ (遍历 route 列表对比)
[block_peer() / close_peer_conn()] → 阻断连接
```

### 2.2 关键设计决策

| 决策 | 理由 |
|------|------|
| Agent 拉取而非 Admin 推送 | Admin 不需要知道 Agent 地址，架构简单；天然支持 Agent 离线场景 |
| JSON 端点无需认证 | 白名单仅包含"允许访问的 IP 列表"，不含敏感信息（如密码、secret）；简化 Agent 实现 |
| Admin 失联时维持最后白名单 | 保证安全性：不会因为 Admin 不可达而放行所有设备 |
| sync-daemon 用 Rust 实现 | 与项目技术栈一致；可复用现有 Dockerfile 构建流程；二进制无外部依赖 |
| 单 Admin 实例 | 用户环境已明确只有 1 个 Admin，不存在多 Admin 写冲突 |

---

## 3. Phase 1：Admin 公开 JSON 端点

### 3.1 目标

暴露 `GET /api/v1/public/whitelist.json`，无需认证，返回完整白名单列表（JSON 数组）。

### 3.2 改动清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `easytier-web/src/restful/admin/whitelist_export.rs` | **新增** | 公开端点 handler |
| `easytier-web/src/restful/admin/mod.rs` | **修改** | 注册模块和路由（在 `build_router` 中，不含 auth middleware） |

### 3.3 关键设计

**Handler 签名**（参考 `ipwhitelist.rs:41-43` 的 `handle_list_whitelist` 模式）：

```rust
use axum::{Extension, Json};
use super::AdminState;
use crate::db::entity;

pub async fn handle_export_whitelist(
    Extension(state): Extension<AdminState>,
) -> Json<Vec<serde_json::Value>> {
    // 查询 ip_whitelist 表，映射为 [{ip, hostname}]，直接返回
}
```

**数据来源**: SQLite `ip_whitelist` 表（`easytier-web/src/db/entity/ip_whitelist.rs:3-13`），字段与现有 entity 一致：`id, ip, comment, hostname, created_by, created_at`。

**输出格式**（与 core 的 `WhitelistEntry` 结构对齐，`global_ctx.rs:60-64`）：

```json
[
  {"ip": "10.0.210.253", "hostname": "my-server"},
  {"ip": "10.0.210.100", "hostname": null}
]
```

`hostname` 为 `null` 表示不限制 hostname 绑定（任何设备使用该 IP 都放行）。

（完整契约见 3.5 节）

**路由注册位置**（`mod.rs:227-231`，在 `.merge(protected)` 之前）：

```rust
// 在 Router::new() 中，紧挨 login 路由下方添加：
.route("/api/v1/public/whitelist.json", get(whitelist_export::handle_export_whitelist))
```

### 3.4 验证标准

```bash
curl http://admin:11211/api/v1/public/whitelist.json
# 预期输出: [{"ip":"10.0.210.253","hostname":"my-server"}, ...]
```

- 返回格式为 JSON 数组
- 包含所有 `ip_whitelist` 表中的记录
- `hostname` 为 null 时不输出（或输出 null，取决于序列化配置）

### 3.5 JSON 端点字段契约

为保证 bind/unbind 操作能够正确同步到所有 agent，Phase 1 实现的 JSON 端点必须遵循以下字段契约。

**响应结构**: 顶层为 JSON 数组，每个元素为一条白名单记录。

**字段定义**:

| 字段 | 类型 | 是否必含 | 说明 |
|------|------|----------|------|
| `ip` | string | ✅ 必含 | 白名单 IP 地址，agent 阻断逻辑的主键 |
| `hostname` | string \| null | ✅ 必含 | 绑定的设备主机名；`null` 表示不限制 hostname（任何设备用该 IP 都放行） |
| `comment` | string \| null | ⚪ 可选 | 备注，agent 不使用，仅 admin 端展示 |
| `created_at` | string | ⚪ 可选 | 创建时间，agent 不使用 |

**核心要求**:

1. **`hostname` 字段必须输出**，即使值为 `null`（不能省略字段名）。这是 bind/unbind 同步的关键。
2. **agent sync-daemon 必须原样透传整个 JSON 数组**到 `IP_WHITELIST_FILE`，不得做字段过滤或重命名。
3. core 端 `load_ip_whitelist()` 已实现兼容两种情况（`hostname` 为字符串或不存在），但**为保证 hostname 绑定生效，admin 端必须输出 `hostname` 字段**。

**反例（禁止行为）**:

```json
// ❌ 错误：缺少 hostname 字段，无法同步 unbind 操作
[{"ip": "10.0.210.253"}]

// ❌ 错误：把 hostname 改成 device_name 之类的别名
[{"ip": "10.0.210.253", "device_name": "my-server"}]

// ✅ 正确：保留 hostname 字段，值为 null 表示未绑定
[{"ip": "10.0.210.253", "hostname": "my-server"}]
[{"ip": "10.0.210.100", "hostname": null}]
```

**与已有代码对齐**:

- SQLite `ip_whitelist` 表（`easytier-web/src/db/entity/ip_whitelist.rs:5-13`）字段：`id, ip, comment, hostname, created_by, created_at`
- Admin `handle_unbind_whitelist`（`ipwhitelist.rs:117-138`）将 `hostname` 置为 `None`——这正是要被同步的状态
- Core `WhitelistEntry`（`easytier/src/common/global_ctx.rs:60-64`）结构：`{ ip, hostname }`
- Core `load_ip_whitelist`（`global_ctx.rs:521-543`）从 JSON 读取 `ip` 和 `hostname` 两个字段

**建议实现**（handler 内的字段映射）：

```rust
// 伪代码
let entries: Vec<serde_json::Value> = rows.into_iter().map(|row| {
    serde_json::json!({
        "ip": row.ip,
        "hostname": row.hostname,  // 保留 null 语义，不要 filter
    })
}).collect();
Json(entries)
```

**验证标准补充**:

```bash
# 1. 创建并绑定 hostname
curl -X POST .../ipwhitelist -d '{"ip":"10.0.210.50","hostname":"device-x"}'
curl http://admin:11211/api/v1/public/whitelist.json
# 预期: [{"ip":"10.0.210.50","hostname":"device-x"}, ...

# 2. 解绑 hostname
curl -X POST .../ipwhitelist/unbind -d '{"id":<id>}'
curl http://admin:11211/api/v1/public/whitelist.json
# 预期: [{"ip":"10.0.210.50","hostname":null}, ...

# 3. 删除条目
curl -X POST .../ipwhitelist/delete -d '{"id":<id>}'
curl http://admin:11211/api/v1/public/whitelist.json
# 预期: 该条目已不在数组中
```

**3 种操作的同步结果**:

| Admin 操作 | Agent `IP_WHITELIST_FILE` 变化 | 生效方式 |
|------------|-------------------------------|----------|
| 创建并绑定 hostname | 新增 `{"ip":..., "hostname":...}` 条目 | 下一个轮询周期（≤30s） |
| 解绑 hostname | 该条目 `hostname` 变 `null` | 下一个轮询周期（≤30s） |
| 删除条目 | 该条目从数组移除 | 下一个轮询周期（≤30s） |

---

## 4. Phase 2：Agent 容器 + Rust 同步守护进程

### 4.1 sync-daemon 设计

**二进制名称**: `easytier-whitelist-sync`

**位置**: `easytier/src/bin/whitelist_sync.rs`（与 `easytier-core.rs`、`easytier-cli.rs` 同级）

**核心逻辑**（伪代码）：

```rust
#[tokio::main]
async fn main() {
    let admin_url = env::var("WHITELIST_SYNC_URL").expect("WHITELIST_SYNC_URL not set");
    let output_file = env::var("IP_WHITELIST_FILE").unwrap_or("/tmp/ip_whitelist.json".into());
    let interval = env::var("WHITELIST_SYNC_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30u64);

    let client = reqwest::Client::new();
    loop {
        match client.get(&admin_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.text().await {
                    // 验证 JSON 有效性后原子写入
                    if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                        let tmp = format!("{}.tmp", output_file);
                        fs::write(&tmp, &body).ok();
                        fs::rename(&tmp, &output_file).ok();
                        tracing::info!("Whitelist synced successfully");
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("Admin returned status {}, keeping last whitelist", resp.status());
            }
            Err(e) => {
                tracing::warn!("Admin unreachable ({}), keeping last whitelist", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
```

**关键依赖**（需要在 `easytier/Cargo.toml` 中添加）：

```toml
# 已存在
tokio = { version = "1", features = ["full"] }
serde_json = "1"

# 需新增
reqwest = { version = "0.12", features = ["json"] }
```

> **注意**: `reqwest` 已在 `[target.'cfg(windows)'.build-dependencies]` 中存在（`0.12.12`），这里需要将其提升为常规依赖或在对应 target 下重复声明。

**环境变量配置**:

| 变量 | 必需 | 默认值 | 说明 |
|------|------|--------|------|
| `WHITELIST_SYNC_URL` | 是 | - | Admin 端点完整 URL，如 `http://10.0.210.1:11211/api/v1/public/whitelist.json` |
| `IP_WHITELIST_FILE` | 否 | `/tmp/ip_whitelist.json` | 白名单 JSON 输出路径，与 core 的 `IP_WHITELIST_FILE` 一致 |
| `WHITELIST_SYNC_INTERVAL` | 否 | `30` | 轮询间隔（秒） |
| `RUST_LOG` | 否 | `info` | 日志级别 |

### 4.2 构建配置

在 `easytier/Cargo.toml` 中添加新的 bin target（`easytier/Cargo.toml:17-20` 之后）：

```toml
[[bin]]
name = "easytier-whitelist-sync"
path = "src/bin/whitelist_sync.rs"
test = false
```

`Dockerfile` 中复制新的二进制（`Dockerfile:60` 之后）：

```dockerfile
COPY --from=builder /app/target/release/easytier-whitelist-sync /usr/local/bin/easytier-whitelist-sync
```

### 4.3 Agent Docker Compose 模板

```yaml
# /opt/easytier-agent/docker-compose.yml
version: "3.8"
services:
  agent:
    image: easytier:latest
    container_name: easytier-agent
    restart: unless-stopped
    network_mode: host
    cap_add:
      - NET_ADMIN
    environment:
      - WHITELIST_SYNC_URL=http://10.0.210.1:11211/api/v1/public/whitelist.json
      - IP_WHITELIST_FILE=/tmp/ip_whitelist.json
      - WHITELIST_SYNC_INTERVAL=30
      - RUST_LOG=info
    volumes:
      - ./core.toml:/etc/easytier/core.toml:ro
      - ./data:/data
    command: >
      sh -c "
        easytier-whitelist-sync &
        IP_WHITELIST_FILE=/tmp/ip_whitelist.json
        easytier-core -c /etc/easytier/core.toml
      "
```

### 4.4 验证标准

1. **白名单生效**: 从非白名单设备 ping agent 的虚拟 IP，无法 ping 通（core 已阻断）
2. **Admin 失联安全**: 停止 admin 容器，agent 仍然阻断白名单外设备（维持最后文件）
3. **恢复同步**: 重启 admin 并添加新 IP 到白名单，60s 内（2 次 daemon 轮询 + 1 次 core 检测）新设备可连接 agent
4. **日志验证**: `docker logs -f easytier-agent` 可见 sync daemon 的同步成功/失败日志

---

## 5. Phase 3：Admin 后台管理 Agent 节点

### 5.1 数据模型

**新增 SQLite 表**（通过 sea-orm migration 或手动建表）：

```sql
CREATE TABLE agent_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    virtual_ip TEXT NOT NULL,
    last_sync_at TIMESTAMP,
    last_sync_status TEXT DEFAULT 'unknown',  -- 'success' | 'failed' | 'unknown'
    created_at TIMESTAMP DEFAULT (datetime('now'))
);
```

**实体文件**: `easytier-web/src/db/entity/agent_node.rs`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "agent_nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub virtual_ip: String,
    pub last_sync_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub last_sync_status: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
```

**注册到 entity mod**: 在 `easytier-web/src/db/entity/mod.rs` 中添加 `pub mod agent_node;`，同步更新 `prelude.rs`。

### 5.2 后端 API

**新增文件**: `easytier-web/src/restful/admin/agent.rs`

**接口列表**:

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| `GET` | `/api/v1/admin/agents` | `handle_list_agents` | 列出所有 agent（需要 admin 认证） |
| `POST` | `/api/v1/admin/agents` | `handle_create_agent` | 注册新 agent（名称 + 虚拟 IP） |
| `DELETE` | `/api/v1/admin/agents/:id` 或 `/api/v1/admin/agents/delete` | `handle_delete_agent` | 删除 agent（通过 JSON body 传递 id） |
| `GET` | `/api/v1/admin/agents/:id/sync_status` | `handle_agent_sync_status` | [可选] 查看同步状态 |

**请求/响应结构**（参考 `ipwhitelist.rs` 模式）：

```rust
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub virtual_ip: String,
}

#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub id: i32,
    pub name: String,
    pub virtual_ip: String,
    pub last_sync_at: Option<String>,
    pub last_sync_status: String,
    pub created_at: String,
}
```

**路由注册**（`mod.rs:214` 的 `protected` Router 内添加）：

```rust
.route("/api/v1/admin/agents", get(agent::handle_list_agents))
.route("/api/v1/admin/agents/create", post(agent::handle_create_agent))
.route("/api/v1/admin/agents/delete", post(agent::handle_delete_agent))
```

路由放在 `protected` 组内，自动经过 `auth_middleware` 认证。

### 5.3 前端页面

**新增组件**: `easytier-web/frontend/src/components/Agents.vue`

参考 `IpWhitelist.vue` 的模式实现：

- 列表展示：名称、虚拟 IP、最后同步时间、同步状态（带颜色标识）
- 注册弹窗：输入名称和虚拟 IP、提交按钮
- 删除操作：按钮 + 确认对话框
- [可选] 手动触发同步按钮

**API Client 扩展**（`easytier-web/frontend-lib/src/modules/api.ts:183` 附近）：

```typescript
// Agent 管理
public async list_agents(): Promise<Array<any>> {
    const response = await this.client.get<any, Array<any>>('/admin/agents');
    return response;
}

public async create_agent(name: string, virtual_ip: string): Promise<undefined> {
    await this.client.post<any>('/admin/agents/create', { name, virtual_ip });
}

public async delete_agent(id: number): Promise<undefined> {
    await this.client.post<any>('/admin/agents/delete', { id });
}
```

### 5.4 同步状态上报 [可选]

Agent sync daemon 成功同步后向 admin 发送心跳：

- `POST /api/v1/agents/:id/heartbeat` — admin 更新 `last_sync_at` 和 `last_sync_status`
- Agent 注册时获得 ID，存储在本地配置文件
- 此功能作为 Phase 3 的可选子任务

### 5.5 验证标准

1. Admin UI 可注册 agent（输入名称 + 虚拟 IP）
2. Agent 列表显示已注册节点，IP 与 docker-compose 配置一致
3. 可删除 agent 节点
4. [可选] 同步状态列显示 `success` / `failed` / `unknown`

---

## 6. 部署与运维

### 6.1 部署步骤

#### 第一阶段：云端 Admin 部署

```bash
# 1. 构建镜像
docker build -t easytier:latest .

# 2. 准备配置
mkdir -p /opt/easytier-admin/data
cp core.toml /opt/easytier-admin/

# 3. 启动 admin
docker run -d \
  --name easytier-admin \
  --network host \
  --cap-add NET_ADMIN \
  -v /opt/easytier-admin/core.toml:/etc/easytier/core.toml:ro \
  -v /opt/easytier-admin/data:/data \
  -e ET_ADMIN_USERNAME=admin \
  -e ET_ADMIN_PASSWORD=your-secure-password \
  -e ET_ADMIN_SECRET=your-random-secret \
  easytier:latest

# 4. 通过 Admin UI 配置白名单
# 访问 http://10.0.210.1:11211 登录后添加 IP 白名单
```

#### 第二阶段：Agent 部署

```bash
# 1. 准备 agent 部署目录
mkdir -p /opt/easytier-agent/{config,data}

# 2. 准备 core.toml（配置 network_name、network_secret、虚拟 IP 等）
cp core.toml.template /opt/easytier-agent/config/core.toml

# 3. 创建 docker-compose.yml（参考 4.3 节模板）

# 4. 启动
cd /opt/easytier-agent && docker-compose up -d

# 5. 验证
docker logs -f easytier-agent
```

### 6.2 监控建议

| 监控项 | 检测方式 | 期望 |
|--------|----------|------|
| Admin 在线 | Agent 日志中 sync daemon 无 `unreachable` 错误 | 持续 "synced successfully" |
| 白名单同步延迟 | 修改 admin 白名单后，检测 agent 阻断变化的时间 | < 60s |
| 阻断连接数 | Core 日志中 `Disconnected peer` 事件 | 正常业务波动 |
| 磁盘空间 | `IP_WHITELIST_FILE` 所在分区 | > 100MB 空闲 |

### 6.3 故障排查

| 现象 | 可能原因 | 排查方法 |
|------|----------|----------|
| Agent sync daemon 报 `unreachable` | Admin 不在线 / 网络不通 | `docker exec easytier-agent ping 10.0.210.1` |
| 白名单外设备仍可连接 | `IP_WHITELIST_FILE` 路径不一致 | 检查 core 和 daemon 的 env var 是否指向同一文件 |
| 白名单内设备被误阻断 | hostname 绑定冲突 / 路由信息延迟 | 查看 core 日志，确认自动绑定信息 |
| sync daemon 未启动 | 二进制缺失 / 命令错误 | `docker exec easytier-agent which easytier-whitelist-sync` |

---

## 7. 风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 白名单同步延迟（最长 60s） | 低 | 业务可接受；降低 `WHITELIST_SYNC_INTERVAL` 到 15s 可减半 |
| Admin 完全失联 | 低 | Agent 维持最后白名单，安全性不降低；Admin 部署在云端有冗余 |
| JSON 端点可被网络内任何设备访问 | 低 | 仅暴露 IP 列表，不含密码/secret；虚拟网段本身需要 name/secret 认证 |
| sync-daemon 进程崩溃 | 中 | Docker 的 `restart: unless-stopped` 确保重启；core 维持最后文件继续工作 |
| 多 Agent 同时部署的复杂度 | 低 | 提供 docker-compose 模板，复制粘贴即可；一键脚本可选 |

---

## 8. 实施时间表（参考）

| Phase | 内容 | 预估工时 | 依赖 |
|-------|------|----------|------|
| Phase 1 | Admin 公开 JSON 端点 | 0.5 - 1 天 | 无 |
| Phase 2 | sync-daemon + Agent 容器 | 1.5 - 2 天 | Phase 1 |
| Phase 3 | Admin agent 管理 UI | 2 - 3 天 | Phase 1 |
| 联调测试 | 端到端验证 | 1 天 | Phase 1+2+3 |
| **合计** | | **5 - 7 天** | |

---

## 9. 关键文件清单

### 已有文件（需了解，不修改）

| 文件 | 角色 |
|------|------|
| `easytier/src/instance/instance.rs:295-465` | Core 白名单检测逻辑（30s 轮询 + 事件驱动） |
| `easytier/src/common/global_ctx.rs:60-64, 521-562` | WhitelistEntry 结构 + load/block/unblock 操作 |
| `easytier-web/src/db/entity/ip_whitelist.rs` | SQLite ip_whitelist 表 Entity 定义 |
| `easytier-web/src/restful/admin/ipwhitelist.rs` | Admin 白名单 CRUD API |
| `easytier-web/frontend/src/components/IpWhitelist.vue` | 前端白名单管理页面 |
| `easytier-web/frontend-lib/src/modules/api.ts` | 前端 API Client（需扩展） |
| `entrypoint.sh` | Admin 容器启动脚本（已有 DB→JSON 同步逻辑） |
| `Dockerfile` | 构建镜像（需添加新二进制） |

### 新增文件

| 文件 | 角色 |
|------|------|
| `easytier-web/src/restful/admin/whitelist_export.rs` | Phase 1: 公开 JSON 端点 handler |
| `easytier/src/bin/whitelist_sync.rs` | Phase 2: sync-daemon 源码 |
| `easytier-web/src/db/entity/agent_node.rs` | Phase 3: agent_nodes 表 Entity |
| `easytier-web/src/restful/admin/agent.rs` | Phase 3: agent 管理 API |
| `easytier-web/frontend/src/components/Agents.vue` | Phase 3: agent 管理前端页面 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `easytier-web/src/restful/admin/mod.rs` | Phase 1: 新增 `whitelist_export` 模块+公开路由；Phase 3: 新增 `agent` 模块+路由 |
| `easytier/Cargo.toml` | Phase 2: 新增 `[[bin]]` target + 添加 `reqwest` 依赖 |
| `Dockerfile` | Phase 2: 复制 `easytier-whitelist-sync` 到镜像 |
| `easytier-web/src/db/entity/mod.rs` | Phase 3: 注册 `agent_node` 模块 |
| `easytier-web/src/db/entity/prelude.rs` | Phase 3: 导出 `agent_node` Entity |
| `easytier-web/frontend-lib/src/modules/api.ts` | Phase 3: 新增 agent 相关 API 方法 |
