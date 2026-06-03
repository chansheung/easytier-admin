# Agent: analyst
# Task: ## 任务：分析 README.md 需要更新哪些内容

### 背景
本次修复了 4 个问题：
1. **admin entrypoint.sh 自动添加自身 IP 到白名单 + hostname 绑定**：启动时从 core.toml 读取 ipv4，INSERT OR IGNORE 写入白名单 + UPDATE 处理旧记录 NULL hostname
2. **agent RUST_LOG 改为 error**：`easytier=warn` → `easytier=error`，消除 Format 1 WARN 日志（~126 处），只保留 ERROR 和 Format 2 日志
3. **instance.rs close_peer 替代 close_peer_conn**：修复多连接场景下 default_conn_id 被清零导致 close 静默失败的 bug
4. **utils.rs console_layer 回滚**：`.without_time()` 改回 `.with_timer(get_logger_timer_rfc3339())`

### 读取文件
1. `/mnt/nvme2t_2/easytier/README.md` — 当前完整内容
2. `/mnt/nvme2t_2/easytier/entrypoint.sh` — 确认 admin auto-add 块
3. `/mnt/nvme2t_2/easytier/easytier/scripts/agent-entrypoint.sh` — 确认 RUST_LOG=error
4. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` — 确认 close_peer 修改

### 分析要点
README 中与以下内容相关的部分需要检查是否需要更新：
- 白名单机制说明（是否需要提到 admin 自动添加自身 IP？）
- agent 日志说明（RUST_LOG 级别）
- 架构说明（是否有新增内容？）
- 部署步骤（是否有变化？）
- 已知问题/故障排查（多连接 bug 是否值得记录？）

### 输出格式
```
## 需要更新的章节
| 章节 | 行号范围 | 需要更新的内容 | 原因 |

## 不需要更新的章节
（简述原因）

## 建议新增的内容
（如果有）
```
# Status: done
# Date: 2026-06-03T12:35:52.328Z

## [user] +058391-06-27T14:19:36.000Z
## Recent context from this session
[2026-06-03T11:28:18.747Z] @analyst: ## 任务：重新排查 agent 间歇性放包的根因

### 背景
agent (10.0.210.253) 对非白名单 client 的 ping 阻断不彻底——约每 7 个包有 1 个能通过。admin (10.0.210.1) 则 100% 阻断。

之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

### 关键事实
1. agent 日志**完全没有** `"peer is blocked by IP whitelist"` 错误 → `is_peer_blocked` 在 `add_new_peer_conn` 中从未返回 true
2. agent 日志**反复出现** `new connection accepted` + `new peer conne
  → I'll continue the root cause investigation by reading the key files identified in the analyst's task.
Let me investigate the critical blocking functions more deeply.
Let me check when `clean_peer_without_conn` is called and how the listener accepts connections:
Now I see the critical issue. Let me verify the default connection ID behavior:
## 根因分析：agent 间歇性放包

### 连接建立流程

```
client → agent listener (WS/TCP/UDP hole punch)
  → add_tunnel_as_server(tunnel, is_directly_connected)
    → add_new_peer_conn(conn)
      → is_peer_blocked(peer_id)? 若 true 则拒绝
      → peers.add_new_peer_conn(conn) → pe

[2026-06-03T11:32:07.280Z] @coder: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

### 根因
peer 有多个连接（WebSocket + UDP hole punch）时，`default_conn_id` 每 5 秒被 `peer.rs` 后台任务清零为全零。`close_peer_conn(零ID)` 静默失败（`NotFound` 错误被 `let _ =` 丢弃），导致已有连接永远不会被关闭，数据包继续通过现有连接流通。

### 修改文件
`/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs`

### 需要修改的 4 处

**修改前**（4 处相同模式）：
```rust
if let Some(conn_id) = pm.get_peer_map().get_peer_defau
  → 现在我将应用所有 4 处修复，将 `close_peer_conn` 替换为 `close_peer`：
现在让我来验证这些修改：
所有 4 处修改已完成。`get_peer_default_conn_id` = 0 次出现，`close_peer` = 4 次调用（行 339, 363, 409, 436）。剩余的 `close_peer_conn` 引用只是 967 行的函数定义。

现在我来验证结构是否有效：
所有 4 处位置均已正确修复。结构干净——`close_peer` 取代了旧的 `if let Some(conn_id) { close_peer_conn }` 模式，且不再有静默失败问题。
网络错误（非编译错误）。正在重试：
由于网络 SSL 问题，编译无法运行（这不是代码错误）。让我通过检查目标方法是否存在来验证 `close_peer` 方法是否存在于 API 中：
`close_peer` 方法在 `peer_map.rs:296` 处存在。所有 4 处修改都正确：

| 位置 | 行 | 任务 |
|----------|------|------|
| 周期性, hostname mismatch | 339 | `close_peer` |
| 周期性, not in whitelist | 363 | `close_peer` |
| 事件驱动, h

[2026-06-03T12:00:22.483Z] @analyst: ## 任务：分析为什么 `[BIND_DEBUG] auto-bound hostname '64c' to IP 10.0.10.1` 每 30 秒重复出现

### 背景
agent 的 periodic task 每次运行都看到 admin (10.0.10.1) 的白名单条目 hostname 为空，触发 auto-bind `[BIND_DEBUG]`。但 admin 的 entrypoint.sh 应该在启动时 `INSERT OR IGNORE` 写入 admin 自身 IP + hostname。

### 需要分析

1. **读取 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 行 310-370**（periodic task），找到 `[BIND_DEBUG]` 日志对应的代码逻辑，理解什么条件下
  → Let me check the `load_ip_whitelist` and `bind_hostname` implementations:
Now let me check the agent's entrypoint to understand how the whitelist JSON is populated on the agent:
Now I see the agent initializes whitelist with `hostname: null`. Let me check the whitelist-sync-daemon:
Let me check what the admin's whitelist API endpoint returns:
Now I have the complete picture. Here is the analysis:

---

## auto-bind 触发条件

**instance.rs:322-355**（periodic task）:
1. 每 30s 读取 whitelist JSON → 找 `WhitelistEntry { ip, hostname }`
2. 管理员的 IP (10.0.10.1) 在 whitelist 中（IP 匹配）
3. `match &entry.hostname`

[2026-06-03T12:05:02.750Z] @analyst: 用户已重建 admin 容器使用最新镜像，但 `[BIND_DEBUG] auto-bound hostname '64c' to IP 10.0.10.1` 仍然每 30 秒重复出现。

请分析以下内容：

1. **读取 `/mnt/nvme2t_2/easytier/entrypoint.sh`** 全文，确认 INSERT OR IGNORE 的逻辑是否正确
2. **读取 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 行 310-370**（periodic task），找到 `[BIND_DEBUG]` auto-bind 的触发条件
3. **分析两个可能性**：
   - 可能 A：admin entrypoint.sh 的 INSERT OR IGNORE 在 easytier-admin 启动之前执
  → Let me check if `easytier-admin` creates the `ip_whitelist` table or adds entries to it.
Let me check the admin container's core.toml config:
现在我们已经有了完整的图景。`agent.Dockerfile` 构建 `easytier-whitelist-sync`，`agent-entrypoint.sh` 同时运行同步守护进程和 `easytier-core`。关键的发现是**第 38 行**，agent 的默认白名单使用了 `"hostname":null`。

但完整的故事涉及两个容器。让我看看 `entrypoint.sh` 的 INSERT 是否真的能工作。

## 根因分析

### BUG #1：entrypoint.sh 第 31 行的 INSERT OR IGNORE 缺少 `created_by` 和 `created_at`

`ip_whitelist` 表模式（来自 `m20250530_000004`）：
```
id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
ip          TEXT NOT NULL
comment     TEXT NULL

[2026-06-03T12:05:49.614Z] @coder: ## 任务：修复 entrypoint.sh 的 INSERT OR IGNORE 缺少 NOT NULL 字段

### 根因
`ip_whitelist` 表的 `created_by` 和 `created_at` 是 `NOT NULL` 字段，但 entrypoint.sh 第 31 行的 INSERT 只提供了 `ip` 和 `hostname`，缺少这两个字段。导致 `INSERT OR IGNORE` 因 NOT NULL 约束失败，`2>/dev/null || true` 静默吞掉了错误。

### 修改文件
`/mnt/nvme2t_2/easytier/entrypoint.sh`

### 修改内容
第 31 行的 INSERT 语句：

**修改前**：
```bash
sqlite3 "$DB_PATH" \
    "INSERT OR IGNORE INT
  → 编辑把文件结构弄乱了——丢失了 `while` 循环块，并且出现了重复行。让我重新正确地编写整个文件。
[PROGRESS: Rewrote entrypoint.sh with INSERT including comment/created_by/created_at + UPDATE for NULL hostname fix]

## 任务：分析 README.md 需要更新哪些内容

### 背景
本次修复了 4 个问题：
1. **admin entrypoint.sh 自动添加自身 IP 到白名单 + hostname 绑定**：启动时从 core.toml 读取 ipv4，INSERT OR IGNORE 写入白名单 + UPDATE 处理旧记录 NULL hostname
2. **agent RUST_LOG 改为 error**：`easytier=warn` → `easytier=error`，消除 Format 1 WARN 日志（~126 处），只保留 ERROR 和 Format 2 日志
3. **instance.rs close_peer 替代 close_peer_conn**：修复多连接场景下 default_conn_id 被清零导致 close 静默失败的 bug
4. **utils.rs console_layer 回滚**：`.without_time()` 改回 `.with_timer(get_logger_timer_rfc3339())`

### 读取文件
1. `/mnt/nvme2t_2/easytier/README.md` — 当前完整内容
2. `/mnt/nvme2t_2/easytier/entrypoint.sh` — 确认 admin auto-add 块
3. `/mnt/nvme2t_2/easytier/easytier/scripts/agent-entrypoint.sh` — 确认 RUST_LOG=error
4. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` — 确认 close_peer 修改

### 分析要点
README 中与以下内容相关的部分需要检查是否需要更新：
- 白名单机制说明（是否需要提到 admin 自动添加自身 IP？）
- agent 日志说明（RUST_LOG 级别）
- 架构说明（是否有新增内容？）
- 部署步骤（是否有变化？）
- 已知问题/故障排查（多连接 bug 是否值得记录？）

### 输出格式
```
## 需要更新的章节
| 章节 | 行号范围 | 需要更新的内容 | 原因 |

## 不需要更新的章节
（简述原因）

