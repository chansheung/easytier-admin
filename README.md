# EasyTier Admin — 半中心化 VPN 管理面板

## 目录

- [项目背景](#项目背景)
  - [为什么二次开发](#为什么二次开发)
  - [半中心化协作架构](#半中心化协作架构)
    - [为什么仅有 admin 端白名单不够](#为什么仅有-admin-端白名单不够)
    - [角色与职责](#角色与职责)
    - [协作流程图](#协作流程图)
    - [关键设计：预置默认白名单](#关键设计预置默认白名单)
- [核心特性](#核心特性)
- [截图示例](#截图示例)
- [Admin 节点部署](#admin-节点部署)
  - [用途](#用途)
  - [快速开始](#快速开始)
  - [core.toml 示例](#coretoml-示例)
  - [IP 白名单机制](#ip-白名单机制)
  - [设备管理](#设备管理)
  - [环境变量](#环境变量)
- [Agent 节点部署](#agent-节点部署)
  - [用途](#用途-1)
  - [与 admin 的关系](#与-admin-的关系)
  - [虚拟 IP 命名约定（.1 后缀）](#虚拟-ip-命名约定1-后缀)
  - [agent 端 core.toml 示例](#agent-端-coretoml-示例)
  - [白名单同步机制](#白名单同步机制)
  - [启动流程](#启动流程)
  - [环境变量](#环境变量-1)
  - [构建与启动](#构建与启动)
  - [重新生成默认白名单](#重新生成默认白名单)
- [兼容性与版本](#兼容性与版本)
- [构建镜像](#构建镜像)
- [一键脚本](#一键脚本)
- [目录结构](#目录结构)
- [常见问题（FAQ）](#常见问题faq)
- [许可与致谢](#许可与致谢)

## 项目背景

### 为什么二次开发

本项目是 [EasyTier](https://github.com/EasyTier/EasyTier) （v2.4.5）的二次发行版，基于团队实际运维需求进行了深度定制。

**典型场景**：团队内部成员日常通过 VPN 访问内网资源，大部分接入 IP 为内部人员；偶尔会有外来人员需要临时使用团队本地服务器上的服务。

**管理诉求**：
- 需要一个集中化的 admin 白名单系统，控制哪些 IP 能接入 VPN
- 需要对关键本地服务器（设备）进行 agent 管理，确保即使在无 admin 参与的情况下，本地也能阻断未授权接入
- 单纯依赖 admin 端白名单无法覆盖所有安全场景（详见下文）

原始项目 EasyTier 遵循 LGPL-3.0 许可证，本项目为衍生作品。

### 半中心化协作架构

#### 为什么仅有 admin 端白名单不够

假设 admin 搭设在带公网的腾讯云服务器上，作为远程中介节点：

1. admin 可以禁止白名单外 IP 与 admin 自己建立连接
2. **但**当白名单内的其他设备（如内网本地服务器）也拥有公网 IPv4/IPv6 时，白名单外的 IP 可以**绕过 admin，直接连接本地服务器**
3. 这导致仅靠 admin 端白名单无法保护关键的本地设备

**解决方案**：二次开发"半中心化" admin + agent 架构：
- **admin**（中心）：部署在云端，集中管控白名单
- **agent**（半中心）：部署在每个关键本地设备上，从 admin 拉取白名单，本地阻断非法接入
- 即便有人通过公网 IP 直连 agent 设备，agent 也会拒绝非白名单连接

#### 角色与职责

| 角色 | 部署位置 | 职责 |
|------|----------|------|
| admin | 云端公网服务器（如腾讯云） | 集中管理 IP 白名单、设备状态；提供 Web UI；对外公开接入点 |
| agent | 内网关键服务器 × N | 从 admin 拉取白名单；本地执行白名单拦截；运行 easytier-core |
| 客户端 | 内部成员设备 / 外来人员设备 | 通过 easytier 客户端接入 VPN |

#### 协作流程图

```
              公网（Internet）
                     │
       ┌─────────────┼─────────────┐
       │             │             │
   ┌───┴────┐   ┌────┴────┐   ┌────┴────┐
   │ admin  │   │ 外部 IP │   │  ...    │
   │ 云服务器│   │ 白名单外│   │  其他   │
   └────────┘   └─────────┘   └─────────┘
       │             ✗             ✗
       │  （白名单外被 admin 阻断）
       │ ✓
       ├──────────────┐
       │              │
   ┌───┴────┐    ┌────┴────┐
   │ agent  │◄───┤ 内部 IP │
   │ 本地服务│    │ 白名单内│
   └────────┘    └─────────┘
       │  ✓
       │  （agent 本地阻断白名单外）
       └──────────────┐
                      │
                  ┌───┴────┐
                  │ 外部 IP │
                  │ 白名单外│
                  └─────────┘
                     ✗
            （agent 本地阻断）

虚线: 数据流
✓: 允许  ✗: 阻断
```

**关键时序**：

1. admin 启动，监听 `0.0.0.0:11211`
2. 客户端 / 外来设备尝试连接 admin → admin 检查白名单 → 通过 / 拒绝
3. agent 启动 → 预置默认白名单（admin IP 兜底）→ 启动 easytier-core
4. core 与 admin 建立 P2P 连接 → agent 定时从 admin 拉取白名单
5. 客户端尝试直接连接 agent → agent 本地检查白名单 → 通过 / 拒绝

#### 关键设计：预置默认白名单

- agent 第一次启动时，`ip_whitelist.json` 不存在
- 入口脚本自动写入 `[{ip: "<admin虚拟IP>", hostname: null}]` 作为兜底
- 这样 core 不会阻断 admin，P2P 连接能立即建立
- sync-daemon 第一次拉取成功后，会用 admin 真实白名单覆盖默认值

## 核心特性

- **半中心化架构**：admin 集中管控 + agent 本地执行，双重白名单防护
- **单镜像融合**：`easytier-core` + `easytier-web` + 自定义 Admin UI，一个容器搞定一切
- **官方客户端兼容**：实测 `easytier` 官方客户端 `2.3.2` 及 `2.4.5` 均可正常连接与管理
- **IP 白名单机制**：仅允许指定 IP 的客户端建立连接，详见下文
- **设备管理**：实时查看在线设备、系统/版本、心跳时间
- **登录鉴权**：管理员账号 + JWT Token 认证
- **多阶段构建**：Rust 编译 + pnpm 前端构建 + rust_embed 静态资源嵌入

## 截图示例

### IP 白名单管理

![IP 白名单](eg/ip-whitelist.png)

*IP 白名单管理界面：默认首页，可添加 / 删除白名单 IP，支持主机名绑定与解绑*

### 设备管理

![设备管理](eg/devices.png)

*设备列表：展示已通过白名单接入的设备，包含机器 ID、主机名、虚拟 IP 和在线状态*

## Admin 节点部署

### 用途

Admin 节点是半中心化架构的控制中心，部署在公网可达的服务器（如腾讯云）上。它负责：
- 提供管理 Web UI（IP 白名单管理、设备管理）
- 作为 VPN 中介节点，连接各 agent 和客户端
- 对外公开接入点，通过白名单过滤非法连接

### 快速开始

最小启动命令：

```bash
docker run -d --restart=always --privileged \
  --name easytier-admin \
  --network host \
  -v $(pwd)/core.toml:/etc/easytier/core.toml \
  -v $(pwd)/data:/data \
  -e ET_ADMIN_PASSWORD=your-strong-password \
  easytier-admin:2.4.5
```

参数说明：

| 参数 | 说明 |
|------|------|
| `--privileged` | `easytier-core` 需要创建 TUN 设备，必须使用特权模式 |
| `--network host` | VPN 流量直接走主机网络栈 |
| `-v .../core.toml` | `easytier-core` 的配置文件（需提前准备好） |
| `-v .../data` | 持久化数据库和运行时数据 |
| `-e ET_ADMIN_PASSWORD` | Admin 管理员密码，**请务必修改** |
| `-e ET_ADMIN_SECRET` | JWT 签名密钥，建议设置（默认值为 `change-me-to-a-random-string`，不安全） |

### core.toml 示例

`core.toml` 是 easytier-core 的主配置文件，挂载到容器的 `/etc/easytier/core.toml`。最简配置示例：

```toml
ipv4 = "10.0.10.1/24"
listeners = [
    "tcp://0.0.0.0:22020",
    "udp://0.0.0.0:22020",
]

[network_identity]
network_name = "your-network-name"       # ← 占位符，请改为自定义名称
network_secret = "change-me-to-a-random-string"  # ← 占位符，请改为强随机密钥

[flags]
private_mode = true
```

字段说明：

- `ipv4` — 本节点在虚拟网络中的 IP 与子网长度（同子网内的客户端可以直接互通）
- `listeners` — easytier-core 监听的协议与端口，客户端通过这些地址连接
- `network_name` — 虚拟网络名称，同一网络的节点必须一致
- `network_secret` — 虚拟网络密钥（用于加密通信），请改成强随机字符串
- `private_mode` — 私有模式（默认不向外转发非本网络流量）

更多字段（如 `peers` 主动连接的节点、`rpc_portal` 管理 API 端口等）请参考 [easytier 官方文档](https://github.com/EasyTier/easytier)。

启动后访问 Admin UI：`http://<your-server-ip>:11211/admin`

### IP 白名单机制

在公网部署 EasyTier 节点时，任何知道连接地址的客户端都可以尝试接入。IP 白名单用于限制只有被授权 IP 的客户端才能与本地节点建立连接，防止未授权访问。

#### 使用方式

登录 Admin UI 后，侧边栏默认进入「IP 白名单」页面。点击「＋ 添加 IP」按钮，在弹出框中输入目标客户端的 easytier 虚拟 IP 地址和备注（可选），即可将该 IP 加入白名单，建议手动分配客户端 IP。

#### 可选字段

- **备注（comment）**：方便标识该 IP 对应的客户端，如"北京办公室出口"。

#### 主机名绑定

当某 IP 的客户端首次连接到节点时，Admin 界面会自动将该客户端的主机名（hostname）绑定到对应的白名单条目，方便运维人员辨认。如需清除绑定，点击该条目的「解绑」按钮即可。

**行为说明**：

| 场景 | 行为 |
|------|------|
| 设备首次连接，白名单条目无主机名 | 自动绑定设备主机名到白名单条目 |
| 设备在线，admin 解除主机名绑定 | 核心会重新自动绑定（需先断开设备再解绑才能持久） |
| 设备断开，admin 解除主机名绑定 | 解绑成功后主机名保持为空，不会重新绑定 |
| 设备重连（已解绑状态） | 重新自动绑定主机名 |
| 空白名单（无任何条目） | 阻止所有连接 |

#### 修复的 Bug

本次修复解决了以下问题：

1. **空白名单放行所有连接**：当白名单文件为空时，原本会跳过所有拦截检查。现已修复为空白名单=阻止所有连接。

2. **blocked_peers 内存状态不持久化**：被阻止的 peer 列表仅存储在内存中，容器重启后丢失。现已持久化到磁盘文件，重启后保留阻止状态。

3. **白名单文件非原子写入**：`sqlite3 ... > file` 写入时会先截断文件，造成竞态条件。现已改为 `> file.tmp && mv file.tmp file` 原子写入。

4. **自动绑定追踪去重阻止重新绑定**：入口脚本中的追踪文件 `/tmp/auto_bound_pairs.json` 永久记录已处理的 (ip, hostname) 对，导致 admin 解绑后无法重新绑定。现已移除追踪去重逻辑。

5. **自动绑定文件持久化导致离线设备重新绑定**：自动绑定文件 `/tmp/ip_hostname_auto.json` 未及时清理，导致已断开连接的设备被反复绑定。现已改为处理后立即删除文件，由核心根据设备在线状态决定是否重建。

6. **先封禁后跳过连接清理导致存活连接残留**：当事件驱动任务先封禁 peer 后，定时任务因检测到 peer 已 blocked 而跳过关闭连接的操作，导致已建立的 UDP 隧道残留，非白名单设备仍能访问其他节点。现已修复为无论 peer 是否已标记 blocked，始终尝试关闭所有剩余连接。
7. **admin 自身 IP 静默插入失败**：`entrypoint.sh` 的 `INSERT OR IGNORE` 缺少 NOT NULL 字段 `created_by`/`created_at`，导致 admin 自身 IP 无法写入白名单。agent 每 30 秒发现 admin IP 无 hostname，反复触发 `[BIND_DEBUG] auto-bound hostname` 日志。现已补全字段。
8. **close_peer_conn 使用默认连接 ID 导致关闭静默失败**：peer 有多连接时 `default_conn_id` 被后台任务清零为全零，`close_peer_conn(零ID)` 静默跳过。现已改用 `close_peer` 直接移除整个 peer。

#### 删除

通过白名单列表中的「删除」按钮移除条目。被移除的 IP 将无法再建立新的连接。

#### 存储

白名单数据保存在 SQLite 数据库（`/data/easytier-admin.db`）中，随容器 `/data` 卷持久化。容器重启后白名单不会丢失。

#### 工作原理

EasyTier 的 IP 白名单实际生效在**网络连接建立时**，由 `easytier-core` 强制执行：

- `easytier-web`（即管理面板后端）只负责白名单的 CRUD 管理 API
- `entrypoint.sh` 每 30 秒将 SQLite 中的 `ip_whitelist` 表导出为 `/tmp/ip_whitelist.json`
- `easytier-core` 启动时通过 `IP_WHITELIST_FILE` 环境变量加载该 JSON 文件
- 当任意 peer 尝试建立连接时，`easytier-core` 检查对方虚拟 IP 是否在白名单内
- 命中白名单 → 正常握手；未命中 → 拒绝握手并记录日志
- 定时任务（每 30 秒）会检测已连接的非白名单设备并主动断开
- admin 容器启动时 entrypoint.sh 自动从 core.toml 读取自身 IP，INSERT OR IGNORE 写入白名单（附带 hostname、created_by、created_at），确保 admin 自身 IP 始终在白名单中，避免 agent 反复触发 auto-bind

#### 注意事项

- 白名单变更后，定时任务（每 30 秒）会自动检测并断开已建立连接的非白名单设备。
- 白名单为空时，所有连接将被阻止（已修复此前的放行问题）。

### 设备管理

Admin UI 提供设备管理功能，实时展示已通过白名单接入的设备信息：

- 机器 ID、主机名
- 虚拟 IP 地址
- 系统版本、easytier 版本
- 心跳时间、在线状态

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `IP_WHITELIST_FILE` | `/tmp/ip_whitelist.json` | IP 白名单文件路径 |
| `BLOCKED_PEERS_FILE` | `/tmp/blocked_peers.json` | 被阻止 peer 持久化文件路径 |
| `IP_HOSTNAME_AUTO_FILE` | `/tmp/ip_hostname_auto.json` | 主机名自动绑定文件路径 |
| `ET_ADMIN_PASSWORD` | — | Admin 管理员密码（**强烈建议设置**） |
| `ET_ADMIN_SECRET` | `change-me-to-a-random-string` | JWT 签名密钥（建议修改） |



## Agent 节点部署

### 用途

对于"半中心化"部署：在关键内网服务器上部署 agent 容器，由 admin 统一管控 IP 白名单。即使该服务器有公网 IP，也只允许 admin 授权的设备访问。

### 与 admin 的关系

- admin 部署在云端（或公网可达的服务器），运行 `easytier-web`
- agent 部署在内网关键服务器上，运行 `easytier-core` + `easytier-whitelist-sync`
- agent 从 admin 拉取白名单，本地阻断白名单外的设备

### 虚拟 IP 命名约定（.1 后缀）

**admin 必须使用 `.1` 后缀**，agent 使用其他后缀：

| 角色 | 虚拟 IP | 说明 |
|------|---------|------|
| admin | `10.0.10.1/24` | 固定为子网的 `.1` |
| agent #1 | `10.0.10.2/24` | 内网服务器 1 |
| agent #2 | `10.0.10.3/24` | 内网服务器 2 |
| 普通客户端 | `10.0.10.x` | 其他设备 |

agent 启动时会从自己的 `core.toml` 读取 `ipv4` 字段（如 `10.0.10.2/24`），自动取前三段（`10.0.10`）拼上 `.1` → 得到 admin 的默认 IP（`10.0.10.1`），写入默认白名单。

### agent 端 core.toml 示例

```toml
ipv4 = "10.0.10.2/24"
listeners = [
    "tcp://0.0.0.0:22020",
    "udp://0.0.0.0:22020",
]

[network_identity]
network_name = "your-network-name"       # ← 占位符，必须与 admin 端一致
network_secret = "change-me-to-a-random-string"  # ← 占位符，必须与 admin 端一致

[[peer]]
uri = "tcp://<admin公网IP>:22020"

[flags]
private_mode = true
```

字段说明：
- `ipv4`：agent 自己的虚拟 IP，**必须与 admin 同子网**，且不能是 `.1`
- `[[peer]]`：admin 的公网/内网 IP + easytier 端口（默认 22020）
- `network_name` / `network_secret`：必须与 admin 端完全一致

### 白名单同步机制

```
1. 读取 core.toml 的 ipv4 → 自动算出 admin 默认 IP（.1 后缀）
2. 写入 /data/ip_whitelist.json 作为预置白名单
3. 启动 whitelist-sync-daemon（向 admin 拉取真实白名单）
4. 启动 easytier-core
5. core 每 30s 检测白名单 + 阻断非白名单设备
6. sync-daemon 每 30s 拉取 admin 白名单覆盖本地文件
```

**admin 失联时的行为**：
- sync-daemon 检测到 admin 不可达 → **不修改本地白名单文件**
- core 继续使用最后一次同步成功的白名单
- 安全性不降低（不会因为 admin 失联而放行新设备）

### 启动流程

agent 完整启动流程：

1. 入口脚本读取 `core.toml` 的 `ipv4`，自动推算 admin IP（`.1` 后缀）
2. 若 `/data/ip_whitelist.json` 不存在，写入默认白名单（仅含 admin IP）
3. 启动 `whitelist-sync-daemon` 后台进程
4. 启动 `easytier-core`
5. core 建立与 admin 的 P2P 连接
6. sync-daemon 定时从 admin 拉取白名单，覆盖本地文件

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `WHITELIST_SYNC_URL` | （必填）| admin 公开白名单端点，如 `http://10.0.10.1:11211/api/v1/public/whitelist.json` |
| `IP_WHITELIST_FILE` | `/data/ip_whitelist.json` | 本地白名单文件路径 |
| `WHITELIST_SYNC_INTERVAL` | `30` | 拉取间隔（秒）|
| `WHITELIST_DEFAULT_IPS` | 自动检测 | 预置白名单的 IP 列表（逗号分隔），默认从 core.toml 自动算 admin 的 `.1` 后缀 |
| `CORE_CONFIG` | `/etc/easytier/core.toml` | core 配置文件路径 |
| `RUST_LOG` | `easytier=error` | tracing 日志级别，只输出 ERROR；Format 2 日志（`[uuid] message`）不受此影响 |

### 构建与启动

```bash
# 1. 构建 agent 镜像
docker build -t easytier-agent:latest -f agent.Dockerfile .

# 2. 准备 core.toml（见上节示例）

# 3. 启动 agent 容器
docker run -d \
  --name easytier-agent \
  --restart unless-stopped \
  --network host \
  --cap-add NET_ADMIN \
  --device /dev/net/tun \
  -v /opt/easytier-agent/config/core.toml:/etc/easytier/core.toml:ro \
  -v easytier-agent-data:/data \
  -e WHITELIST_SYNC_URL=http://10.0.10.1:11211/api/v1/public/whitelist.json \
  -e WHITELIST_SYNC_INTERVAL=30 \
  easytier-agent:latest

# 4. 验证
docker logs -f easytier-agent
# 应看到：
#   [agent] auto-detected admin IP from core.toml: 10.0.10.1 (agent=10.0.10.2)
#   [agent] initializing default whitelist (kills chicken-and-egg)
#   [agent] default whitelist: [{"ip":"10.0.10.1","hostname":null}]
#   whitelist-sync-daemon starting: ...
#   Whitelist synced: N entries
```

### 重新生成默认白名单

如果需要重新触发默认白名单逻辑（例如网络重命名），删除数据卷：

```bash
docker stop easytier-agent
docker rm easytier-agent
docker volume rm easytier-agent-data
# 然后重新 docker run
```

## 兼容性与版本

本镜像内置的 `easytier-core` 与 `easytier-web` 与官方版本保持一致，仅以下文件有定制修改：

- `easytier-web/admin-frontend/index.html` — 自定义管理 UI（IP 白名单、设备管理、时间本地化）
- `Dockerfile` — 增加了 `tzdata` 和 `ENV TZ=Asia/Shanghai` 配置

实测兼容的官方客户端版本：

| 客户端版本 | 状态 |
|-----------|------|
| 2.3.2     | 正常 |
| 2.4.5     | 正常 |

## 构建镜像

本镜像采用 Docker 多阶段构建：

1. **Builder 阶段**：Rust 1.89 编译 `easytier-core` 和 `easytier-web`；pnpm 编译前端（`frontend-lib` + `frontend`）；`rust_embed` 将 `admin-frontend/index.html` 嵌入二进制
2. **Runtime 阶段**：`debian:bookworm-slim` 基础镜像，安装运行时依赖，复制编译产物

构建命令：

```bash
docker build --platform linux/amd64 -t easytier-admin:2.4.5 .
```

首次构建需要下载 Rust crate 依赖并完整编译，耗时约 10–30 分钟（视网络和机器性能而定）。后续构建会利用 Docker 层缓存，仅重新编译变更部分。

## 一键脚本

仓库根目录提供 `build-and-run.sh`（Linux amd64），支持一键构建并启动容器：

```bash
./build-and-run.sh                                                      # 默认参数
./build-and-run.sh --tag 2.4.5 --password your-password --container my-easytier
```

所有可调参数：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--image` | 镜像名称 | `easytier-admin` |
| `--tag` | 镜像标签 | `2.4.5` |
| `--container` | 容器名称 | `easytier-admin` |
| `--password` | Admin 密码 | `changeme-please` |
| `--web-port` | Admin UI 端口（仅显示用） | `11211` |
| `--vpn-port` | VPN 监听端口（仅显示用） | `22020` |
| `--no-cache` | 不使用 Docker 构建缓存 | 关闭 |

## 目录结构

```
.
├── Dockerfile              # 多阶段构建文件
├── agent.Dockerfile       # agent 节点镜像（半中心化部署）
├── entrypoint.sh           # 容器入口脚本
├── build-and-run.sh        # 一键构建运行脚本
├── README.md               # 本文档
├── .gitignore
├── LICENSE                 # 上游 LGPL-3.0 许可证
├── Cargo.toml              # Rust workspace 根配置
├── Cargo.lock
├── pnpm-workspace.yaml     # pnpm monorepo 配置
├── pnpm-lock.yaml
├── easytier/               # easytier-core 源码
│   ├── Cargo.toml
│   └── src/
├── easytier-rpc-build/     # protobuf RPC 编译辅助
│   ├── Cargo.toml
│   └── src/
├── easytier-web/           # 管理后台源码
│   ├── Cargo.toml
│   ├── src/                # Rust 后端（REST API、IP 白名单、SQLite）
│   ├── admin-frontend/     # 自定义管理 UI（HTML+JS，被 rust_embed 嵌入二进制）
│   ├── frontend/           # 官方 Web 前端（Vue+TS，编译后作为依赖）
│   └── frontend-lib/       # 前端组件库
├── vendor/                 # 离线构建的 vendored Rust 依赖
└── .cargo/                 # Cargo 构建配置
```

## 常见问题（FAQ）

**Q: Admin 默认账号是什么？**
A: 用户名 `admin`，密码由 `ET_ADMIN_PASSWORD` 环境变量设置。首次启动后通过 Admin UI 注册。

**Q: 忘记密码怎么办？**
A: 删除 `/data/easytier-admin.db` 文件后重启容器，系统会重新初始化（**注意：会丢失所有 IP 白名单和设备数据**）。

**Q: 容器内时区是什么？**
A: 已设为 `Asia/Shanghai`，Admin UI 中所有时间显示为北京时间（UTC+8）。

**Q: 如何升级 easytier-core 版本？**
A: 更新 `easytier/Cargo.toml` 中的版本号，然后重新构建镜像。

**Q: 为什么需要 `--privileged`？**
A: `easytier-core` 需要创建 TUN 网络设备，这需要特权模式。

**Q: agent 无法同步白名单怎么办？**
A: 检查 `WHITELIST_SYNC_URL` 是否正确、admin 是否可达。admin 失联时 agent 会使用最后一次同步的白名单，不会放行新设备。

**Q: 日志中反复出现 `[BIND_DEBUG] auto-bound hostname` 是什么意思？**
A: 表示某白名单条目的 hostname 为空，核心自动将当前连接设备的主机名绑定到该 IP。如果 admin 自身 IP（如 10.0.10.1）每 30 秒出现此日志，说明 admin 启动时未成功写入自身白名单条目。请检查容器启动日志中是否有 SQLite 错误。

## 许可与致谢

本项目基于 [EasyTier](https://github.com/EasyTier/EasyTier) 进行二次开发，遵循上游 [LGPL-3.0](LICENSE) 许可证。

感谢 EasyTier 开源社区提供的优秀组网方案。