## 建议新增的内容
（如果有）
```

[PROGRESS] After each sub-step, output: [PROGRESS: what you just did]. Keep it under 120 chars.

## [assistant] +058391-06-27T14:19:55.000Z
[thinking] The user seems to be asking me to continue work on a project. Let me look at the context more carefully. It appears they were in the middle of a multi-step investigation and fix for an "easytier" project involving:

1. A bug where an agent intermittently lets packets through
2. Fixes to `close_peer` vs `close_peer_conn`
3. Fixes to entrypoint.sh for SQLite INSERT
4. RUST_LOG changes
5. README.md updates

Let me check what the current state is - they seem to have done a lot of work already. Let me see if there's a specific question or task I need to address now.

The last message from @analyst was about analyzing README.md updates needed. Let me check what files exist and what the current state is.

Actually, looking at this more carefully, it seems like I'm being presented with a history of conversation from prior sessions with @analyst and @coder agents. The last thing mentioned is a task to analyze README.md updates.

Let me check the current state of the files to understand what needs to be done.
--- read({"filePath":"/mnt/nvme2t_2/easytier"}) [completed]
<path>/mnt/nvme2t_2/easytier</path>
<type>directory</type>
<entries>
.cargo/
.flowcraft/
.git/
.gitignore
agent.Dockerfile
build-and-run.sh
Cargo.lock
Cargo.toml
core.toml
data/
docker-compose.agent.yml
Dockerfile
docs/
easytier-rpc-build/
easytier-web/
easytier/
eg/
entrypoint.sh
ip_whitelist.json/
LICENSE
pnpm-lock.yaml
pnpm-workspace.yaml
README.md
target/
vendor/

(25 entries)
</entries>
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/README.md"}) [completed]
1#ZZjfrQ|# EasyTier Admin — 半中心化 VPN 管理面板
2#47DEQp|
3#5-0Y3r|## 目录
4#47DEQp|
5#1JdMU3|- [项目背景](#项目背景)
6#unBgNt|  - [为什么二次开发](#为什么二次开发)
7#vf83Lc|  - [半中心化协作架构](#半中心化协作架构)
8#efZ2e6|    - [为什么仅有 admin 端白名单不够](#为什么仅有-admin-端白名单不够)
9#vZqAg6|    - [角色与职责](#角色与职责)
10#AdI7SY|    - [协作流程图](#协作流程图)
11#L9_p7i|    - [关键设计：预置默认白名单](#关键设计预置默认白名单)
12#li2yTk|- [核心特性](#核心特性)
13#ZYHliB|- [截图示例](#截图示例)
14#bA7C9D|- [Admin 节点部署](#admin-节点部署)
15#FwY7CX|  - [用途](#用途)
16#ECs4-d|  - [快速开始](#快速开始)
17#CIkLmw|  - [core.toml 示例](#coretoml-示例)
18#A6OTTL|  - [IP 白名单机制](#ip-白名单机制)
19#f267Y5|  - [设备管理](#设备管理)
20#VBerWo|  - [环境变量](#环境变量)
21#wISEfY|- [Agent 节点部署](#agent-节点部署)
22#3l2dQf|  - [用途](#用途-1)
23#JJJaiS|  - [与 admin 的关系](#与-admin-的关系)
24#2oGdtM|  - [虚拟 IP 命名约定（.1 后缀）](#虚拟-ip-命名约定1-后缀)
25#EWFmiC|  - [agent 端 core.toml 示例](#agent-端-coretoml-示例)
26#_ZrvD4|  - [白名单同步机制](#白名单同步机制)
27#WZOBmQ|  - [启动流程](#启动流程)
28#0W1yY1|  - [环境变量](#环境变量-1)
29#gFr6Gh|  - [构建与启动](#构建与启动)
30#UhL-ph|  - [重新生成默认白名单](#重新生成默认白名单)
31#oe17fS|- [兼容性与版本](#兼容性与版本)
32#tMbAQh|- [构建镜像](#构建镜像)
33#oepkXh|- [一键脚本](#一键脚本)
34#oKKWE3|- [目录结构](#目录结构)
35#87OG_G|- [常见问题（FAQ）](#常见问题faq)
36#TgQ2Yw|- [许可与致谢](#许可与致谢)
37#47DEQp|
38#xMpY_Y|## 项目背景
39#47DEQp|
40#_kTRak|### 为什么二次开发
41#47DEQp|
42#ga5iE9|本项目是 [EasyTier](https://github.com/EasyTier/EasyTier) （v2.4.5）的二次发行版，基于团队实际运维需求进行了深度定制。
43#47DEQp|
44#hVje6t|**典型场景**：团队内部成员日常通过 VPN 访问内网资源，大部分接入 IP 为内部人员；偶尔会有外来人员需要临时使用团队本地服务器上的服务。
45#47DEQp|
46#Pi2SEt|**管理诉求**：
47#xUZhsO|- 需要一个集中化的 admin 白名单系统，控制哪些 IP 能接入 VPN
48#e9TZr6|- 需要对关键本地服务器（设备）进行 agent 管理，确保即使在无 admin 参与的情况下，本地也能阻断未授权接入
49#PDPAO2|- 单纯依赖 admin 端白名单无法覆盖所有安全场景（详见下文）
50#47DEQp|
51#KcmDbQ|原始项目 EasyTier 遵循 LGPL-3.0 许可证，本项目为衍生作品。
52#47DEQp|
53#vSzyNV|### 半中心化协作架构
54#47DEQp|
55#KOUPa8|#### 为什么仅有 admin 端白名单不够
56#47DEQp|
57#W4UXk2|假设 admin 搭设在带公网的腾讯云服务器上，作为远程中介节点：
58#47DEQp|
59#kgf2-e|1. admin 可以禁止白名单外 IP 与 admin 自己建立连接
60#FHsC_w|2. **但**当白名单内的其他设备（如内网本地服务器）也拥有公网 IPv4/IPv6 时，白名单外的 IP 可以**绕过 admin，直接连接本地服务器**
61#rmaiW6|3. 这导致仅靠 admin 端白名单无法保护关键的本地设备
62#47DEQp|
63#I6F5Lr|**解决方案**：二次开发"半中心化" admin + agent 架构：
64#3P5ytT|- **admin**（中心）：部署在云端，集中管控白名单
65#1Av3NU|- **agent**（半中心）：部署在每个关键本地设备上，从 admin 拉取白名单，本地阻断非法接入
66#KG4AoJ|- 即便有人通过公网 IP 直连 agent 设备，agent 也会拒绝非白名单连接
67#47DEQp|
68#DosgeQ|#### 角色与职责
69#47DEQp|
70#yM60mQ|| 角色 | 部署位置 | 职责 |
71#kJ7itv||------|----------|------|
72#yLa1lG|| admin | 云端公网服务器（如腾讯云） | 集中管理 IP 白名单、设备状态；提供 Web UI；对外公开接入点 |
73#PWTwO2|| agent | 内网关键服务器 × N | 从 admin 拉取白名单；本地执行白名单拦截；运行 easytier-core |
74#0T3Gz-|| 客户端 | 内部成员设备 / 外来人员设备 | 通过 easytier 客户端接入 VPN |
75#47DEQp|
76#izpmFB|#### 协作流程图
77#47DEQp|
78#8bkBhH|```
79#h3FWpv|              公网（Internet）
80#N880xj|                     │
81#kB5_Nm|       ┌─────────────┼─────────────┐
82#IqFZDD|       │             │             │
83#lLiNwT|   ┌───┴────┐   ┌────┴────┐   ┌────┴────┐
84#pMVsI-|   │ admin  │   │ 外部 IP │   │  ...    │
85#O4y0qN|   │ 云服务器│   │ 白名单外│   │  其他   │
86#Az-uxy|   └────────┘   └─────────┘   └─────────┘
87#e6rdx3|       │             ✗             ✗
88#65W2Lh|       │  （白名单外被 admin 阻断）
89#Nnwgp6|       │ ✓
90#plXQC4|       ├──────────────┐
91#Ol0JxZ|       │              │
92#ssr8E_|   ┌───┴────┐    ┌────┴────┐
93#JY8tgA|   │ agent  │◄───┤ 内部 IP │
94#a5LkoI|   │ 本地服务│    │ 白名单内│
95#Na31fI|   └────────┘    └─────────┘
96#FcI73b|       │  ✓
97#8E2yYr|       │  （agent 本地阻断白名单外）
98#owv5h8|       └──────────────┐
99#rKeIR-|                      │
100#WY__Q8|                  ┌───┴────┐
101#-s7Ncj|                  │ 外部 IP │
102#--q0ng|                  │ 白名单外│
103#5ZiZnS|                  └─────────┘
104#stFkTz|                     ✗
105#8P7DN8|            （agent 本地阻断）
106#47DEQp|
107#jVfgDr|虚线: 数据流
108#XgW0lG|✓: 允许  ✗: 阻断
109#8bkBhH|```
110#47DEQp|
111#SfjfWV|**关键时序**：
112#47DEQp|
113#xb11BA|1. admin 启动，监听 `0.0.0.0:11211`
114#b6yA_h|2. 客户端 / 外来设备尝试连接 admin → admin 检查白名单 → 通过 / 拒绝
115#y0605K|3. agent 启动 → 预置默认白名单（admin IP 兜底）→ 启动 easytier-core
116#7WpMeV|4. core 与 admin 建立 P2P 连接 → agent 定时从 admin 拉取白名单
117#0vWWHF|5. 客户端尝试直接连接 agent → agent 本地检查白名单 → 通过 / 拒绝
118#47DEQp|
119#LeIK_A|#### 关键设计：预置默认白名单
120#47DEQp|
121#oXXxdS|- agent 第一次启动时，`ip_whitelist.json` 不存在
122#BOi4Cr|- 入口脚本自动写入 `[{ip: "<admin虚拟IP>", hostname: null}]` 作为兜底
123#H2u0yD|- 这样 core 不会阻断 admin，P2P 连接能立即建立
124#RsikcZ|- sync-daemon 第一次拉取成功后，会用 admin 真实白名单覆盖默认值
125#47DEQp|
126#wbISpy|## 核心特性
127#47DEQp|
128#fQHYiJ|- **半中心化架构**：admin 集中管控 + agent 本地执行，双重白名单防护
129#VDbp_m|- **单镜像融合**：`easytier-core` + `easytier-web` + 自定义 Admin UI，一个容器搞定一切
130#D6Cd0A|- **官方客户端兼容**：实测 `easytier` 官方客户端 `2.3.2` 及 `2.4.5` 均可正常连接与管理
131#AiM6q2|- **IP 白名单机制**：仅允许指定 IP 的客户端建立连接，详见下文
132#YsZvtU|- **设备管理**：实时查看在线设备、系统/版本、心跳时间
133#agdldl|- **登录鉴权**：管理员账号 + JWT Token 认证
134#Zs97tr|- **多阶段构建**：Rust 编译 + pnpm 前端构建 + rust_embed 静态资源嵌入
135#47DEQp|
136#x76YqT|## 截图示例
137#47DEQp|
138#J14vqJ|### IP 白名单管理
139#47DEQp|
140#ZxljEg|![IP 白名单](eg/ip-whitelist.png)
141#47DEQp|
142#1faKpA|*IP 白名单管理界面：默认首页，可添加 / 删除白名单 IP，支持主机名绑定与解绑*
143#47DEQp|
144#XugOa0|### 设备管理
145#47DEQp|
146#kJ1Zd2|![设备管理](eg/devices.png)
147#47DEQp|
148#kzUMtK|*设备列表：展示已通过白名单接入的设备，包含机器 ID、主机名、虚拟 IP 和在线状态*
149#47DEQp|
150#UzLqj-|## Admin 节点部署
151#47DEQp|
152#s71GML|### 用途
153#47DEQp|
154#q3opSy|Admin 节点是半中心化架构的控制中心，部署在公网可达的服务器（如腾讯云）上。它负责：
155#SN9Et1|- 提供管理 Web UI（IP 白名单管理、设备管理）
156#6p04H_|- 作为 VPN 中介节点，连接各 agent 和客户端
157#dCqtAd|- 对外公开接入点，通过白名单过滤非法连接
158#47DEQp|
159#wtDI9_|### 快速开始
160#47DEQp|
161#F8bkga|最小启动命令：
162#47DEQp|
163#rTMjNc|```bash
164#0UsXWC|docker run -d --restart=always --privileged \
165#x8eXCM|  --name easytier-admin \
166#50fDoD|  --network host \
167#1Dqlpp|  -v $(pwd)/core.toml:/etc/easytier/core.toml \
168#3rMnZb|  -v $(pwd)/data:/data \
169#y-Yq29|  -e ET_ADMIN_PASSWORD=your-strong-password \
170#B2RZga|  easytier-admin:2.4.5
171#8bkBhH|```
172#47DEQp|
173#RU56x3|参数说明：
174#47DEQp|
175#1aK0CQ|| 参数 | 说明 |
176#Q1UY1c||------|------|
177#2FMX0_|| `--privileged` | `easytier-core` 需要创建 TUN 设备，必须使用特权模式 |
178#uDeyWG|| `--network host` | VPN 流量直接走主机网络栈 |
179#WvurlM|| `-v .../core.toml` | `easytier-core` 的配置文件（需提前准备好） |
180#dKpB3C|| `-v .../data` | 持久化数据库和运行时数据 |
181#I-50Fg|| `-e ET_ADMIN_PASSWORD` | Admin 管理员密码，**请务必修改** |
182#M2s10m|| `-e ET_ADMIN_SECRET` | JWT 签名密钥，建议设置（默认值为 `change-me-to-a-random-string`，不安全） |
183#47DEQp|
184#VH-Qlf|### core.toml 示例
185#47DEQp|
186#IFlRSq|`core.toml` 是 easytier-core 的主配置文件，挂载到容器的 `/etc/easytier/core.toml`。最简配置示例：
187#47DEQp|
188#E3uXOp|```toml
189#KZmrVz|ipv4 = "10.0.10.1/24"
190#BuulPS|listeners = [
191#_uyvOZ|    "tcp://0.0.0.0:22020",
192#g0U3Mz|    "udp://0.0.0.0:22020",
193#z64NQk|]
194#47DEQp|
195#3gQyNR|[network_identity]
196#ahTX2O|network_name = "your-network-name"       # ← 占位符，请改为自定义名称
197#wLC4yt|network_secret = "change-me-to-a-random-string"  # ← 占位符，请改为强随机密钥
198#47DEQp|
199#wCb8fa|[flags]
200#Thaouz|private_mode = true
201#8bkBhH|```
202#47DEQp|
203#UxzPdW|字段说明：
204#47DEQp|
205#HyP7-Z|- `ipv4` — 本节点在虚拟网络中的 IP 与子网长度（同子网内的客户端可以直接互通）
206#fq0kO5|- `listeners` — easytier-core 监听的协议与端口，客户端通过这些地址连接
207#JwdfvR|- `network_name` — 虚拟网络名称，同一网络的节点必须一致
208#jXX2qY|- `network_secret` — 虚拟网络密钥（用于加密通信），请改成强随机字符串
209#pOOwDK|- `private_mode` — 私有模式（默认不向外转发非本网络流量）
210#47DEQp|
211#seEGai|更多字段（如 `peers` 主动连接的节点、`rpc_portal` 管理 API 端口等）请参考 [easytier 官方文档](https://github.com/EasyTier/easytier)。
212#47DEQp|
213#Z6qkBH|启动后访问 Admin UI：`http://<your-server-ip>:11211/admin`
214#47DEQp|
215#lehMfz|### IP 白名单机制
216#47DEQp|
217#yxgaoe|在公网部署 EasyTier 节点时，任何知道连接地址的客户端都可以尝试接入。IP 白名单用于限制只有被授权 IP 的客户端才能与本地节点建立连接，防止未授权访问。
218#47DEQp|
219#L24zVV|#### 使用方式
220#47DEQp|
221#_DG4ix|登录 Admin UI 后，侧边栏默认进入「IP 白名单」页面。点击「＋ 添加 IP」按钮，在弹出框中输入目标客户端的 easytier 虚拟 IP 地址和备注（可选），即可将该 IP 加入白名单，建议手动分配客户端 IP。
222#47DEQp|
223#rZ527U|#### 可选字段
224#47DEQp|
225#OjzVdF|- **备注（comment）**：方便标识该 IP 对应的客户端，如"北京办公室出口"。
226#47DEQp|
227#JLGE3y|#### 主机名绑定
228#47DEQp|
229#GSoWhR|当某 IP 的客户端首次连接到节点时，Admin 界面会自动将该客户端的主机名（hostname）绑定到对应的白名单条目，方便运维人员辨认。如需清除绑定，点击该条目的「解绑」按钮即可。
230#47DEQp|
231#2fqg4Q|**行为说明**：
232#47DEQp|
233#3a8KNt|| 场景 | 行为 |
234#Q1UY1c||------|------|
235#cMyLRt|| 设备首次连接，白名单条目无主机名 | 自动绑定设备主机名到白名单条目 |
236#voG2yD|| 设备在线，admin 解除主机名绑定 | 核心会重新自动绑定（需先断开设备再解绑才能持久） |
237#bd66HW|| 设备断开，admin 解除主机名绑定 | 解绑成功后主机名保持为空，不会重新绑定 |
238#P0RlxG|| 设备重连（已解绑状态） | 重新自动绑定主机名 |
239#sYmIKz|| 空白名单（无任何条目） | 阻止所有连接 |
240#47DEQp|
241#LdDasj|#### 修复的 Bug
242#47DEQp|
243#wXiAEg|本次修复解决了以下问题：
244#47DEQp|
245#gLXwMw|1. **空白名单放行所有连接**：当白名单文件为空时，原本会跳过所有拦截检查。现已修复为空白名单=阻止所有连接。
246#47DEQp|
247#UcS6SX|2. **blocked_peers 内存状态不持久化**：被阻止的 peer 列表仅存储在内存中，容器重启后丢失。现已持久化到磁盘文件，重启后保留阻止状态。
248#47DEQp|
249#4sSDZ5|3. **白名单文件非原子写入**：`sqlite3 ... > file` 写入时会先截断文件，造成竞态条件。现已改为 `> file.tmp && mv file.tmp file` 原子写入。
250#47DEQp|
251#8CICw9|4. **自动绑定追踪去重阻止重新绑定**：入口脚本中的追踪文件 `/tmp/auto_bound_pairs.json` 永久记录已处理的 (ip, hostname) 对，导致 admin 解绑后无法重新绑定。现已移除追踪去重逻辑。
252#47DEQp|
253#7gXCGp|5. **自动绑定文件持久化导致离线设备重新绑定**：自动绑定文件 `/tmp/ip_hostname_auto.json` 未及时清理，导致已断开连接的设备被反复绑定。现已改为处理后立即删除文件，由核心根据设备在线状态决定是否重建。
254#47DEQp|
255#OVydSe|6. **先封禁后跳过连接清理导致存活连接残留**：当事件驱动任务先封禁 peer 后，定时任务因检测到 peer 已 blocked 而跳过关闭连接的操作，导致已建立的 UDP 隧道残留，非白名单设备仍能访问其他节点。现已修复为无论 peer 是否已标记 blocked，始终尝试关闭所有剩余连接。
256#47DEQp|
257#l0tY6B|#### 删除
258#47DEQp|
259#OQdXoY|通过白名单列表中的「删除」按钮移除条目。被移除的 IP 将无法再建立新的连接。
260#47DEQp|
261#IzOnL4|#### 存储
262#47DEQp|
263#LT1Bh2|白名单数据保存在 SQLite 数据库（`/data/easytier-admin.db`）中，随容器 `/data` 卷持久化。容器重启后白名单不会丢失。
264#47DEQp|
265#-T2Blm|#### 工作原理
266#47DEQp|
267#xTEE4G|EasyTier 的 IP 白名单实际生效在**网络连接建立时**，由 `easytier-core` 强制执行：
268#47DEQp|
269#ygqITJ|- `easytier-web`（即管理面板后端）只负责白名单的 CRUD 管理 API
270#qD6R03|- `entrypoint.sh` 每 30 秒将 SQLite 中的 `ip_whitelist` 表导出为 `/tmp/ip_whitelist.json`
271#Zw_0RS|- `easytier-core` 启动时通过 `IP_WHITELIST_FILE` 环境变量加载该 JSON 文件
272#tzT43x|- 当任意 peer 尝试建立连接时，`easytier-core` 检查对方虚拟 IP 是否在白名单内
273#5Yf_nG|- 命中白名单 → 正常握手；未命中 → 拒绝握手并记录日志
274#930rag|- 定时任务（每 30 秒）会检测已连接的非白名单设备并主动断开
275#47DEQp|
276#g7a1R4|#### 注意事项
277#47DEQp|
278#MW9ROy|- 白名单变更后，定时任务（每 30 秒）会自动检测并断开已建立连接的非白名单设备。
279#3YvSN-|- 白名单为空时，所有连接将被阻止（已修复此前的放行问题）。
280#47DEQp|
281#XugOa0|### 设备管理
282#47DEQp|
283#UQ1M1K|Admin UI 提供设备管理功能，实时展示已通过白名单接入的设备信息：
284#47DEQp|
285#Sc-GBd|- 机器 ID、主机名
286#3RDozC|- 虚拟 IP 地址
287#wtkjaR|- 系统版本、easytier 版本
288#qux512|- 心跳时间、在线状态
289#47DEQp|
290#ReTUvI|### 环境变量
291#47DEQp|
292#58-cI-|| 变量 | 默认值 | 说明 |
293#28EPHU||------|--------|------|
294#rWKlEu|| `IP_WHITELIST_FILE` | `/tmp/ip_whitelist.json` | IP 白名单文件路径 |
295#KY1NoW|| `BLOCKED_PEERS_FILE` | `/tmp/blocked_peers.json` | 被阻止 peer 持久化文件路径 |
296#-tRY5-|| `IP_HOSTNAME_AUTO_FILE` | `/tmp/ip_hostname_auto.json` | 主机名自动绑定文件路径 |
297#bdIlId|| `ET_ADMIN_PASSWORD` | — | Admin 管理员密码（**强烈建议设置**） |
298#zn2R2C|| `ET_ADMIN_SECRET` | `change-me-to-a-random-string` | JWT 签名密钥（建议修改） |
299#47DEQp|
300#47DEQp|
301#47DEQp|
302#ciax_W|## Agent 节点部署
303#47DEQp|
304#s71GML|### 用途
305#47DEQp|
306#olkAxI|对于"半中心化"部署：在关键内网服务器上部署 agent 容器，由 admin 统一管控 IP 白名单。即使该服务器有公网 IP，也只允许 admin 授权的设备访问。
307#47DEQp|
308#3KL90K|### 与 admin 的关系
309#47DEQp|
310#XAAN7P|- admin 部署在云端（或公网可达的服务器），运行 `easytier-web`
311#7G0VXx|- agent 部署在内网关键服务器上，运行 `easytier-core` + `easytier-whitelist-sync`
312#oYFiQU|- agent 从 admin 拉取白名单，本地阻断白名单外的设备
313#47DEQp|
314#xCLYxo|### 虚拟 IP 命名约定（.1 后缀）
315#47DEQp|
316#uCz4bV|**admin 必须使用 `.1` 后缀**，agent 使用其他后缀：
317#47DEQp|
318#noiE5l|| 角色 | 虚拟 IP | 说明 |
319#x8pw5q||------|---------|------|
320#0q2BCP|| admin | `10.0.10.1/24` | 固定为子网的 `.1` |
321#3cGf87|| agent #1 | `10.0.10.2/24` | 内网服务器 1 |
322#tK-m3v|| agent #2 | `10.0.10.3/24` | 内网服务器 2 |
323#v0bALt|| 普通客户端 | `10.0.10.x` | 其他设备 |
324#47DEQp|
325#m4qPoj|agent 启动时会从自己的 `core.toml` 读取 `ipv4` 字段（如 `10.0.10.2/24`），自动取前三段（`10.0.10`）拼上 `.1` → 得到 admin 的默认 IP（`10.0.10.1`），写入默认白名单。
326#47DEQp|
327#xvnD-c|### agent 端 core.toml 示例
328#47DEQp|
329#E3uXOp|```toml
330#Gxm1E5|ipv4 = "10.0.10.2/24"
331#BuulPS|listeners = [
332#_uyvOZ|    "tcp://0.0.0.0:22020",
333#g0U3Mz|    "udp://0.0.0.0:22020",
334#z64NQk|]
335#47DEQp|
336#3gQyNR|[network_identity]
337#g-rX_3|network_name = "your-network-name"       # ← 占位符，必须与 admin 端一致
338#TeQYSt|network_secret = "change-me-to-a-random-string"  # ← 占位符，必须与 admin 端一致
339#47DEQp|
340#kaOTDv|[[peer]]
341#eRr1eX|uri = "tcp://<admin公网IP>:22020"
342#47DEQp|
343#wCb8fa|[flags]
344#Thaouz|private_mode = true
345#8bkBhH|```
346#47DEQp|
347#UxzPdW|字段说明：
348#4BQaQH|- `ipv4`：agent 自己的虚拟 IP，**必须与 admin 同子网**，且不能是 `.1`
349#KTn0CD|- `[[peer]]`：admin 的公网/内网 IP + easytier 端口（默认 22020）
350#M33gDp|- `network_name` / `network_secret`：必须与 admin 端完全一致
351#47DEQp|
352#KKq652|### 白名单同步机制
353#47DEQp|
354#8bkBhH|```
355#Hhr98J|1. 读取 core.toml 的 ipv4 → 自动算出 admin 默认 IP（.1 后缀）
356#Cc1lXn|2. 写入 /data/ip_whitelist.json 作为预置白名单
357#DA5637|3. 启动 whitelist-sync-daemon（向 admin 拉取真实白名单）
358#RF-9VI|4. 启动 easytier-core
359#RdEolD|5. core 每 30s 检测白名单 + 阻断非白名单设备
360#FGAMEA|6. sync-daemon 每 30s 拉取 admin 白名单覆盖本地文件
361#8bkBhH|```
362#47DEQp|
363#JVs39y|**admin 失联时的行为**：
364#SiFNsb|- sync-daemon 检测到 admin 不可达 → **不修改本地白名单文件**
365#a8l-A9|- core 继续使用最后一次同步成功的白名单
366#RjDKmP|- 安全性不降低（不会因为 admin 失联而放行新设备）
367#47DEQp|
368#Mr6GVj|### 启动流程
369#47DEQp|
370#7eGG6C|agent 完整启动流程：
371#47DEQp|
372#vD8OC1|1. 入口脚本读取 `core.toml` 的 `ipv4`，自动推算 admin IP（`.1` 后缀）
373#OsCiCQ|2. 若 `/data/ip_whitelist.json` 不存在，写入默认白名单（仅含 admin IP）
374#4LIRIh|3. 启动 `whitelist-sync-daemon` 后台进程
375#Sk2mUC|4. 启动 `easytier-core`
376#fxk7U_|5. core 建立与 admin 的 P2P 连接
377#uiLp9f|6. sync-daemon 定时从 admin 拉取白名单，覆盖本地文件
378#47DEQp|
379#ReTUvI|### 环境变量
380#47DEQp|
381#58-cI-|| 变量 | 默认值 | 说明 |
382#28EPHU||------|--------|------|
383#BOFPVU|| `WHITELIST_SYNC_URL` | （必填）| admin 公开白名单端点，如 `http://10.0.10.1:11211/api/v1/public/whitelist.json` |
384#LXU9Ie|| `IP_WHITELIST_FILE` | `/data/ip_whitelist.json` | 本地白名单文件路径 |
385#KeKoxG|| `WHITELIST_SYNC_INTERVAL` | `30` | 拉取间隔（秒）|
386#68EtGO|| `WHITELIST_DEFAULT_IPS` | 自动检测 | 预置白名单的 IP 列表（逗号分隔），默认从 core.toml 自动算 admin 的 `.1` 后缀 |
387#zIkF7S|| `CORE_CONFIG` | `/etc/easytier/core.toml` | core 配置文件路径 |
388#47DEQp|
389#60-Pds|### 构建与启动
390#47DEQp|
391#rTMjNc|```bash
392#bgWP1J|# 1. 构建 agent 镜像
393#jw8pC-|docker build -t easytier-agent:latest -f agent.Dockerfile .
394#47DEQp|
395#HWMUH-|# 2. 准备 core.toml（见上节示例）
396#47DEQp|
397#-ip2ij|# 3. 启动 agent 容器
398#ojlfEE|docker run -d \
399#UKuRsr|  --name easytier-agent \
400#B-kZQG|  --restart unless-stopped \
401#50fDoD|  --network host \
402#YKwbih|  --cap-add NET_ADMIN \
403#MDolmc|  --device /dev/net/tun \
404#OIGbOt|  -v /opt/easytier-agent/config/core.toml:/etc/easytier/core.toml:ro \
405#cH-9Ud|  -v easytier-agent-data:/data \
406#9OMu4W|  -e WHITELIST_SYNC_URL=http://10.0.10.1:11211/api/v1/public/whitelist.json \
407#MYmvFG|  -e WHITELIST_SYNC_INTERVAL=30 \
408#mUAg9u|  easytier-agent:latest
409#47DEQp|
410#Jk1Ni_|# 4. 验证
411#O27Cf3|docker logs -f easytier-agent
412#VXePP-|# 应看到：
413#85lw65|#   [agent] auto-detected admin IP from core.toml: 10.0.10.1 (agent=10.0.10.2)
414#f8VGEu|#   [agent] initializing default whitelist (kills chicken-and-egg)
415#1x8oTj|#   [agent] default whitelist: [{"ip":"10.0.10.1","hostname":null}]
416#NfJYFq|#   whitelist-sync-daemon starting: ...
417#8OwnbJ|#   Whitelist synced: N entries
418#8bkBhH|```
419#47DEQp|
420#Xlq3tX|### 重新生成默认白名单
421#47DEQp|
422#ijPU3b|如果需要重新触发默认白名单逻辑（例如网络重命名），删除数据卷：
423#47DEQp|
424#rTMjNc|```bash
425#A9wpVF|docker stop easytier-agent
426#OnOOsl|docker rm easytier-agent
427#G_Sr2D|docker volume rm easytier-agent-data
428#6NMkwI|# 然后重新 docker run
429#8bkBhH|```
430#47DEQp|
431#X1AAmN|## 兼容性与版本
432#47DEQp|
433#A72gb0|本镜像内置的 `easytier-core` 与 `easytier-web` 与官方版本保持一致，仅以下文件有定制修改：
434#47DEQp|
435#fROwi6|- `easytier-web/admin-frontend/index.html` — 自定义管理 UI（IP 白名单、设备管理、时间本地化）
436#6E3Mf9|- `Dockerfile` — 增加了 `tzdata` 和 `ENV TZ=Asia/Shanghai` 配置
437#47DEQp|
438#O2n2fe|实测兼容的官方客户端版本：
439#47DEQp|
440#icoT-e|| 客户端版本 | 状态 |
441#kFf8YZ||-----------|------|
442#svadrI|| 2.3.2     | 正常 |
443#nj2Sth|| 2.4.5     | 正常 |
444#47DEQp|
445#lh1-4f|## 构建镜像
446#47DEQp|
447#gBkQXM|本镜像采用 Docker 多阶段构建：
448#47DEQp|
449#6QaWUE|1. **Builder 阶段**：Rust 1.89 编译 `easytier-core` 和 `easytier-web`；pnpm 编译前端（`frontend-lib` + `frontend`）；`rust_embed` 将 `admin-frontend/index.html` 嵌入二进制
450#_rJtYZ|2. **Runtime 阶段**：`debian:bookworm-slim` 基础镜像，安装运行时依赖，复制编译产物
451#47DEQp|
452#gX_EZt|构建命令：
453#47DEQp|
454#rTMjNc|```bash
455#xReK-R|docker build --platform linux/amd64 -t easytier-admin:2.4.5 .
456#8bkBhH|```
457#47DEQp|
458#B5ckTg|首次构建需要下载 Rust crate 依赖并完整编译，耗时约 10–30 分钟（视网络和机器性能而定）。后续构建会利用 Docker 层缓存，仅重新编译变更部分。
459#47DEQp|
460#ZrcCRO|## 一键脚本
461#47DEQp|
462#1bwCuP|仓库根目录提供 `build-and-run.sh`（Linux amd64），支持一键构建并启动容器：
463#47DEQp|
464#rTMjNc|```bash
465#ZZ_zW7|./build-and-run.sh                                                      # 默认参数
466#mpL6Sf|./build-and-run.sh --tag 2.4.5 --password your-password --container my-easytier
467#8bkBhH|```
468#47DEQp|
469#z_LSEy|所有可调参数：
470#47DEQp|
471#ckRseV|| 参数 | 说明 | 默认值 |
472#Zteqcb||------|------|--------|
473#L8L8e1|| `--image` | 镜像名称 | `easytier-admin` |
474#h9vuwY|| `--tag` | 镜像标签 | `2.4.5` |
475#Rnsph5|| `--container` | 容器名称 | `easytier-admin` |
476#p13jLy|| `--password` | Admin 密码 | `changeme-please` |
477#EKaSoH|| `--web-port` | Admin UI 端口（仅显示用） | `11211` |
478#s87WdJ|| `--vpn-port` | VPN 监听端口（仅显示用） | `22020` |
479#G52l0j|| `--no-cache` | 不使用 Docker 构建缓存 | 关闭 |
480#47DEQp|
481#xSldlR|## 目录结构
482#47DEQp|
483#8bkBhH|```
484#zbTuKu|.
485#XK0mpi|├── Dockerfile              # 多阶段构建文件
486#Q_lSF2|├── agent.Dockerfile       # agent 节点镜像（半中心化部署）
487#YJmtR4|├── entrypoint.sh           # 容器入口脚本
488#FaB6Zc|├── build-and-run.sh        # 一键构建运行脚本
489#hUenHg|├── README.md               # 本文档
490#Dm3O1B|├── .gitignore
491#CjXFCu|├── LICENSE                 # 上游 LGPL-3.0 许可证
492#hhpxgH|├── Cargo.toml              # Rust workspace 根配置
493#SalN2g|├── Cargo.lock
494#O3bPW2|├── pnpm-workspace.yaml     # pnpm monorepo 配置
495#vLIrLO|├── pnpm-lock.yaml
496#RZBhAW|├── easytier/               # easytier-core 源码
497#ei_v68|│   ├── Cargo.toml
498#oZHzVr|│   └── src/
499#Dny7HH|├── easytier-rpc-build/     # protobuf RPC 编译辅助
500#ei_v68|│   ├── Cargo.toml
501#oZHzVr|│   └── src/
502#my9-C2|├── easytier-web/           # 管理后台源码
503#ei_v68|│   ├── Cargo.toml
504#i4-TQt|│   ├── src/                # Rust 后端（REST API、IP 白名单、SQLite）
505#tPup5f|│   ├── admin-frontend/     # 自定义管理 UI（HTML+JS，被 rust_embed 嵌入二进制）
506#IbqGs9|│   ├── frontend/           # 官方 Web 前端（Vue+TS，编译后作为依赖）
507#gadNq4|│   └── frontend-lib/       # 前端组件库
508#BZH1im|├── vendor/                 # 离线构建的 vendored Rust 依赖
509#gW82sD|└── .cargo/                 # Cargo 构建配置
510#8bkBhH|```
511#47DEQp|
512#ZVfp8_|## 常见问题（FAQ）
513#47DEQp|
514#-cRN8e|**Q: Admin 默认账号是什么？**
515#NQwtol|A: 用户名 `admin`，密码由 `ET_ADMIN_PASSWORD` 环境变量设置。首次启动后通过 Admin UI 注册。
516#47DEQp|
517#O1W0PP|**Q: 忘记密码怎么办？**
518#Gcp7CX|A: 删除 `/data/easytier-admin.db` 文件后重启容器，系统会重新初始化（**注意：会丢失所有 IP 白名单和设备数据**）。
519#47DEQp|
520#wt4VrT|**Q: 容器内时区是什么？**
521#xBQKOo|A: 已设为 `Asia/Shanghai`，Admin UI 中所有时间显示为北京时间（UTC+8）。
522#47DEQp|
523#UrLTV2|**Q: 如何升级 easytier-core 版本？**
524#GLu21b|A: 更新 `easytier/Cargo.toml` 中的版本号，然后重新构建镜像。
525#47DEQp|
526#as2RBp|**Q: 为什么需要 `--privileged`？**
527#BdrSkt|A: `easytier-core` 需要创建 TUN 网络设备，这需要特权模式。
528#47DEQp|
529#pFt8Tp|**Q: agent 无法同步白名单怎么办？**
530#9quEII|A: 检查 `WHITELIST_SYNC_URL` 是否正确、admin 是否可达。admin 失联时 agent 会使用最后一次同步的白名单，不会放行新设备。
531#47DEQp|
532#d4XDlx|## 许可与致谢
533#47DEQp|
534#wq6wT7|本项目基于 [EasyTier](https://github.com/EasyTier/EasyTier) 进行二次开发，遵循上游 [LGPL-3.0](LICENSE) 许可证。
535#47DEQp|
536#gy7y6A|感谢 EasyTier 开源社区提供的优秀组网方案。
537#47DEQp|

## [assistant] +058391-06-27T15:33:43.000Z
[thinking] Let me understand the current state and what the user is asking. Looking at the conversation history:

1. They found a bug where agent intermittently lets packets through
2. Fixed by replacing `close_peer_conn` with `close_peer` in `instance.rs`
3. Found that `entrypoint.sh` INSERT OR IGNORE was failing silently because `created_by` and `created_at` NOT NULL fields were missing
4. Fixed the entrypoint.sh
5. Changed RUST_LOG from `easytier=warn` to `easytier=error`
6. Rolled back `utils.rs console_layer` to include timestamps again

Now the last task from @analyst was about analyzing what needs to be updated in README.md. Let me read the current state of all the modified files to understand what changes were actually made, then I can figure out what README updates are needed.

Let me check the current state of the files that were modified.
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/entrypoint.sh"}) [completed]
1#Nme7aS|#!/bin/bash
2#Lhp2Od|set +e
3#47DEQp|
4#23rhQ8|DB_PATH=${DB_PATH:-"/data/et.db"}
5#-BtQON|SECRETS_FILE=${SECRETS_FILE:-"/tmp/network_secrets.json"}
6#VnW7rh|IP_WHITELIST_FILE=${IP_WHITELIST_FILE:-"/tmp/ip_whitelist.json"}
7#FKsehc|IP_HOSTNAME_AUTO_FILE=${IP_HOSTNAME_AUTO_FILE:-"/tmp/ip_hostname_auto.json"}
8#WTtRLL|BLOCKED_PEERS_FILE=${BLOCKED_PEERS_FILE:-"/tmp/blocked_peers.json"}
9#8ZIz-M|CORE_CONFIG=${CORE_CONFIG:-"/etc/easytier/core.toml"}
10#47DEQp|
11#9K7W-P|echo "Starting easytier-core..."
12#aq1au8|if [ -f "$CORE_CONFIG" ]; then
13#m_Tzd9|    ADMIN_API_URL="http://127.0.0.1:11211" IP_WHITELIST_FILE="$IP_WHITELIST_FILE" IP_HOSTNAME_AUTO_FILE="$IP_HOSTNAME_AUTO_FILE" BLOCKED_PEERS_FILE="$BLOCKED_PEERS_FILE" EXTRA_SECRETS_FILE="$SECRETS_FILE" easytier-core -c "$CORE_CONFIG" &
14#tfnLg8|    CORE_PID=$!
15#fdUwxN|else
16#yCfuAt|    echo "Warning: No core config found at $CORE_CONFIG. Core not started."
17#tL3ISB|fi
18#47DEQp|
19#8eAjxH|echo "Starting easytier-admin..."
20#VRnQZ-|easytier-admin --db "$DB_PATH" --web-server-port 11211 -p tcp &
21#_JQXVc|ADMIN_PID=$!
22#47DEQp|
23#aq1au8|if [ -f "$CORE_CONFIG" ]; then
24#gGs8VR|    ADMIN_IP=$(grep -E '^[[:space:]]*ipv4[[:space:]]*=' "$CORE_CONFIG" | \
25#5Y802a|        sed -E 's/^[[:space:]]*ipv4[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/' | \
26#YEQ8mc|        cut -d'/' -f1)
27#Vc2gJv|    HOSTNAME_VAL=$(hostname 2>/dev/null || echo "admin")
28#dPRbOh|    if [ -n "$ADMIN_IP" ]; then
29#5BI-tM|        echo "[entrypoint] auto-adding admin self ($ADMIN_IP) to whitelist with hostname=$HOSTNAME_VAL"
30#rkp9Et|        sqlite3 "$DB_PATH" \
31#C6DoxM|            "INSERT OR IGNORE INTO ip_whitelist (ip, hostname, comment, created_by, created_at) VALUES ('$ADMIN_IP', '$HOSTNAME_VAL', 'auto', 'entrypoint', datetime('now'))" \
32#JDccrY|            2>/dev/null || true
33#rkp9Et|        sqlite3 "$DB_PATH" \
34#oAjdNc|            "UPDATE ip_whitelist SET hostname='$HOSTNAME_VAL' WHERE ip='$ADMIN_IP' AND hostname IS NULL" \
35#JDccrY|            2>/dev/null || true
36#uA3yug|    fi
37#tL3ISB|fi
38#47DEQp|
39#XnWuEq|echo "Starting secret sync..."
40#9hW1xj|while true; do
41#Oy7ZPK|    sqlite3 "$DB_PATH" \
42#4j6y2F|        "SELECT json_group_array(json_object('secret', secret))
43#cfVRWv|         FROM network_secrets
44#UXfET0|         WHERE is_active = 1 
45#cnbXwp|           AND (expires_at IS NULL OR expires_at > datetime('now'))" \
46#V-CD39|        2>/dev/null > "$SECRETS_FILE" || true
47#47DEQp|
48#FR0JSN|    # Process auto-hostname bindings from core
49#pTIofk|    if [ -f "$IP_HOSTNAME_AUTO_FILE" ]; then
50#ynau6Y|        cat "$IP_HOSTNAME_AUTO_FILE" | python3 -c "
51#M-E9R8|import json, sys
52#0hhcHk|bindings = json.load(sys.stdin)
53#2MYoo9|for b in bindings:
54#XFR2Al|    ip = b.get('ip', '')
55#nOJKrB|    hostname = b.get('hostname', '')
56#T12jji|    if ip and hostname:
57#VpFb9h|        print(f\"UPDATE ip_whitelist SET hostname='{hostname}' WHERE ip='{ip}' AND hostname IS NULL\")
58#QOUu9s|" 2>/dev/null | while read sql; do
59#SyUiza|            sqlite3 "$DB_PATH" "$sql" 2>/dev/null || true
60#o5MEoY|        done
61#Onzp5z|        rm -f "$IP_HOSTNAME_AUTO_FILE"
62#uA3yug|    fi
63#47DEQp|
64#Oy7ZPK|    sqlite3 "$DB_PATH" \
65#EZwWIm|        "SELECT json_group_array(json_object('ip', ip, 'hostname', hostname)) FROM ip_whitelist" \
66#ggAAYE|        2>/dev/null > "$IP_WHITELIST_FILE.tmp" && mv "$IP_WHITELIST_FILE.tmp" "$IP_WHITELIST_FILE" || true
67#N1667T|    sleep 30
68#pkaSH4|done &
69#m7VEmQ|SYNC_PID=$!
70#47DEQp|
71#jUJNVV|wait 2>/dev/null || true
72#B6zVyC|echo "Shutting down..."
73#AWwGFw|kill $CORE_PID $ADMIN_PID $SYNC_PID 2>/dev/null || true
74#jUJNVV|wait 2>/dev/null || true
75#47DEQp|
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/scripts/agent-entrypoint.sh"}) [completed]
1#Nme7aS|#!/bin/bash
2#gldp7t|set -e
3#47DEQp|
4#uCMZTa|: "${WHITELIST_SYNC_URL:?WHITELIST_SYNC_URL must be set}"
5#0_RR3V|: "${IP_WHITELIST_FILE:=/data/ip_whitelist.json}"
6#t4w34X|: "${WHITELIST_SYNC_INTERVAL:=30}"
7#k3sqja|: "${CORE_CONFIG:=/etc/easytier/core.toml}"
8#ZJT_a9|: "${RUST_LOG:=easytier=error}"
9#LRKWuX|export RUST_LOG
10#LarChN|export IP_WHITELIST_FILE
11#nGhEQd|export WHITELIST_SYNC_INTERVAL
12#47DEQp|
13#DpzXU5|# 后续 sync-daemon 会用 admin 真实白名单覆盖此默认值。
14#-jPprj|DEFAULT_WHITELIST_IPS="${WHITELIST_DEFAULT_IPS:-}"
15#47DEQp|
16#zrb-Us|if [ -z "$DEFAULT_WHITELIST_IPS" ] && [ -f "$CORE_CONFIG" ]; then
17#uPbGmF|    AGENT_IP=$(grep -E '^[[:space:]]*ipv4[[:space:]]*=' "$CORE_CONFIG" | \
18#5Y802a|        sed -E 's/^[[:space:]]*ipv4[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/' | \
19#YEQ8mc|        cut -d'/' -f1)
20#SIvPbH|    if [ -n "$AGENT_IP" ]; then
21#fcMeVW|        DEFAULT_ADMIN_IP=$(echo "$AGENT_IP" | awk -F. '{print $1"."$2"."$3".1"}')
22#n9OBCt|        DEFAULT_WHITELIST_IPS="$DEFAULT_ADMIN_IP"
23#z_vegj|        echo "[agent] auto-detected admin IP from core.toml: $DEFAULT_ADMIN_IP (agent=$AGENT_IP)"
24#uA3yug|    fi
25#tL3ISB|fi
26#47DEQp|
27#zvU8MW|if [ ! -f "$IP_WHITELIST_FILE" ] && [ -n "$DEFAULT_WHITELIST_IPS" ]; then
28#owPflu|    echo "[agent] initializing default whitelist (kills chicken-and-egg)"
29#GAyLSX|    mkdir -p "$(dirname "$IP_WHITELIST_FILE")"
30#ssVAyK|    DEFAULT_JSON=$(echo "$DEFAULT_WHITELIST_IPS" | awk '
31#5eWLjU|    BEGIN { printf "[" }
32#DT0vWo|    {
33#Vy3_VT|        n = split($0, arr, ",")
34#iSWuhr|        for (i = 1; i <= n; i++) {
35#9hNbZ0|            gsub(/^[ \t]+|[ \t]+$/, "", arr[i])
36#twCwS9|            if (arr[i] != "") {
37#J_3lmB|                if (i > 1) printf ","
38#Mte3rE|                printf "{\"ip\":\"%s\",\"hostname\":null}", arr[i]
39#hq_fcK|            }
40#KkQWgx|        }
41#KNhneG|    }
42#eKZdC0|    END { printf "]\n" }
43#yWwQC3|    ')
44#5Z8qiH|    printf '%s' "$DEFAULT_JSON" > "$IP_WHITELIST_FILE"
45#2Dozcl|    echo "[agent] default whitelist: $DEFAULT_JSON"
46#k7_u0m|elif [ -f "$IP_WHITELIST_FILE" ]; then
47#0KMnK5|    echo "[agent] whitelist file exists, skipping default init"
48#tL3ISB|fi
49#47DEQp|
50#VZ4ZCm|echo "[agent] starting whitelist-sync-daemon..."
51#EqDRX4|easytier-whitelist-sync &
52#m7VEmQ|SYNC_PID=$!
53#47DEQp|
54#aq5u9C|sleep 5
55#47DEQp|
56#8Zr0vy|echo "[agent] starting easytier-core..."
57#aq1au8|if [ -f "$CORE_CONFIG" ]; then
58#CeaDmq|    IP_WHITELIST_FILE="$IP_WHITELIST_FILE" \
59#R4vUhD|    easytier-core -c "$CORE_CONFIG" &
60#tfnLg8|    CORE_PID=$!
61#fdUwxN|else
62#EIYlGT|    echo "[agent] ERROR: core config not found at $CORE_CONFIG"
63#yyiJ64|    kill $SYNC_PID
64#Q81I2A|    exit 1
65#tL3ISB|fi
66#47DEQp|
67#nwvYsh|trap "echo '[agent] shutting down'; kill $SYNC_PID $CORE_PID 2>/dev/null; wait" SIGTERM SIGINT
68#cW7Ku0|wait
69#47DEQp|
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs","limit":50}) [completed]
1#idcYPx|use std::any::Any;
2#rHAMkI|use std::collections::HashSet;
3#3i1UbG|use std::net::{IpAddr, Ipv4Addr};
4#TX2tS7|use std::sync::atomic::{AtomicBool, Ordering};
5#Y1NUBR|use std::sync::{Arc, Weak};
6#bd0W_A|use std::time::Duration;
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#CPbhxq|use cidr::{IpCidr, Ipv4Inet};
10#47DEQp|
11#J-XYtp|use futures::FutureExt;
12#BJN22x|use tokio::sync::{oneshot, Notify};
13#6_VUed|use tokio::{sync::Mutex, task::JoinSet};
14#HycC7X|use tokio_util::sync::CancellationToken;
15#47DEQp|
16#4DVW7a|use crate::common::acl_processor::AclRuleBuilder;
17#ueHy0Y|use crate::common::config::ConfigLoader;
18#1ldaD4|use crate::common::error::Error;
19#9wFsCH|use crate::common::global_ctx::{ArcGlobalCtx, GlobalCtx, GlobalCtxEvent};
20#mvlxEN|use crate::common::scoped_task::ScopedTask;
21#_WoPHh|use crate::common::PeerId;
22#cPOIRF|use crate::connector::direct::DirectConnectorManager;
23#hogYE0|use crate::connector::manual::{ConnectorManagerRpcService, ManualConnectorManager};
24#erQ5iU|use crate::connector::udp_hole_punch::UdpHolePunchConnector;
25#ejR0qf|use crate::gateway::icmp_proxy::IcmpProxy;
26#bJmizf|use crate::gateway::kcp_proxy::{KcpProxyDst, KcpProxyDstRpcService, KcpProxySrc};
27#rE-Sqy|use crate::gateway::quic_proxy::{QUICProxyDst, QUICProxyDstRpcService, QUICProxySrc};
28#cqr7_4|use crate::gateway::tcp_proxy::{NatDstTcpConnector, TcpProxy, TcpProxyRpcService};
29#8GSF2g|use crate::gateway::udp_proxy::UdpProxy;
30#zcBr4d|use crate::peer_center::instance::PeerCenterInstance;
31#9lvsA5|use crate::peers::peer_conn::PeerConnId;
32#LoFxpZ|use crate::peers::peer_manager::{PeerManager, RouteAlgoType};
33#-AoE18|use crate::peers::rpc_service::PeerManagerRpcService;
34#xeImgP|use crate::peers::{create_packet_recv_chan, recv_packet_from_chan, PacketRecvChanReceiver};
35#OcwcBy|use crate::proto::cli::VpnPortalRpc;
36#uPqIqx|use crate::proto::cli::{
37#NVPEsM|    AddPortForwardRequest, AddPortForwardResponse, GetPrometheusStatsRequest,
38#rjIKkU|    GetPrometheusStatsResponse, GetStatsRequest, GetStatsResponse, ListMappedListenerRequest,
39#jUlX81|    ListMappedListenerResponse, ListPortForwardRequest, ListPortForwardResponse,
40#Y1yxDo|    ManageMappedListenerRequest, ManageMappedListenerResponse, MappedListener,
41#RDmcQw|    MappedListenerManageAction, MappedListenerManageRpc, MetricSnapshot, PortForwardManageRpc,
42#ba8G6V|    RemovePortForwardRequest, RemovePortForwardResponse, StatsRpc,
43#3zmC_W|};
44#OCkFn3|use crate::proto::cli::{GetVpnPortalInfoRequest, GetVpnPortalInfoResponse, VpnPortalInfo};
45#IvycE5|use crate::proto::common::{PortForwardConfigPb, TunnelInfo};
46#3hsMSR|use crate::proto::peer_rpc::PeerCenterRpcServer;
47#0sz7R6|use crate::proto::rpc_impl::standalone::{RpcServerHook, StandAloneServer};
48#gh0Oxf|use crate::proto::rpc_types;
49#RYyBjw|use crate::proto::rpc_types::controller::BaseController;
50#iLWKio|use crate::tunnel::tcp::TcpTunnelListener;
51#63WiJD|use crate::vpn_portal::{self, VpnPortal};
52#47DEQp|
53#3H8X_F|use super::dns_server::runner::DnsRunner;
54#JkrRdm|use super::dns_server::MAGIC_DNS_FAKE_IP;
55#E9G50B|use super::listeners::ListenerManager;
56#47DEQp|
57#eX70dr|#[cfg(feature = "socks5")]
58#I370cj|use crate::gateway::socks5::Socks5Server;
59#47DEQp|
60#fG4NjB|#[derive(Clone)]
61#qdeGvi|struct IpProxy {
62#uj-EDs|    tcp_proxy: Arc<TcpProxy<NatDstTcpConnector>>,
63#LXeGDc|    icmp_proxy: Arc<IcmpProxy>,
64#d456fQ|    udp_proxy: Arc<UdpProxy>,
65#sfM3Lj|    global_ctx: ArcGlobalCtx,
66#tNRg3L|    started: Arc<AtomicBool>,
67#0Qs2qn|}
68#47DEQp|
69#Oo4MM_|impl IpProxy {
70#XiF5hg|    fn new(global_ctx: ArcGlobalCtx, peer_manager: Arc<PeerManager>) -> Result<Self, Error> {
71#Zt5NnB|        let tcp_proxy = TcpProxy::new(peer_manager.clone(), NatDstTcpConnector {});
72#EtbHXp|        let icmp_proxy = IcmpProxy::new(global_ctx.clone(), peer_manager.clone())
73#MFCTIJ|            .with_context(|| "create icmp proxy failed")?;
74#0oBQk7|        let udp_proxy = UdpProxy::new(global_ctx.clone(), peer_manager.clone())
75#IsE2n_|            .with_context(|| "create udp proxy failed")?;
76#Z9PvxG|        Ok(IpProxy {
77#bM3rd8|            tcp_proxy,
78#CA9DHN|            icmp_proxy,
79#CYr6HS|            udp_proxy,
80#PvFYEj|            global_ctx,
81#0tmpZx|            started: Arc::new(AtomicBool::new(false)),
82#9I500R|        })
83#KNhneG|    }
84#47DEQp|
85#G2-hVv|    async fn start(&self) -> Result<(), Error> {
86#qxvdE7|        if (self.global_ctx.config.get_proxy_cidrs().is_empty()
87#kgqUjD|            || self.started.load(Ordering::Relaxed))
88#40DQ43|            && !self.global_ctx.enable_exit_node()
89#XRxEXn|            && !self.global_ctx.no_tun()
90#qWZHcZ|        {
91#6WCzfY|            return Ok(());
92#KkQWgx|        }
93#47DEQp|
94#Pk-dvt|        // Actually, if this node is enabled as an exit node,
95#OkAh4W|        // we still can use the system stack to forward packets.
96#1ANCVU|        if self.global_ctx.proxy_forward_by_system() && !self.global_ctx.no_tun() {
97#6WCzfY|            return Ok(());
98#KkQWgx|        }
99#47DEQp|
100#fqoH0J|        self.started.store(true, Ordering::Relaxed);
101#2IVC4_|        self.tcp_proxy.start(true).await?;
102#Epq6hx|        if let Err(e) = self.icmp_proxy.start().await {
103#G-36sE|            tracing::error!("start icmp proxy failed: {:?}", e);
104#oZxrgb|            if cfg!(not(any(target_os = "android", target_env = "ohos"))) {
105#_qBBE_|                // android and ohos not support icmp proxy
106#EnZEc5|                return Err(e);
107#hq_fcK|            }
108#KkQWgx|        }
109#Lj2lP2|        self.udp_proxy.start().await?;
110#sX41Gp|        Ok(())
111#KNhneG|    }
112#0Qs2qn|}
113#47DEQp|
114#dU1RB6|#[cfg(feature = "tun")]
115#EnBOzL|type NicCtx = super::virtual_nic::NicCtx;
116#XgxVd_|#[cfg(not(feature = "tun"))]
117#6Qvb67|struct NicCtx;
118#XgxVd_|#[cfg(not(feature = "tun"))]
119#KvhMrR|impl NicCtx {
120#4SQmJP|    pub fn new(
121#n4qY0i|        _global_ctx: ArcGlobalCtx,
122#eQg1SJ|        _peer_manager: &Arc<PeerManager>,
123#r0jWyK|        _peer_packet_receiver: Arc<Mutex<PacketRecvChanReceiver>>,
124#P1yxtn|    ) -> Self {
125#6op1sO|        Self
126#KNhneG|    }
127#47DEQp|
128#i0D5JW|    pub async fn run(&mut self, _ipv4_addr: Ipv4Addr) -> Result<(), Error> {
129#sX41Gp|        Ok(())
130#KNhneG|    }
131#0Qs2qn|}
132#47DEQp|
133#Yu4a9V|struct MagicDnsContainer {
134#HBUsu9|    dns_runner_task: ScopedTask<()>,
135#Nf2F4C|    dns_runner_cancel_token: CancellationToken,
136#0Qs2qn|}
137#47DEQp|
138#9xNxPl|// nic container will be cleared when dhcp ip changed
139#6oN5oh|pub(crate) struct NicCtxContainer {
140#VwfTlr|    nic_ctx: Option<Box<dyn Any + 'static + Send>>,
141#rua5bu|    magic_dns: Option<MagicDnsContainer>,
142#0Qs2qn|}
143#47DEQp|
144#WJVBM9|impl NicCtxContainer {
145#drm3O2|    fn new(nic_ctx: NicCtx, dns_runner: Option<DnsRunner>) -> Self {
146#zEGEEo|        if let Some(mut dns_runner) = dns_runner {
147#UqagTr|            let token = CancellationToken::new();
148#DYkYW8|            let token_clone = token.clone();
149#c7htKU|            let task = tokio::spawn(async move {
150#0qhyJs|                let _ = dns_runner.run(token_clone).await;
151#-JdJBA|            });
152#3Q48M3|            Self {
153#lLJC-i|                nic_ctx: Some(Box::new(nic_ctx)),
154#Mu7wwF|                magic_dns: Some(MagicDnsContainer {
155#vyTj9I|                    dns_runner_task: task.into(),
156#U85dxh|                    dns_runner_cancel_token: token,
157#MsptEA|                }),
158#hq_fcK|            }
159#d4ha1c|        } else {
160#3Q48M3|            Self {
161#lLJC-i|                nic_ctx: Some(Box::new(nic_ctx)),
162#vewqsl|                magic_dns: None,
163#hq_fcK|            }
164#KkQWgx|        }
165#KNhneG|    }
166#47DEQp|
167#rbEQLK|    fn new_with_any<T: 'static + Send>(ctx: T) -> Self {
168#qMx6zU|        Self {
169#gPdDMI|            nic_ctx: Some(Box::new(ctx)),
170#Y75cMA|            magic_dns: None,
171#KkQWgx|        }
172#KNhneG|    }
173#0Qs2qn|}
174#47DEQp|
175#hMqpOX|type ArcNicCtx = Arc<Mutex<Option<NicCtxContainer>>>;
176#47DEQp|
177#dgc-_o|pub struct InstanceRpcServerHook {
178#h8fA_m|    rpc_portal_whitelist: Vec<IpCidr>,
179#0Qs2qn|}
180#47DEQp|
181#I0xZDG|impl InstanceRpcServerHook {
182#WD4xNj|    pub fn new(rpc_portal_whitelist: Option<Vec<IpCidr>>) -> Self {
183#mICM0-|        let rpc_portal_whitelist = rpc_portal_whitelist
184#f5UENG|            .unwrap_or_else(|| vec!["127.0.0.0/8".parse().unwrap(), "::1/128".parse().unwrap()]);
185#1FjQgy|        InstanceRpcServerHook {
186#I_qCl5|            rpc_portal_whitelist,
187#KkQWgx|        }
188#KNhneG|    }
189#0Qs2qn|}
190#47DEQp|
191#31qlYx|#[async_trait::async_trait]
192#zSMbJH|impl RpcServerHook for InstanceRpcServerHook {
193#61wmMd|    async fn on_new_client(
194#7BRTWY|        &self,
195#9t8xpt|        tunnel_info: Option<TunnelInfo>,
196#Saslb4|    ) -> Result<Option<TunnelInfo>, anyhow::Error> {
197#BlX9xo|        let tunnel_info = tunnel_info.ok_or_else(|| anyhow::anyhow!("tunnel info is None"))?;
198#47DEQp|
199#nUob4y|        let remote_url = tunnel_info
200#A-hApy|            .remote_addr
201#f5TNOB|            .clone()
202#ClFVg8|            .ok_or_else(|| anyhow::anyhow!("remote_addr is None"))?;
203#47DEQp|
204#Qpjg5R|        let url_str = &remote_url.url;
205#OHRp5k|        let url = url::Url::parse(url_str)
206#t9HSXw|            .map_err(|e| anyhow::anyhow!("Failed to parse remote URL '{}': {}", url_str, e))?;
207#47DEQp|
208#dN5dPe|        let host = url
209#bHOvnW|            .host_str()
210#1DuEKI|            .ok_or_else(|| anyhow::anyhow!("No host found in remote URL '{}'", url_str))?;
211#47DEQp|
212#Tm9CUn|        let ip_addr: IpAddr = host
213#C8wNTO|            .parse()
214#PN-fNI|            .map_err(|e| anyhow::anyhow!("Failed to parse IP address '{}': {}", host, e))?;
215#47DEQp|
216#jv2gSQ|        for cidr in &self.rpc_portal_whitelist {
217#M7L3vD|            if cidr.contains(&ip_addr) {
218#h8l3SL|                return Ok(Some(tunnel_info));
219#hq_fcK|            }
220#KkQWgx|        }
221#iMhqMb|        return Err(anyhow::anyhow!(
222#NsInze|            "Rpc portal client IP {} not in whitelist: {:?}, ignoring client.",
223#LcYU0v|            ip_addr,
224#f_A8wo|            self.rpc_portal_whitelist
225#XNrhe2|        ));
226#KNhneG|    }
227#0Qs2qn|}
228#47DEQp|
229#IVkNK4|pub struct Instance {
230#DxGnbT|    inst_name: String,
231#47DEQp|
232#padI3k|    id: uuid::Uuid,
233#47DEQp|
234#Yf_ThY|    nic_ctx: ArcNicCtx,
235#47DEQp|
236#jSZA3M|    peer_packet_receiver: Arc<Mutex<PacketRecvChanReceiver>>,
237#w-V8cy|    peer_manager: Arc<PeerManager>,
238#7MKwyP|    listener_manager: Arc<Mutex<ListenerManager<PeerManager>>>,
239#S9cacv|    conn_manager: Arc<ManualConnectorManager>,
240#jNB13j|    direct_conn_manager: Arc<DirectConnectorManager>,
241#ncaS3L|    udp_hole_puncher: Arc<Mutex<UdpHolePunchConnector>>,
242#47DEQp|
243#zFEUuN|    ip_proxy: Option<IpProxy>,
244#47DEQp|
245#RLRkGX|    kcp_proxy_src: Option<KcpProxySrc>,
246#3W9MHn|    kcp_proxy_dst: Option<KcpProxyDst>,
247#47DEQp|
248#CTM6XI|    quic_proxy_src: Option<QUICProxySrc>,
249#Mqb1AC|    quic_proxy_dst: Option<QUICProxyDst>,
250#47DEQp|
251#4f9KOd|    peer_center: Arc<PeerCenterInstance>,
252#47DEQp|
253#NDyVjL|    vpn_portal: Arc<Mutex<Box<dyn VpnPortal>>>,
254#47DEQp|
255#iZ3y8V|    #[cfg(feature = "socks5")]
256#IToVwl|    socks5_server: Arc<Socks5Server>,
257#47DEQp|
258#sR36nA|    rpc_server: Option<StandAloneServer<TcpTunnelListener>>,
259#47DEQp|
260#sfM3Lj|    global_ctx: ArcGlobalCtx,
261#0Qs2qn|}
262#47DEQp|
263#gthTws|impl Instance {
264#GSa_XI|    pub fn new(config: impl ConfigLoader + 'static) -> Self {
265#vNDgW1|        let global_ctx = Arc::new(GlobalCtx::new(config));
266#47DEQp|
267#nAKADg|        if let Ok(secrets_file) = std::env::var("EXTRA_SECRETS_FILE") {
268#ElPxrf|            global_ctx.set_extra_secrets_file(secrets_file);
269#15dn6u|            let ctx = global_ctx.clone();
270#gNYhpy|            tokio::spawn(async move {
271#nZx_YA|                loop {
272#YsB-tr|                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
273#WXii6j|                    ctx.load_extra_identities_from_json();
274#C_CgiK|                }
275#-JdJBA|            });
276#KkQWgx|        }
277#47DEQp|
278#3_Ehij|        tracing::info!(
279#bwgXBe|            "[INIT] instance creating. config: {}",
280#GZi-e-|            global_ctx.config.dump()
281#bq_4ei|        );
282#47DEQp|
283#Bnd_wL|        let (peer_packet_sender, peer_packet_receiver) = create_packet_recv_chan();
284#47DEQp|
285#YEEA2w|        let id = global_ctx.get_id();
286#47DEQp|
287#Oox-R_|        let peer_manager = Arc::new(PeerManager::new(
288#QuqIan|            RouteAlgoType::Ospf,
289#2rw4tz|            global_ctx.clone(),
290#KUNG1B|            peer_packet_sender.clone(),
291#XNrhe2|        ));
292#47DEQp|
293#YgmKIe|        peer_manager.set_allow_loopback_tunnel(false);
294#47DEQp|
295#HwC_7G|        println!("[WHITELIST_DEBUG] Checking IP_WHITELIST_FILE env var...");
296#Y6ccPr|        if let Ok(whitelist_file) = std::env::var("IP_WHITELIST_FILE") {
297#FbBcyj|            global_ctx.set_ip_whitelist_file(whitelist_file.clone());
298#8H00ac|            println!("[WHITELIST_DEBUG] IP_WHITELIST_FILE = {}", whitelist_file);
299#7y7Y2x|            if let Ok(auto_file) = std::env::var("IP_HOSTNAME_AUTO_FILE") {
300#Dj3FBI|                global_ctx.set_ip_hostname_auto_file(auto_file);
301#CE4VEs|            } else {
302#lygC7k|                global_ctx.set_ip_hostname_auto_file("/tmp/ip_hostname_auto.json".to_string());
303#hq_fcK|            }
304#75e08Y|            if let Ok(blocked_file) = std::env::var("BLOCKED_PEERS_FILE") {
305#hDqSDp|                global_ctx.set_blocked_peers_file(blocked_file);
306#hq_fcK|            }
307#pO4HZ-|            global_ctx.load_blocked_peers_from_file();
308#15dn6u|            let ctx = global_ctx.clone();
309#MZmfqG|            let pm = peer_manager.clone();
310#3TxkcS|            let ev_ctx = global_ctx.clone();
311#H3IO3F|            let ev_pm = peer_manager.clone();
312#gNYhpy|            tokio::spawn(async move {
313#BQj6Kq|                println!("[WHITELIST_DEBUG] Periodic task started");
314#nZx_YA|                loop {
315#YsB-tr|                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
316#APVDIL|                    println!("[WHITELIST_DEBUG] Periodic task running");
317#hlKQ5M|                    let whitelist = ctx.load_ip_whitelist();
318#lXByWl|                    println!("[WHITELIST_DEBUG] whitelist has {} entries", whitelist.len());
319#g4CF2k|                    let routes = pm.list_routes().await;
320#SfhhUa|                    println!("[WHITELIST_DEBUG] routes from list_routes(): {} total", routes.len());
321#FRdFOq|                    for route in &routes {
322#qQRRTP|                        let ip_str = route.ipv4_addr.as_ref().and_then(|ip| ip.address.as_ref()).map(|a| format!("{}", a));
323#Tf98w_|                        println!("[WHITELIST_DEBUG] route peer_id={}, ipv4={:?}", route.peer_id, ip_str);
324#uNGRgX|                    }
325#MfNsRN|                    let mut allowed_peers = Vec::new();
326#FRdFOq|                    for route in &routes {
327#qQRRTP|                        let ip_str = route.ipv4_addr.as_ref().and_then(|ip| ip.address.as_ref()).map(|a| format!("{}", a));
328#-eCupi|                        println!("[WHITELIST_DEBUG] checking peer_id={}, ipv4={:?}", route.peer_id, ip_str);
329#hqUjAn|                        if let Some(ip) = ip_str {
330#YdpUYy|                            let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip);
331#KmrKzE|                            if whitelist_ip_match {
332#r3L4T3|                                let entry = whitelist.iter().find(|e| e.ip == ip).unwrap();
333#NC8Vkn|                                match &entry.hostname {
334#EvDRLJ|                                    Some(bound_hostname) if !bound_hostname.is_empty() => {
335#9Q5Z7R|                                        if route.hostname != *bound_hostname {
336#3yGnp6|                                            let peer_id = route.peer_id;
337#hAJek2|                                            if !ctx.is_peer_blocked(peer_id) {
338#otUFjq|                                                ctx.block_peer(peer_id);
339#ZxYt6M|                                                let _ = pm.get_peer_map().close_peer(peer_id).await;
340#ugZoKr|                                                tracing::info!("Disconnected peer {}: IP {} hostname '{}' doesn't match bound '{}'", peer_id, ip, route.hostname, bound_hostname);
341#47DEQp|
342#47DEQp|
343#f45sRb|                                            }
344#aDfwqA|                                        } else {
345#bLnURe|                                            allowed_peers.push(route.peer_id);
346#E1hbvZ|                                        }
347#R9QnXW|                                    }
348#S8FaK9|                                    _ => {
349#2MU51-|                                        // hostname is None or "" — allow peer, auto-bind for both
350#x-ZhCA|                                        if !route.hostname.is_empty() {
351#FvtkKm|                                            ctx.bind_hostname(&ip, &route.hostname);
352#N0ItBQ|                                            println!("[BIND_DEBUG] auto-bound hostname '{}' to IP {}", route.hostname, ip);
353#E1hbvZ|                                        }
354#0ttO5y|                                        allowed_peers.push(route.peer_id);
355#R9QnXW|                                    }
356#BCocYH|                                }
357#47DEQp|
358#qjCoUo|                            } else {
359#Zj9VBb|                                let peer_id = route.peer_id;
360#Qcx7Nv|                                if !ctx.is_peer_blocked(peer_id) {
361#obcQX5|                                    ctx.block_peer(peer_id);
362#BCocYH|                                }
363#Z2GJMw|                                let _ = pm.get_peer_map().close_peer(peer_id).await;
364#iIaTj1|                                tracing::info!("Disconnected peer {}: IP {} not in whitelist", peer_id, ip);
365#47DEQp|
366#47DEQp|
367#ELKDGY|                            }
368#qrgf36|                        }
369#uNGRgX|                    }
370#irF0k1|                    for peer_id in allowed_peers {
371#h5et1K|                        if ctx.is_peer_blocked(peer_id) {
372#QzaLcb|                            ctx.unblock_peer(peer_id);
373#KxvMLh|                            tracing::info!("Unblocked peer {}: IP now in whitelist", peer_id);
374#qrgf36|                        }
375#uNGRgX|                    }
376#C_CgiK|                }
377#-JdJBA|            });
378#gNYhpy|            tokio::spawn(async move {
379#_yJDU_|                println!("[WHITELIST_DEBUG] Event-driven task started, subscribing to events");
380#ODc_zy|                let mut rx = ev_ctx.subscribe();
381#nZx_YA|                loop {
382#wgu-5F|                    match rx.recv().await {
383#WAZdK-|                        Ok(GlobalCtxEvent::PeerAdded(peer_id)) => {
384#DyW1xi|                            tracing::info!("Event-driven: PeerAdded({})", peer_id);
385#VNvpns|                            let ctx = ev_ctx.clone();
386#6hSFVL|                            let pm = ev_pm.clone();
387#OmSrLH|                            tokio::spawn(async move {
388#UcsuWj|                                let whitelist = ctx.load_ip_whitelist();
389#y1UM0Q|                                tracing::info!("Event-driven: whitelist has {} entries", whitelist.len());
390#B65_R_|                                for iteration in 0..120 {
391#ZLg3bh|                                    tokio::time::sleep(Duration::from_secs(1)).await;
392#DdqAwu|                                    if iteration % 30 == 0 {
393#ttCMv7|                                        tracing::info!("Event-driven: polling route info for peer {}, iteration {}", peer_id, iteration);
394#R9QnXW|                                    }
395#L6Swvq|                                    match pm.get_peer_map().get_route_peer_info(peer_id).await {
396#K21kxm|                                        Some(info) => {
397#xnzUlN|                                            if let Some(ref ip) = info.ipv4_addr {
398#iDSegq|                                                let ip_str = format!("{}", ip);
399#FK3Cgo|                                                tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
400#j9LGIx|                                                let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
401#tmY8le|                                                if whitelist_ip_match {
402#1QIYW3|                                                    let entry = whitelist.iter().find(|e| e.ip == ip_str).unwrap();
403#GW_QUn|                                                    let peer_hostname = info.hostname.as_deref().unwrap_or("");
404#vuwwfG|                                                    match &entry.hostname {
405#7bJFHl|                                                        Some(bound_hostname) if !bound_hostname.is_empty() => {
406#7EPl-h|                                                            if peer_hostname != bound_hostname.as_str() {
407#EhVB5x|                                                                if !ctx.is_peer_blocked(peer_id) {
408#1LUYXe|                                                                    ctx.block_peer(peer_id);
409#YOPWew|                                                                    let _ = pm.get_peer_map().close_peer(peer_id).await;
410#qsEMJV|                                                                    tracing::info!("Event-driven: disconnected peer {}: IP {} hostname '{}' doesn't match bound '{}'", peer_id, ip_str, peer_hostname, bound_hostname);
411#47DEQp|
412#47DEQp|
413#2LGh5n|                                                                }
414#hKoEmG|                                                            } else {
415#nvg4X2|                                                                if ctx.is_peer_blocked(peer_id) {
416#adiUqJ|                                                                    ctx.unblock_peer(peer_id);
417#1BAOWN|                                                                    tracing::info!("Event-driven: unblocked peer {}: IP {} in whitelist with matching hostname", peer_id, ip_str);
418#DMrtXW|                                                            }
419#cjSFTt|                                                        }
420#cjSFTt|                                                        }
421#tKztQP|                                                        _ => {
422#0ukzCP|                                                            // hostname is None or "" — allow peer, auto-bind for both
423#GIolFH|                                                            if let Some(ref h) = info.hostname {
424#UDVNh-|                                                                ctx.bind_hostname(&ip_str, h);
425#DMrtXW|                                                            }
426#G9lklR|                                                            if ctx.is_peer_blocked(peer_id) {
427#-LImmd|                                                                ctx.unblock_peer(peer_id);
428#D_OyrS|                                                                tracing::info!("Event-driven: unblocked peer {}: IP {} in whitelist (auto-bound hostname)", peer_id, ip_str);
429#DMrtXW|                                                            }
430#cjSFTt|                                                        }
431#TNZU2s|                                                    }
432#qrgSP7|                                                } else {
433#7cHHyj|                                                    if !ctx.is_peer_blocked(peer_id) {
434#Evye2j|                                                        ctx.block_peer(peer_id);
435#TNZU2s|                                                    }
436#_4InoV|                                                    let _ = pm.get_peer_map().close_peer(peer_id).await;
437#i_tcjm|                                                    tracing::info!("Event-driven: disconnected peer {}: IP {} not in whitelist", peer_id, ip_str);
438#47DEQp|
439#47DEQp|
440#kay0JC|                                                    return;
441#paYJX6|                                                }
442#S4Yvyf|                                            } else {
443#Ee5bn0|                                                tracing::debug!("Event-driven: peer {} has route info but ipv4_addr is None (iteration {})", peer_id, iteration);
444#f45sRb|                                            }
445#E1hbvZ|                                        }
446#yBZKr8|                                        None => {
447#AB6QzC|                                            if iteration % 30 == 0 {
448#nmZcwa|                                                tracing::info!("Event-driven: no route info yet for peer {} (iteration {})", peer_id, iteration);
449#f45sRb|                                            }
450#E1hbvZ|                                        }
451#R9QnXW|                                    }
452#BCocYH|                                }
453#4GVTj3|                                tracing::warn!("Event-driven: no route info for peer {} within 120s", peer_id);
454#eMXesW|                            });
455#qrgf36|                        }
456#RId2go|                        Ok(_) => {}
457#MvSNkR|                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
458#IXT1u8|                            tracing::warn!("Whitelist event subscriber skipped {} events", n);
459#qrgf36|                        }
460#kz9RqG|                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
461#Ko0WZT|                            break;
462#qrgf36|                        }
463#uNGRgX|                    }
464#C_CgiK|                }
465#-JdJBA|            });
466#KkQWgx|        }
467#47DEQp|
468#MMg2WC|        if let Ok(admin_api_url) = std::env::var("ADMIN_API_URL") {
469#MZmfqG|            let pm = peer_manager.clone();
470#gNYhpy|            tokio::spawn(async move {
471#_BO9BR|                let url = format!("{}/api/v1/admin/peers/report", admin_api_url);
472#nZx_YA|                loop {
473#YsB-tr|                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
474#g4CF2k|                    let routes = pm.list_routes().await;
475#rb7flX|                    let peer_list: Vec<serde_json::Value> = routes.iter().map(|r| {
476#dG9X8n|                        let ip = r.ipv4_addr.as_ref().and_then(|ip| ip.address.as_ref()).map(|a| format!("{}", a));
477#58Ls1D|                        serde_json::json!({
478#m032oa|                            "peer_id": r.peer_id,
479#FxYqv-|                            "ip": ip,
480#FiE7m6|                            "hostname": if r.hostname.is_empty() { None } else { Some(&r.hostname) },
481#d3u6EI|                        })
482#zZW95O|                    }).collect();
483#2B7ufJ|                    let body = serde_json::json!({"peers": peer_list}).to_string();
484#d_V89O|                    let url_clone = url.clone();
485#RNIldj|                    let body_clone = body.clone();
486#-z-ILg|                    let count = peer_list.len();
487#yd1OPy|                    tokio::task::spawn_blocking(move || {
488#3uWpmw|                        let mut resp = Vec::new();
489#A0xCRA|                        let uri = match http_req::uri::Uri::try_from(url_clone.as_str()) {
490#NpJFbA|                            Ok(u) => u,
491#7_PbPZ|                            Err(e) => {
492#vuLy6u|                                tracing::warn!("Failed to parse admin API URL: {}", e);
493#Tl9d-2|                                return;
494#ELKDGY|                            }
495#KJVpRp|                        };
496#Bf_yhF|                        match http_req::request::Request::new(&uri)
497#lOank0|                            .method(http_req::request::Method::POST)
498#en4OZm|                            .header("Content-Type", "application/json")
499#uZmVVN|                            .body(&body_clone.as_bytes())
500#7MdiM_|                            .send(&mut resp)
501#Fzqoyl|                        {
502#ItGech|                            Ok(r) => {
503#0cPbvw|                                tracing::debug!("Reported {} peers to admin, status: {}", count, r.status_code());
504#ELKDGY|                            }
505#7_PbPZ|                            Err(e) => {
506#oR-sd5|                                tracing::warn!("Failed to report peers to admin: {}", e);
507#ELKDGY|                            }
508#qrgf36|                        }
509#twrv0v|                    }).await.ok();
510#C_CgiK|                }
511#-JdJBA|            });
512#KkQWgx|        }
513#47DEQp|
514#axEiGY|        let listener_manager = Arc::new(Mutex::new(ListenerManager::new(
515#2rw4tz|            global_ctx.clone(),
516#LD1IjY|            peer_manager.clone(),
517#e8LcOr|        )));
518#47DEQp|
519#WN3bGk|        let conn_manager = Arc::new(ManualConnectorManager::new(
520#2rw4tz|            global_ctx.clone(),
521#LD1IjY|            peer_manager.clone(),
522#XNrhe2|        ));
523#47DEQp|
524#RP36Kb|        let mut direct_conn_manager =
525#yLL9aR|            DirectConnectorManager::new(global_ctx.clone(), peer_manager.clone());
526#DJbWXm|        direct_conn_manager.run();
527#47DEQp|
528#q5YBnw|        let udp_hole_puncher = UdpHolePunchConnector::new(peer_manager.clone());
529#47DEQp|
530#SHU_qF|        let peer_center = Arc::new(PeerCenterInstance::new(peer_manager.clone()));
531#47DEQp|
532#yrddWf|        #[cfg(feature = "wireguard")]
533#SPluBe|        let vpn_portal_inst = vpn_portal::wireguard::WireGuard::default();
534#p_lYtD|        #[cfg(not(feature = "wireguard"))]
535#jsEgRb|        let vpn_portal_inst = vpn_portal::NullVpnPortal;
536#47DEQp|
537#4QLRSp|        #[cfg(feature = "socks5")]
538#RvhDZt|        let socks5_server = Socks5Server::new(global_ctx.clone(), peer_manager.clone(), None);
539#47DEQp|
540#_mt4YY|        let rpc_server = global_ctx.config.get_rpc_portal().map(|s| {
541#P7OeBD|            StandAloneServer::new(TcpTunnelListener::new(
542#WpIiww|                format!("tcp://{}", s).parse().unwrap(),
543#rDIp7f|            ))
544#NQ-BLD|        });
545#47DEQp|
546#W0e19o|        Instance {
547#BKUYwA|            inst_name: global_ctx.inst_name.clone(),
548#Uu1o0c|            id,
549#47DEQp|
550#1mQK77|            peer_packet_receiver: Arc::new(Mutex::new(peer_packet_receiver)),
551#dJNg24|            nic_ctx: Arc::new(Mutex::new(None)),
552#47DEQp|
553#dw8tKr|            peer_manager,
554#VhQWYC|            listener_manager,
555#I0Rrlw|            conn_manager,
556#yFtD5M|            direct_conn_manager: Arc::new(direct_conn_manager),
557#nAW6UE|            udp_hole_puncher: Arc::new(Mutex::new(udp_hole_puncher)),
558#47DEQp|
559#iacbjf|            ip_proxy: None,
560#Ir4uG6|            kcp_proxy_src: None,
561#P30bk9|            kcp_proxy_dst: None,
562#47DEQp|
563#j5NmjS|            quic_proxy_src: None,
564#7oyUU4|            quic_proxy_dst: None,
565#47DEQp|
566#9qRuJZ|            peer_center,
567#47DEQp|
568#lzBXqu|            vpn_portal: Arc::new(Mutex::new(Box::new(vpn_portal_inst))),
569#47DEQp|
570#3_KqOA|            #[cfg(feature = "socks5")]
571#FixAsK|            socks5_server,
572#47DEQp|
573#f9drFe|            rpc_server,
574#47DEQp|
575#PvFYEj|            global_ctx,
576#KkQWgx|        }
577#KNhneG|    }
578#47DEQp|
579#4Yy50y|    pub fn get_conn_manager(&self) -> Arc<ManualConnectorManager> {
580#r3_Mgw|        self.conn_manager.clone()
581#KNhneG|    }
582#47DEQp|
583#WSoBkr|    async fn add_initial_peers(&mut self) -> Result<(), Error> {
584#z8T3g0|        for peer in self.global_ctx.config.get_peers().iter() {
585#qusht7|            self.get_conn_manager()
586#UVbR8L|                .add_connector_by_url(peer.uri.as_str())
587#DJQPO4|                .await?;
588#KkQWgx|        }
589#sX41Gp|        Ok(())
590#KNhneG|    }
591#47DEQp|
592#2UogTR|    // use a mock nic ctx to consume packets.
593#xABTdI|    async fn clear_nic_ctx(
594#5BZA5f|        arc_nic_ctx: ArcNicCtx,
595#aKDH_7|        packet_recv: Arc<Mutex<PacketRecvChanReceiver>>,
596#5PWdJw|    ) {
597#30t35U|        if let Some(old_ctx) = arc_nic_ctx.lock().await.take() {
598#bUDgN0|            if let Some(dns_runner) = old_ctx.magic_dns {
599#-qMiEw|                dns_runner.dns_runner_cancel_token.cancel();
600#aIUPVk|                tracing::debug!("cancelling dns runner task");
601#7Q0uKb|                let ret = dns_runner.dns_runner_task.await;
602#dM8WjK|                tracing::debug!("dns runner task cancelled, ret: {:?}", ret);
603#hq_fcK|            }
604#8OSK9L|        };
605#47DEQp|
606#W8HYf4|        let mut tasks = JoinSet::new();
607#2pyWyL|        tasks.spawn(async move {
608#RjWL-T|            let mut packet_recv = packet_recv.lock().await;
609#OcolFQ|            while let Ok(packet) = recv_packet_from_chan(&mut packet_recv).await {
610#UFDRhu|                tracing::trace!("packet consumed by mock nic ctx: {:?}", packet);
611#hq_fcK|            }
612#NQ-BLD|        });
613#d0nTd9|        arc_nic_ctx
614#dJo_S-|            .lock()
615#icqUnQ|            .await
616#jJy7CD|            .replace(NicCtxContainer::new_with_any(tasks));
617#47DEQp|
618#5m3CB2|        tracing::debug!("nic ctx cleared.");
619#KNhneG|    }
620#47DEQp|
621#0lvszs|    fn create_magic_dns_runner(
622#dE-qy1|        peer_mgr: Arc<PeerManager>,
623#TFhHQl|        tun_dev: Option<String>,
624#VO2Ds2|        tun_ip: Ipv4Inet,
625#vxOOYI|    ) -> Option<DnsRunner> {
626#IXeuPM|        let ctx = peer_mgr.get_global_ctx();
627#IhdXrW|        if !ctx.config.get_flags().accept_dns {
628#ihJ_TO|            return None;
629#KkQWgx|        }
630#47DEQp|
631#qUSgYL|        let runner = DnsRunner::new(
632#paGBW0|            peer_mgr,
633#fgGBJg|            tun_dev,
634#kb65ll|            tun_ip,
635#tUM033|            MAGIC_DNS_FAKE_IP.parse().unwrap(),
636#bq_4ei|        );
637#MN1FOf|        Some(runner)
638#KNhneG|    }
639#47DEQp|
640#gCwH2G|    async fn use_new_nic_ctx(
641#5BZA5f|        arc_nic_ctx: ArcNicCtx,
642#dWiDiJ|        nic_ctx: NicCtx,
643#T4no7Q|        magic_dns: Option<DnsRunner>,
644#5PWdJw|    ) {
645#72FJGX|        let mut g = arc_nic_ctx.lock().await;
646#Ztr8yR|        *g = Some(NicCtxContainer::new(nic_ctx, magic_dns));
647#AM6aAy|        tracing::debug!("nic ctx updated.");
648#KNhneG|    }
649#47DEQp|
650#CFJcL8|    // Warning, if there is an IP conflict in the network when using DHCP, the IP will be automatically changed.
651#6mvQVP|    fn check_dhcp_ip_conflict(&self) {
652#jLvuzY|        use rand::Rng;
653#AlEPxM|        let peer_manager_c = Arc::downgrade(&self.peer_manager.clone());
654#VoJb5z|        let global_ctx_c = self.get_global_ctx();
655#8rNZKH|        let nic_ctx = self.nic_ctx.clone();
656#_oLoWA|        let _peer_packet_receiver = self.peer_packet_receiver.clone();
657#b9LDnJ|        tokio::spawn(async move {
658#-qr4cg|            let default_ipv4_addr = Ipv4Inet::new(Ipv4Addr::new(10, 126, 126, 0), 24).unwrap();
659#yz0_gd|            let mut current_dhcp_ip: Option<Ipv4Inet> = None;
660#jsIpwb|            let mut next_sleep_time = 0;
661#tRqDHS|            let nic_closed_notifier = Arc::new(Notify::new());
662#elI2Dp|            loop {
663#jUBW0T|                tokio::time::sleep(std::time::Duration::from_secs(next_sleep_time)).await;
664#47DEQp|
665#fmzOYC|                let Some(peer_manager_c) = peer_manager_c.upgrade() else {
666#QlLGsp|                    tracing::warn!("peer manager is dropped, stop dhcp check.");
667#eGvmHB|                    return;
668#JBqzQb|                };
669#47DEQp|
670#qk0lUc|                if nic_closed_notifier.notified().now_or_never().is_some() {
671#pQHvlx|                    tracing::debug!("nic ctx is closed, try recreate it");
672#H_TDIW|                    current_dhcp_ip = None;
673#C_CgiK|                }
674#47DEQp|
675#cADn7t|                // do not allocate ip if no peer connected
676#W_NE21|                let routes = peer_manager_c.list_routes().await;
677#h1t70J|                if routes.is_empty() {
678#QXEZP5|                    next_sleep_time = 1;
679#_01Y-5|                    continue;
680#ZtcDW9|                } else {
681#Yag3tG|                    next_sleep_time = rand::thread_rng().gen_range(5..10);
682#C_CgiK|                }
683#47DEQp|
684#VrpMDQ|                let mut used_ipv4 = HashSet::new();
685#X8M3Yo|                for route in routes {
686#zqGPcf|                    let Some(peer_ipv4_addr) = route.ipv4_addr else {
687#W98xla|                        continue;
688#cAQ1mR|                    };
689#47DEQp|
690#nyU2lH|                    used_ipv4.insert(peer_ipv4_addr.into());
691#C_CgiK|                }
692#47DEQp|
693#pLdSNB|                let dhcp_inet = used_ipv4.iter().next().unwrap_or(&default_ipv4_addr);
694#gE5zvr|                // if old ip is already in this subnet and not conflicted, use it
695#1WWvlm|                if let Some(ip) = current_dhcp_ip {
696#I9V0DO|                    if ip.network() == dhcp_inet.network() && !used_ipv4.contains(&ip) {
697#W98xla|                        continue;
698#uNGRgX|                    }
699#C_CgiK|                }
700#47DEQp|
701#hCtrHk|                // find an available ip in the subnet
702#aNrfr9|                let candidate_ipv4_addr = dhcp_inet.network().iter().find(|ip| {
703#jsnBDw|                    ip.address() != dhcp_inet.first_address()
704#7m1MWB|                        && ip.address() != dhcp_inet.last_address()
705#co0Ibs|                        && !used_ipv4.contains(ip)
706#RDMkP3|                });
707#47DEQp|
708#BmQenu|                if current_dhcp_ip == candidate_ipv4_addr {
709#_01Y-5|                    continue;
710#C_CgiK|                }
711#47DEQp|
712#wBfP8N|                let last_ip = current_dhcp_ip;
713#CW4xPB|                tracing::debug!(
714#inCt5j|                    ?current_dhcp_ip,
715#se6PTf|                    ?candidate_ipv4_addr,
716#26zwEk|                    "dhcp start changing ip"
717#iyW0eL|                );
718#47DEQp|
719#z5k8Ow|                Self::clear_nic_ctx(nic_ctx.clone(), _peer_packet_receiver.clone()).await;
720#47DEQp|
721#FIItvA|                if let Some(ip) = candidate_ipv4_addr {
722#-v1J2u|                    if global_ctx_c.no_tun() {
723#XJaVx-|                        current_dhcp_ip = Some(ip);
724#8whqFG|                        global_ctx_c.set_ipv4(Some(ip));
725#nWRHx7|                        global_ctx_c
726#WSJOWD|                            .issue_event(GlobalCtxEvent::DhcpIpv4Changed(last_ip, Some(ip)));
727#W98xla|                        continue;
728#uNGRgX|                    }
729#47DEQp|
730#vachCR|                    #[cfg(not(any(target_os = "android", target_env = "ohos")))]
731#VoYf0M|                    {
732#XfRcMR|                        let mut new_nic_ctx = NicCtx::new(
733#iKHcnj|                            global_ctx_c.clone(),
734#vVyZ2b|                            &peer_manager_c,
735#ARyIZb|                            _peer_packet_receiver.clone(),
736#8LzzdV|                            nic_closed_notifier.clone(),
737#OatpD-|                        );
738#dJuv0d|                        if let Err(e) = new_nic_ctx.run(Some(ip), global_ctx_c.get_ipv6()).await {
739#afLG1Y|                            tracing::error!(
740#O728vV|                                ?current_dhcp_ip,
741#nQ_Q4O|                                ?candidate_ipv4_addr,
742#W9SJyj|                                ?e,
743#TTnUfl|                                "add ip failed"
744#LPL0Oo|                            );
745#4aPRmy|                            global_ctx_c.set_ipv4(None);
746#XVbxgL|                            continue;
747#qrgf36|                        }
748#n6alTU|                        let ifname = new_nic_ctx.ifname().await;
749#8SMpg3|                        Self::use_new_nic_ctx(
750#HnUf5O|                            nic_ctx.clone(),
751#xFjQKb|                            new_nic_ctx,
752#0A5Nsy|                            Self::create_magic_dns_runner(peer_manager_c.clone(), ifname, ip),
753#Cd3EFV|                        )
754#ElveAT|                        .await;
755#uNGRgX|                    }
756#47DEQp|
757#-P6jOd|                    current_dhcp_ip = Some(ip);
758#8kNnmp|                    global_ctx_c.set_ipv4(Some(ip));
759#OWHKso|                    global_ctx_c.issue_event(GlobalCtxEvent::DhcpIpv4Changed(last_ip, Some(ip)));
760#ZtcDW9|                } else {
761#H_TDIW|                    current_dhcp_ip = None;
762#CHZbbl|                    global_ctx_c.set_ipv4(None);
763#YLWPFJ|                    global_ctx_c.issue_event(GlobalCtxEvent::DhcpIpv4Conflicted(last_ip));
764#C_CgiK|                }
765#hq_fcK|            }
766#NQ-BLD|        });
767#KNhneG|    }
768#47DEQp|
769#qBP9XI|    fn check_for_static_ip(&self, first_round_output: oneshot::Sender<Result<(), Error>>) {
770#SCUtpj|        let ipv4_addr = self.global_ctx.get_ipv4();
771#4v4QcJ|        let ipv6_addr = self.global_ctx.get_ipv6();
772#47DEQp|
773#Ndr9n4|        // Only run if we have at least one IP address (IPv4 or IPv6)
774#NghAie|        if ipv4_addr.is_none() && ipv6_addr.is_none() {
775#z1rpOi|            let _ = first_round_output.send(Ok(()));
776#NhdEjk|            return;
777#KkQWgx|        }
778#47DEQp|
779#8rNZKH|        let nic_ctx = self.nic_ctx.clone();
780#KmTcS8|        let peer_mgr = Arc::downgrade(&self.peer_manager);
781#eUK-pc|        let peer_packet_receiver = self.peer_packet_receiver.clone();
782#47DEQp|
783#b9LDnJ|        tokio::spawn(async move {
784#r-SzFE|            let mut output_tx = Some(first_round_output);
785#elI2Dp|            loop {
786#MONONJ|                let Some(peer_manager) = peer_mgr.upgrade() else {
787#k01Gbu|                    tracing::warn!("peer manager is dropped, stop static ip check.");
788#CSF2L9|                    if let Some(output_tx) = output_tx.take() {
789#dkn1fX|                        let _ = output_tx.send(Err(Error::Unknown));
790#QEZAyt|                        return;
791#uNGRgX|                    }
792#eGvmHB|                    return;
793#JBqzQb|                };
794#47DEQp|
795#S25wh0|                let close_notifier = Arc::new(Notify::new());
796#iLpAjQ|                let mut new_nic_ctx = NicCtx::new(
797#jloGsb|                    peer_manager.get_global_ctx(),
798#rzQdkE|                    &peer_manager,
799#xsmJ98|                    peer_packet_receiver.clone(),
800#rGvzV-|                    close_notifier.clone(),
801#iyW0eL|                );
802#47DEQp|
803#BUCRkg|                if let Err(e) = new_nic_ctx.run(ipv4_addr, ipv6_addr).await {
804#CSF2L9|                    if let Some(output_tx) = output_tx.take() {
805#PFgA6P|                        let _ = output_tx.send(Err(e));
806#QEZAyt|                        return;
807#uNGRgX|                    }
808#5p4i4i|                    tracing::error!("failed to create new nic ctx, err: {:?}", e);
809#1wzcqo|                    tokio::time::sleep(Duration::from_secs(1)).await;
810#_01Y-5|                    continue;
811#C_CgiK|                }
812#k3WJoJ|                let ifname = new_nic_ctx.ifname().await;
813#47DEQp|
814#UDcuB2|                // Create Magic DNS runner only if we have IPv4
815#bYxdXW|                let dns_runner = if let Some(ipv4) = ipv4_addr {
816#6PikpA|                    Self::create_magic_dns_runner(peer_manager, ifname, ipv4)
817#ZtcDW9|                } else {
818#JqjnWy|                    None
819#JBqzQb|                };
820#ZD-V7h|                Self::use_new_nic_ctx(nic_ctx.clone(), new_nic_ctx, dns_runner).await;
821#47DEQp|
822#8KayAt|                if let Some(output_tx) = output_tx.take() {
823#PHGs54|                    let _ = output_tx.send(Ok(()));
824#C_CgiK|                }
825#47DEQp|
826#iNt-FX|                // NOTICE: make sure we do not hold the peer manager here,
827#U9iCDg|                while close_notifier.notified().now_or_never().is_none() {
828#1wzcqo|                    tokio::time::sleep(Duration::from_secs(1)).await;
829#mAu-bf|                    if peer_mgr.strong_count() == 0 {
830#dgQteq|                        tracing::warn!("peer manager is dropped, stop static ip check.");
831#QEZAyt|                        return;
832#uNGRgX|                    }
833#C_CgiK|                }
834#hq_fcK|            }
835#NQ-BLD|        });
836#KNhneG|    }
837#47DEQp|
838#HbUasO|    async fn run_quic_dst(&mut self) -> Result<(), Error> {
839#4BMUF0|        if self.global_ctx.get_flags().disable_quic_input {
840#6WCzfY|            return Ok(());
841#KkQWgx|        }
842#47DEQp|
843#kTRvjE|        let route = Arc::new(self.peer_manager.get_route());
844#k7LP5S|        let quic_dst = QUICProxyDst::new(self.global_ctx.clone(), route)?;
845#dLRxsX|        quic_dst.start().await?;
846#G6v1RW|        self.global_ctx
847#GDZ0dE|            .set_quic_proxy_port(Some(quic_dst.local_addr()?.port()));
848#Y66GYz|        self.quic_proxy_dst = Some(quic_dst);
849#sX41Gp|        Ok(())
850#KNhneG|    }
851#47DEQp|
852#xZKdNb|    pub async fn run(&mut self) -> Result<(), Error> {
853#1KNbtu|        self.listener_manager
854#dJo_S-|            .lock()
855#icqUnQ|            .await
856#T7taUf|            .prepare_listeners()
857#hmaiLi|            .await?;
858#zJGLpz|        self.listener_manager.lock().await.run().await?;
859#UAYZz2|        self.peer_manager.run().await?;
860#47DEQp|
861#KG_Zn6|        Self::clear_nic_ctx(self.nic_ctx.clone(), self.peer_packet_receiver.clone()).await;
862#47DEQp|
863#ISndmD|        if !self.global_ctx.config.get_flags().no_tun {
864#V2FGZF|            #[cfg(not(any(target_os = "android", target_env = "ohos")))]
865#eP03rc|            {
866#fzB6B-|                let (output_tx, output_rx) = oneshot::channel();
867#bjHtZI|                self.check_for_static_ip(output_tx);
868#Ymg5ZK|                output_rx.await.unwrap()?;
869#hq_fcK|            }
870#KkQWgx|        }
871#47DEQp|
872#_ET3LU|        if self.global_ctx.config.get_dhcp() {
873#t2bWOK|            self.check_dhcp_ip_conflict();
874#KkQWgx|        }
875#47DEQp|
876#sGr8lh|        if self.global_ctx.get_flags().enable_kcp_proxy {
877#8P25AQ|            let src_proxy = KcpProxySrc::new(self.get_peer_manager()).await;
878#vDI5JQ|            src_proxy.start().await;
879#gncCm0|            self.kcp_proxy_src = Some(src_proxy);
880#KkQWgx|        }
881#47DEQp|
882#EiwuWF|        if !self.global_ctx.get_flags().disable_kcp_input {
883#MxzdMf|            let mut dst_proxy = KcpProxyDst::new(self.get_peer_manager()).await;
884#5HN4pq|            dst_proxy.start().await;
885#MsvW2d|            self.kcp_proxy_dst = Some(dst_proxy);
886#KkQWgx|        }
887#47DEQp|
888#qQ_AOS|        if self.global_ctx.get_flags().enable_quic_proxy {
889#mcQ2cl|            let quic_src = QUICProxySrc::new(self.get_peer_manager()).await;
890#xZntYI|            quic_src.start().await;
891#Ir2gJn|            self.quic_proxy_src = Some(quic_src);
892#KkQWgx|        }
893#47DEQp|
894#r5jXI_|        if !self.global_ctx.get_flags().disable_quic_input {
895#jIgZXB|            if let Err(e) = self.run_quic_dst().await {
896#gCEV1K|                eprintln!(
897#oYbvni|                    "quic input start failed: {:?} (some platforms may not support)",
898#Hnsyov|                    e
899#iyW0eL|                );
900#hq_fcK|            }
901#KkQWgx|        }
902#47DEQp|
903#G6v1RW|        self.global_ctx
904#iscZip|            .get_acl_filter()
905#gA5jAm|            .reload_rules(AclRuleBuilder::build(&self.global_ctx)?.as_ref());
906#47DEQp|
907#4vbLVZ|        // run after tun device created, so listener can bind to tun device, which may be required by win 10
908#pcuz8m|        self.ip_proxy = Some(IpProxy::new(
909#GWiuO2|            self.get_global_ctx(),
910#YKJnRz|            self.get_peer_manager(),
911#r0SYoC|        )?);
912#6GUbqB|        self.run_ip_proxy().await?;
913#47DEQp|
914#TAzS4c|        self.udp_hole_puncher.lock().await.run().await?;
915#47DEQp|
916#6YFYOP|        self.peer_center.init().await;
917#MvW_h-|        let route_calc = self.peer_center.get_cost_calculator();
918#Yavkf7|        self.peer_manager
919#Y6skBZ|            .get_route()
920#ZMnG8A|            .set_route_cost_fn(route_calc)
921#agFGm5|            .await;
922#47DEQp|
923#-NAjqc|        self.add_initial_peers().await?;
924#47DEQp|
925#urBnUU|        if self.global_ctx.get_vpn_portal_cidr().is_some() {
926#TL--pq|            self.run_vpn_portal().await?;
927#KkQWgx|        }
928#47DEQp|
929#4QLRSp|        #[cfg(feature = "socks5")]
930#HwMDAJ|        self.socks5_server
931#X6ISs9|            .run(
932#GCKXbV|                self.kcp_proxy_src
933#__E1rg|                    .as_ref()
934#NlN7S-|                    .map(|x| Arc::downgrade(&x.get_kcp_endpoint())),
935#jOhom-|            )
936#hmaiLi|            .await?;
937#47DEQp|
938#Ba_4tM|        self.run_rpc_server().await?;
939#47DEQp|
940#sX41Gp|        Ok(())
941#KNhneG|    }
942#47DEQp|
943#KZ1HmO|    pub async fn run_ip_proxy(&mut self) -> Result<(), Error> {
944#SekgpM|        if self.ip_proxy.is_none() {
945#hsE5lI|            return Err(anyhow::anyhow!("ip proxy not enabled.").into());
946#KkQWgx|        }
947#hlinIA|        self.ip_proxy.as_ref().unwrap().start().await?;
948#sX41Gp|        Ok(())
949#KNhneG|    }
950#47DEQp|
951#6fBJne|    pub async fn run_vpn_portal(&mut self) -> Result<(), Error> {
952#hpcO-p|        if self.global_ctx.get_vpn_portal_cidr().is_none() {
953#hjGw9u|            return Err(anyhow::anyhow!("vpn portal cidr not set.").into());
954#KkQWgx|        }
955#dskYEq|        self.vpn_portal
956#dJo_S-|            .lock()
957#icqUnQ|            .await
958#xyH5zM|            .start(self.get_global_ctx(), self.get_peer_manager())
959#hmaiLi|            .await?;
960#sX41Gp|        Ok(())
961#KNhneG|    }
962#47DEQp|
963#BKR7Ql|    pub fn get_peer_manager(&self) -> Arc<PeerManager> {
964#2Pvyau|        self.peer_manager.clone()
965#KNhneG|    }
966#47DEQp|
967#FiOS6u|    pub async fn close_peer_conn(
968#HmXHKy|        &mut self,
969#ECN-1O|        peer_id: PeerId,
970#uPXWFD|        conn_id: &PeerConnId,
971#Ew16UP|    ) -> Result<(), Error> {
972#Yavkf7|        self.peer_manager
973#zy2jK-|            .get_peer_map()
974#rsE5vT|            .close_peer_conn(peer_id, conn_id)
975#hmaiLi|            .await?;
976#sX41Gp|        Ok(())
977#KNhneG|    }
978#47DEQp|
979#ExwZ5e|    pub async fn wait(&self) {
980#3rrjR7|        self.peer_manager.wait().await;
981#KNhneG|    }

...26873 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d7b62ba001EuKNAoXiOzifC8
Use Grep to search the full content or Read with offset/limit to view specific sections.
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/utils.rs","limit":50}) [completed]
1#TZU_WO|use std::{fs::OpenOptions, str::FromStr};
2#47DEQp|
3#woeOSu|use anyhow::Context;
4#1j3GBC|use tracing::level_filters::LevelFilter;
5#PSmYQT|use tracing_subscriber::{
6#NZNTXF|    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
7#3zmC_W|};
8#47DEQp|
9#DHI-Wd|use crate::common::{
10#2NRlFh|    config::LoggingConfigLoader, get_logger_timer_rfc3339, tracing_rolling_appender::*,
11#3zmC_W|};
12#47DEQp|
13#093jCP|pub type PeerRoutePair = crate::proto::cli::PeerRoutePair;
14#47DEQp|
15#3tkHoo|pub fn cost_to_str(cost: i32) -> String {
16#I7SqrU|    if cost == 1 {
17#N2z58F|        "p2p".to_string()
18#1iXXq0|    } else {
19#UY-PcW|        format!("relay({})", cost)
20#KNhneG|    }
21#0Qs2qn|}
22#47DEQp|
23#z3q8eF|pub fn float_to_str(f: f64, precision: usize) -> String {
24#TtmnGc|    format!("{:.1$}", f, precision)
25#0Qs2qn|}
26#47DEQp|
27#QleuZT|pub type NewFilterSender = std::sync::mpsc::Sender<String>;
28#47DEQp|
29#oPL1v_|pub fn init_logger(
30#Z4EVd5|    config: impl LoggingConfigLoader,
31#Uoe6-c|    need_reload: bool,
32#GU-JHU|) -> Result<Option<NewFilterSender>, anyhow::Error> {
33#PcsNA5|    use crate::instance::logger_rpc_service::{CURRENT_LOG_LEVEL, LOGGER_LEVEL_SENDER};
34#47DEQp|
35#iEK46F|    let file_config = config.get_file_logger_config();
36#hZefsZ|    let file_level = file_config
37#ov7MtD|        .level
38#mWZT2p|        .map(|s| s.parse().unwrap())
39#dFG-HC|        .unwrap_or(LevelFilter::OFF);
40#47DEQp|
41#8NDxn5|    let mut ret_sender: Option<NewFilterSender> = None;
42#47DEQp|
43#lx-n3n|    // logger to rolling file
44#sYmaZ5|    let mut file_layer = None;
45#2uaTy2|    if file_level != LevelFilter::OFF || need_reload {
46#eJphwD|        let mut l = tracing_subscriber::fmt::layer();
47#O5WZga|        l.set_ansi(false);
48#SnFtGu|        let file_filter = EnvFilter::builder()
49#5KswEr|            .with_default_directive(file_level.into())
50#bkiW9A|            .from_env()
51#e2ZiLF|            .with_context(|| "failed to create file filter")?;
52#uK9YYq|        let (file_filter, file_filter_reloader) =
53#SSshq0|            tracing_subscriber::reload::Layer::new(file_filter);
54#47DEQp|
55#aIR2iX|        if need_reload {
56#TqSA16|            let (sender, recver) = std::sync::mpsc::channel();
57#V4dIE_|            ret_sender = Some(sender.clone());
58#47DEQp|
59#CTMIji|            // 初始化全局状态
60#4r-tml|            let _ = LOGGER_LEVEL_SENDER.set(std::sync::Mutex::new(sender));
61#xaW3kr|            let _ = CURRENT_LOG_LEVEL.set(std::sync::Mutex::new(file_level.to_string()));
62#47DEQp|
63#urRuWv|            std::thread::spawn(move || {
64#iQo-lW|                println!("Start log filter reloader");
65#Rci45i|                while let Ok(lf) = recver.recv() {
66#s06MxD|                    let e = file_filter_reloader.modify(|f| {
67#u5VpRP|                        if let Ok(nf) = EnvFilter::builder()
68#g8lC3L|                            .with_default_directive(lf.parse::<LevelFilter>().unwrap().into())
69#i9dT1a|                            .from_env()
70#91V51w|                            .with_context(|| "failed to create file filter")
71#Fzqoyl|                        {
72#E6x57w|                            println!("Reload log filter succeed, new filter level: {:?}", lf);
73#RVeTzk|                            *f = nf;
74#qrgf36|                        }
75#x8RDhW|                    });
76#XU3e4R|                    if e.is_err() {
77#oD3Xf-|                        println!("Failed to reload log filter: {:?}", e);
78#uNGRgX|                    }
79#C_CgiK|                }
80#BnF8In|                println!("Stop log filter reloader");
81#-JdJBA|            });
82#KkQWgx|        }
83#47DEQp|
84#R9lKMQ|        let dir = file_config.dir.as_deref().unwrap_or(".");
85#2dWrDR|        let file = file_config.file.as_deref().unwrap_or("easytier.log");
86#H2kvyE|        let path = std::path::Path::new(dir).join(file);
87#QqddKh|        let path_str = path.to_string_lossy().into_owned();
88#47DEQp|
89#NwpngO|        let builder = RollingFileAppenderBase::builder();
90#jfhWXV|        let file_appender = builder
91#T4JTWr|            .filename(path_str)
92#rIDikj|            .condition_daily()
93#bk3EXD|            .max_filecount(file_config.count.unwrap_or(10))
94#SpeylA|            .condition_max_file_size(file_config.size_mb.unwrap_or(100) * 1024 * 1024)
95#tueSmz|            .build()
96#WEGyMY|            .unwrap();
97#47DEQp|
98#O1zYjE|        let wrapper = FileAppenderWrapper::new(file_appender);
99#47DEQp|
100#9mCnGt|        // Create a simple wrapper that implements MakeWriter
101#mfpfii|        file_layer = Some(
102#DGWZYq|            l.with_writer(wrapper)
103#DizRMt|                .with_timer(get_logger_timer_rfc3339())
104#idtqdd|                .with_filter(file_filter),
105#bq_4ei|        );
106#KNhneG|    }
107#47DEQp|
108#M9XOTd|    // logger to console
109#5d0KFv|    let console_config = config.get_console_logger_config();
110#PBc5ah|    let console_level = console_config
111#ov7MtD|        .level
112#mWZT2p|        .map(|s| s.parse().unwrap())
113#dFG-HC|        .unwrap_or(LevelFilter::OFF);
114#47DEQp|
115#Mf-Dsl|    let console_filter = EnvFilter::builder()
116#QwFcV9|        .with_default_directive(console_level.into())
117#2fm9KY|        .from_env()
118#Du2ckX|        .unwrap();
119#47DEQp|
120#NSd_cK|    let console_layer = tracing_subscriber::fmt::layer()
121#E-xD1a|        .pretty()
122#yPvrq0|        .with_timer(get_logger_timer_rfc3339())
123#K1h8fv|        .with_writer(std::io::stderr)
124#BoDzRN|        .with_filter(console_filter);
125#47DEQp|
126#Ly4v4g|    let registry = Registry::default();
127#47DEQp|
128#yK289h|    #[cfg(not(feature = "tracing"))]
129#DT0vWo|    {
130#kuSUqw|        registry.with(console_layer).with(file_layer).init();
131#KNhneG|    }
132#47DEQp|
133#Zv_3gk|    #[cfg(feature = "tracing")]
134#DT0vWo|    {
135#OyUU_Y|        let console_subscriber_layer = console_subscriber::ConsoleLayer::builder().spawn();
136#kjM7cN|        registry
137#aZWjI-|            .with(console_layer)
138#RvV4jb|            .with(file_layer)
139#W6-evN|            .with(console_subscriber_layer)
140#69UnR2|            .init();
141#KNhneG|    }
142#47DEQp|
143#vIFgXK|    Ok(ret_sender)
144#0Qs2qn|}
145#47DEQp|
146#QwXpHu|#[cfg(target_os = "windows")]
147#M9oOGR|pub fn utf8_or_gbk_to_string(s: &[u8]) -> String {
148#sGP7co|    use encoding::{all::GBK, DecoderTrap, Encoding};
149#bA46Vt|    if let Ok(utf8_str) = String::from_utf8(s.to_vec()) {
150#ULJnHE|        utf8_str
151#1iXXq0|    } else {
152#05PR8t|        // 如果解码失败，则尝试使用GBK解码
153#QdUq3x|        if let Ok(gbk_str) = GBK.decode(s, DecoderTrap::Strict) {
154#3gAAiS|            gbk_str
155#d4ha1c|        } else {
156#1gidW5|            String::from_utf8_lossy(s).to_string()
157#KkQWgx|        }
158#KNhneG|    }
159#0Qs2qn|}
160#47DEQp|
161#B4bxfc|thread_local! {
162#eiheXd|    static PANIC_COUNT : std::cell::RefCell<u32> = const { std::cell::RefCell::new(0) };
163#0Qs2qn|}
164#47DEQp|
165#djpxYs|pub fn setup_panic_handler() {
166#9SqYFN|    use std::backtrace;
167#79BY0u|    use std::io::Write;
168#cYUpr4|    std::panic::set_hook(Box::new(|info| {
169#1GT7Ia|        PANIC_COUNT.with(|c| {
170#EwjdgP|            let mut count = c.borrow_mut();
171#4ebzT8|            *count += 1;
172#NQ-BLD|        });
173#al0RcU|        let panic_count = PANIC_COUNT.with(|c| *c.borrow());
174#mXDRNE|        if panic_count > 1 {
175#PR4bAR|            println!("panic happened more than once, exit immediately");
176#_AElVY|            std::process::exit(1);
177#KkQWgx|        }
178#47DEQp|
179#KbgbvK|        let payload = info.payload();
180#srIgxp|        let payload_str: Option<&str> = if let Some(s) = payload.downcast_ref::<&str>() {
181#fdXmom|            Some(s)
182#cjsZ_c|        } else if let Some(s) = payload.downcast_ref::<String>() {
183#fdXmom|            Some(s)
184#d4ha1c|        } else {
185#T3LJbU|            None
186#8OSK9L|        };
187#zjgMxA|        let payload_str = payload_str.unwrap_or("<unknown panic info>");
188#yxlvGi|        // The current implementation always returns `Some`.
189#68krQ1|        let location = info.location().unwrap();
190#vWfNWj|        let thread = std::thread::current();
191#OI1DbK|        let thread = thread.name().unwrap_or("<unnamed>");
192#47DEQp|
193#GGV9Sr|        let tmp_path = std::env::temp_dir().join("easytier-panic.log");
194#MppD01|        let candidate_path = [
195#4gkq30|            std::path::PathBuf::from_str("easytier-panic.log").ok(),
196#BjDXNE|            Some(tmp_path),
197#f9uDyt|        ];
198#FiWqnc|        let mut file = None;
199#M1WwCu|        let mut file_path = None;
200#zslgm2|        for path in candidate_path.iter().filter_map(|p| p.clone()) {
201#pO7SSS|            file = OpenOptions::new()
202#4TX9og|                .create(true)
203#Fusb0W|                .append(true)
204#eXMotD|                .open(path.clone())
205#r6b1kw|                .ok();
206#DY3la8|            if file.is_some() {
207#D0dMZg|                file_path = Some(path);
208#FA_X37|                break;
209#hq_fcK|            }
210#KkQWgx|        }
211#47DEQp|
212#wlVBzO|        println!("{}", rust_i18n::t!("core_app.panic_backtrace_save"));
213#47DEQp|
214#Dmn3Un|        // write str to stderr & file
215#2dcVUW|        let write_err = |s: String| {
216#9kbnA2|            let mut stderr = std::io::stderr();
217#uFlkQc|            let content = format!("{}: {}", chrono::Local::now(), s);
218#sMC4Yn|            let _ = writeln!(stderr, "{}", content);
219#cL9jQA|            if let Some(mut f) = file.as_ref() {
220#pUE5Mf|                let _ = writeln!(f, "{}", content);
221#hq_fcK|            }
222#8OSK9L|        };
223#47DEQp|
224#9rS27d|        write_err("panic occurred, if this is a bug, please report this issue on github (https://github.com/easytier/easytier/issues)".to_string());
225#SlfkoC|        write_err(format!("easytier version: {}", crate::VERSION));
226#s5Ia4z|        write_err(format!("os version: {}", std::env::consts::OS));
227#zNGMCv|        write_err(format!("arch: {}", std::env::consts::ARCH));
228#4M1fCs|        write_err(format!(
229#LkSVFu|            "panic is recorded in: {}",
230#Te_3UI|            file_path
231#bfvHc4|                .and_then(|p| p.to_str().map(|x| x.to_string()))
232#vsnw1g|                .unwrap_or("<no file>".to_string())
233#XNrhe2|        ));
234#k1b2FS|        write_err(format!("thread: {}", thread));
235#ZL4hAR|        write_err(format!("time: {}", chrono::Local::now()));
236#uui_Wy|        write_err(format!("location: {}", location));
237#6E5Cht|        write_err(format!("panic info: {}", payload_str));
238#47DEQp|
239#KHw2PN|        // backtrace is risky, so use it last
240#Y_F_6m|        let backtrace = backtrace::Backtrace::force_capture();
241#HetN4z|        write_err(format!("backtrace: {:#?}", backtrace));
242#47DEQp|
243#f8L8tU|        std::process::exit(1);
244#HeCT59|    }));
245#0Qs2qn|}
246#47DEQp|
247#Uj8kHs|pub fn check_tcp_available(port: u16) -> bool {
248#RFx7GB|    use std::net::TcpListener;
249#m-m-oP|    let s = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port);
250#9Afb5k|    TcpListener::bind(s).is_ok()
251#0Qs2qn|}
252#47DEQp|
253#9OHYyr|pub fn find_free_tcp_port(mut range: std::ops::Range<u16>) -> Option<u16> {
254#X-nMvM|    range.find(|&port| check_tcp_available(port))
255#0Qs2qn|}
256#47DEQp|
257#O6JvtA|#[cfg(test)]
258#FQleaY|mod tests {
259#TyIp5_|    use crate::common::config::{self};
260#47DEQp|
261#6ffN3e|    use super::*;
262#47DEQp|
263#ZaEm3G|    async fn test_logger_reload() {
264#tkSWhp|        println!("current working dir: {:?}", std::env::current_dir());
265#9MG76o|        let config = config::LoggingConfigBuilder::default().build().unwrap();
266#-d_DQf|        let s = init_logger(&config, true).unwrap();
267#KYVJvw|        tracing::debug!("test not display debug");
268#ulZSPb|        s.unwrap().send(LevelFilter::DEBUG.to_string()).unwrap();
269#3A-exV|        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
270#F8PoWg|        tracing::debug!("test display debug");
271#KNhneG|    }
272#0Qs2qn|}
273#47DEQp|

## [assistant] +058391-06-27T16:59:22.000Z
[thinking] Now I have a clear picture of all the changes made. Let me summarize what needs to be updated in the README:

1. **entrypoint.sh** - Now has auto-add admin IP to whitelist logic (lines 28-36). The INSERT includes `comment, created_by, created_at` fields, and there's an UPDATE for NULL hostname fix.

2. **agent-entrypoint.sh** - `RUST_LOG` changed from `easytier=warn` to `easytier=error` (line 8).

3. **instance.rs** - `close_peer` replaced `close_peer_conn` in 4 places (lines 339, 363, 409, 436). There are also many debug log lines that were added during investigation.

4. **utils.rs** - Console layer now uses `.with_timer(get_logger_timer_rfc3339())` instead of `.without_time()`.

Now let me analyze the README to see what needs updating:

Looking at the README:

- **"修复的 Bug" section (line 241-256)**: This already lists 6 bugs that were fixed. The bug about `close_peer_conn` vs `close_peer` (multi-connection cleanup) is listed as bug #6. The entrypoint INSERT fix isn't listed. The RUST_LOG change isn't mentioned. The timer rollback isn't mentioned.

Let me check if I should update README. Let me look at the README sections more carefully:

1. **IP白名单机制** section (line 215-280): Describes whitelist mechanism. The "修复的 Bug" subsection (241-256) lists 6 bugs fixed. The entrypoint auto-add behavior could be mentioned here.

2. **Agent环境变量** section (line 379-388): Lists agent env vars. `RUST_LOG` is now set to `easytier=error` by default in agent-entrypoint.sh, but this isn't documented.

3. **启动流程** (line 368-377): Agent startup flow. This already mentions "入口脚本读取 core.toml 的 ipv4，自动推算 admin IP（.1 后缀）". 

Let me check what needs to be updated more carefully. Actually, I think the user's analyst was in the middle of analyzing what README updates are needed. Let me provide that analysis to the user.
--- grep({"pattern":"(修复|修复了|fixed|bug|Bug)","include":"*.md"}) [completed]
Found 295 matches (showing first 100)
/mnt/nvme2t_2/easytier/.flowcraft/logs/coder-ses_172a053e2ffefhkghG2wOg4Sm7-2026-06-03T12-05-49-613Z.md:
  Line 2: # Task: ## 任务：修复 entrypoint.sh 的 INSERT OR IGNORE 缺少 NOT NULL 字段

  Line 85: 之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

  Line 105: [2026-06-03T11:32:07.280Z] @coder: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

  Line 118:   → 现在我将应用所有 4 处修复，将 `close_peer_conn` 替换为 `close_peer`：

  Line 123: 所有 4 处位置均已正确修复。结构干净——`close_peer` 取代了旧的 `if let Some(conn_id) { close_peer_conn }` 模式，且不再有静默失败问题。

  Line 181: ## 任务：修复 entrypoint.sh 的 INSERT OR IGNORE 缺少 NOT NULL 字段


/mnt/nvme2t_2/easytier/.flowcraft/logs/analyst-ses_172a535f1ffesh0FSLtcaivI1t-2026-06-03T12-05-02-748Z.summary.md:
  Line 35: 在随后的重建（包括您修复的 hostname INSERT）中，BUG #1 阻止了新行的插入。数据库仍然有旧行（因为 `/data` 被挂载）→ `hostname` 永远保持 `NULL`。


/mnt/nvme2t_2/easytier/.flowcraft/logs/analyst-ses_172a535f1ffesh0FSLtcaivI1t-2026-06-03T12-05-02-748Z.md:
  Line 21: [2026-06-03T11:03:16.058Z] @coder: ## 任务：修复 event-driven 任务的 IP 比较格式 bug

  Line 26: ### 修复

  Line 36: The periodic task (line 322) already uses `.address()` — confirmed from the earlier grep which only found one match for `format!("{}", ip)` (the one we just fixed

  Line 38: [2026-06-03T11:11:16.395Z] @analyst: ## 任务：分析并规划两个问题的修复方案

  Line 53: ## 问题 1 修复方案

  Line 95: 之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

  Line 115: [2026-06-03T11:32:07.280Z] @coder: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

  Line 128:   → 现在我将应用所有 4 处修复，将 `close_peer_conn` 替换为 `close_peer`：

  Line 133: 所有 4 处位置均已正确修复。结构干净——`close_peer` 取代了旧的 `if let Some(conn_id) { close_peer_conn }` 模式，且不再有静默失败问题。

  Line 685: 443#Ee5bn0|                                                tracing::debug!("Event-driven: peer {} has route info but ipv4_addr is None (iteration {})", peer_id, iteration);

  Line 745: 503#0cPbvw|                                tracing::debug!("Reported {} peers to admin, status: {}", count, r.status_code());

  Line 842: 600#aIUPVk|                tracing::debug!("cancelling dns runner task");

  Line 844: 602#dM8WjK|                tracing::debug!("dns runner task cancelled, ret: {:?}", ret);

  Line 860: 618#5m3CB2|        tracing::debug!("nic ctx cleared.");

  Line 889: 647#AM6aAy|        tracing::debug!("nic ctx updated.");

  Line 913: 671#pQHvlx|                    tracing::debug!("nic ctx is closed, try recreate it");

  Line 955: 713#CW4xPB|                tracing::debug!(

  Line 1334: 29#Irva_7|#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]

  Line 1365: 60#P-GtIR|#[derive(Debug, Clone)]

  Line 1416: 111#U3NXHB|impl std::fmt::Debug for GlobalCtx {

  Line 1418: 113#-uunom|        f.debug_struct("GlobalCtx")

  Line 2285: 12#zundQZ|#[derive(Debug, Serialize)]

  Line 2373: 15#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2379: 21#zundQZ|#[derive(Debug, Serialize)]

  Line 2389: 31#zundQZ|#[derive(Debug, Serialize)]

  Line 2394: 36#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2436: 78#vz5LgR|        created_at: Set(Utc::now().fixed_offset()),

  Line 2470: 112#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2520: 11#MmJkO6|#[derive(Debug, Clone, Serialize, Deserialize)]

  Line 2527: 18#zundQZ|#[derive(Debug, Serialize)]

  Line 2532: 23#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2939: 15#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2946: 22#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2951: 27#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2958: 34#zundQZ|#[derive(Debug, Serialize)]

  Line 2970: 46#zundQZ|#[derive(Debug, Serialize)]

  Line 3009: 85#ahXxsU|    let now = Utc::now().fixed_offset();

  Line 3076: 152#D-apIG|            .fixed_offset(),

  Line 3079: 155#zAOyKP|    agent.updated_at = Set(Utc::now().fixed_offset());

  Line 3191: 3#980-z7|#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]

  Line 3203: 15#yVJwY9|#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]

  Line 3427: 在随后的重建（包括您修复的 hostname INSERT）中，BUG #1 阻止了新行的插入。数据库仍然有旧行（因为 `/data` 被挂载）→ `hostname` 永远保持 `NULL`。

  Line 3471: # 7. hostname 修复后的验证：


/mnt/nvme2t_2/easytier/.flowcraft/logs/analyst-ses_172a7127fffe4o770h2iES8oga-2026-06-03T12-00-22-482Z.summary.md:
  Line 37: ## 修复方案

  Line 55: **选项 B（更完整）：检查并修复 `WHITELIST_SYNC_URL`**


/mnt/nvme2t_2/easytier/.flowcraft/logs/analyst-ses_172a7127fffe4o770h2iES8oga-2026-06-03T12-00-22-482Z.md:
  Line 38: ## 修复方案

  Line 46: [2026-06-03T11:03:16.058Z] @coder: ## 任务：修复 event-driven 任务的 IP 比较格式 bug

  Line 51: ### 修复

  Line 61: The periodic task (line 322) already uses `.address()` — confirmed from the earlier grep which only found one match for `format!("{}", ip)` (the one we just fixed

  Line 63: [2026-06-03T11:11:16.395Z] @analyst: ## 任务：分析并规划两个问题的修复方案

  Line 78: ## 问题 1 修复方案

  Line 120: 之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

  Line 140: [2026-06-03T11:32:07.280Z] @coder: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

  Line 153:   → 现在我将应用所有 4 处修复，将 `close_peer_conn` 替换为 `close_peer`：

  Line 158: 所有 4 处位置均已正确修复。结构干净——`close_peer` 取代了旧的 `if let Some(conn_id) { close_peer_conn }` 模式，且不再有静默失败问题。

  Line 205: ## 修复方案

  Line 663: 443#Ee5bn0|                                                tracing::debug!("Event-driven: peer {} has route info but ipv4_addr is None (iteration {})", peer_id, iteration);

  Line 723: 503#0cPbvw|                                tracing::debug!("Reported {} peers to admin, status: {}", count, r.status_code());

  Line 820: 600#aIUPVk|                tracing::debug!("cancelling dns runner task");

  Line 822: 602#dM8WjK|                tracing::debug!("dns runner task cancelled, ret: {:?}", ret);

  Line 838: 618#5m3CB2|        tracing::debug!("nic ctx cleared.");

  Line 867: 647#AM6aAy|        tracing::debug!("nic ctx updated.");

  Line 891: 671#pQHvlx|                    tracing::debug!("nic ctx is closed, try recreate it");

  Line 933: 713#CW4xPB|                tracing::debug!(

  Line 1358: 29#Irva_7|#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]

  Line 1389: 60#P-GtIR|#[derive(Debug, Clone)]

  Line 1440: 111#U3NXHB|impl std::fmt::Debug for GlobalCtx {

  Line 1442: 113#-uunom|        f.debug_struct("GlobalCtx")

  Line 2402: 12#zundQZ|#[derive(Debug, Serialize)]

  Line 2447: 15#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2453: 21#zundQZ|#[derive(Debug, Serialize)]

  Line 2463: 31#zundQZ|#[derive(Debug, Serialize)]

  Line 2468: 36#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2510: 78#vz5LgR|        created_at: Set(Utc::now().fixed_offset()),

  Line 2544: 112#gqPAVP|#[derive(Debug, Deserialize)]

  Line 2647: ## 修复方案

  Line 2665: **选项 B（更完整）：检查并修复 `WHITELIST_SYNC_URL`**


/mnt/nvme2t_2/easytier/.flowcraft/logs/coder-ses_172c20779ffeT3xvr4uKOOKKTZ-2026-06-03T11-32-07-279Z.md:
  Line 2: # Task: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

  Line 52: [2026-06-02T16:56:21.236Z] @coder: ## 任务：验证 README.md 修复后内容并提交到 GitHub

  Line 59: #### 1. 验证修复（5 个 grep 检查）

  Line 91: [2026-06-03T11:03:16.058Z] @coder: ## 任务：修复 event-driven 任务的 IP 比较格式 bug

  Line 96: ### 修复

  Line 106: The periodic task (line 322) already uses `.address()` — confirmed from the earlier grep which only found one match for `format!("{}", ip)` (the one we just fixed

  Line 108: [2026-06-03T11:11:16.395Z] @analyst: ## 任务：分析并规划两个问题的修复方案

  Line 123: ## 问题 1 修复方案

  Line 165: 之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

  Line 185: ## 任务：修复 agent 间歇性放包 — 用 close_peer 替代 close_peer_conn

  Line 678: 443#Ee5bn0|                                                tracing::debug!("Event-driven: peer {} has route info but ipv4_addr is None (iteration {})", peer_id, iteration);

  Line 738: 503#0cPbvw|                                tracing::debug!("Reported {} peers to admin, status: {}", count, r.status_code());

  Line 835: 600#aIUPVk|                tracing::debug!("cancelling dns runner task");

  Line 837: 602#dM8WjK|                tracing::debug!("dns runner task cancelled, ret: {:?}", ret);

  Line 853: 618#5m3CB2|        tracing::debug!("nic ctx cleared.");

  Line 882: 647#AM6aAy|        tracing::debug!("nic ctx updated.");


(Results truncated: showing 100 of 295 matches (195 hidden). Consider using a more specific path or pattern.)

## [assistant] +058391-06-27T19:05:43.000Z
[thinking] OK so this is a lot of context. Let me now analyze what README changes are needed based on my understanding of all 4 fixes:

1. **entrypoint.sh fix**: admin container now auto-adds its own IP to whitelist on startup with `INSERT OR IGNORE ... (ip, hostname, comment, created_by, created_at)` + `UPDATE ... WHERE hostname IS NULL`. This solves the `[BIND_DEBUG]` issue.

2. **RUST_LOG change**: agent-entrypoint.sh now uses `easytier=error` instead of `easytier=warn`.

3. **instance.rs fix**: `close_peer` replaces `close_peer_conn` in 4 places to fix the multi-connection cleanup bug.

4. **utils.rs fix**: Console logger reverted from `.without_time()` to `.with_timer(get_logger_timer_rfc3339())`.

Now, analyzing the README:

**Section "IP 白名单机制 > 修复的 Bug"** (lines 241-256): Lists 6 bugs. Should add:
- Bug #7: entrypoint.sh INSERT OR IGNORE missing NOT NULL fields (created_by, created_at) causing silent failure

**Section "IP 白名单机制 > 工作原理"** (lines 267-275): Could mention that admin entrypoint.sh auto-adds admin IP to whitelist on startup.

**Section "Agent 节点部署 > 环境变量"** (lines 379-388): Could mention `RUST_LOG` default is now `easytier=error`.

**Section "常见问题（FAQ）"** (lines 513-531): Could add an entry about the `[BIND_DEBUG]` log.

Actually, looking at this from the user's perspective, the analyst task was to analyze what needs updating. Let me produce a clear analysis.
## README 更新分析

### 需要更新的章节

| 章节 | 行号 | 内容 | 原因 |
|------|------|------|------|
| **IP 白名单机制 > 工作原理** | 267-275 | 新增"entrypoint.sh 自动将 admin 自身 IP 写入白名单" | admin 启动时自动 INSERT OR IGNORE + UPDATE NULL hostname |
| **IP 白名单机制 > 修复的 Bug** | 241-256 | 新增 Bug #7：`INSERT OR IGNORE` 缺少 `created_by`/`created_at` NOT NULL 字段导致静默失败 | 这是本次修复的核心问题之一 |
| **Agent 节点部署 > 环境变量** | 379-388 | 新增 `RUST_LOG` 行或注明默认值 `easytier=error` | agent-entrypoint.sh 第 8 行默认值已变更 |
| **常见问题（FAQ）** | 513-531 | 新增 FAQ：`[BIND_DEBUG]` 日志含义及排查 | 用户在生产中遇到此问题 |

### 不需要更新的章节
- **核心特性**、**架构说明**、**部署步骤**、**目录结构**、**构建镜像** — 无变化
- **Agent 启动流程**（行 368-377）— 已正确描述自动推算 admin IP 的行为

### 建议新增内容
- **IP 白名单机制** 下新增小节："Admin 自动注册自身 IP" — 说明 admin 容器启动时从 `core.toml` 读取 ipv4，`INSERT OR IGNORE` 写入白名单 + `UPDATE` 修复旧记录 NULL hostname
- **Agent 日志** 注明 `RUST_LOG=easytier=error` 只输出 ERROR 级别（不含 Format 1 WARN 日志）
