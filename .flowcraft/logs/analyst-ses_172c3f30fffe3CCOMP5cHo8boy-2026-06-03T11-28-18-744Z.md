# Agent: analyst
# Task: ## 任务：重新排查 agent 间歇性放包的根因

### 背景
agent (10.0.210.253) 对非白名单 client 的 ping 阻断不彻底——约每 7 个包有 1 个能通过。admin (10.0.210.1) 则 100% 阻断。

之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

### 关键事实
1. agent 日志**完全没有** `"peer is blocked by IP whitelist"` 错误 → `is_peer_blocked` 在 `add_new_peer_conn` 中从未返回 true
2. agent 日志**反复出现** `new connection accepted` + `new peer connection added` 同一个 peer_id（非白名单 peer 不断被接受）
3. admin 的 `add_new_peer_conn` 能 100% 阻断非白名单 peer
4. agent 和 admin 的白名单检测代码相同（instance.rs 的 periodic task + event-driven task）
5. 之前的分析发现 `block_peer` 将 peer_id 加入 `blocked_peers` DashSet（内存），`add_new_peer_conn` 检查 `is_peer_blocked` 拒绝连接

### 需要分析的文件
请读取以下文件，理解完整的连接建立和阻断流程：

1. `/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs` — `add_new_peer_conn` 函数（搜索 `is_peer_blocked` 调用），理解连接建立时的白名单检查
2. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 行 300-470 — periodic task 和 event-driven task 的白名单检查 + block/disconnect 逻辑
3. `/mnt/nvme2t_2/easytier/easytier/src/common/global_ctx.rs` — `block_peer` / `is_peer_blocked` / `unblock_peer` 函数
4. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` — 搜索 `unblock_peer`，确认是否有地方在意外解除 block
5. `/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs` — `close_peer_conn` 函数，理解断开连接后 peer 能否通过其他路径重连

### 需要排查的假设

1. **`close_peer_conn` 只关闭默认连接，peer 通过非默认连接继续通信**：event-driven task 调用 `pm.close_peer_conn(peer_id, &conn_id)` 只关闭一个连接 ID，但 peer 可能有多个并发连接
2. **`block_peer` 添加的 blocked_peers 在连接断开后被清理**：peer 断开后，某个 cleanup 逻辑可能从 blocked_peers 中移除 peer_id，导致重连时 `is_peer_blocked` 返回 false
3. **event-driven task 的 `block_peer` 和 periodic task 的 block 逻辑冲突**：一个 block 后另一个 unblock
4. **UDP hole punch / WireGuard 多路径绕过**：非白名单 peer 通过 UDP 直连或 WG 隧道绕过 `add_new_peer_conn` 检查
5. **periodic task 每 30 秒运行一次，但白名单加载频率是否足够？** 白名单文件更新后是否在下次 periodic task 才生效？
6. **agent 的 event-driven task 是否真的被触发？** 搜索 `PeerAdded` event 处理

### 输出格式

```
## 连接建立流程分析
（从 peer 连接到 agent 的完整路径）

## 阻断流程分析
（从白名单检查到 block/disconnect 的完整路径）

## 假设验证结果
| # | 假设 | 验证结果 | 证据（行号） |
|---|------|----------|-------------|

## 根因分析
（最可能的根因）

## 修复建议
（具体的代码修改方案）
```
# Status: done
# Date: 2026-06-03T11:28:18.744Z

## [user] +058391-05-10T15:51:22.000Z
## Recent context from this session
[2026-06-02T16:53:12.289Z] @coder: ## 任务：根据 3 个 reviewer 的反馈修复 README.md

### 文件路径
`/mnt/nvme2t_2/easytier/README.md`

### 修复清单（共 6 项）

#### Fix 1: 截图文件名对调（行 140、146）
**当前**：
- 行 140：`![IP 白名单](eg/devices.png)` ← 文件名错了
- 行 146：`![设备管理](eg/ip-whitelist.png)` ← 文件名错了

**修改为**：
- 行 140：`![IP 白名单](eg/ip-whitelist.png)`
- 行 146：`![设备管理](eg/devices.png)`

#### Fix 2: VPN 端口统一为 22020
**背景**：README core.toml 示例用 `22022`，但实际 Dockerfile EXPO
  → 现在我已经获取到完整文件及其哈希值。让我来应用所有 6 处修复：
现在我来添加缺失的内容。Fix 3 需要将旧的描述替换为完整的要点。Fix 6 需要删除行 267 之后的空行，并用新内容替换。让我读取受影响区域的当前状态：
我还需要将完整的要点列表添加到修复3，并清理修复6中已删除的常见问题。让我来做这两件事：
现在，我还将删除 Fix 5 中多余的旧的“角色与职责”行。让我检查 TOC 区域：
修复 5 和修复 6 需要更多的工作。目录仍然在管理部分（第 21 行）中有“常见问题”条目，但实际上该部分已被删除。让我来修复这个问题，并验证目录中代理部分没有“常见问题”。
现在让我检查关键部分，以验证所有 6 个修复：
等等 - grep 搜索 `### 常见问题` 没有返回任何匹配项。让我检查一下 — 全局 FAQ 标题使用了 `## 常见问题（FAQ）` 而不是 `### 常见问题`。我来验证一下：
修复6已验证：管理员章节的 `### 常见问题` 子章节已移除。只剩下全局的 `## 常见问题（FAQ）`。

现在验证修复2——`grep`显示没有`22022`的匹配项。让我再确认一下`22020`是否存在于预期的位置：
6 项修复全部完成，验证通过：

| Fix | 状态 | 验证结果 |
|-----|------|---------|
| 1. 截图文件名对调 | 成功

[2026-06-02T16:56:21.236Z] @coder: ## 任务：验证 README.md 修复后内容并提交到 GitHub

### 文件路径
`/mnt/nvme2t_2/easytier/README.md`

### 步骤

#### 1. 验证修复（5 个 grep 检查）
```bash
cd /mnt/nvme2t_2/easytier

# 检查 1: 截图文件名引用正确
grep -n "ip-whitelist.png\|devices.png" README.md

# 检查 2: 端口统一（无 22022 残留）
grep -c "22022" README.md    # 应为 0
grep -c "22020" README.md    # 应为多个

# 检查 3: 工作原理已重写
grep -n "easytier-core.*强制执行\|easytier-core.*启动时通过" README.md

# 检
  → 5 个检查全部通过。现在运行隐私检查：
7 个隐私检查全部为 0。现在提交并推送：
`origin` 指向上游 EasyTier 仓库（无推送权限）。需要推送到你的 fork：
全部完成。汇总：

| 项目 | 结果 |
|------|------|
| 截图文件名 | `ip-whitelist.png` ↔ `devices.png` 引用正确 |
| 端口统一 | 0 处 `22022`，7 处 `22020` |
| 工作原理重写 | 已确认（行 267、271） |
| agent.Dockerfile | 已包含（行 393、486） |
| 目录缺失章节 | 已补全（行 8 + 55） |
| 隐私检查 | 7/7 全为 0 |
| 提交 hash | `59f9097` |
| 推送 | `user-repo main` 成功 |
| 最终行数 | **536 行** |

[2026-06-03T11:03:16.058Z] @coder: ## 任务：修复 event-driven 任务的 IP 比较格式 bug

### 背景
`easytier/src/instance/instance.rs` 的 event-driven 任务（行 ~397）使用 `format!("{}", ip)` 格式化 `Ipv4Inet`，输出 `"10.0.210.5/24"`（带子网掩码）。但白名单 JSON 存储的是 `"10.0.210.5"`（不带掩码）。比较永远失败，导致所有 peer 被视为非白名单 → 全部断开 → 断开后 peer 多路径重连 → 部分包漏过。

### 修复
修改 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 的 event-driven 任务。

**位置**：event-driven 任务中，`if let Some(ref ip
  → Found it at line 398. The `ip` is of type `Ipv4Inet` (from the `cidr` crate), and `format!("{}", ip)` outputs `"10.0.210.5/24"` with subnet mask. Need to use `ip.address()` to get just the IP.
[PROGRESS: Fixed line 398: `format!("{}", ip)` → `format!("{}", ip.address())` to strip /24 subnet mask]

Now let me verify the periodic task (line 322) already uses `.address()`:
Fix verified. Line 398 now reads `format!("{}", ip.address())`.

The periodic task (line 322) already uses `.address()` — confirmed from the earlier grep which only found one match for `format!("{}", ip)` (the one we just fixed

[2026-06-03T11:11:16.395Z] @analyst: ## 任务：分析并规划两个问题的修复方案

### 文件路径
- `/mnt/nvme2t_2/easytier/entrypoint.sh`（admin 入口脚本）
- `/mnt/nvme2t_2/easytier/easytier/scripts/agent-entrypoint.sh`（agent 入口脚本）
- `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs`（白名单检测逻辑）
- `/mnt/nvme2t_2/easytier/easytier/src/common/global_ctx.rs`（blocked_peers、whitelist 相关）

### 问题 1：admin 容器启动时自动添加自身虚拟 IP (.1) 到白名单 + 自动绑定 hostname

**需求**：admin 容器启动时，自动
  → 以下为分析报告：

---

## 问题 1 修复方案

- **修改文件**：`/mnt/nvme2t_2/easytier/entrypoint.sh`
- **修改位置**：行 24（sync loop `while true` 之前），即 secrets export 循环前
- **具体修改内容**：

```bash
# 在行 24（while true; do）前面插入：
# 自动将 admin 自身虚拟 IP (.1) 加入白名单
if [ -f "$CORE_CONFIG" ]; then
    ADMIN_IP=$(grep -E '^[[:space:]]*ipv4[[:space:]]*=' "$CORE_CONFIG" | \
        sed -E 's/^[[:space:]]*ipv4[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/' | \
        cut -d'/' -f1)
    HOSTNAME=$(hostname 2>/dev/null || echo "admin")
    if [ -n "$ADMIN_IP" ]; then
        echo "[entrypoint] auto-adding admin self ($ADMIN_IP) to whitelist with

[2026-06-03T11:19:37.478Z] @analyst: ## 任务：分析如何消除 agent 的 Format 1 日志（tracing warn/error 带 nanosecond 时间戳 + 文件:行号），只保留 Format 2 日志（println! 的 print_event 带 UUID 前缀 + second 精度时间戳）

### 背景
agent 容器有两种日志：
- **Format 1**：`tracing_subscriber` 输出，如 `2026-06-03T19:17:57.111181209+08:00  WARN easytier::peers::peer: notified that peer conn is closed... at easytier/src/peers/peer.rs:65`
- **Format 2**：`println!` 输出的 `print_event`（`instance_ma
  → ## 方案对比

| 方案 | RUST_LOG 值 | `tracing::warn!` 日志 | `tracing::error!` 日志 | sync-daemon 日志 | Format 2 日志 | 推荐度 |
|------|-------------|----------------------|----------------------|------------------|---------------|--------|
| A | `easytier=error` | ❌ 全部移除 (~126处) | ✅ 保留 (~162处) | ✅ error 保留 | ✅ 不受影响 | ⭐⭐⭐ |
| B | `easytier=off` | ❌ 全部移除 | ❌ 全部移除 | ❌ 全部移除 | ✅ 不受影响 | ⭐⭐ |
| C | `off` | ❌ 全部移除 | ❌ 全部移除 | ❌ 全部移除 | ✅ 不受影响 | ⭐ |
| 当前 | `easytier=warn` | ✅ 显示 (~126处) | ✅ 显示 (~162处) | ✅ warn+error 显示 | ✅ 不受影响 | — |

## 推荐方案：方案 A（`easytier=error`）

**理由**：
1. **安全底线**：`tracing::error!` 共 ~162 处调用，涵盖关键故

## 任务：重新排查 agent 间歇性放包的根因

### 背景
agent (10.0.210.253) 对非白名单 client 的 ping 阻断不彻底——约每 7 个包有 1 个能通过。admin (10.0.210.1) 则 100% 阻断。

之前怀疑的 `/24` 掩码 bug 已排除（`info.ipv4_addr` 是 `Ipv4Addr` 不是 `Ipv4Inet`，`Display` 输出纯 IP）。

### 关键事实
1. agent 日志**完全没有** `"peer is blocked by IP whitelist"` 错误 → `is_peer_blocked` 在 `add_new_peer_conn` 中从未返回 true
2. agent 日志**反复出现** `new connection accepted` + `new peer connection added` 同一个 peer_id（非白名单 peer 不断被接受）
3. admin 的 `add_new_peer_conn` 能 100% 阻断非白名单 peer
4. agent 和 admin 的白名单检测代码相同（instance.rs 的 periodic task + event-driven task）
5. 之前的分析发现 `block_peer` 将 peer_id 加入 `blocked_peers` DashSet（内存），`add_new_peer_conn` 检查 `is_peer_blocked` 拒绝连接

### 需要分析的文件
请读取以下文件，理解完整的连接建立和阻断流程：

1. `/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs` — `add_new_peer_conn` 函数（搜索 `is_peer_blocked` 调用），理解连接建立时的白名单检查
2. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 行 300-470 — periodic task 和 event-driven task 的白名单检查 + block/disconnect 逻辑
3. `/mnt/nvme2t_2/easytier/easytier/src/common/global_ctx.rs` — `block_peer` / `is_peer_blocked` / `unblock_peer` 函数
4. `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` — 搜索 `unblock_peer`，确认是否有地方在意外解除 block
5. `/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs` — `close_peer_conn` 函数，理解断开连接后 peer 能否通过其他路径重连

### 需要排查的假设

1. **`close_peer_conn` 只关闭默认连接，peer 通过非默认连接继续通信**：event-driven task 调用 `pm.close_peer_conn(peer_id, &conn_id)` 只关闭一个连接 ID，但 peer 可能有多个并发连接
2. **`block_peer` 添加的 blocked_peers 在连接断开后被清理**：peer 断开后，某个 cleanup 逻辑可能从 blocked_peers 中移除 peer_id，导致重连时 `is_peer_blocked` 返回 false
3. **event-driven task 的 `block_peer` 和 periodic task 的 block 逻辑冲突**：一个 block 后另一个 unblock
4. **UDP hole punch / WireGuard 多路径绕过**：非白名单 peer 通过 UDP 直连或 WG 隧道绕过 `add_new_peer_conn` 检查
5. **periodic task 每 30 秒运行一次，但白名单加载频率是否足够？** 白名单文件更新后是否在下次 periodic task 才生效？
6. **agent 的 event-driven task 是否真的被触发？** 搜索 `PeerAdded` event 处理

### 输出格式

```
## 连接建立流程分析
（从 peer 连接到 agent 的完整路径）

## 阻断流程分析
（从白名单检查到 block/disconnect 的完整路径）

## 假设验证结果
| # | 假设 | 验证结果 | 证据（行号） |
|---|------|----------|-------------|

## 根因分析
（最可能的根因）

## 修复建议
（具体的代码修改方案）
```

[PROGRESS] After each sub-step, output: [PROGRESS: what you just did]. Keep it under 120 chars.

## [assistant] +058391-05-10T15:51:41.000Z
[thinking] The user is continuing a conversation about debugging an intermittent packet filtering issue in the EasyTier project. The last message from the analyst was a task analysis about finding the root cause of why the agent sometimes lets non-whitelisted packets through.

Let me look at the recent context more carefully:

1. The first session was about fixing README.md with 6 fixes
2. The second session was about verifying and pushing the README changes
3. The third session was about fixing the IP comparison format bug in instance.rs (format!("{}", ip) → format!("{}", ip.address()))
4. The fourth session was about analyzing and planning fixes for admin container auto-adding self IP to whitelist
5. The fifth session was about analyzing how to eliminate Format 1 logging from agent
6. The last session was about the root cause analysis of intermittent packet filtering

The user seems to have been working with @coder and @analyst agents to investigate a bug where the agent (10.0.210.253) intermittently lets non-whitelisted packets through, while the admin (10.0.210.1) blocks 100%.

The user now seems to want me to continue with the investigation. But there's no explicit user message here - it seems like the context was just provided. Let me see if there's a question.

Actually, looking at this more carefully, this appears to be a continuation of the session where the analyst was tasked with analyzing the root cause. The last entry is the task description for the root cause analysis. The user might want me to continue that work.

Let me start by reading the relevant files to understand the connection establishment and blocking flow, as outlined in the analyst's task.
I'll continue the root cause investigation by reading the key files identified in the analyst's task.
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs"}) [completed]
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
339#dEVHj6|                                                if let Some(conn_id) = pm.get_peer_map().get_peer_default_conn_id(peer_id).await {
340#WIMkGT|                                                    let _ = pm.close_peer_conn(peer_id, &conn_id).await;
341#zA3t4p|                                                    tracing::info!("Disconnected peer {}: IP {} hostname '{}' doesn't match bound '{}'", peer_id, ip, route.hostname, bound_hostname);
342#paYJX6|                                                }
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
363#BC3Zqv|                                if let Some(conn_id) = pm.get_peer_map().get_peer_default_conn_id(peer_id).await {
364#jx6X3U|                                    let _ = pm.close_peer_conn(peer_id, &conn_id).await;
365#eR36Tr|                                    tracing::info!("Disconnected peer {}: IP {} not in whitelist", peer_id, ip);
366#BCocYH|                                }
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
409#K0GaNI|                                                                    if let Some(conn_id) = pm.get_peer_map().get_peer_default_conn_id(peer_id).await {
410#Puaj3h|                                                                        let _ = pm.close_peer_conn(peer_id, &conn_id).await;
411#DTThP9|                                                                        tracing::info!("Event-driven: disconnected peer {}: IP {} hostname '{}' doesn't match bound '{}'", peer_id, ip_str, peer_hostname, bound_hostname);
412#D71piW|                                                                    }
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
436#662HKW|                                                    if let Some(conn_id) = pm.get_peer_map().get_peer_default_conn_id(peer_id).await {
437#iU8HQy|                                                        let _ = pm.close_peer_conn(peer_id, &conn_id).await;
438#Lm7IaW|                                                        tracing::info!("Event-driven: disconnected peer {}: IP {} not in whitelist", peer_id, ip_str);
439#TNZU2s|                                                    }
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

...27703 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3c228e001wCC2F4B2qvxns1
Use Grep to search the full content or Read with offset/limit to view specific sections.
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/common/global_ctx.rs"}) [completed]
1#X4GxpJ|use std::collections::hash_map::DefaultHasher;
2#dlKC0M|use std::net::IpAddr;
3#JkvAPI|use std::{
4#Mib_GL|    hash::Hasher,
5#uOCKyA|    sync::{Arc, Mutex},
6#3zmC_W|};
7#47DEQp|
8#e36N5L|use crate::common::config::ProxyNetworkConfig;
9#1Jgbdw|use crate::common::stats_manager::StatsManager;
10#klb8AZ|use crate::common::token_bucket::TokenBucketManager;
11#tpw6kh|use crate::peers::acl_filter::AclFilter;
12#The7dO|use crate::proto::acl::GroupIdentity;
13#EZ9nlL|use crate::proto::cli::PeerConnInfo;
14#aQk00-|use crate::proto::common::{PeerFeatureFlag, PortForwardConfigPb};
15#8Yfa11|use crate::proto::peer_rpc::PeerGroupInfo;
16#yfXgtS|use crossbeam::atomic::AtomicCell;
17#gsg7kL|use dashmap::DashSet;
18#47DEQp|
19#Z2Y0cv|use super::{
20#ye-c1W|    config::{ConfigLoader, Flags},
21#BhyPE5|    netns::NetNS,
22#JcVOc3|    network::IPCollector,
23#KIzYzm|    stun::{StunInfoCollector, StunInfoCollectorTrait},
24#mTFds7|    PeerId,
25#3zmC_W|};
26#47DEQp|
27#JDvCiX|pub type NetworkIdentity = crate::common::config::NetworkIdentity;
28#47DEQp|
29#Irva_7|#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
30#DGIkf8|pub enum GlobalCtxEvent {
31#ceh4Qd|    TunDeviceReady(String),
32#sohcHb|    TunDeviceError(String),
33#47DEQp|
34#RAb2Ce|    PeerAdded(PeerId),
35#nNXOMa|    PeerRemoved(PeerId),
36#9xGWsq|    PeerConnAdded(PeerConnInfo),
37#W92YSA|    PeerConnRemoved(PeerConnInfo),
38#47DEQp|
39#Pd5W48|    ListenerAdded(url::Url),
40#D_weAe|    ListenerAddFailed(url::Url, String), // (url, error message)
41#iyZofy|    ListenerAcceptFailed(url::Url, String), // (url, error message)
42#-jr-qZ|    ConnectionAccepted(String, String),  // (local url, remote url)
43#uE8KjG|    ConnectionError(String, String, String), // (local url, remote url, error message)
44#47DEQp|
45#-pQvX_|    Connecting(url::Url),
46#YwEHDE|    ConnectError(String, String, String), // (dst, ip version, error message)
47#47DEQp|
48#twcj_7|    VpnPortalStarted(String),                    // (portal)
49#xUbwS0|    VpnPortalClientConnected(String, String),    // (portal, client ip)
50#sSfGSo|    VpnPortalClientDisconnected(String, String), // (portal, client ip)
51#47DEQp|
52#Fua0A4|    DhcpIpv4Changed(Option<cidr::Ipv4Inet>, Option<cidr::Ipv4Inet>), // (old, new)
53#lqBKkb|    DhcpIpv4Conflicted(Option<cidr::Ipv4Inet>),
54#47DEQp|
55#64GXkI|    PortForwardAdded(PortForwardConfigPb),
56#0Qs2qn|}
57#47DEQp|
58#UZFcWr|pub type EventBus = tokio::sync::broadcast::Sender<GlobalCtxEvent>;
59#UvwwOh|pub type EventBusSubscriber = tokio::sync::broadcast::Receiver<GlobalCtxEvent>;
60#P-GtIR|#[derive(Debug, Clone)]
61#057xa5|pub struct WhitelistEntry {
62#5zLO9R|    pub ip: String,
63#mZNBwO|    pub hostname: Option<String>,
64#0Qs2qn|}
65#47DEQp|
66#BFLPn6|pub struct GlobalCtx {
67#5rmcHk|    pub inst_name: String,
68#TWNuvU|    pub id: uuid::Uuid,
69#CJudpb|    pub config: Box<dyn ConfigLoader>,
70#HTkEfg|    pub net_ns: NetNS,
71#JUP8ke|    pub network: NetworkIdentity,
72#47DEQp|
73#8vxITD|    event_bus: EventBus,
74#47DEQp|
75#GFY-XM|    cached_ipv4: AtomicCell<Option<cidr::Ipv4Inet>>,
76#Q8DFSd|    cached_ipv6: AtomicCell<Option<cidr::Ipv6Inet>>,
77#e_XMLh|    cached_proxy_cidrs: AtomicCell<Option<Vec<ProxyNetworkConfig>>>,
78#47DEQp|
79#x1P1lL|    ip_collector: Mutex<Option<Arc<IPCollector>>>,
80#47DEQp|
81#rkYMPc|    hostname: Mutex<String>,
82#47DEQp|
83#ef81B3|    stun_info_collection: Mutex<Arc<dyn StunInfoCollectorTrait>>,
84#47DEQp|
85#JZPmLl|    running_listeners: Mutex<Vec<url::Url>>,
86#47DEQp|
87#EUa7V_|    enable_exit_node: bool,
88#sHwSnx|    proxy_forward_by_system: bool,
89#PrIf0t|    no_tun: bool,
90#47DEQp|
91#GNT79r|    feature_flags: AtomicCell<PeerFeatureFlag>,
92#47DEQp|
93#icFpeR|    quic_proxy_port: AtomicCell<Option<u16>>,
94#47DEQp|
95#DGL2vW|    token_bucket_manager: TokenBucketManager,
96#47DEQp|
97#vH3fpT|    stats_manager: Arc<StatsManager>,
98#47DEQp|
99#yupftT|    acl_filter: Arc<AclFilter>,
100#47DEQp|
101#33ijhb|    extra_network_identities: Mutex<Vec<NetworkIdentity>>,
102#47DEQp|
103#KJ4cqk|    extra_secrets_file: Mutex<Option<String>>,
104#47DEQp|
105#VE9tLu|    ip_whitelist_file: Mutex<Option<String>>,
106#Ue-_jN|    ip_hostname_auto_file: Mutex<Option<String>>,
107#UTwtbV|    blocked_peers_file: Mutex<Option<String>>,
108#cgMQbh|    blocked_peers: DashSet<PeerId>,
109#0Qs2qn|}
110#47DEQp|
111#U3NXHB|impl std::fmt::Debug for GlobalCtx {
112#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
113#-uunom|        f.debug_struct("GlobalCtx")
114#Ulo3wI|            .field("inst_name", &self.inst_name)
115#BvIefu|            .field("id", &self.id)
116#B6qMPq|            .field("net_ns", &self.net_ns.name())
117#Xe482f|            .field("event_bus", &"EventBus")
118#fXqIMg|            .field("ipv4", &self.cached_ipv4)
119#uPgCZz|            .finish()
120#KNhneG|    }
121#0Qs2qn|}
122#47DEQp|
123#shJ7Dy|pub type ArcGlobalCtx = std::sync::Arc<GlobalCtx>;
124#47DEQp|
125#X4GYN4|impl GlobalCtx {
126#eU39qN|    pub fn new(config_fs: impl ConfigLoader + 'static) -> Self {
127#S5l5B4|        let id = config_fs.get_id();
128#_-Bnfi|        let network = config_fs.get_network_identity();
129#hR0cbZ|        let net_ns = NetNS::new(config_fs.get_netns());
130#_5APAk|        let hostname = config_fs.get_hostname();
131#47DEQp|
132#Y__ban|        let (event_bus, _) = tokio::sync::broadcast::channel(8);
133#47DEQp|
134#MwZCQl|        let stun_info_collector = StunInfoCollector::new_with_default_servers();
135#47DEQp|
136#jC6gFA|        if let Some(stun_servers) = config_fs.get_stun_servers() {
137#nzRBgY|            stun_info_collector.set_stun_servers(stun_servers);
138#d4ha1c|        } else {
139#bt6pkS|            stun_info_collector.set_stun_servers(StunInfoCollector::get_default_servers());
140#KkQWgx|        }
141#47DEQp|
142#cfMuxJ|        if let Some(stun_servers) = config_fs.get_stun_servers_v6() {
143#QMfkTW|            stun_info_collector.set_stun_servers_v6(stun_servers);
144#d4ha1c|        } else {
145#1II6Wa|            stun_info_collector.set_stun_servers_v6(StunInfoCollector::get_default_servers_v6());
146#KkQWgx|        }
147#47DEQp|
148#47g67e|        let stun_info_collector = Arc::new(stun_info_collector);
149#47DEQp|
150#kxOP1u|        let enable_exit_node = config_fs.get_flags().enable_exit_node || cfg!(target_env = "ohos");
151#_Q1J8v|        let proxy_forward_by_system = config_fs.get_flags().proxy_forward_by_system;
152#UopQwH|        let no_tun = config_fs.get_flags().no_tun;
153#47DEQp|
154#YjCHto|        let feature_flags = PeerFeatureFlag {
155#1XmcwS|            kcp_input: !config_fs.get_flags().disable_kcp_input,
156#h4hRUg|            no_relay_kcp: config_fs.get_flags().disable_relay_kcp,
157#TE1PNK|            ..Default::default()
158#8OSK9L|        };
159#47DEQp|
160#jrj7E7|        GlobalCtx {
161#UXdbQJ|            inst_name: config_fs.get_inst_name(),
162#Uu1o0c|            id,
163#cpfkg5|            config: Box::new(config_fs),
164#YRo7BQ|            net_ns: net_ns.clone(),
165#LEhu53|            network,
166#47DEQp|
167#vYP4rH|            event_bus,
168#MUoemX|            cached_ipv4: AtomicCell::new(None),
169#eTUJHu|            cached_ipv6: AtomicCell::new(None),
170#xOjjSL|            cached_proxy_cidrs: AtomicCell::new(None),
171#47DEQp|
172#cSdj9Q|            ip_collector: Mutex::new(Some(Arc::new(IPCollector::new(
173#lAoCwR|                net_ns,
174#6ZthXf|                stun_info_collector.clone(),
175#pQy6zL|            )))),
176#47DEQp|
177#x9aMLc|            hostname: Mutex::new(hostname),
178#47DEQp|
179#CEKWq8|            stun_info_collection: Mutex::new(stun_info_collector),
180#47DEQp|
181#yJE8y2|            running_listeners: Mutex::new(Vec::new()),
182#47DEQp|
183#ZUU485|            enable_exit_node,
184#n-w4VI|            proxy_forward_by_system,
185#mLvEwW|            no_tun,
186#47DEQp|
187#afi-Mv|            feature_flags: AtomicCell::new(feature_flags),
188#RoO6k_|            quic_proxy_port: AtomicCell::new(None),
189#47DEQp|
190#rYWCRP|            token_bucket_manager: TokenBucketManager::new(),
191#47DEQp|
192#MIjpq4|            stats_manager: Arc::new(StatsManager::new()),
193#47DEQp|
194#VzhxO6|            acl_filter: Arc::new(AclFilter::new()),
195#47DEQp|
196#egiXVp|            extra_network_identities: Mutex::new(vec![]),
197#vlMl65|            extra_secrets_file: Mutex::new(None),
198#JXSUaC|            ip_whitelist_file: Mutex::new(None),
199#T3O2TU|            ip_hostname_auto_file: Mutex::new(None),
200#BeoEth|            blocked_peers_file: Mutex::new(None),
201#GRkLBK|            blocked_peers: DashSet::new(),
202#KkQWgx|        }
203#KNhneG|    }
204#47DEQp|
205#ifx1Hr|    pub fn subscribe(&self) -> EventBusSubscriber {
206#0oBmpD|        self.event_bus.subscribe()
207#KNhneG|    }
208#47DEQp|
209#xahnvK|    pub fn issue_event(&self, event: GlobalCtxEvent) {
210#xb5Ek9|        if let Err(e) = self.event_bus.send(event.clone()) {
211#eQBeMq|            tracing::warn!(
212#hUqOCO|                "Failed to send event: {:?}, error: {:?}, receiver count: {}",
213#wYwV9l|                event,
214#yJkmrS|                e,
215#4WdRD2|                self.event_bus.receiver_count()
216#TP8Q3D|            );
217#KkQWgx|        }
218#KNhneG|    }
219#47DEQp|
220#kHnEYN|    pub fn check_network_in_whitelist(&self, network_name: &str) -> Result<(), anyhow::Error> {
221#r7Kmin|        if self
222#w9YV8E|            .get_flags()
223#ELP4v2|            .relay_network_whitelist
224#TB85iJ|            .split(" ")
225#5bVLK8|            .map(wildmatch::WildMatch::new)
226#rt82Ss|            .any(|wl| wl.matches(network_name))
227#qWZHcZ|        {
228#Y1ZmOf|            Ok(())
229#d4ha1c|        } else {
230#YfwIa9|            Err(anyhow::anyhow!("network {} not in whitelist", network_name))
231#KkQWgx|        }
232#KNhneG|    }
233#47DEQp|
234#p041U4|    pub fn get_ipv4(&self) -> Option<cidr::Ipv4Inet> {
235#0Wt00q|        if let Some(ret) = self.cached_ipv4.load() {
236#_CrpQ-|            return Some(ret);
237#KkQWgx|        }
238#kubRLq|        let addr = self.config.get_ipv4();
239#pPXGNM|        self.cached_ipv4.store(addr);
240#IWsmZW|        addr
241#KNhneG|    }
242#47DEQp|
243#Mr2GVq|    pub fn set_ipv4(&self, addr: Option<cidr::Ipv4Inet>) {
244#ZPNabJ|        self.config.set_ipv4(addr);
245#gSN-FJ|        self.cached_ipv4.store(None);
246#KNhneG|    }
247#47DEQp|
248#pSFUoR|    pub fn get_ipv6(&self) -> Option<cidr::Ipv6Inet> {
249#0wqZX0|        if let Some(ret) = self.cached_ipv6.load() {
250#_CrpQ-|            return Some(ret);
251#KkQWgx|        }
252#F-Xhoz|        let addr = self.config.get_ipv6();
253#efZJ4F|        self.cached_ipv6.store(addr);
254#IWsmZW|        addr
255#KNhneG|    }
256#47DEQp|
257#NDLx-H|    pub fn set_ipv6(&self, addr: Option<cidr::Ipv6Inet>) {
258#BU6Xbz|        self.config.set_ipv6(addr);
259#TbzfO7|        self.cached_ipv6.store(None);
260#KNhneG|    }
261#47DEQp|
262#-GvpNz|    pub fn get_id(&self) -> uuid::Uuid {
263#_1JpYS|        self.config.get_id()
264#KNhneG|    }
265#47DEQp|
266#QVVaRf|    pub fn is_ip_in_same_network(&self, ip: &IpAddr) -> bool {
267#YhhzlR|        match ip {
268#nHV2kL|            IpAddr::V4(v4) => self.get_ipv4().map(|x| x.contains(v4)).unwrap_or(false),
269#wkVZyn|            IpAddr::V6(v6) => self.get_ipv6().map(|x| x.contains(v6)).unwrap_or(false),
270#KkQWgx|        }
271#KNhneG|    }
272#47DEQp|
273#VJfi7I|    pub fn get_network_identity(&self) -> NetworkIdentity {
274#4V3b1y|        self.config.get_network_identity()
275#KNhneG|    }
276#47DEQp|
277#g0jIex|    pub fn get_network_name(&self) -> String {
278#YrmnoZ|        self.get_network_identity().network_name
279#KNhneG|    }
280#47DEQp|
281#gBWJzz|    pub fn get_ip_collector(&self) -> Arc<IPCollector> {
282#IIPH07|        self.ip_collector.lock().unwrap().as_ref().unwrap().clone()
283#KNhneG|    }
284#47DEQp|
285#i-4bAU|    pub fn get_hostname(&self) -> String {
286#rGNed3|        return self.hostname.lock().unwrap().clone();
287#KNhneG|    }
288#47DEQp|
289#rHcJT3|    pub fn set_hostname(&self, hostname: String) {
290#dmBZlr|        *self.hostname.lock().unwrap() = hostname;
291#KNhneG|    }
292#47DEQp|
293#nkdozj|    pub fn get_stun_info_collector(&self) -> Arc<dyn StunInfoCollectorTrait> {
294#puEs-y|        self.stun_info_collection.lock().unwrap().clone()
295#KNhneG|    }
296#47DEQp|
297#DDFt8d|    pub fn replace_stun_info_collector(&self, collector: Box<dyn StunInfoCollectorTrait>) {
298#ULdpz5|        let arc_collector: Arc<dyn StunInfoCollectorTrait> = Arc::new(collector);
299#p36OOa|        *self.stun_info_collection.lock().unwrap() = arc_collector.clone();
300#47DEQp|
301#wA3syq|        // rebuild the ip collector
302#ITJ6MH|        *self.ip_collector.lock().unwrap() = Some(Arc::new(IPCollector::new(
303#mkO6w4|            self.net_ns.clone(),
304#sRwg3Q|            arc_collector,
305#e8LcOr|        )));
306#KNhneG|    }
307#47DEQp|
308#SY5nV7|    pub fn get_running_listeners(&self) -> Vec<url::Url> {
309#Pz6CTH|        self.running_listeners.lock().unwrap().clone()
310#KNhneG|    }
311#47DEQp|
312#dfbeKJ|    pub fn add_running_listener(&self, url: url::Url) {
313#AVMsNT|        let mut l = self.running_listeners.lock().unwrap();
314#wwjdx6|        if !l.contains(&url) {
315#pJSVr7|            l.push(url);
316#KkQWgx|        }
317#KNhneG|    }
318#47DEQp|
319#JuUJki|    pub fn get_vpn_portal_cidr(&self) -> Option<cidr::Ipv4Cidr> {
320#p7_b4f|        self.config.get_vpn_portal_config().map(|x| x.client_cidr)
321#KNhneG|    }
322#47DEQp|
323#zTNAb6|    pub fn get_flags(&self) -> Flags {
324#1TtXmz|        self.config.get_flags()
325#KNhneG|    }
326#47DEQp|
327#3-rcfR|    pub fn set_flags(&self, flags: Flags) {
328#x16Zod|        self.config.set_flags(flags);
329#KNhneG|    }
330#47DEQp|
331#cYYuoI|    pub fn get_128_key(&self) -> [u8; 16] {
332#cZjPIE|        let mut key = [0u8; 16];
333#mMFmqH|        let secret = self
334#8yoTWr|            .config
335#Ga99pq|            .get_network_identity()
336#aid5Dh|            .network_secret
337#PQmuKE|            .unwrap_or_default();
338#uPPN8c|        // fill key according to network secret
339#nSazqs|        let mut hasher = DefaultHasher::new();
340#sHYRcY|        hasher.write(secret.as_bytes());
341#vJ5HOq|        key[0..8].copy_from_slice(&hasher.finish().to_be_bytes());
342#2cDIOn|        hasher.write(&key[0..8]);
343#3IDExA|        key[8..16].copy_from_slice(&hasher.finish().to_be_bytes());
344#R7mdkg|        hasher.write(&key[0..16]);
345#hA2iAK|        key
346#KNhneG|    }
347#47DEQp|
348#kje5EJ|    pub fn get_256_key(&self) -> [u8; 32] {
349#_Q6bzS|        let mut key = [0u8; 32];
350#mMFmqH|        let secret = self
351#8yoTWr|            .config
352#Ga99pq|            .get_network_identity()
353#aid5Dh|            .network_secret
354#PQmuKE|            .unwrap_or_default();
355#uPPN8c|        // fill key according to network secret
356#nSazqs|        let mut hasher = DefaultHasher::new();
357#sHYRcY|        hasher.write(secret.as_bytes());
358#w8kvFE|        hasher.write(b"easytier-256bit-key"); // 添加固定盐值以区分128位和256位密钥
359#47DEQp|
360#MWlZUb|        // 生成32字节密钥
361#GLyPi3|        for i in 0..4 {
362#Prd5ZS|            let chunk_start = i * 8;
363#VKcasr|            let chunk_end = chunk_start + 8;
364#q1E7YZ|            hasher.write(&key[0..chunk_start]);
365#ARO321|            hasher.write(&[i as u8]); // 添加索引以确保每个8字节块都不同
366#mhSx9W|            key[chunk_start..chunk_end].copy_from_slice(&hasher.finish().to_be_bytes());
367#KkQWgx|        }
368#hA2iAK|        key
369#KNhneG|    }
370#47DEQp|
371#42nCbq|    pub fn enable_exit_node(&self) -> bool {
372#40l295|        self.enable_exit_node
373#KNhneG|    }
374#47DEQp|
375#slGuYw|    pub fn proxy_forward_by_system(&self) -> bool {
376#yGZrfc|        self.proxy_forward_by_system
377#KNhneG|    }
378#47DEQp|
379#oAplGE|    pub fn no_tun(&self) -> bool {
380#mejHwP|        self.no_tun
381#KNhneG|    }
382#47DEQp|
383#jfKyuX|    pub fn get_feature_flags(&self) -> PeerFeatureFlag {
384#LIUES2|        self.feature_flags.load()
385#KNhneG|    }
386#47DEQp|
387#NHFFr8|    pub fn set_feature_flags(&self, flags: PeerFeatureFlag) {
388#VKOyDZ|        self.feature_flags.store(flags);
389#KNhneG|    }
390#47DEQp|
391#B9Xfj9|    pub fn get_quic_proxy_port(&self) -> Option<u16> {
392#Z1EiA5|        self.quic_proxy_port.load()
393#KNhneG|    }
394#47DEQp|
395#mzjJss|    pub fn set_quic_proxy_port(&self, port: Option<u16>) {
396#mfVt0-|        self.acl_filter.set_quic_udp_port(port.unwrap_or(0));
397#dnvSYF|        self.quic_proxy_port.store(port);
398#KNhneG|    }
399#47DEQp|
400#iWsZd-|    pub fn token_bucket_manager(&self) -> &TokenBucketManager {
401#NBNnzF|        &self.token_bucket_manager
402#KNhneG|    }
403#47DEQp|
404#miae6F|    pub fn stats_manager(&self) -> &Arc<StatsManager> {
405#7xuxok|        &self.stats_manager
406#KNhneG|    }
407#47DEQp|
408#qCWtRz|    pub fn get_acl_filter(&self) -> &Arc<AclFilter> {
409#E-HLNG|        &self.acl_filter
410#KNhneG|    }
411#47DEQp|
412#uBNs-d|    pub fn is_valid_network_identity(&self, peer_identity: &NetworkIdentity) -> bool {
413#zHaDoa|        if &self.get_network_identity() == peer_identity {
414#KXQCUd|            return true;
415#KkQWgx|        }
416#yUNvGz|        let extra = self.extra_network_identities.lock().unwrap();
417#17W-so|        extra.contains(peer_identity)
418#KNhneG|    }
419#47DEQp|
420#h-FFXF|    pub fn set_extra_network_identities(&self, identities: Vec<NetworkIdentity>) {
421#0neN13|        *self.extra_network_identities.lock().unwrap() = identities;
422#KNhneG|    }
423#47DEQp|
424#lIhJ1z|    pub fn set_extra_secrets_file(&self, path: String) {
425#wH-ECf|        *self.extra_secrets_file.lock().unwrap() = Some(path);
426#KNhneG|    }
427#47DEQp|
428#Ud091Q|    pub fn load_extra_identities_from_json(&self) {
429#why9o0|        let path = self.extra_secrets_file.lock().unwrap().clone();
430#y0eBlx|        let path = match path {
431#aFFYbW|            Some(p) => p,
432#Mhs0Il|            None => return,
433#8OSK9L|        };
434#6qG0f8|        let content = match std::fs::read_to_string(&path) {
435#V3X4Zd|            Ok(c) => c,
436#-FYpvt|            Err(_) => return,
437#8OSK9L|        };
438#ijMohe|        let secrets: Vec<serde_json::Value> = match serde_json::from_str(&content) {
439#WuUW_8|            Ok(s) => s,
440#-FYpvt|            Err(_) => return,
441#8OSK9L|        };
442#4ogMEx|        let primary_name = self.get_network_identity().network_name.clone();
443#sU70WS|        let identities: Vec<NetworkIdentity> = secrets
444#mGJIm4|            .iter()
445#0pk_kT|            .filter_map(|s| {
446#XRgkok|                let secret = s.get("secret")?.as_str()?;
447#DlE0gn|                Some(NetworkIdentity::new(primary_name.clone(), secret.to_string()))
448#Smfm2F|            })
449#elbMXN|            .collect();
450#dCE4TN|        if !identities.is_empty() {
451#IESFP7|            self.set_extra_network_identities(identities);
452#KkQWgx|        }
453#KNhneG|    }
454#47DEQp|
455#k7l_Z7|    pub fn set_ip_whitelist_file(&self, path: String) {
456#dR8k0L|        *self.ip_whitelist_file.lock().unwrap() = Some(path);
457#KNhneG|    }
458#47DEQp|
459#wx_kpK|    pub fn set_ip_hostname_auto_file(&self, path: String) {
460#LpsCkm|        *self.ip_hostname_auto_file.lock().unwrap() = Some(path);
461#KNhneG|    }
462#47DEQp|
463#4pzmsQ|    pub fn set_blocked_peers_file(&self, path: String) {
464#hLg20c|        *self.blocked_peers_file.lock().unwrap() = Some(path);
465#KNhneG|    }
466#47DEQp|
467#FK25ow|    fn save_blocked_peers(&self) {
468#mrgumt|        let path = self.blocked_peers_file.lock().unwrap().clone();
469#y0eBlx|        let path = match path {
470#aFFYbW|            Some(p) => p,
471#Mhs0Il|            None => return,
472#8OSK9L|        };
473#5q6ErL|        let peers: Vec<u32> = self.blocked_peers.iter().map(|p| *p).collect();
474#PMHJkW|        if let Ok(content) = serde_json::to_string(&peers) {
475#9_U4CX|            let _ = std::fs::write(&path, content);
476#KkQWgx|        }
477#KNhneG|    }
478#47DEQp|
479#_o6YKh|    pub fn load_blocked_peers_from_file(&self) {
480#mrgumt|        let path = self.blocked_peers_file.lock().unwrap().clone();
481#y0eBlx|        let path = match path {
482#aFFYbW|            Some(p) => p,
483#Mhs0Il|            None => return,
484#8OSK9L|        };
485#6qG0f8|        let content = match std::fs::read_to_string(&path) {
486#V3X4Zd|            Ok(c) => c,
487#-FYpvt|            Err(_) => return,
488#8OSK9L|        };
489#kPaQIe|        let peers: Vec<u32> = match serde_json::from_str(&content) {
490#uty70W|            Ok(p) => p,
491#-FYpvt|            Err(_) => return,
492#8OSK9L|        };
493#xK23P-|        for peer_id in peers {
494#wYfVL8|            self.blocked_peers.insert(peer_id);
495#KkQWgx|        }
496#KNhneG|    }
497#47DEQp|
498#vY-Htn|    pub fn bind_hostname(&self, ip: &str, hostname: &str) {
499#R0Xvdq|        if hostname.is_empty() {
500#lKeqdA|            println!("[BIND_DEBUG] bind_hostname called with empty hostname for IP {}, skipping", ip);
501#NhdEjk|            return;
502#KkQWgx|        }
503#rRQ7ap|        let path = self.ip_hostname_auto_file.lock().unwrap().clone();
504#y0eBlx|        let path = match path {
505#aFFYbW|            Some(p) => p,
506#Mhs0Il|            None => return,
507#8OSK9L|        };
508#4CX0zG|        let mut bindings: Vec<serde_json::Value> = std::fs::read_to_string(&path)
509#BqNu03|            .ok()
510#ciVoXR|            .and_then(|c| serde_json::from_str(&c).ok())
511#PQmuKE|            .unwrap_or_default();
512#FBLxjO|        let exists = bindings.iter().any(|v| v.get("ip").and_then(|s| s.as_str()) == Some(ip));
513#LYoDzj|        if !exists {
514#he7tk7|            bindings.push(serde_json::json!({ "ip": ip, "hostname": hostname }));
515#yKeDyc|            if let Ok(content) = serde_json::to_string(&bindings) {
516#MjNNnA|                let _ = std::fs::write(&path, content);
517#hq_fcK|            }
518#KkQWgx|        }
519#KNhneG|    }
520#47DEQp|
521#s1cfy7|    pub fn load_ip_whitelist(&self) -> Vec<WhitelistEntry> {
522#ZsSU9Y|        let path = self.ip_whitelist_file.lock().unwrap().clone();
523#y0eBlx|        let path = match path {
524#aFFYbW|            Some(p) => p,
525#DmiOeW|            None => return vec![],
526#8OSK9L|        };
527#6qG0f8|        let content = match std::fs::read_to_string(&path) {
528#V3X4Zd|            Ok(c) => c,
529#3b4Pa9|            Err(_) => return vec![],
530#8OSK9L|        };
531#0eNuQf|        let entries: Vec<serde_json::Value> = match serde_json::from_str(&content) {
532#Cw5rqW|            Ok(e) => e,
533#3b4Pa9|            Err(_) => return vec![],
534#8OSK9L|        };
535#5t-qnh|        entries
536#mGJIm4|            .iter()
537#vojTFS|            .filter_map(|e| {
538#rdcKkd|                let ip = e.get("ip")?.as_str()?.to_string();
539#5GLdmp|                let hostname = e.get("hostname").and_then(|v| v.as_str()).map(|s| s.to_string());
540#M6iJJx|                Some(WhitelistEntry { ip, hostname })
541#Smfm2F|            })
542#b1dGJO|            .collect()
543#KNhneG|    }
544#47DEQp|
545#oPY4yQ|    pub fn block_peer(&self, peer_id: PeerId) {
546#lz3X0B|        self.blocked_peers.insert(peer_id);
547#5XT2Km|        self.save_blocked_peers();
548#KNhneG|    }
549#47DEQp|
550#9FLy0J|    pub fn unblock_peer(&self, peer_id: PeerId) {
551#RqYznr|        self.blocked_peers.remove(&peer_id);
552#5XT2Km|        self.save_blocked_peers();
553#KNhneG|    }
554#47DEQp|
555#dyOqRE|    pub fn is_peer_blocked(&self, peer_id: PeerId) -> bool {
556#xwepZi|        self.blocked_peers.contains(&peer_id)
557#KNhneG|    }
558#47DEQp|
559#3fCbgP|    pub fn clear_blocked_peers(&self) {
560#1idEWL|        self.blocked_peers.clear();
561#5XT2Km|        self.save_blocked_peers();
562#KNhneG|    }
563#47DEQp|
564#iS0tHg|    pub fn get_acl_groups(&self, peer_id: PeerId) -> Vec<PeerGroupInfo> {
565#Oi0Nai|        use std::collections::HashSet;
566#JPHVkY|        self.config
567#yJ7WnA|            .get_acl()
568#eI2HG2|            .and_then(|acl| acl.acl_v1)
569#M-UvT2|            .and_then(|acl_v1| acl_v1.group)
570#jo2JI4|            .map_or_else(Vec::new, |group| {
571#BzO2xc|                let memberships: HashSet<_> = group.members.iter().collect();
572#3X_NQV|                group
573#I866Sy|                    .declares
574#NCjcpM|                    .iter()
575#gzZCw8|                    .filter(|g| memberships.contains(&g.group_name))
576#VK981I|                    .map(|g| {
577#KxYiJm|                        PeerGroupInfo::generate_with_proof(
578#lZcdTh|                            g.group_name.clone(),
579#sGiEu8|                            g.group_secret.clone(),
580#8c51aN|                            peer_id,
581#Cd3EFV|                        )
582#O1Rl7p|                    })
583#Y9IuRp|                    .collect()
584#Smfm2F|            })
585#KNhneG|    }
586#47DEQp|
587#FMB8DY|    pub fn get_acl_group_declarations(&self) -> Vec<GroupIdentity> {
588#JPHVkY|        self.config
589#yJ7WnA|            .get_acl()
590#eI2HG2|            .and_then(|acl| acl.acl_v1)
591#M-UvT2|            .and_then(|acl_v1| acl_v1.group)
592#kOMhTe|            .map_or_else(Vec::new, |group| group.declares.to_vec())
593#KNhneG|    }
594#0Qs2qn|}
595#47DEQp|
596#O6JvtA|#[cfg(test)]
597#w39nbD|pub mod tests {
598#iA5wSn|    use crate::{
599#kFWzQG|        common::{config::TomlConfigLoader, new_peer_id, stun::MockStunInfoCollector},
600#3qf7N9|        proto::common::NatType,
601#7em-_b|    };
602#47DEQp|
603#6ffN3e|    use super::*;
604#47DEQp|
605#y2wAzC|    #[tokio::test]
606#H36-AE|    async fn test_global_ctx() {
607#0alfTx|        let config = TomlConfigLoader::default();
608#4fuIj7|        let global_ctx = GlobalCtx::new(config);
609#47DEQp|
610#tfcL7-|        let mut subscriber = global_ctx.subscribe();
611#H2UMdM|        let peer_id = new_peer_id();
612#Y7Pf-O|        global_ctx.issue_event(GlobalCtxEvent::PeerAdded(peer_id));
613#mRtB3l|        global_ctx.issue_event(GlobalCtxEvent::PeerRemoved(peer_id));
614#Kciwv-|        global_ctx.issue_event(GlobalCtxEvent::PeerConnAdded(PeerConnInfo::default()));
615#cIt_a8|        global_ctx.issue_event(GlobalCtxEvent::PeerConnRemoved(PeerConnInfo::default()));
616#47DEQp|
617#6EJm6b|        assert_eq!(
618#WnUVtL|            subscriber.recv().await.unwrap(),
619#nDMh2G|            GlobalCtxEvent::PeerAdded(peer_id)
620#bq_4ei|        );
621#6EJm6b|        assert_eq!(
622#WnUVtL|            subscriber.recv().await.unwrap(),
623#8xQAAr|            GlobalCtxEvent::PeerRemoved(peer_id)
624#bq_4ei|        );
625#6EJm6b|        assert_eq!(
626#WnUVtL|            subscriber.recv().await.unwrap(),
627#k22WBQ|            GlobalCtxEvent::PeerConnAdded(PeerConnInfo::default())
628#bq_4ei|        );
629#6EJm6b|        assert_eq!(
630#WnUVtL|            subscriber.recv().await.unwrap(),
631#0FILMl|            GlobalCtxEvent::PeerConnRemoved(PeerConnInfo::default())
632#bq_4ei|        );
633#KNhneG|    }
634#47DEQp|
635#1opDWc|    pub fn get_mock_global_ctx_with_network(
636#gKxBaX|        network_identy: Option<NetworkIdentity>,
637#mfwGx0|    ) -> ArcGlobalCtx {
638#4v6zhi|        let config_fs = TomlConfigLoader::default();
639#kc0Hvn|        config_fs.set_inst_name(format!("test_{}", config_fs.get_id()));
640#THUXjB|        config_fs.set_network_identity(network_identy.unwrap_or_default());
641#47DEQp|
642#GBjgbD|        let ctx = Arc::new(GlobalCtx::new(config_fs));
643#OB1-qs|        ctx.replace_stun_info_collector(Box::new(MockStunInfoCollector {
644#lJP5si|            udp_nat_type: NatType::Unknown,
645#5u6gdO|        }));
646#-0F7Ba|        ctx
647#KNhneG|    }
648#47DEQp|
649#3W5TYf|    pub fn get_mock_global_ctx() -> ArcGlobalCtx {
650#ceX5Ag|        get_mock_global_ctx_with_network(None)
651#KNhneG|    }
652#0Qs2qn|}
653#47DEQp|
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs"}) [completed]
1#JkvAPI|use std::{
2#n-mlx5|    fmt::Debug,
3#0miUK5|    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
4#2oOvB1|    sync::{atomic::AtomicBool, Arc, Weak},
5#nkq1zO|    time::{Instant, SystemTime},
6#3zmC_W|};
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#7mQoz2|use async_trait::async_trait;
10#47DEQp|
11#pF_Tl6|use dashmap::DashMap;
12#47DEQp|
13#McK12P|use tokio::{
14#6SaBei|    sync::{
15#yXmr9W|        mpsc::{self, UnboundedReceiver, UnboundedSender},
16#keoZXw|        Mutex, RwLock,
17#WGNt2R|    },
18#8wmH-9|    task::JoinSet,
19#3zmC_W|};
20#47DEQp|
21#8h8Z46|use crate::{
22#hIkMG-|    common::{
23#7LA2Ap|        compressor::{Compressor as _, DefaultCompressor},
24#yGOl1X|        constants::EASYTIER_VERSION,
25#Qez2zE|        error::Error,
26#IOwpvn|        global_ctx::{ArcGlobalCtx, NetworkIdentity},
27#-CQTtT|        stats_manager::{CounterHandle, LabelSet, LabelType, MetricName},
28#TN8nTI|        stun::StunInfoCollectorTrait,
29#yr1lub|        PeerId,
30#WGNt2R|    },
31#bBQPjc|    peers::{
32#r7_WyB|        peer_conn::PeerConn,
33#ag2bHw|        peer_rpc::PeerRpcManagerTransport,
34#Fkytt7|        recv_packet_from_chan,
35#wGxRrF|        route_trait::{ForeignNetworkRouteInfoMap, MockRoute, NextHopPolicy, RouteInterface},
36#DlpQe-|        PeerPacketFilter,
37#WGNt2R|    },
38#Js7gIl|    proto::{
39#2Obpu4|        cli::{
40#jjjzE7|            self, list_global_foreign_network_response::OneForeignNetwork,
41#XNpGX2|            ListGlobalForeignNetworkResponse,
42#Vev0I6|        },
43#GHt5hR|        peer_rpc::{
44#LNbBxd|            ForeignNetworkRouteInfoEntry, ForeignNetworkRouteInfoKey, RouteForeignNetworkSummary,
45#Vev0I6|        },
46#WGNt2R|    },
47#5fAXpE|    tunnel::{
48#tnN8Km|        self,
49#Ce4DhK|        packet_def::{CompressorAlgo, PacketType, ZCPacket},
50#BHnU6g|        Tunnel, TunnelConnector,
51#WGNt2R|    },
52#3zmC_W|};
53#47DEQp|
54#Z2Y0cv|use super::{
55#xTI2Qn|    create_packet_recv_chan,
56#TV_3YQ|    encrypt::{Encryptor, NullCipher},
57#nXLxyb|    foreign_network_client::ForeignNetworkClient,
58#oGbRre|    foreign_network_manager::{ForeignNetworkManager, GlobalForeignNetworkAccessor},
59#rXyyCw|    peer_conn::PeerConnId,
60#mZdjXu|    peer_map::PeerMap,
61#XRTSKE|    peer_ospf_route::PeerRoute,
62#s2iwev|    peer_rpc::PeerRpcManager,
63#LWqQ_9|    route_trait::{ArcRoute, Route},
64#_s6yp8|    BoxNicPacketFilter, BoxPeerPacketFilter, PacketRecvChan, PacketRecvChanReceiver,
65#3zmC_W|};
66#47DEQp|
67#XQRd2S|struct RpcTransport {
68#ug-G5V|    my_peer_id: PeerId,
69#3hRIgc|    peers: Weak<PeerMap>,
70#4FpEM7|    // TODO: this seems can be removed
71#IiaVCa|    foreign_peers: Mutex<Option<Weak<ForeignNetworkClient>>>,
72#47DEQp|
73#Ta7SHq|    packet_recv: Mutex<UnboundedReceiver<ZCPacket>>,
74#JTTivI|    peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
75#47DEQp|
76#HzqH-Y|    encryptor: Arc<dyn Encryptor>,
77#0Qs2qn|}
78#47DEQp|
79#31qlYx|#[async_trait::async_trait]
80#bWsjuU|impl PeerRpcManagerTransport for RpcTransport {
81#2-s-Hc|    fn my_peer_id(&self) -> PeerId {
82#t_Rp3F|        self.my_peer_id
83#KNhneG|    }
84#47DEQp|
85#iLULEg|    async fn send(&self, mut msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
86#5WJPcd|        let peers = self.peers.upgrade().ok_or(Error::Unknown)?;
87#B8dSEy|        // NOTE: if route info is not exchanged, this will return None. treat it as public server.
88#vGxGid|        let is_dst_peer_public_server = peers
89#cg69RI|            .get_route_peer_info(dst_peer_id)
90#icqUnQ|            .await
91#5_bb6B|            .and_then(|x| x.feature_flag.map(|x| x.is_public_server))
92#7cxG-C|            // if dst is directly connected, it's must not public server
93#7Oh1uK|            .unwrap_or(!peers.has_peer(dst_peer_id));
94#PqGi0l|        if !is_dst_peer_public_server {
95#r_wdly|            self.encryptor
96#nsRc7J|                .encrypt(&mut msg)
97#lPh9vD|                .with_context(|| "encrypt failed")?;
98#KkQWgx|        }
99#EV3zNG|        // send to self and this packet will be forwarded in peer_recv loop
100#FaOb0U|        peers.send_msg_directly(msg, self.my_peer_id).await
101#KNhneG|    }
102#47DEQp|
103#wYJVUI|    async fn recv(&self) -> Result<ZCPacket, Error> {
104#EhjLK5|        if let Some(o) = self.packet_recv.lock().await.recv().await {
105#2qgdDw|            Ok(o)
106#d4ha1c|        } else {
107#7mEL_z|            Err(Error::Unknown)
108#KkQWgx|        }
109#KNhneG|    }
110#0Qs2qn|}
111#47DEQp|
112#e7IyZm|pub enum RouteAlgoType {
113#Ai27Qg|    Ospf,
114#gW4I1O|    None,
115#0Qs2qn|}
116#47DEQp|
117#Vim1EE|enum RouteAlgoInst {
118#cMn152|    Ospf(Arc<PeerRoute>),
119#gW4I1O|    None,
120#0Qs2qn|}
121#47DEQp|
122#RYRGuJ|struct SelfTxCounters {
123#D-V-tR|    self_tx_packets: CounterHandle,
124#6V0uMP|    self_tx_bytes: CounterHandle,
125#zGXsep|    compress_tx_bytes_before: CounterHandle,
126#DukShZ|    compress_tx_bytes_after: CounterHandle,
127#0Qs2qn|}
128#47DEQp|
129#A-4Ewh|pub struct PeerManager {
130#ug-G5V|    my_peer_id: PeerId,
131#47DEQp|
132#sfM3Lj|    global_ctx: ArcGlobalCtx,
133#kfhNSA|    nic_channel: PacketRecvChan,
134#47DEQp|
135#FwTrUD|    tasks: Mutex<JoinSet<()>>,
136#47DEQp|
137#_j4kdf|    packet_recv: Arc<Mutex<Option<PacketRecvChanReceiver>>>,
138#47DEQp|
139#tGvkRl|    peers: Arc<PeerMap>,
140#47DEQp|
141#8VMx0Q|    peer_rpc_mgr: Arc<PeerRpcManager>,
142#FeSjdm|    peer_rpc_tspt: Arc<RpcTransport>,
143#47DEQp|
144#YPcpo0|    peer_packet_process_pipeline: Arc<RwLock<Vec<BoxPeerPacketFilter>>>,
145#bNnuK9|    nic_packet_process_pipeline: Arc<RwLock<Vec<BoxNicPacketFilter>>>,
146#47DEQp|
147#65LL3T|    route_algo_inst: RouteAlgoInst,
148#47DEQp|
149#KBjRVM|    foreign_network_manager: Arc<ForeignNetworkManager>,
150#HP62aE|    foreign_network_client: Arc<ForeignNetworkClient>,
151#47DEQp|
152#JTtOhW|    encryptor: Arc<dyn Encryptor + 'static>,
153#ajfTlp|    data_compress_algo: CompressorAlgo,
154#47DEQp|
155#MzW6ir|    exit_nodes: Vec<IpAddr>,
156#47DEQp|
157#-OFllv|    reserved_my_peer_id_map: DashMap<String, PeerId>,
158#47DEQp|
159#yPvr-Q|    allow_loopback_tunnel: AtomicBool,
160#47DEQp|
161#EB2QlC|    self_tx_counters: SelfTxCounters,
162#0Qs2qn|}
163#47DEQp|
164#VEJ-Nv|impl Debug for PeerManager {
165#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
166#GfxoC0|        f.debug_struct("PeerManager")
167#yUbTEK|            .field("my_peer_id", &self.my_peer_id())
168#pZnIWM|            .field("instance_name", &self.global_ctx.inst_name)
169#jwchZi|            .field("net_ns", &self.global_ctx.net_ns.name())
170#uPgCZz|            .finish()
171#KNhneG|    }
172#0Qs2qn|}
173#47DEQp|
174#WaSJgC|impl PeerManager {
175#4SQmJP|    pub fn new(
176#84qYcS|        route_algo: RouteAlgoType,
177#V5_-NZ|        global_ctx: ArcGlobalCtx,
178#itxN3d|        nic_channel: PacketRecvChan,
179#P1yxtn|    ) -> Self {
180#Ei27Hm|        let my_peer_id = rand::random();
181#47DEQp|
182#bF21tz|        let (packet_send, packet_recv) = create_packet_recv_chan();
183#-k9OTl|        let peers = Arc::new(PeerMap::new(
184#TcJi4O|            packet_send.clone(),
185#2rw4tz|            global_ctx.clone(),
186#907ngS|            my_peer_id,
187#XNrhe2|        ));
188#47DEQp|
189#BCJiZI|        let encryptor = if global_ctx.get_flags().enable_encryption {
190#TrvYkF|            // 只有在启用加密时才使用工厂函数选择算法
191#kL1789|            let algorithm = &global_ctx.get_flags().encryption_algorithm;
192#dtVNcK|            super::encrypt::create_encryptor(
193#LTVAb5|                algorithm,
194#xEkbYr|                global_ctx.get_128_key(),
195#tezR8V|                global_ctx.get_256_key(),
196#jOhom-|            )
197#d4ha1c|        } else {
198#DlGJ0c|            // disable_encryption = true 时使用 NullCipher
199#P1gqj_|            Arc::new(NullCipher)
200#8OSK9L|        };
201#47DEQp|
202#cnaP6d|        if global_ctx
203#gL1TGg|            .check_network_in_whitelist(&global_ctx.get_network_name())
204#UZAafT|            .is_err()
205#qWZHcZ|        {
206#9cFfax|            // if local network is not in whitelist, avoid relay data when exist any other route path
207#55FUIS|            let mut f = global_ctx.get_feature_flags();
208#7NEeJZ|            f.avoid_relay_data = true;
209#ghRxLc|            global_ctx.set_feature_flags(f);
210#KkQWgx|        }
211#47DEQp|
212#eC7ab7|        // TODO: remove these because we have impl pipeline processor.
213#4QNvTE|        let (peer_rpc_tspt_sender, peer_rpc_tspt_recv) = mpsc::unbounded_channel();
214#odkilt|        let rpc_tspt = Arc::new(RpcTransport {
215#907ngS|            my_peer_id,
216#irHp49|            peers: Arc::downgrade(&peers),
217#XFt7Gs|            foreign_peers: Mutex::new(None),
218#-q3GY9|            packet_recv: Mutex::new(peer_rpc_tspt_recv),
219#rnjVTo|            peer_rpc_tspt_sender,
220#QCQqiA|            encryptor: encryptor.clone(),
221#NQ-BLD|        });
222#dTWeBi|        let peer_rpc_mgr = Arc::new(PeerRpcManager::new_with_stats_manager(
223#cL_fh1|            rpc_tspt.clone(),
224#UXBsZE|            global_ctx.stats_manager().clone(),
225#XNrhe2|        ));
226#47DEQp|
227#YyP1mU|        let route_algo_inst = match route_algo {
228#Y9u96Z|            RouteAlgoType::Ospf => RouteAlgoInst::Ospf(PeerRoute::new(
229#hGlxsx|                my_peer_id,
230#_bhH_r|                global_ctx.clone(),
231#LQuUWT|                peer_rpc_mgr.clone(),
232#bt9-a0|            )),
233#DFHyU5|            RouteAlgoType::None => RouteAlgoInst::None,
234#8OSK9L|        };
235#47DEQp|
236#cHpfI6|        let foreign_network_manager = Arc::new(ForeignNetworkManager::new(
237#907ngS|            my_peer_id,
238#2rw4tz|            global_ctx.clone(),
239#TcJi4O|            packet_send.clone(),
240#6wEwdz|            Self::build_foreign_network_manager_accessor(&peers),
241#XNrhe2|        ));
242#IqUQio|        let foreign_network_client = Arc::new(ForeignNetworkClient::new(
243#2rw4tz|            global_ctx.clone(),
244#TcJi4O|            packet_send.clone(),
245#DdNEnS|            peer_rpc_mgr.clone(),
246#907ngS|            my_peer_id,
247#XNrhe2|        ));
248#47DEQp|
249#JOS7nS|        let data_compress_algo = global_ctx
250#w9YV8E|            .get_flags()
251#CMuCL3|            .data_compress_algo()
252#aJofO-|            .try_into()
253#_byGCt|            .expect("invalid data compress algo, maybe some features not enabled");
254#47DEQp|
255#dA-spH|        let exit_nodes = global_ctx.config.get_exit_nodes();
256#47DEQp|
257#dvFkte|        let stats_manager = global_ctx.stats_manager();
258#ZcR7Qb|        let self_tx_counters = SelfTxCounters {
259#UwSzpA|            self_tx_packets: stats_manager.get_counter(
260#I7V2Ec|                MetricName::TrafficPacketsSelfTx,
261#qRrZQ1|                LabelSet::new()
262#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
263#XsBKuC|            ),
264#HPPGMJ|            self_tx_bytes: stats_manager.get_counter(
265#zOOjtD|                MetricName::TrafficBytesSelfTx,
266#qRrZQ1|                LabelSet::new()
267#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
268#XsBKuC|            ),
269#bc-v55|            compress_tx_bytes_before: stats_manager.get_counter(
270#xCAqxm|                MetricName::CompressionBytesTxBefore,
271#qRrZQ1|                LabelSet::new()
272#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
273#XsBKuC|            ),
274#u85wqm|            compress_tx_bytes_after: stats_manager.get_counter(
275#Frn6d_|                MetricName::CompressionBytesTxAfter,
276#qRrZQ1|                LabelSet::new()
277#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
278#XsBKuC|            ),
279#8OSK9L|        };
280#47DEQp|
281#eSauPW|        PeerManager {
282#907ngS|            my_peer_id,
283#47DEQp|
284#PvFYEj|            global_ctx,
285#3JnIN-|            nic_channel,
286#47DEQp|
287#V0fnw8|            tasks: Mutex::new(JoinSet::new()),
288#47DEQp|
289#chi1qM|            packet_recv: Arc::new(Mutex::new(Some(packet_recv))),
290#47DEQp|
291#N9VvX9|            peers: peers.clone(),
292#47DEQp|
293#V4bq0M|            peer_rpc_mgr,
294#AkAixq|            peer_rpc_tspt: rpc_tspt,
295#47DEQp|
296#RDG8zn|            peer_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
297#AVkW1r|            nic_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
298#47DEQp|
299#vtQn2d|            route_algo_inst,
300#47DEQp|
301#Sj49Bi|            foreign_network_manager,
302#mDTqnb|            foreign_network_client,
303#47DEQp|
304#QC8bmd|            encryptor,
305#Pu-vJW|            data_compress_algo,
306#47DEQp|
307#4PpgxG|            exit_nodes,
308#47DEQp|
309#MFZYOb|            reserved_my_peer_id_map: DashMap::new(),
310#47DEQp|
311#jn9OJ3|            allow_loopback_tunnel: AtomicBool::new(true),
312#47DEQp|
313#qTdfR3|            self_tx_counters,
314#KkQWgx|        }
315#KNhneG|    }
316#47DEQp|
317#XCaz50|    pub fn set_allow_loopback_tunnel(&self, allow_loopback_tunnel: bool) {
318#9uXnfd|        self.allow_loopback_tunnel
319#5i63Ur|            .store(allow_loopback_tunnel, std::sync::atomic::Ordering::Relaxed);
320#KNhneG|    }
321#47DEQp|
322#bSQhio|    fn build_foreign_network_manager_accessor(
323#dEPfKJ|        peer_map: &Arc<PeerMap>,
324#f_qBdl|    ) -> Box<dyn GlobalForeignNetworkAccessor> {
325#O4kh_j|        struct T {
326#mqjLcH|            peer_map: Weak<PeerMap>,
327#KkQWgx|        }
328#47DEQp|
329#kD-9ea|        #[async_trait::async_trait]
330#cwfbLX|        impl GlobalForeignNetworkAccessor for T {
331#-FwRBf|            async fn list_global_foreign_peer(
332#Xibb9x|                &self,
333#HOFVLU|                network_identity: &NetworkIdentity,
334#b6Lxu3|            ) -> Vec<PeerId> {
335#1C2Yt4|                let Some(peer_map) = self.peer_map.upgrade() else {
336#W_IXmF|                    return vec![];
337#JBqzQb|                };
338#47DEQp|
339#aWxCLN|                peer_map
340#lEVlUJ|                    .list_peers_own_foreign_network(network_identity)
341#lYNw0O|                    .await
342#hq_fcK|            }
343#KkQWgx|        }
344#47DEQp|
345#D2hsWL|        Box::new(T {
346#jYzaNh|            peer_map: Arc::downgrade(peer_map),
347#9I500R|        })
348#KNhneG|    }
349#47DEQp|
350#2lSRhY|    async fn add_new_peer_conn(&self, peer_conn: PeerConn) -> Result<(), Error> {
351#mxf9Cw|        if !self.global_ctx.is_valid_network_identity(&peer_conn.get_network_identity()) {
352#DvThUD|            return Err(Error::SecretKeyError(
353#cEWqcp|                "network identity not match".to_string(),
354#1oJHlw|            ));
355#KkQWgx|        }
356#ofM7n0|        let peer_id = peer_conn.get_peer_id();
357#Rar11V|        if self.global_ctx.is_peer_blocked(peer_id) {
358#DvThUD|            return Err(Error::SecretKeyError(
359#YTPquW|                "peer is blocked by IP whitelist".to_string(),
360#1oJHlw|            ));
361#KkQWgx|        }
362#aoqkXN|        self.peers.add_new_peer_conn(peer_conn).await;
363#sX41Gp|        Ok(())
364#KNhneG|    }
365#47DEQp|
366#14lYQ5|    pub async fn add_client_tunnel(
367#7BRTWY|        &self,
368#TMFJR1|        tunnel: Box<dyn Tunnel>,
369#GkW2PL|        is_directly_connected: bool,
370#QfKvkA|    ) -> Result<(PeerId, PeerConnId), Error> {
371#c0IlOn|        let mut peer = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
372#_IfWok|        peer.set_is_hole_punched(!is_directly_connected);
373#5yUaks|        peer.do_handshake_as_client().await?;
374#7JL2LP|        let conn_id = peer.get_conn_id();
375#RU1WKB|        let peer_id = peer.get_peer_id();
376#kTrXVm|        if peer.get_network_identity().network_name
377#ZrhvYf|            == self.global_ctx.get_network_identity().network_name
378#qWZHcZ|        {
379#f-Qeac|            self.add_new_peer_conn(peer).await?;
380#d4ha1c|        } else {
381#A7ODMm|            self.foreign_network_client.add_new_peer_conn(peer).await;
382#KkQWgx|        }
383#EPLuW8|        Ok((peer_id, conn_id))
384#KNhneG|    }
385#47DEQp|
386#jJc1fo|    pub fn has_directly_connected_conn(&self, peer_id: PeerId) -> bool {
387#3bpcwG|        if let Some(peer) = self.peers.get_peer_by_id(peer_id) {
388#Zq_A7q|            peer.has_directly_connected_conn()
389#d4ha1c|        } else {
390#kjtVBH|            self.foreign_network_client.get_peer_map().has_peer(peer_id)
391#KkQWgx|        }
392#KNhneG|    }
393#47DEQp|
394#pwuqRZ|    #[tracing::instrument]
395#o0DshP|    pub async fn try_direct_connect<C>(
396#7BRTWY|        &self,
397#i1Ryys|        mut connector: C,
398#EvgalJ|    ) -> Result<(PeerId, PeerConnId), Error>
399#JSvlCU|    where
400#rdvGRT|        C: TunnelConnector + Debug,
401#DT0vWo|    {
402#FQRxlN|        let ns = self.global_ctx.net_ns.clone();
403#09Wskl|        let t = ns
404#s0FDIJ|            .run_async(|| async move { connector.connect().await })
405#hmaiLi|            .await?;
406#9oIpNF|        self.add_client_tunnel(t, true).await
407#KNhneG|    }
408#47DEQp|
409#k9IM0H|    // avoid loop back to virtual network
410#4-_0cQ|    fn check_remote_addr_not_from_virtual_network(
411#7BRTWY|        &self,
412#6OnkCQ|        tunnel: &dyn Tunnel,
413#UnHDae|    ) -> Result<(), anyhow::Error> {
414#sd1I2C|        tracing::info!("check remote addr not from virtual network");
415#ClcXQQ|        let Some(tunnel_info) = tunnel.info() else {
416#PdNkBX|            anyhow::bail!("tunnel info is not set");
417#8OSK9L|        };
418#W0tgbh|        let Some(src) = tunnel_info.remote_addr.map(url::Url::from) else {
419#ol3jOX|            anyhow::bail!("tunnel info remote addr is not set");
420#8OSK9L|        };
421#u-6BoE|        if src.scheme() == "ring" {
422#6WCzfY|            return Ok(());
423#KkQWgx|        }
424#u9Qbc0|        let src_host = match src.socket_addrs(|| Some(1)) {
425#elIV8e|            Ok(addrs) => addrs,
426#gJq-2M|            Err(_) => {
427#PnbuE3|                // if the tunnel is not rely on ip address, skip check
428#LEBeyD|                return Ok(());
429#hq_fcK|            }
430#8OSK9L|        };
431#YrPT6Z|        let virtual_ipv4 = self.global_ctx.get_ipv4().map(|ip| ip.network());
432#MnnRwK|        let virtual_ipv6 = self.global_ctx.get_ipv6().map(|ip| ip.network());
433#3_Ehij|        tracing::info!(
434#5owbgP|            ?virtual_ipv4,
435#P0zdad|            ?virtual_ipv6,
436#jKufAz|            "check remote addr not from virtual network"
437#bq_4ei|        );
438#H79IuK|        for addr in src_host {
439#5YpsUk|            // if no-tun is enabled, the src ip of packet in virtual network is converted to loopback address
440#uqyRvu|            if addr.ip().is_loopback()
441#aSe1bi|                && !self
442#GwaNY6|                    .allow_loopback_tunnel
443#wg0XBg|                    .load(std::sync::atomic::Ordering::Relaxed)
444#eP03rc|            {
445#-2-r1E|                anyhow::bail!("tunnel src host is loopback address");
446#hq_fcK|            }
447#47DEQp|
448#Vs2UUY|            match addr {
449#Fkp5F3|                SocketAddr::V4(addr) => {
450#u2nxdD|                    if let Some(virtual_ipv4) = virtual_ipv4 {
451#JYPb6X|                        if virtual_ipv4.contains(addr.ip()) {
452#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
453#qrgf36|                        }
454#uNGRgX|                    }
455#C_CgiK|                }
456#Yj08jy|                SocketAddr::V6(addr) => {
457#qo9J_v|                    if let Some(virtual_ipv6) = virtual_ipv6 {
458#U0F33o|                        if virtual_ipv6.contains(addr.ip()) {
459#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
460#qrgf36|                        }
461#uNGRgX|                    }
462#C_CgiK|                }
463#hq_fcK|            }
464#KkQWgx|        }
465#sX41Gp|        Ok(())
466#KNhneG|    }
467#47DEQp|
468#U7Knbg|    #[tracing::instrument(ret)]
469#pg0BwE|    pub async fn add_tunnel_as_server(
470#7BRTWY|        &self,
471#TMFJR1|        tunnel: Box<dyn Tunnel>,
472#GkW2PL|        is_directly_connected: bool,
473#Ew16UP|    ) -> Result<(), Error> {
474#c2-UuG|        tracing::info!("add tunnel as server start");
475#6b66k9|        self.check_remote_addr_not_from_virtual_network(&tunnel)?;
476#47DEQp|
477#c3DuKs|        let mut conn = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
478#1CaMg8|        conn.do_handshake_as_server_ext(|peer, msg| {
479#2MbFGD|            if msg.network_name
480#z_HzRK|                == self.global_ctx.get_network_identity().network_name
481#eP03rc|            {
482#LEBeyD|                return Ok(());
483#hq_fcK|            }
484#47DEQp|
485#HMq8Dh|            if self.global_ctx.config.get_flags().private_mode {
486#Pvb-m7|                return Err(Error::SecretKeyError(
487#rq_jVx|                    "private mode is turned on, network identity not match".to_string(),
488#uRtctN|                ));
489#hq_fcK|            }
490#47DEQp|
491#Wvjjso|            let mut peer_id = self
492#WhSa3T|                .foreign_network_manager
493#ujWxtI|                .get_network_peer_id(&msg.network_name);
494#r-UKkg|            if peer_id.is_none() {
495#f8G6ks|                peer_id = Some(*self.reserved_my_peer_id_map.entry(msg.network_name.clone()).or_insert_with(|| {
496#cUHymN|                    rand::random::<PeerId>()
497#JxOJH_|                }).value());
498#hq_fcK|            }
499#OyZvBS|            peer.set_peer_id(peer_id.unwrap());
500#47DEQp|
501#8JJIOp|            tracing::info!(
502#QAVlTO|                ?peer_id,
503#XOCU_s|                ?msg.network_name,
504#YBX1Va|                "handshake as server with foreign network, new peer id: {}, peer id in foreign manager: {:?}",
505#jsth7q|                peer.get_my_peer_id(), peer_id
506#TP8Q3D|            );
507#47DEQp|
508#Y1ZmOf|            Ok(())
509#9I500R|        })
510#GGG_xT|        .await?;
511#47DEQp|
512#xVC9Et|        let peer_network_name = conn.get_network_identity().network_name.clone();
513#47DEQp|
514#Srsdme|        conn.set_is_hole_punched(!is_directly_connected);
515#47DEQp|
516#ia8Sn0|        if peer_network_name == self.global_ctx.get_network_identity().network_name {
517#32dI9Z|            self.add_new_peer_conn(conn).await?;
518#d4ha1c|        } else {
519#MIGy32|            self.foreign_network_manager.add_peer_conn(conn).await?;
520#KkQWgx|        }
521#47DEQp|
522#eakY8y|        self.reserved_my_peer_id_map.remove(&peer_network_name);
523#47DEQp|
524#093W8_|        tracing::info!("add tunnel as server done");
525#sX41Gp|        Ok(())
526#KNhneG|    }
527#47DEQp|
528#yBJw-S|    async fn try_handle_foreign_network_packet(
529#sCE1BR|        mut packet: ZCPacket,
530#kIT2Ch|        my_peer_id: PeerId,
531#huG0PK|        peer_map: &PeerMap,
532#ZvH2o7|        foreign_network_mgr: &ForeignNetworkManager,
533#u88bzm|    ) -> Result<(), ZCPacket> {
534#bhZeMH|        let pm_header = packet.peer_manager_header().unwrap();
535#fDsfht|        if pm_header.packet_type != PacketType::ForeignNetworkPacket as u8 {
536#KmX_9i|            return Err(packet);
537#KkQWgx|        }
538#47DEQp|
539#flN_K6|        let from_peer_id = pm_header.from_peer_id.get();
540#FRT-lF|        let to_peer_id = pm_header.to_peer_id.get();
541#47DEQp|
542#y4SHAG|        let foreign_hdr = packet.foreign_network_hdr().unwrap();
543#aXbqEm|        let foreign_network_name = foreign_hdr.get_network_name(packet.payload());
544#8quVoo|        let foreign_peer_id = foreign_hdr.get_dst_peer_id();
545#47DEQp|
546#IxuZAU|        let foreign_network_my_peer_id =
547#QNcXt2|            foreign_network_mgr.get_network_peer_id(&foreign_network_name);
548#47DEQp|
549#6MHiqH|        let buf_len = packet.buf_len();
550#-vAIvE|        let stats_manager = peer_map.get_global_ctx().stats_manager().clone();
551#5HN3PJ|        let label_set =
552#8VYKuw|            LabelSet::new().with_label_type(LabelType::NetworkName(foreign_network_name.clone()));
553#GxRWuo|        let add_counter = move |bytes_metric, packets_metric| {
554#S_bDbN|            stats_manager
555#XTcotD|                .get_counter(bytes_metric, label_set.clone())
556#nLeZsi|                .add(buf_len as u64);
557#9fhBOM|            stats_manager.get_counter(packets_metric, label_set).inc();
558#8OSK9L|        };
559#47DEQp|
560#YbVzBx|        // NOTICE: the to peer id is modified by the src from foreign network my peer id to the origin my peer id
561#W0MF1w|        if to_peer_id == my_peer_id {
562#viIycQ|            // packet sent from other peer to me, extract the inner packet and forward it
563#YUb47C|            add_counter(
564#mPzK_l|                MetricName::TrafficBytesForeignForwardRx,
565#h0W-Nu|                MetricName::TrafficPacketsForeignForwardRx,
566#TP8Q3D|            );
567#fdRDE9|            if let Err(e) = foreign_network_mgr
568#z4HPvh|                .send_msg_to_peer(
569#KnDqPt|                    &foreign_network_name,
570#5wz9iB|                    foreign_peer_id,
571#HFocUC|                    packet.foreign_network_packet(),
572#cN3xS-|                )
573#PcpKA_|                .await
574#eP03rc|            {
575#CW4xPB|                tracing::debug!(
576#2EUTrQ|                    ?e,
577#EwHmim|                    ?foreign_network_name,
578#P4MAxf|                    ?foreign_peer_id,
579#H3knNr|                    "foreign network mgr send_msg_to_peer failed"
580#iyW0eL|                );
581#hq_fcK|            }
582#Y1ZmOf|            Ok(())
583#YIUK0N|        } else if Some(from_peer_id) == foreign_network_my_peer_id {
584#mwhHwz|            // to_peer_id is my peer id for the foreign network, need to convert to the origin my_peer_id of dst
585#1Cllfa|            let Some(to_peer_id) = peer_map
586#vOHCEe|                .get_origin_my_peer_id(&foreign_network_name, to_peer_id)
587#PcpKA_|                .await
588#HO4mJ1|            else {
589#CW4xPB|                tracing::debug!(
590#EwHmim|                    ?foreign_network_name,
591#64bVg4|                    ?to_peer_id,
592#Fa_5NR|                    "cannot find origin my peer id for foreign network."
593#iyW0eL|                );
594#1QMLIr|                return Err(packet);
595#huZslw|            };
596#47DEQp|
597#YUb47C|            add_counter(
598#isNHJA|                MetricName::TrafficBytesForeignForwardTx,
599#Yut8ka|                MetricName::TrafficPacketsForeignForwardTx,
600#TP8Q3D|            );
601#47DEQp|
602#qFCK98|            // modify the to_peer id from foreign network my peer id to the origin my peer id
603#1qxWD1|            packet
604#0lMrqe|                .mut_peer_manager_header()
605#gQk2jP|                .unwrap()
606#2oYuph|                .to_peer_id
607#1FcmIL|                .set(to_peer_id);
608#47DEQp|
609#WWRUyf|            // packet is generated from foreign network mgr and should be forward to other peer
610#GjxVjo|            if let Err(e) = peer_map
611#pWezG5|                .send_msg(packet, to_peer_id, NextHopPolicy::LeastHop)
612#PcpKA_|                .await
613#eP03rc|            {
614#CW4xPB|                tracing::debug!(
615#2EUTrQ|                    ?e,
616#64bVg4|                    ?to_peer_id,
617#0xHh2s|                    "send_msg_directly failed when forward local generated foreign network packet"
618#iyW0eL|                );
619#hq_fcK|            }
620#Y1ZmOf|            Ok(())
621#d4ha1c|        } else {
622#b_Rzck|            // target is not me, forward it. try get origin peer id
623#YUb47C|            add_counter(
624#iZmaoy|                MetricName::TrafficBytesForeignForwardForwarded,
625#1fxT_q|                MetricName::TrafficPacketsForeignForwardForwarded,
626#TP8Q3D|            );
627#BKXjm4|            Err(packet)
628#KkQWgx|        }
629#KNhneG|    }
630#47DEQp|
631#FHuK_1|    async fn start_peer_recv(&self) {
632#37sTF-|        let mut recv = self.packet_recv.lock().await.take().unwrap();
633#vnR7aH|        let my_peer_id = self.my_peer_id;
634#nmPmgi|        let peers = self.peers.clone();
635#97eb5s|        let pipe_line = self.peer_packet_process_pipeline.clone();
636#8aTJMr|        let foreign_client = self.foreign_network_client.clone();
637#C546Zb|        let foreign_mgr = self.foreign_network_manager.clone();
638#k_2-Ol|        let encryptor = self.encryptor.clone();
639#mPlkH9|        let compress_algo = self.data_compress_algo;
640#GRlbc0|        let acl_filter = self.global_ctx.get_acl_filter().clone();
641#ZGLiLv|        let global_ctx = self.global_ctx.clone();
642#UZrWMo|        let stats_mgr = self.global_ctx.stats_manager().clone();
643#9sGRWS|        let route = self.get_route();
644#47DEQp|
645#5HN3PJ|        let label_set =
646#aDVoKb|            LabelSet::new().with_label_type(LabelType::NetworkName(global_ctx.get_network_name()));
647#47DEQp|
648#mEGOr9|        let self_tx_bytes = self.self_tx_counters.self_tx_bytes.clone();
649#T7N9pS|        let self_tx_packets = self.self_tx_counters.self_tx_packets.clone();
650#hCUNVJ|        let self_rx_bytes =
651#80PMSq|            stats_mgr.get_counter(MetricName::TrafficBytesSelfRx, label_set.clone());
652#DmhFt-|        let self_rx_packets =
653#DmhvAH|            stats_mgr.get_counter(MetricName::TrafficPacketsSelfRx, label_set.clone());
654#FP31iZ|        let forward_tx_bytes =
655#mIb08E|            stats_mgr.get_counter(MetricName::TrafficBytesForwarded, label_set.clone());
656#N6gzzb|        let forward_tx_packets =
657#nCT_oP|            stats_mgr.get_counter(MetricName::TrafficPacketsForwarded, label_set.clone());
658#47DEQp|
659#9nvSTI|        let compress_tx_bytes_before = self.self_tx_counters.compress_tx_bytes_before.clone();
660#8Ehyj-|        let compress_tx_bytes_after = self.self_tx_counters.compress_tx_bytes_after.clone();
661#dMW85W|        let compress_rx_bytes_before =
662#Jryksd|            stats_mgr.get_counter(MetricName::CompressionBytesRxBefore, label_set.clone());
663#CJs0ZN|        let compress_rx_bytes_after =
664#OkiaLh|            stats_mgr.get_counter(MetricName::CompressionBytesRxAfter, label_set.clone());
665#47DEQp|
666#DDENzd|        self.tasks.lock().await.spawn(async move {
667#s1vPO9|            tracing::trace!("start_peer_recv");
668#z3Goih|            while let Ok(ret) = recv_packet_from_chan(&mut recv).await {
669#w5bSOZ|                let Err(mut ret) =
670#E9vP_K|                    Self::try_handle_foreign_network_packet(ret, my_peer_id, &peers, &foreign_mgr)
671#o6c0n-|                        .await
672#QIIDCb|                else {
673#_01Y-5|                    continue;
674#JBqzQb|                };
675#47DEQp|
676#z4xbBM|                let buf_len = ret.buf_len();
677#sLsb9g|                let Some(hdr) = ret.mut_peer_manager_header() else {
678#eP2t8R|                    tracing::warn!(?ret, "invalid packet, skip");
679#_01Y-5|                    continue;
680#JBqzQb|                };
681#47DEQp|
682#ax0YAR|                tracing::trace!(?hdr, "peer recv a packet...");
683#G1tVuV|                let from_peer_id = hdr.from_peer_id.get();
684#i2EURd|                let to_peer_id = hdr.to_peer_id.get();
685#Cuycos|                if to_peer_id != my_peer_id {
686#JRJHei|                    if hdr.forward_counter > 7 {
687#F8mduZ|                        tracing::warn!(?hdr, "forward counter exceed, drop packet");
688#W98xla|                        continue;
689#uNGRgX|                    }
690#47DEQp|
691#pwyl0-|                    if hdr.forward_counter > 2 && hdr.is_latency_first() {
692#uVS30W|                        tracing::trace!(?hdr, "set_latency_first false because too many hop");
693#9ndRqd|                        hdr.set_latency_first(false);
694#uNGRgX|                    }
695#47DEQp|
696#VdVaow|                    hdr.forward_counter += 1;
697#47DEQp|
698#ZT0lS_|                    if from_peer_id == my_peer_id {
699#uroO3I|                        compress_tx_bytes_before.add(buf_len as u64);
700#47DEQp|
701#iGD8zw|                        if hdr.packet_type == PacketType::Data as u8
702#GABpVR|                            || hdr.packet_type == PacketType::KcpSrc as u8
703#Z6rbMg|                            || hdr.packet_type == PacketType::KcpDst as u8
704#Fzqoyl|                        {
705#1LI0Vo|                            let _ =
706#JUJFc0|                                Self::try_compress_and_encrypt(compress_algo, &encryptor, &mut ret)
707#1yUlMs|                                    .await;
708#qrgf36|                        }
709#47DEQp|
710#T0tOoJ|                        compress_tx_bytes_after.add(ret.buf_len() as u64);
711#tANDt7|                        self_tx_bytes.add(ret.buf_len() as u64);
712#f8O7M7|                        self_tx_packets.inc();
713#HyGTCY|                    } else {
714#Pguetm|                        forward_tx_bytes.add(buf_len as u64);
715#gmyQh8|                        forward_tx_packets.inc();
716#uNGRgX|                    }
717#47DEQp|
718#kSLxh3|                    tracing::trace!(?to_peer_id, ?my_peer_id, "need forward");
719#b8Js0t|                    let ret =
720#EZHc3h|                        Self::send_msg_internal(&peers, &foreign_client, ret, to_peer_id).await;
721#LXf7AG|                    if ret.is_err() {
722#RK5a5R|                        tracing::error!(?ret, ?to_peer_id, ?from_peer_id, "forward packet error");
723#uNGRgX|                    }
724#ZtcDW9|                } else {
725#fn_fyy|                    if let Err(e) = encryptor.decrypt(&mut ret) {
726#HZCGH0|                        tracing::error!(?e, "decrypt failed");
727#W98xla|                        continue;
728#uNGRgX|                    }
729#47DEQp|
730#Ln7UBJ|                    self_rx_bytes.add(buf_len as u64);
731#XNjtrJ|                    self_rx_packets.inc();
732#BHaA-C|                    compress_rx_bytes_before.add(buf_len as u64);
733#47DEQp|
734#QblX98|                    let compressor = DefaultCompressor {};
735#GMZW0O|                    if let Err(e) = compressor.decompress(&mut ret).await {
736#gbQ6SV|                        tracing::error!(?e, "decompress failed");
737#W98xla|                        continue;
738#uNGRgX|                    }
739#47DEQp|
740#Hjej0y|                    compress_rx_bytes_after.add(ret.buf_len() as u64);
741#47DEQp|
742#2dQ5pH|                    if !acl_filter.process_packet_with_acl(
743#NQJi05|                        &ret,
744#VaXc6J|                        true,
745#NFzhEu|                        global_ctx.get_ipv4().map(|x| x.address()),
746#NvLih8|                        global_ctx.get_ipv6().map(|x| x.address()),
747#5sWIyd|                        &route,
748#SwK0Oa|                    ) {
749#W98xla|                        continue;
750#uNGRgX|                    }
751#47DEQp|
752#cqHypd|                    let mut processed = false;
753#fYLeTn|                    let mut zc_packet = Some(ret);
754#IhX6Kq|                    for (idx, pipeline) in pipe_line.read().await.iter().rev().enumerate() {
755#r-rtuq|                        tracing::trace!(?zc_packet, ?idx, "try_process_packet_from_peer");
756#0x6iPo|                        zc_packet = pipeline
757#D_qrXs|                            .try_process_packet_from_peer(zc_packet.unwrap())
758#IY999y|                            .await;
759#Ew5lXA|                        if zc_packet.is_none() {
760#10AbF1|                            processed = true;
761#Ko0WZT|                            break;
762#qrgf36|                        }
763#uNGRgX|                    }
764#R2opY7|                    if !processed {
765#BN52j-|                        tracing::error!(?zc_packet, "unhandled packet");
766#uNGRgX|                    }
767#C_CgiK|                }
768#hq_fcK|            }
769#H99HIN|            panic!("done_peer_recv");
770#NQ-BLD|        });
771#KNhneG|    }
772#47DEQp|
773#q_JTCC|    pub async fn add_packet_process_pipeline(&self, pipeline: BoxPeerPacketFilter) {
774#5xAyFb|        // newest pipeline will be executed first
775#9k-qPG|        self.peer_packet_process_pipeline
776#jtBdos|            .write()
777#icqUnQ|            .await
778#gbkGf8|            .push(pipeline);
779#KNhneG|    }
780#47DEQp|
781#ekn-0q|    pub async fn add_nic_packet_process_pipeline(&self, pipeline: BoxNicPacketFilter) {
782#5xAyFb|        // newest pipeline will be executed first
783#lgMoPV|        self.nic_packet_process_pipeline
784#jtBdos|            .write()
785#icqUnQ|            .await
786#gbkGf8|            .push(pipeline);
787#KNhneG|    }
788#47DEQp|
789#rwu4HQ|    async fn init_packet_process_pipeline(&self) {
790#LJfv5W|        // for tun/tap ip/eth packet.
791#hwmngc|        struct NicPacketProcessor {
792#My7Qci|            nic_channel: PacketRecvChan,
793#KkQWgx|        }
794#kD-9ea|        #[async_trait::async_trait]
795#OtOTlR|        impl PeerPacketFilter for NicPacketProcessor {
796#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
797#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
798#L2pk7f|                if hdr.packet_type == PacketType::Data as u8 {
799#dx_3aF|                    tracing::trace!(?packet, "send packet to nic channel");
800#9RlVs2|                    // TODO: use a function to get the body ref directly for zero copy
801#D1yJAP|                    let _ = self.nic_channel.send(packet).await;
802#JqjnWy|                    None
803#ZtcDW9|                } else {
804#jLWmFO|                    Some(packet)
805#C_CgiK|                }
806#hq_fcK|            }
807#KkQWgx|        }
808#zvD7Bg|        self.add_packet_process_pipeline(Box::new(NicPacketProcessor {
809#PTKjfM|            nic_channel: self.nic_channel.clone(),
810#AVIbAB|        }))
811#10YU6B|        .await;
812#47DEQp|
813#s7MHfd|        // for peer rpc packet
814#N1Iaxj|        struct PeerRpcPacketProcessor {
815#Lx--BG|            peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
816#KkQWgx|        }
817#47DEQp|
818#kD-9ea|        #[async_trait::async_trait]
819#D1zWOn|        impl PeerPacketFilter for PeerRpcPacketProcessor {
820#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
821#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
822#CrfTXZ|                if hdr.packet_type == PacketType::TaRpc as u8
823#TBQp0z|                    || hdr.packet_type == PacketType::RpcReq as u8
824#6aiYqm|                    || hdr.packet_type == PacketType::RpcResp as u8
825#NV9RY5|                {
826#DWqudi|                    self.peer_rpc_tspt_sender.send(packet).unwrap();
827#JqjnWy|                    None
828#ZtcDW9|                } else {
829#jLWmFO|                    Some(packet)
830#C_CgiK|                }
831#hq_fcK|            }
832#KkQWgx|        }
833#4XfYK9|        self.add_packet_process_pipeline(Box::new(PeerRpcPacketProcessor {
834#Nc02qB|            peer_rpc_tspt_sender: self.peer_rpc_tspt.peer_rpc_tspt_sender.clone(),
835#AVIbAB|        }))
836#10YU6B|        .await;
837#KNhneG|    }
838#47DEQp|
839#qnv61H|    pub async fn add_route<T>(&self, route: T)
840#JSvlCU|    where
841#s0mP8g|        T: Route + PeerPacketFilter + Send + Sync + Clone + 'static,
842#DT0vWo|    {
843#YR42TL|        // for route
844#DOPJKV|        self.add_packet_process_pipeline(Box::new(route.clone()))
845#agFGm5|            .await;
846#47DEQp|
847#hY75HP|        struct Interface {
848#y1Tqlc|            my_peer_id: PeerId,
849#rrAPV3|            peers: Weak<PeerMap>,
850#21pgeS|            foreign_network_client: Weak<ForeignNetworkClient>,
851#8Gbfqs|            foreign_network_manager: Weak<ForeignNetworkManager>,
852#KkQWgx|        }
853#47DEQp|
854#KGY7Sc|        #[async_trait]
855#DkYunO|        impl RouteInterface for Interface {
856#l_VjGY|            async fn list_peers(&self) -> Vec<PeerId> {
857#XDZ2Vk|                let Some(foreign_client) = self.foreign_network_client.upgrade() else {
858#W_IXmF|                    return vec![];
859#JBqzQb|                };
860#47DEQp|
861#PUDAeF|                let Some(peer_map) = self.peers.upgrade() else {
862#W_IXmF|                    return vec![];
863#JBqzQb|                };
864#47DEQp|
865#XQDNH1|                let mut peers = foreign_client.list_public_peers().await;
866#njwKeW|                peers.extend(peer_map.list_peers_with_conn().await);
867#MTa5cE|                peers
868#hq_fcK|            }
869#47DEQp|
870#LFp0oo|            fn my_peer_id(&self) -> PeerId {
871#AE5UEx|                self.my_peer_id
872#hq_fcK|            }
873#47DEQp|
874#uonLKK|            async fn list_foreign_networks(&self) -> ForeignNetworkRouteInfoMap {
875#5THHmO|                let ret = DashMap::new();
876#swf1dg|                let Some(foreign_mgr) = self.foreign_network_manager.upgrade() else {
877#b5Svxi|                    return ret;
878#JBqzQb|                };
879#47DEQp|
880#Y8o-Yf|                let networks = foreign_mgr.list_foreign_networks().await;
881#ioetyj|                for (network_name, info) in networks.foreign_networks.iter() {
882#JV3_bG|                    if info.peers.is_empty() {
883#W98xla|                        continue;
884#uNGRgX|                    }
885#47DEQp|
886#LK7acJ|                    let last_update = foreign_mgr
887#JxC-fm|                        .get_foreign_network_last_update(network_name)
888#v0277T|                        .unwrap_or(SystemTime::now());
889#4cuRRV|                    ret.insert(
890#zuxFX7|                        ForeignNetworkRouteInfoKey {
891#FvjDof|                            peer_id: self.my_peer_id,
892#MBDbjS|                            network_name: network_name.clone(),
893#xt3MAa|                        },
894#WTdtwn|                        ForeignNetworkRouteInfoEntry {
895#nSGc8d|                            foreign_peer_ids: info.peers.iter().map(|x| x.peer_id).collect(),
896#DEQ9Fe|                            last_update: Some(last_update.into()),
897#QBzroT|                            version: 0,
898#rLkInO|                            network_secret_digest: info.network_secret_digest.clone(),
899#5efuR1|                            my_peer_id_for_this_network: info.my_peer_id_for_this_network,
900#xt3MAa|                        },
901#iTFhZ0|                    );
902#C_CgiK|                }
903#0qn6Vo|                ret
904#hq_fcK|            }
905#KkQWgx|        }
906#47DEQp|
907#vnR7aH|        let my_peer_id = self.my_peer_id;
908#T7G1Zk|        let _route_id = route
909#SfQrJo|            .open(Box::new(Interface {
910#hGlxsx|                my_peer_id,
911#0F1sFB|                peers: Arc::downgrade(&self.peers),
912#ER66ww|                foreign_network_client: Arc::downgrade(&self.foreign_network_client),
913#yRkAf3|                foreign_network_manager: Arc::downgrade(&self.foreign_network_manager),
914#LufvaI|            }))
915#icqUnQ|            .await
916#WEGyMY|            .unwrap();
917#47DEQp|
918#ORaZ3y|        let arc_route: ArcRoute = Arc::new(Box::new(route));
919#fY7N7T|        self.peers.add_route(arc_route).await;
920#KNhneG|    }
921#47DEQp|
922#rj8ACA|    pub fn get_route(&self) -> Box<dyn Route + Send + Sync + 'static> {
923#aWnGNt|        match &self.route_algo_inst {
924#ind_vT|            RouteAlgoInst::Ospf(route) => Box::new(route.clone()),
925#3ENyBM|            RouteAlgoInst::None => Box::new(MockRoute {}),
926#KkQWgx|        }
927#KNhneG|    }
928#47DEQp|
929#ggsQcx|    pub async fn list_routes(&self) -> Vec<cli::Route> {
930#buqXa2|        self.get_route().list_routes().await
931#KNhneG|    }
932#47DEQp|
933#SQwiYH|    pub async fn get_route_peer_info_last_update_time(&self) -> Instant {
934#o0Hn44|        self.get_route().get_peer_info_last_update_time().await
935#KNhneG|    }
936#47DEQp|
937#rMN99o|    pub async fn dump_route(&self) -> String {
938#NjIrQ5|        self.get_route().dump().await
939#KNhneG|    }
940#47DEQp|
941#qHMIGl|    pub async fn list_global_foreign_network(&self) -> ListGlobalForeignNetworkResponse {
942#dSN10n|        let mut resp = ListGlobalForeignNetworkResponse::default();
943#0GRtAb|        let ret = self.get_route().list_foreign_network_info().await;
944#PfLspy|        for info in ret.infos.iter() {
945#los8mn|            let entry = resp
946#cAKnz7|                .foreign_networks
947#x4ul2a|                .entry(info.key.as_ref().unwrap().peer_id)
948#ctxgtd|                .or_insert_with(Default::default);
949#MiV-NN|            let Some(route_info) = info.value.as_ref() else {
950#YqjFPd|                continue;
951#huZslw|            };
952#47DEQp|
953#ubAyqu|            let f = OneForeignNetwork {
954#1WNjy7|                network_name: info.key.as_ref().unwrap().network_name.clone(),
955#LogZYQ|                peer_ids: route_info.foreign_peer_ids.clone(),
956#RIHZBu|                last_updated: format!("{}", route_info.last_update.unwrap()),
957#hHIWcX|                version: route_info.version,
958#huZslw|            };
959#47DEQp|
960#TZwkyd|            entry.foreign_networks.push(f);
961#KkQWgx|        }
962#47DEQp|
963#ZudY7T|        resp
964#KNhneG|    }
965#47DEQp|
966#J0LLTy|    pub async fn get_foreign_network_summary(&self) -> RouteForeignNetworkSummary {
967#yFVVTB|        self.get_route().get_foreign_network_summary().await
968#KNhneG|    }
969#47DEQp|
970#WFyYU9|    async fn run_nic_packet_process_pipeline(&self, data: &mut ZCPacket) {
971#81LGkF|        if !self.global_ctx.get_acl_filter().process_packet_with_acl(
972#RSksDL|            data,
973#bBjPxJ|            false,
974#SKXxAq|            None,
975#SKXxAq|            None,
976#HUVukP|            &self.get_route(),
977#HceTsT|        ) {
978#NhdEjk|            return;
979#KkQWgx|        }
980#47DEQp|
981#AzFZTs|        for pipeline in self.nic_packet_process_pipeline.read().await.iter().rev() {
982#iJa1xN|            let _ = pipeline.try_process_packet_from_nic(data).await;
983#KkQWgx|        }
984#KNhneG|    }
985#47DEQp|
986#LNmNxR|    pub async fn remove_nic_packet_process_pipeline(&self, id: String) -> Result<(), Error> {
987#kzaKYh|        let mut pipelines = self.nic_packet_process_pipeline.write().await;
988#aIOBXg|        if let Some(pos) = pipelines.iter().position(|x| x.id() == id) {
989#pyEoMy|            pipelines.remove(pos);
990#Y1ZmOf|            Ok(())
991#d4ha1c|        } else {
992#j9KMDj|            Err(Error::NotFound)
993#KkQWgx|        }
994#KNhneG|    }
995#47DEQp|
996#QwGVbl|    fn get_next_hop_policy(is_first_latency: bool) -> NextHopPolicy {
997#Prtm2m|        if is_first_latency {
998#lrGTD9|            NextHopPolicy::LeastCost
999#d4ha1c|        } else {
1000#0t3aFq|            NextHopPolicy::LeastHop
1001#KkQWgx|        }
1002#KNhneG|    }
1003#47DEQp|
1004#YcV_Zw|    pub async fn send_msg(&self, msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
1005#0Y1ljl|        self.self_tx_counters
1006#7boxeQ|            .self_tx_bytes
1007#OREiCd|            .add(msg.buf_len() as u64);
1008#P1nqM0|        self.self_tx_counters.self_tx_packets.inc();
1009#X4VtRb|        let msg_len = msg.buf_len() as u64;
1010#X_Kl5i|        let result =
1011#Pd9f-1|            Self::send_msg_internal(&self.peers, &self.foreign_network_client, msg, dst_peer_id)
1012#mju4UJ|                .await;
1013#ZxIhWe|        if result.is_ok() {
1014#wEbH0X|            self.self_tx_counters.self_tx_bytes.add(msg_len);
1015#zhexxv|            self.self_tx_counters.self_tx_packets.inc();
1016#KkQWgx|        }
1017#NMBvw3|        result
1018#KNhneG|    }
1019#47DEQp|
1020#zhhnC3|    async fn send_msg_internal(
1021#swj2Tw|        peers: &Arc<PeerMap>,
1022#rQ13r4|        foreign_network_client: &Arc<ForeignNetworkClient>,
1023#i65YQK|        msg: ZCPacket,
1024#LiSCIf|        dst_peer_id: PeerId,
1025#Ew16UP|    ) -> Result<(), Error> {
1026#Vg3OfO|        let policy =
1027#s6Hg6i|            Self::get_next_hop_policy(msg.peer_manager_header().unwrap().is_latency_first());
1028#47DEQp|
1029#S-yTsc|        if let Some(gateway) = peers.get_gateway_peer_id(dst_peer_id, policy.clone()).await {
1030#CVigK7|            if peers.has_peer(gateway) {
1031#5Mp0l-|                peers.send_msg_directly(msg, gateway).await
1032#JP80iE|            } else if foreign_network_client.has_next_hop(gateway) {
1033#znEXzS|                foreign_network_client.send_msg(msg, gateway).await
1034#CE4VEs|            } else {
1035#tw81bP|                tracing::warn!(
1036#kk6F7t|                    ?gateway,
1037#_vSdBB|                    ?dst_peer_id,
1038#5aLh6k|                    "cannot send msg to peer through gateway"
1039#iyW0eL|                );
1040#rw6i9c|                Err(Error::RouteError(None))
1041#hq_fcK|            }
1042#OEJCrt|        } else if foreign_network_client.has_next_hop(dst_peer_id) {
1043#hzCk3v|            // check foreign network again. so in happy path we can avoid extra check
1044#YLJ9ty|            foreign_network_client.send_msg(msg, dst_peer_id).await
1045#d4ha1c|        } else {
1046#kVR-Se|            tracing::debug!(?dst_peer_id, "no gateway for peer");
1047#NuwARt|            Err(Error::RouteError(None))
1048#KkQWgx|        }
1049#KNhneG|    }
1050#47DEQp|
1051#n2tvl8|    pub async fn get_msg_dst_peer(&self, ipv4_addr: &Ipv4Addr) -> (Vec<PeerId>, bool) {
1052#YR8kEa|        let mut is_exit_node = false;
1053#5eR7LF|        let mut dst_peers = vec![];
1054#FZMhE_|        let network_length = self
1055#7t5vZi|            .global_ctx
1056#THImhK|            .get_ipv4()
1057#8fe7wM|            .map(|x| x.network_length())
1058#y5RfWx|            .unwrap_or(24);
1059#D0sue1|        let ipv4_inet = cidr::Ipv4Inet::new(*ipv4_addr, network_length).unwrap();
1060#vpDCah|        if ipv4_addr.is_broadcast()
1061#lwUXjr|            || ipv4_addr.is_multicast()
1062#QttO4f|            || *ipv4_addr == ipv4_inet.last_address()
1063#qWZHcZ|        {
1064#poVYD0|            dst_peers.extend(self.peers.list_routes().await.iter().filter_map(|x| {
1065#r49BH8|                if *x.key() != self.my_peer_id {
1066#RZUfe_|                    Some(*x.key())
1067#ZtcDW9|                } else {
1068#JqjnWy|                    None
1069#C_CgiK|                }
1070#Hsxh2D|            }));
1071#vflSHi|        } else if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(ipv4_addr).await {
1072#otf58d|            dst_peers.push(peer_id);
1073#pegr-g|        } else if !self
1074#7t5vZi|            .global_ctx
1075#VvsySw|            .is_ip_in_same_network(&std::net::IpAddr::V4(*ipv4_addr))
1076#qWZHcZ|        {
1077#jQL-Kg|            for exit_node in &self.exit_nodes {
1078#yDUUhE|                let IpAddr::V4(exit_node) = exit_node else {
1079#_01Y-5|                    continue;
1080#JBqzQb|                };
1081#t182g9|                if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(exit_node).await {

...35440 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3c23ff001XcOZBCU1AymbWa
Use Grep to search the full content or Read with offset/limit to view specific sections.

## [assistant] +058391-05-10T17:31:58.000Z
[thinking] I now have the three key files. Let me analyze the flow:

1. **Connection acceptance flow**: `add_tunnel_as_server` (peer_manager.rs:469) calls `add_new_peer_conn` (peer_manager.rs:350) which checks `is_peer_blocked` → if blocked, returns error.

2. **Blocking flow in periodic task** (instance.rs:312-377): Every 30s:
   - Loads whitelist
   - Lists routes
   - For each route with IP, checks whitelist
   - If not in whitelist: `block_peer` + `close_peer_conn`
   - If in whitelist (with hostname mismatch): `block_peer` + `close_peer_conn`
   
3. **Blocking flow in event-driven task** (instance.rs:378-465): On `PeerAdded` event:
   - Same logic as periodic task
   - Polls for 120 iterations (120s) waiting for route info

4. **Key issue**: `close_peer_conn` only closes the **default** connection:
   - Line 339-340: `pm.get_peer_map().get_peer_default_conn_id(peer_id).await`
   - Line 363-364: same
   - Line 436-437: same

5. The `blocked_peers` is a `DashSet` - no automatic cleanup.

Let me now look at `close_peer_conn` in peer_manager.rs to understand what it does, and check for `unblock_peer` in instance.rs.

Let me check what happens with `close_peer_conn` and whether peers can have multiple connections.
Let me investigate the critical blocking functions more deeply.
--- grep({"pattern":"pub async fn close_peer_conn","path":"/mnt/nvme2t_2/easytier/easytier/src/peers"}) [completed]
Found 4 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/peer.rs:
  Line 174:     pub async fn close_peer_conn(&self, conn_id: &PeerConnId) -> Result<(), Error> {


/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs:
  Line 1356:     pub async fn close_peer_conn(


/mnt/nvme2t_2/easytier/easytier/src/peers/peer_map.rs:
  Line 284:     pub async fn close_peer_conn(


/mnt/nvme2t_2/easytier/easytier/src/peers/foreign_network_manager.rs:
  Line 732:     pub async fn close_peer_conn(

--- grep({"pattern":"get_peer_default_conn_id\\|list_conn_ids\\|all_conn_ids","path":"/mnt/nvme2t_2/easytier/easytier/src/peers"}) [completed]
No files found
--- grep({"pattern":"fn unblock_peer\\|unblock_peer\\|clear_blocked_peers","path":"/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs"}) [completed]
No files found

## [assistant] +058391-05-10T19:38:02.000Z
[thinking] Interesting! There's no `unblock_peer` or `clear_blocked_peers` in instance.rs. So the only unblocking happens in the periodic task (line 370-375) and event-driven task (lines 416-417, 426-428). Both follow the pattern: if peer was blocked but is now in whitelist, unblock them.

Let me look at `close_peer_conn` in peer_manager.rs and peer_map.rs to understand the connection closing behavior, and also check how `add_tunnel_as_server` gets called (to understand the flow when a peer reconnects).
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs","offset":1350,"limit":50}) [completed]
1#JkvAPI|use std::{
2#n-mlx5|    fmt::Debug,
3#0miUK5|    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
4#2oOvB1|    sync::{atomic::AtomicBool, Arc, Weak},
5#nkq1zO|    time::{Instant, SystemTime},
6#3zmC_W|};
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#7mQoz2|use async_trait::async_trait;
10#47DEQp|
11#pF_Tl6|use dashmap::DashMap;
12#47DEQp|
13#McK12P|use tokio::{
14#6SaBei|    sync::{
15#yXmr9W|        mpsc::{self, UnboundedReceiver, UnboundedSender},
16#keoZXw|        Mutex, RwLock,
17#WGNt2R|    },
18#8wmH-9|    task::JoinSet,
19#3zmC_W|};
20#47DEQp|
21#8h8Z46|use crate::{
22#hIkMG-|    common::{
23#7LA2Ap|        compressor::{Compressor as _, DefaultCompressor},
24#yGOl1X|        constants::EASYTIER_VERSION,
25#Qez2zE|        error::Error,
26#IOwpvn|        global_ctx::{ArcGlobalCtx, NetworkIdentity},
27#-CQTtT|        stats_manager::{CounterHandle, LabelSet, LabelType, MetricName},
28#TN8nTI|        stun::StunInfoCollectorTrait,
29#yr1lub|        PeerId,
30#WGNt2R|    },
31#bBQPjc|    peers::{
32#r7_WyB|        peer_conn::PeerConn,
33#ag2bHw|        peer_rpc::PeerRpcManagerTransport,
34#Fkytt7|        recv_packet_from_chan,
35#wGxRrF|        route_trait::{ForeignNetworkRouteInfoMap, MockRoute, NextHopPolicy, RouteInterface},
36#DlpQe-|        PeerPacketFilter,
37#WGNt2R|    },
38#Js7gIl|    proto::{
39#2Obpu4|        cli::{
40#jjjzE7|            self, list_global_foreign_network_response::OneForeignNetwork,
41#XNpGX2|            ListGlobalForeignNetworkResponse,
42#Vev0I6|        },
43#GHt5hR|        peer_rpc::{
44#LNbBxd|            ForeignNetworkRouteInfoEntry, ForeignNetworkRouteInfoKey, RouteForeignNetworkSummary,
45#Vev0I6|        },
46#WGNt2R|    },
47#5fAXpE|    tunnel::{
48#tnN8Km|        self,
49#Ce4DhK|        packet_def::{CompressorAlgo, PacketType, ZCPacket},
50#BHnU6g|        Tunnel, TunnelConnector,
51#WGNt2R|    },
52#3zmC_W|};
53#47DEQp|
54#Z2Y0cv|use super::{
55#xTI2Qn|    create_packet_recv_chan,
56#TV_3YQ|    encrypt::{Encryptor, NullCipher},
57#nXLxyb|    foreign_network_client::ForeignNetworkClient,
58#oGbRre|    foreign_network_manager::{ForeignNetworkManager, GlobalForeignNetworkAccessor},
59#rXyyCw|    peer_conn::PeerConnId,
60#mZdjXu|    peer_map::PeerMap,
61#XRTSKE|    peer_ospf_route::PeerRoute,
62#s2iwev|    peer_rpc::PeerRpcManager,
63#LWqQ_9|    route_trait::{ArcRoute, Route},
64#_s6yp8|    BoxNicPacketFilter, BoxPeerPacketFilter, PacketRecvChan, PacketRecvChanReceiver,
65#3zmC_W|};
66#47DEQp|
67#XQRd2S|struct RpcTransport {
68#ug-G5V|    my_peer_id: PeerId,
69#3hRIgc|    peers: Weak<PeerMap>,
70#4FpEM7|    // TODO: this seems can be removed
71#IiaVCa|    foreign_peers: Mutex<Option<Weak<ForeignNetworkClient>>>,
72#47DEQp|
73#Ta7SHq|    packet_recv: Mutex<UnboundedReceiver<ZCPacket>>,
74#JTTivI|    peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
75#47DEQp|
76#HzqH-Y|    encryptor: Arc<dyn Encryptor>,
77#0Qs2qn|}
78#47DEQp|
79#31qlYx|#[async_trait::async_trait]
80#bWsjuU|impl PeerRpcManagerTransport for RpcTransport {
81#2-s-Hc|    fn my_peer_id(&self) -> PeerId {
82#t_Rp3F|        self.my_peer_id
83#KNhneG|    }
84#47DEQp|
85#iLULEg|    async fn send(&self, mut msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
86#5WJPcd|        let peers = self.peers.upgrade().ok_or(Error::Unknown)?;
87#B8dSEy|        // NOTE: if route info is not exchanged, this will return None. treat it as public server.
88#vGxGid|        let is_dst_peer_public_server = peers
89#cg69RI|            .get_route_peer_info(dst_peer_id)
90#icqUnQ|            .await
91#5_bb6B|            .and_then(|x| x.feature_flag.map(|x| x.is_public_server))
92#7cxG-C|            // if dst is directly connected, it's must not public server
93#7Oh1uK|            .unwrap_or(!peers.has_peer(dst_peer_id));
94#PqGi0l|        if !is_dst_peer_public_server {
95#r_wdly|            self.encryptor
96#nsRc7J|                .encrypt(&mut msg)
97#lPh9vD|                .with_context(|| "encrypt failed")?;
98#KkQWgx|        }
99#EV3zNG|        // send to self and this packet will be forwarded in peer_recv loop
100#FaOb0U|        peers.send_msg_directly(msg, self.my_peer_id).await
101#KNhneG|    }
102#47DEQp|
103#wYJVUI|    async fn recv(&self) -> Result<ZCPacket, Error> {
104#EhjLK5|        if let Some(o) = self.packet_recv.lock().await.recv().await {
105#2qgdDw|            Ok(o)
106#d4ha1c|        } else {
107#7mEL_z|            Err(Error::Unknown)
108#KkQWgx|        }
109#KNhneG|    }
110#0Qs2qn|}
111#47DEQp|
112#e7IyZm|pub enum RouteAlgoType {
113#Ai27Qg|    Ospf,
114#gW4I1O|    None,
115#0Qs2qn|}
116#47DEQp|
117#Vim1EE|enum RouteAlgoInst {
118#cMn152|    Ospf(Arc<PeerRoute>),
119#gW4I1O|    None,
120#0Qs2qn|}
121#47DEQp|
122#RYRGuJ|struct SelfTxCounters {
123#D-V-tR|    self_tx_packets: CounterHandle,
124#6V0uMP|    self_tx_bytes: CounterHandle,
125#zGXsep|    compress_tx_bytes_before: CounterHandle,
126#DukShZ|    compress_tx_bytes_after: CounterHandle,
127#0Qs2qn|}
128#47DEQp|
129#A-4Ewh|pub struct PeerManager {
130#ug-G5V|    my_peer_id: PeerId,
131#47DEQp|
132#sfM3Lj|    global_ctx: ArcGlobalCtx,
133#kfhNSA|    nic_channel: PacketRecvChan,
134#47DEQp|
135#FwTrUD|    tasks: Mutex<JoinSet<()>>,
136#47DEQp|
137#_j4kdf|    packet_recv: Arc<Mutex<Option<PacketRecvChanReceiver>>>,
138#47DEQp|
139#tGvkRl|    peers: Arc<PeerMap>,
140#47DEQp|
141#8VMx0Q|    peer_rpc_mgr: Arc<PeerRpcManager>,
142#FeSjdm|    peer_rpc_tspt: Arc<RpcTransport>,
143#47DEQp|
144#YPcpo0|    peer_packet_process_pipeline: Arc<RwLock<Vec<BoxPeerPacketFilter>>>,
145#bNnuK9|    nic_packet_process_pipeline: Arc<RwLock<Vec<BoxNicPacketFilter>>>,
146#47DEQp|
147#65LL3T|    route_algo_inst: RouteAlgoInst,
148#47DEQp|
149#KBjRVM|    foreign_network_manager: Arc<ForeignNetworkManager>,
150#HP62aE|    foreign_network_client: Arc<ForeignNetworkClient>,
151#47DEQp|
152#JTtOhW|    encryptor: Arc<dyn Encryptor + 'static>,
153#ajfTlp|    data_compress_algo: CompressorAlgo,
154#47DEQp|
155#MzW6ir|    exit_nodes: Vec<IpAddr>,
156#47DEQp|
157#-OFllv|    reserved_my_peer_id_map: DashMap<String, PeerId>,
158#47DEQp|
159#yPvr-Q|    allow_loopback_tunnel: AtomicBool,
160#47DEQp|
161#EB2QlC|    self_tx_counters: SelfTxCounters,
162#0Qs2qn|}
163#47DEQp|
164#VEJ-Nv|impl Debug for PeerManager {
165#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
166#GfxoC0|        f.debug_struct("PeerManager")
167#yUbTEK|            .field("my_peer_id", &self.my_peer_id())
168#pZnIWM|            .field("instance_name", &self.global_ctx.inst_name)
169#jwchZi|            .field("net_ns", &self.global_ctx.net_ns.name())
170#uPgCZz|            .finish()
171#KNhneG|    }
172#0Qs2qn|}
173#47DEQp|
174#WaSJgC|impl PeerManager {
175#4SQmJP|    pub fn new(
176#84qYcS|        route_algo: RouteAlgoType,
177#V5_-NZ|        global_ctx: ArcGlobalCtx,
178#itxN3d|        nic_channel: PacketRecvChan,
179#P1yxtn|    ) -> Self {
180#Ei27Hm|        let my_peer_id = rand::random();
181#47DEQp|
182#bF21tz|        let (packet_send, packet_recv) = create_packet_recv_chan();
183#-k9OTl|        let peers = Arc::new(PeerMap::new(
184#TcJi4O|            packet_send.clone(),
185#2rw4tz|            global_ctx.clone(),
186#907ngS|            my_peer_id,
187#XNrhe2|        ));
188#47DEQp|
189#BCJiZI|        let encryptor = if global_ctx.get_flags().enable_encryption {
190#TrvYkF|            // 只有在启用加密时才使用工厂函数选择算法
191#kL1789|            let algorithm = &global_ctx.get_flags().encryption_algorithm;
192#dtVNcK|            super::encrypt::create_encryptor(
193#LTVAb5|                algorithm,
194#xEkbYr|                global_ctx.get_128_key(),
195#tezR8V|                global_ctx.get_256_key(),
196#jOhom-|            )
197#d4ha1c|        } else {
198#DlGJ0c|            // disable_encryption = true 时使用 NullCipher
199#P1gqj_|            Arc::new(NullCipher)
200#8OSK9L|        };
201#47DEQp|
202#cnaP6d|        if global_ctx
203#gL1TGg|            .check_network_in_whitelist(&global_ctx.get_network_name())
204#UZAafT|            .is_err()
205#qWZHcZ|        {
206#9cFfax|            // if local network is not in whitelist, avoid relay data when exist any other route path
207#55FUIS|            let mut f = global_ctx.get_feature_flags();
208#7NEeJZ|            f.avoid_relay_data = true;
209#ghRxLc|            global_ctx.set_feature_flags(f);
210#KkQWgx|        }
211#47DEQp|
212#eC7ab7|        // TODO: remove these because we have impl pipeline processor.
213#4QNvTE|        let (peer_rpc_tspt_sender, peer_rpc_tspt_recv) = mpsc::unbounded_channel();
214#odkilt|        let rpc_tspt = Arc::new(RpcTransport {
215#907ngS|            my_peer_id,
216#irHp49|            peers: Arc::downgrade(&peers),
217#XFt7Gs|            foreign_peers: Mutex::new(None),
218#-q3GY9|            packet_recv: Mutex::new(peer_rpc_tspt_recv),
219#rnjVTo|            peer_rpc_tspt_sender,
220#QCQqiA|            encryptor: encryptor.clone(),
221#NQ-BLD|        });
222#dTWeBi|        let peer_rpc_mgr = Arc::new(PeerRpcManager::new_with_stats_manager(
223#cL_fh1|            rpc_tspt.clone(),
224#UXBsZE|            global_ctx.stats_manager().clone(),
225#XNrhe2|        ));
226#47DEQp|
227#YyP1mU|        let route_algo_inst = match route_algo {
228#Y9u96Z|            RouteAlgoType::Ospf => RouteAlgoInst::Ospf(PeerRoute::new(
229#hGlxsx|                my_peer_id,
230#_bhH_r|                global_ctx.clone(),
231#LQuUWT|                peer_rpc_mgr.clone(),
232#bt9-a0|            )),
233#DFHyU5|            RouteAlgoType::None => RouteAlgoInst::None,
234#8OSK9L|        };
235#47DEQp|
236#cHpfI6|        let foreign_network_manager = Arc::new(ForeignNetworkManager::new(
237#907ngS|            my_peer_id,
238#2rw4tz|            global_ctx.clone(),
239#TcJi4O|            packet_send.clone(),
240#6wEwdz|            Self::build_foreign_network_manager_accessor(&peers),
241#XNrhe2|        ));
242#IqUQio|        let foreign_network_client = Arc::new(ForeignNetworkClient::new(
243#2rw4tz|            global_ctx.clone(),
244#TcJi4O|            packet_send.clone(),
245#DdNEnS|            peer_rpc_mgr.clone(),
246#907ngS|            my_peer_id,
247#XNrhe2|        ));
248#47DEQp|
249#JOS7nS|        let data_compress_algo = global_ctx
250#w9YV8E|            .get_flags()
251#CMuCL3|            .data_compress_algo()
252#aJofO-|            .try_into()
253#_byGCt|            .expect("invalid data compress algo, maybe some features not enabled");
254#47DEQp|
255#dA-spH|        let exit_nodes = global_ctx.config.get_exit_nodes();
256#47DEQp|
257#dvFkte|        let stats_manager = global_ctx.stats_manager();
258#ZcR7Qb|        let self_tx_counters = SelfTxCounters {
259#UwSzpA|            self_tx_packets: stats_manager.get_counter(
260#I7V2Ec|                MetricName::TrafficPacketsSelfTx,
261#qRrZQ1|                LabelSet::new()
262#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
263#XsBKuC|            ),
264#HPPGMJ|            self_tx_bytes: stats_manager.get_counter(
265#zOOjtD|                MetricName::TrafficBytesSelfTx,
266#qRrZQ1|                LabelSet::new()
267#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
268#XsBKuC|            ),
269#bc-v55|            compress_tx_bytes_before: stats_manager.get_counter(
270#xCAqxm|                MetricName::CompressionBytesTxBefore,
271#qRrZQ1|                LabelSet::new()
272#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
273#XsBKuC|            ),
274#u85wqm|            compress_tx_bytes_after: stats_manager.get_counter(
275#Frn6d_|                MetricName::CompressionBytesTxAfter,
276#qRrZQ1|                LabelSet::new()
277#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
278#XsBKuC|            ),
279#8OSK9L|        };
280#47DEQp|
281#eSauPW|        PeerManager {
282#907ngS|            my_peer_id,
283#47DEQp|
284#PvFYEj|            global_ctx,
285#3JnIN-|            nic_channel,
286#47DEQp|
287#V0fnw8|            tasks: Mutex::new(JoinSet::new()),
288#47DEQp|
289#chi1qM|            packet_recv: Arc::new(Mutex::new(Some(packet_recv))),
290#47DEQp|
291#N9VvX9|            peers: peers.clone(),
292#47DEQp|
293#V4bq0M|            peer_rpc_mgr,
294#AkAixq|            peer_rpc_tspt: rpc_tspt,
295#47DEQp|
296#RDG8zn|            peer_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
297#AVkW1r|            nic_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
298#47DEQp|
299#vtQn2d|            route_algo_inst,
300#47DEQp|
301#Sj49Bi|            foreign_network_manager,
302#mDTqnb|            foreign_network_client,
303#47DEQp|
304#QC8bmd|            encryptor,
305#Pu-vJW|            data_compress_algo,
306#47DEQp|
307#4PpgxG|            exit_nodes,
308#47DEQp|
309#MFZYOb|            reserved_my_peer_id_map: DashMap::new(),
310#47DEQp|
311#jn9OJ3|            allow_loopback_tunnel: AtomicBool::new(true),
312#47DEQp|
313#qTdfR3|            self_tx_counters,
314#KkQWgx|        }
315#KNhneG|    }
316#47DEQp|
317#XCaz50|    pub fn set_allow_loopback_tunnel(&self, allow_loopback_tunnel: bool) {
318#9uXnfd|        self.allow_loopback_tunnel
319#5i63Ur|            .store(allow_loopback_tunnel, std::sync::atomic::Ordering::Relaxed);
320#KNhneG|    }
321#47DEQp|
322#bSQhio|    fn build_foreign_network_manager_accessor(
323#dEPfKJ|        peer_map: &Arc<PeerMap>,
324#f_qBdl|    ) -> Box<dyn GlobalForeignNetworkAccessor> {
325#O4kh_j|        struct T {
326#mqjLcH|            peer_map: Weak<PeerMap>,
327#KkQWgx|        }
328#47DEQp|
329#kD-9ea|        #[async_trait::async_trait]
330#cwfbLX|        impl GlobalForeignNetworkAccessor for T {
331#-FwRBf|            async fn list_global_foreign_peer(
332#Xibb9x|                &self,
333#HOFVLU|                network_identity: &NetworkIdentity,
334#b6Lxu3|            ) -> Vec<PeerId> {
335#1C2Yt4|                let Some(peer_map) = self.peer_map.upgrade() else {
336#W_IXmF|                    return vec![];
337#JBqzQb|                };
338#47DEQp|
339#aWxCLN|                peer_map
340#lEVlUJ|                    .list_peers_own_foreign_network(network_identity)
341#lYNw0O|                    .await
342#hq_fcK|            }
343#KkQWgx|        }
344#47DEQp|
345#D2hsWL|        Box::new(T {
346#jYzaNh|            peer_map: Arc::downgrade(peer_map),
347#9I500R|        })
348#KNhneG|    }
349#47DEQp|
350#2lSRhY|    async fn add_new_peer_conn(&self, peer_conn: PeerConn) -> Result<(), Error> {
351#mxf9Cw|        if !self.global_ctx.is_valid_network_identity(&peer_conn.get_network_identity()) {
352#DvThUD|            return Err(Error::SecretKeyError(
353#cEWqcp|                "network identity not match".to_string(),
354#1oJHlw|            ));
355#KkQWgx|        }
356#ofM7n0|        let peer_id = peer_conn.get_peer_id();
357#Rar11V|        if self.global_ctx.is_peer_blocked(peer_id) {
358#DvThUD|            return Err(Error::SecretKeyError(
359#YTPquW|                "peer is blocked by IP whitelist".to_string(),
360#1oJHlw|            ));
361#KkQWgx|        }
362#aoqkXN|        self.peers.add_new_peer_conn(peer_conn).await;
363#sX41Gp|        Ok(())
364#KNhneG|    }
365#47DEQp|
366#14lYQ5|    pub async fn add_client_tunnel(
367#7BRTWY|        &self,
368#TMFJR1|        tunnel: Box<dyn Tunnel>,
369#GkW2PL|        is_directly_connected: bool,
370#QfKvkA|    ) -> Result<(PeerId, PeerConnId), Error> {
371#c0IlOn|        let mut peer = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
372#_IfWok|        peer.set_is_hole_punched(!is_directly_connected);
373#5yUaks|        peer.do_handshake_as_client().await?;
374#7JL2LP|        let conn_id = peer.get_conn_id();
375#RU1WKB|        let peer_id = peer.get_peer_id();
376#kTrXVm|        if peer.get_network_identity().network_name
377#ZrhvYf|            == self.global_ctx.get_network_identity().network_name
378#qWZHcZ|        {
379#f-Qeac|            self.add_new_peer_conn(peer).await?;
380#d4ha1c|        } else {
381#A7ODMm|            self.foreign_network_client.add_new_peer_conn(peer).await;
382#KkQWgx|        }
383#EPLuW8|        Ok((peer_id, conn_id))
384#KNhneG|    }
385#47DEQp|
386#jJc1fo|    pub fn has_directly_connected_conn(&self, peer_id: PeerId) -> bool {
387#3bpcwG|        if let Some(peer) = self.peers.get_peer_by_id(peer_id) {
388#Zq_A7q|            peer.has_directly_connected_conn()
389#d4ha1c|        } else {
390#kjtVBH|            self.foreign_network_client.get_peer_map().has_peer(peer_id)
391#KkQWgx|        }
392#KNhneG|    }
393#47DEQp|
394#pwuqRZ|    #[tracing::instrument]
395#o0DshP|    pub async fn try_direct_connect<C>(
396#7BRTWY|        &self,
397#i1Ryys|        mut connector: C,
398#EvgalJ|    ) -> Result<(PeerId, PeerConnId), Error>
399#JSvlCU|    where
400#rdvGRT|        C: TunnelConnector + Debug,
401#DT0vWo|    {
402#FQRxlN|        let ns = self.global_ctx.net_ns.clone();
403#09Wskl|        let t = ns
404#s0FDIJ|            .run_async(|| async move { connector.connect().await })
405#hmaiLi|            .await?;
406#9oIpNF|        self.add_client_tunnel(t, true).await
407#KNhneG|    }
408#47DEQp|
409#k9IM0H|    // avoid loop back to virtual network
410#4-_0cQ|    fn check_remote_addr_not_from_virtual_network(
411#7BRTWY|        &self,
412#6OnkCQ|        tunnel: &dyn Tunnel,
413#UnHDae|    ) -> Result<(), anyhow::Error> {
414#sd1I2C|        tracing::info!("check remote addr not from virtual network");
415#ClcXQQ|        let Some(tunnel_info) = tunnel.info() else {
416#PdNkBX|            anyhow::bail!("tunnel info is not set");
417#8OSK9L|        };
418#W0tgbh|        let Some(src) = tunnel_info.remote_addr.map(url::Url::from) else {
419#ol3jOX|            anyhow::bail!("tunnel info remote addr is not set");
420#8OSK9L|        };
421#u-6BoE|        if src.scheme() == "ring" {
422#6WCzfY|            return Ok(());
423#KkQWgx|        }
424#u9Qbc0|        let src_host = match src.socket_addrs(|| Some(1)) {
425#elIV8e|            Ok(addrs) => addrs,
426#gJq-2M|            Err(_) => {
427#PnbuE3|                // if the tunnel is not rely on ip address, skip check
428#LEBeyD|                return Ok(());
429#hq_fcK|            }
430#8OSK9L|        };
431#YrPT6Z|        let virtual_ipv4 = self.global_ctx.get_ipv4().map(|ip| ip.network());
432#MnnRwK|        let virtual_ipv6 = self.global_ctx.get_ipv6().map(|ip| ip.network());
433#3_Ehij|        tracing::info!(
434#5owbgP|            ?virtual_ipv4,
435#P0zdad|            ?virtual_ipv6,
436#jKufAz|            "check remote addr not from virtual network"
437#bq_4ei|        );
438#H79IuK|        for addr in src_host {
439#5YpsUk|            // if no-tun is enabled, the src ip of packet in virtual network is converted to loopback address
440#uqyRvu|            if addr.ip().is_loopback()
441#aSe1bi|                && !self
442#GwaNY6|                    .allow_loopback_tunnel
443#wg0XBg|                    .load(std::sync::atomic::Ordering::Relaxed)
444#eP03rc|            {
445#-2-r1E|                anyhow::bail!("tunnel src host is loopback address");
446#hq_fcK|            }
447#47DEQp|
448#Vs2UUY|            match addr {
449#Fkp5F3|                SocketAddr::V4(addr) => {
450#u2nxdD|                    if let Some(virtual_ipv4) = virtual_ipv4 {
451#JYPb6X|                        if virtual_ipv4.contains(addr.ip()) {
452#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
453#qrgf36|                        }
454#uNGRgX|                    }
455#C_CgiK|                }
456#Yj08jy|                SocketAddr::V6(addr) => {
457#qo9J_v|                    if let Some(virtual_ipv6) = virtual_ipv6 {
458#U0F33o|                        if virtual_ipv6.contains(addr.ip()) {
459#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
460#qrgf36|                        }
461#uNGRgX|                    }
462#C_CgiK|                }
463#hq_fcK|            }
464#KkQWgx|        }
465#sX41Gp|        Ok(())
466#KNhneG|    }
467#47DEQp|
468#U7Knbg|    #[tracing::instrument(ret)]
469#pg0BwE|    pub async fn add_tunnel_as_server(
470#7BRTWY|        &self,
471#TMFJR1|        tunnel: Box<dyn Tunnel>,
472#GkW2PL|        is_directly_connected: bool,
473#Ew16UP|    ) -> Result<(), Error> {
474#c2-UuG|        tracing::info!("add tunnel as server start");
475#6b66k9|        self.check_remote_addr_not_from_virtual_network(&tunnel)?;
476#47DEQp|
477#c3DuKs|        let mut conn = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
478#1CaMg8|        conn.do_handshake_as_server_ext(|peer, msg| {
479#2MbFGD|            if msg.network_name
480#z_HzRK|                == self.global_ctx.get_network_identity().network_name
481#eP03rc|            {
482#LEBeyD|                return Ok(());
483#hq_fcK|            }
484#47DEQp|
485#HMq8Dh|            if self.global_ctx.config.get_flags().private_mode {
486#Pvb-m7|                return Err(Error::SecretKeyError(
487#rq_jVx|                    "private mode is turned on, network identity not match".to_string(),
488#uRtctN|                ));
489#hq_fcK|            }
490#47DEQp|
491#Wvjjso|            let mut peer_id = self
492#WhSa3T|                .foreign_network_manager
493#ujWxtI|                .get_network_peer_id(&msg.network_name);
494#r-UKkg|            if peer_id.is_none() {
495#f8G6ks|                peer_id = Some(*self.reserved_my_peer_id_map.entry(msg.network_name.clone()).or_insert_with(|| {
496#cUHymN|                    rand::random::<PeerId>()
497#JxOJH_|                }).value());
498#hq_fcK|            }
499#OyZvBS|            peer.set_peer_id(peer_id.unwrap());
500#47DEQp|
501#8JJIOp|            tracing::info!(
502#QAVlTO|                ?peer_id,
503#XOCU_s|                ?msg.network_name,
504#YBX1Va|                "handshake as server with foreign network, new peer id: {}, peer id in foreign manager: {:?}",
505#jsth7q|                peer.get_my_peer_id(), peer_id
506#TP8Q3D|            );
507#47DEQp|
508#Y1ZmOf|            Ok(())
509#9I500R|        })
510#GGG_xT|        .await?;
511#47DEQp|
512#xVC9Et|        let peer_network_name = conn.get_network_identity().network_name.clone();
513#47DEQp|
514#Srsdme|        conn.set_is_hole_punched(!is_directly_connected);
515#47DEQp|
516#ia8Sn0|        if peer_network_name == self.global_ctx.get_network_identity().network_name {
517#32dI9Z|            self.add_new_peer_conn(conn).await?;
518#d4ha1c|        } else {
519#MIGy32|            self.foreign_network_manager.add_peer_conn(conn).await?;
520#KkQWgx|        }
521#47DEQp|
522#eakY8y|        self.reserved_my_peer_id_map.remove(&peer_network_name);
523#47DEQp|
524#093W8_|        tracing::info!("add tunnel as server done");
525#sX41Gp|        Ok(())
526#KNhneG|    }
527#47DEQp|
528#yBJw-S|    async fn try_handle_foreign_network_packet(
529#sCE1BR|        mut packet: ZCPacket,
530#kIT2Ch|        my_peer_id: PeerId,
531#huG0PK|        peer_map: &PeerMap,
532#ZvH2o7|        foreign_network_mgr: &ForeignNetworkManager,
533#u88bzm|    ) -> Result<(), ZCPacket> {
534#bhZeMH|        let pm_header = packet.peer_manager_header().unwrap();
535#fDsfht|        if pm_header.packet_type != PacketType::ForeignNetworkPacket as u8 {
536#KmX_9i|            return Err(packet);
537#KkQWgx|        }
538#47DEQp|
539#flN_K6|        let from_peer_id = pm_header.from_peer_id.get();
540#FRT-lF|        let to_peer_id = pm_header.to_peer_id.get();
541#47DEQp|
542#y4SHAG|        let foreign_hdr = packet.foreign_network_hdr().unwrap();
543#aXbqEm|        let foreign_network_name = foreign_hdr.get_network_name(packet.payload());
544#8quVoo|        let foreign_peer_id = foreign_hdr.get_dst_peer_id();
545#47DEQp|
546#IxuZAU|        let foreign_network_my_peer_id =
547#QNcXt2|            foreign_network_mgr.get_network_peer_id(&foreign_network_name);
548#47DEQp|
549#6MHiqH|        let buf_len = packet.buf_len();
550#-vAIvE|        let stats_manager = peer_map.get_global_ctx().stats_manager().clone();
551#5HN3PJ|        let label_set =
552#8VYKuw|            LabelSet::new().with_label_type(LabelType::NetworkName(foreign_network_name.clone()));
553#GxRWuo|        let add_counter = move |bytes_metric, packets_metric| {
554#S_bDbN|            stats_manager
555#XTcotD|                .get_counter(bytes_metric, label_set.clone())
556#nLeZsi|                .add(buf_len as u64);
557#9fhBOM|            stats_manager.get_counter(packets_metric, label_set).inc();
558#8OSK9L|        };
559#47DEQp|
560#YbVzBx|        // NOTICE: the to peer id is modified by the src from foreign network my peer id to the origin my peer id
561#W0MF1w|        if to_peer_id == my_peer_id {
562#viIycQ|            // packet sent from other peer to me, extract the inner packet and forward it
563#YUb47C|            add_counter(
564#mPzK_l|                MetricName::TrafficBytesForeignForwardRx,
565#h0W-Nu|                MetricName::TrafficPacketsForeignForwardRx,
566#TP8Q3D|            );
567#fdRDE9|            if let Err(e) = foreign_network_mgr
568#z4HPvh|                .send_msg_to_peer(
569#KnDqPt|                    &foreign_network_name,
570#5wz9iB|                    foreign_peer_id,
571#HFocUC|                    packet.foreign_network_packet(),
572#cN3xS-|                )
573#PcpKA_|                .await
574#eP03rc|            {
575#CW4xPB|                tracing::debug!(
576#2EUTrQ|                    ?e,
577#EwHmim|                    ?foreign_network_name,
578#P4MAxf|                    ?foreign_peer_id,
579#H3knNr|                    "foreign network mgr send_msg_to_peer failed"
580#iyW0eL|                );
581#hq_fcK|            }
582#Y1ZmOf|            Ok(())
583#YIUK0N|        } else if Some(from_peer_id) == foreign_network_my_peer_id {
584#mwhHwz|            // to_peer_id is my peer id for the foreign network, need to convert to the origin my_peer_id of dst
585#1Cllfa|            let Some(to_peer_id) = peer_map
586#vOHCEe|                .get_origin_my_peer_id(&foreign_network_name, to_peer_id)
587#PcpKA_|                .await
588#HO4mJ1|            else {
589#CW4xPB|                tracing::debug!(
590#EwHmim|                    ?foreign_network_name,
591#64bVg4|                    ?to_peer_id,
592#Fa_5NR|                    "cannot find origin my peer id for foreign network."
593#iyW0eL|                );
594#1QMLIr|                return Err(packet);
595#huZslw|            };
596#47DEQp|
597#YUb47C|            add_counter(
598#isNHJA|                MetricName::TrafficBytesForeignForwardTx,
599#Yut8ka|                MetricName::TrafficPacketsForeignForwardTx,
600#TP8Q3D|            );
601#47DEQp|
602#qFCK98|            // modify the to_peer id from foreign network my peer id to the origin my peer id
603#1qxWD1|            packet
604#0lMrqe|                .mut_peer_manager_header()
605#gQk2jP|                .unwrap()
606#2oYuph|                .to_peer_id
607#1FcmIL|                .set(to_peer_id);
608#47DEQp|
609#WWRUyf|            // packet is generated from foreign network mgr and should be forward to other peer
610#GjxVjo|            if let Err(e) = peer_map
611#pWezG5|                .send_msg(packet, to_peer_id, NextHopPolicy::LeastHop)
612#PcpKA_|                .await
613#eP03rc|            {
614#CW4xPB|                tracing::debug!(
615#2EUTrQ|                    ?e,
616#64bVg4|                    ?to_peer_id,
617#0xHh2s|                    "send_msg_directly failed when forward local generated foreign network packet"
618#iyW0eL|                );
619#hq_fcK|            }
620#Y1ZmOf|            Ok(())
621#d4ha1c|        } else {
622#b_Rzck|            // target is not me, forward it. try get origin peer id
623#YUb47C|            add_counter(
624#iZmaoy|                MetricName::TrafficBytesForeignForwardForwarded,
625#1fxT_q|                MetricName::TrafficPacketsForeignForwardForwarded,
626#TP8Q3D|            );
627#BKXjm4|            Err(packet)
628#KkQWgx|        }
629#KNhneG|    }
630#47DEQp|
631#FHuK_1|    async fn start_peer_recv(&self) {
632#37sTF-|        let mut recv = self.packet_recv.lock().await.take().unwrap();
633#vnR7aH|        let my_peer_id = self.my_peer_id;
634#nmPmgi|        let peers = self.peers.clone();
635#97eb5s|        let pipe_line = self.peer_packet_process_pipeline.clone();
636#8aTJMr|        let foreign_client = self.foreign_network_client.clone();
637#C546Zb|        let foreign_mgr = self.foreign_network_manager.clone();
638#k_2-Ol|        let encryptor = self.encryptor.clone();
639#mPlkH9|        let compress_algo = self.data_compress_algo;
640#GRlbc0|        let acl_filter = self.global_ctx.get_acl_filter().clone();
641#ZGLiLv|        let global_ctx = self.global_ctx.clone();
642#UZrWMo|        let stats_mgr = self.global_ctx.stats_manager().clone();
643#9sGRWS|        let route = self.get_route();
644#47DEQp|
645#5HN3PJ|        let label_set =
646#aDVoKb|            LabelSet::new().with_label_type(LabelType::NetworkName(global_ctx.get_network_name()));
647#47DEQp|
648#mEGOr9|        let self_tx_bytes = self.self_tx_counters.self_tx_bytes.clone();
649#T7N9pS|        let self_tx_packets = self.self_tx_counters.self_tx_packets.clone();
650#hCUNVJ|        let self_rx_bytes =
651#80PMSq|            stats_mgr.get_counter(MetricName::TrafficBytesSelfRx, label_set.clone());
652#DmhFt-|        let self_rx_packets =
653#DmhvAH|            stats_mgr.get_counter(MetricName::TrafficPacketsSelfRx, label_set.clone());
654#FP31iZ|        let forward_tx_bytes =
655#mIb08E|            stats_mgr.get_counter(MetricName::TrafficBytesForwarded, label_set.clone());
656#N6gzzb|        let forward_tx_packets =
657#nCT_oP|            stats_mgr.get_counter(MetricName::TrafficPacketsForwarded, label_set.clone());
658#47DEQp|
659#9nvSTI|        let compress_tx_bytes_before = self.self_tx_counters.compress_tx_bytes_before.clone();
660#8Ehyj-|        let compress_tx_bytes_after = self.self_tx_counters.compress_tx_bytes_after.clone();
661#dMW85W|        let compress_rx_bytes_before =
662#Jryksd|            stats_mgr.get_counter(MetricName::CompressionBytesRxBefore, label_set.clone());
663#CJs0ZN|        let compress_rx_bytes_after =
664#OkiaLh|            stats_mgr.get_counter(MetricName::CompressionBytesRxAfter, label_set.clone());
665#47DEQp|
666#DDENzd|        self.tasks.lock().await.spawn(async move {
667#s1vPO9|            tracing::trace!("start_peer_recv");
668#z3Goih|            while let Ok(ret) = recv_packet_from_chan(&mut recv).await {
669#w5bSOZ|                let Err(mut ret) =
670#E9vP_K|                    Self::try_handle_foreign_network_packet(ret, my_peer_id, &peers, &foreign_mgr)
671#o6c0n-|                        .await
672#QIIDCb|                else {
673#_01Y-5|                    continue;
674#JBqzQb|                };
675#47DEQp|
676#z4xbBM|                let buf_len = ret.buf_len();
677#sLsb9g|                let Some(hdr) = ret.mut_peer_manager_header() else {
678#eP2t8R|                    tracing::warn!(?ret, "invalid packet, skip");
679#_01Y-5|                    continue;
680#JBqzQb|                };
681#47DEQp|
682#ax0YAR|                tracing::trace!(?hdr, "peer recv a packet...");
683#G1tVuV|                let from_peer_id = hdr.from_peer_id.get();
684#i2EURd|                let to_peer_id = hdr.to_peer_id.get();
685#Cuycos|                if to_peer_id != my_peer_id {
686#JRJHei|                    if hdr.forward_counter > 7 {
687#F8mduZ|                        tracing::warn!(?hdr, "forward counter exceed, drop packet");
688#W98xla|                        continue;
689#uNGRgX|                    }
690#47DEQp|
691#pwyl0-|                    if hdr.forward_counter > 2 && hdr.is_latency_first() {
692#uVS30W|                        tracing::trace!(?hdr, "set_latency_first false because too many hop");
693#9ndRqd|                        hdr.set_latency_first(false);
694#uNGRgX|                    }
695#47DEQp|
696#VdVaow|                    hdr.forward_counter += 1;
697#47DEQp|
698#ZT0lS_|                    if from_peer_id == my_peer_id {
699#uroO3I|                        compress_tx_bytes_before.add(buf_len as u64);
700#47DEQp|
701#iGD8zw|                        if hdr.packet_type == PacketType::Data as u8
702#GABpVR|                            || hdr.packet_type == PacketType::KcpSrc as u8
703#Z6rbMg|                            || hdr.packet_type == PacketType::KcpDst as u8
704#Fzqoyl|                        {
705#1LI0Vo|                            let _ =
706#JUJFc0|                                Self::try_compress_and_encrypt(compress_algo, &encryptor, &mut ret)
707#1yUlMs|                                    .await;
708#qrgf36|                        }
709#47DEQp|
710#T0tOoJ|                        compress_tx_bytes_after.add(ret.buf_len() as u64);
711#tANDt7|                        self_tx_bytes.add(ret.buf_len() as u64);
712#f8O7M7|                        self_tx_packets.inc();
713#HyGTCY|                    } else {
714#Pguetm|                        forward_tx_bytes.add(buf_len as u64);
715#gmyQh8|                        forward_tx_packets.inc();
716#uNGRgX|                    }
717#47DEQp|
718#kSLxh3|                    tracing::trace!(?to_peer_id, ?my_peer_id, "need forward");
719#b8Js0t|                    let ret =
720#EZHc3h|                        Self::send_msg_internal(&peers, &foreign_client, ret, to_peer_id).await;
721#LXf7AG|                    if ret.is_err() {
722#RK5a5R|                        tracing::error!(?ret, ?to_peer_id, ?from_peer_id, "forward packet error");
723#uNGRgX|                    }
724#ZtcDW9|                } else {
725#fn_fyy|                    if let Err(e) = encryptor.decrypt(&mut ret) {
726#HZCGH0|                        tracing::error!(?e, "decrypt failed");
727#W98xla|                        continue;
728#uNGRgX|                    }
729#47DEQp|
730#Ln7UBJ|                    self_rx_bytes.add(buf_len as u64);
731#XNjtrJ|                    self_rx_packets.inc();
732#BHaA-C|                    compress_rx_bytes_before.add(buf_len as u64);
733#47DEQp|
734#QblX98|                    let compressor = DefaultCompressor {};
735#GMZW0O|                    if let Err(e) = compressor.decompress(&mut ret).await {
736#gbQ6SV|                        tracing::error!(?e, "decompress failed");
737#W98xla|                        continue;
738#uNGRgX|                    }
739#47DEQp|
740#Hjej0y|                    compress_rx_bytes_after.add(ret.buf_len() as u64);
741#47DEQp|
742#2dQ5pH|                    if !acl_filter.process_packet_with_acl(
743#NQJi05|                        &ret,
744#VaXc6J|                        true,
745#NFzhEu|                        global_ctx.get_ipv4().map(|x| x.address()),
746#NvLih8|                        global_ctx.get_ipv6().map(|x| x.address()),
747#5sWIyd|                        &route,
748#SwK0Oa|                    ) {
749#W98xla|                        continue;
750#uNGRgX|                    }
751#47DEQp|
752#cqHypd|                    let mut processed = false;
753#fYLeTn|                    let mut zc_packet = Some(ret);
754#IhX6Kq|                    for (idx, pipeline) in pipe_line.read().await.iter().rev().enumerate() {
755#r-rtuq|                        tracing::trace!(?zc_packet, ?idx, "try_process_packet_from_peer");
756#0x6iPo|                        zc_packet = pipeline
757#D_qrXs|                            .try_process_packet_from_peer(zc_packet.unwrap())
758#IY999y|                            .await;
759#Ew5lXA|                        if zc_packet.is_none() {
760#10AbF1|                            processed = true;
761#Ko0WZT|                            break;
762#qrgf36|                        }
763#uNGRgX|                    }
764#R2opY7|                    if !processed {
765#BN52j-|                        tracing::error!(?zc_packet, "unhandled packet");
766#uNGRgX|                    }
767#C_CgiK|                }
768#hq_fcK|            }
769#H99HIN|            panic!("done_peer_recv");
770#NQ-BLD|        });
771#KNhneG|    }
772#47DEQp|
773#q_JTCC|    pub async fn add_packet_process_pipeline(&self, pipeline: BoxPeerPacketFilter) {
774#5xAyFb|        // newest pipeline will be executed first
775#9k-qPG|        self.peer_packet_process_pipeline
776#jtBdos|            .write()
777#icqUnQ|            .await
778#gbkGf8|            .push(pipeline);
779#KNhneG|    }
780#47DEQp|
781#ekn-0q|    pub async fn add_nic_packet_process_pipeline(&self, pipeline: BoxNicPacketFilter) {
782#5xAyFb|        // newest pipeline will be executed first
783#lgMoPV|        self.nic_packet_process_pipeline
784#jtBdos|            .write()
785#icqUnQ|            .await
786#gbkGf8|            .push(pipeline);
787#KNhneG|    }
788#47DEQp|
789#rwu4HQ|    async fn init_packet_process_pipeline(&self) {
790#LJfv5W|        // for tun/tap ip/eth packet.
791#hwmngc|        struct NicPacketProcessor {
792#My7Qci|            nic_channel: PacketRecvChan,
793#KkQWgx|        }
794#kD-9ea|        #[async_trait::async_trait]
795#OtOTlR|        impl PeerPacketFilter for NicPacketProcessor {
796#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
797#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
798#L2pk7f|                if hdr.packet_type == PacketType::Data as u8 {
799#dx_3aF|                    tracing::trace!(?packet, "send packet to nic channel");
800#9RlVs2|                    // TODO: use a function to get the body ref directly for zero copy
801#D1yJAP|                    let _ = self.nic_channel.send(packet).await;
802#JqjnWy|                    None
803#ZtcDW9|                } else {
804#jLWmFO|                    Some(packet)
805#C_CgiK|                }
806#hq_fcK|            }
807#KkQWgx|        }
808#zvD7Bg|        self.add_packet_process_pipeline(Box::new(NicPacketProcessor {
809#PTKjfM|            nic_channel: self.nic_channel.clone(),
810#AVIbAB|        }))
811#10YU6B|        .await;
812#47DEQp|
813#s7MHfd|        // for peer rpc packet
814#N1Iaxj|        struct PeerRpcPacketProcessor {
815#Lx--BG|            peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
816#KkQWgx|        }
817#47DEQp|
818#kD-9ea|        #[async_trait::async_trait]
819#D1zWOn|        impl PeerPacketFilter for PeerRpcPacketProcessor {
820#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
821#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
822#CrfTXZ|                if hdr.packet_type == PacketType::TaRpc as u8
823#TBQp0z|                    || hdr.packet_type == PacketType::RpcReq as u8
824#6aiYqm|                    || hdr.packet_type == PacketType::RpcResp as u8
825#NV9RY5|                {
826#DWqudi|                    self.peer_rpc_tspt_sender.send(packet).unwrap();
827#JqjnWy|                    None
828#ZtcDW9|                } else {
829#jLWmFO|                    Some(packet)
830#C_CgiK|                }
831#hq_fcK|            }
832#KkQWgx|        }
833#4XfYK9|        self.add_packet_process_pipeline(Box::new(PeerRpcPacketProcessor {
834#Nc02qB|            peer_rpc_tspt_sender: self.peer_rpc_tspt.peer_rpc_tspt_sender.clone(),
835#AVIbAB|        }))
836#10YU6B|        .await;
837#KNhneG|    }
838#47DEQp|
839#qnv61H|    pub async fn add_route<T>(&self, route: T)
840#JSvlCU|    where
841#s0mP8g|        T: Route + PeerPacketFilter + Send + Sync + Clone + 'static,
842#DT0vWo|    {
843#YR42TL|        // for route
844#DOPJKV|        self.add_packet_process_pipeline(Box::new(route.clone()))
845#agFGm5|            .await;
846#47DEQp|
847#hY75HP|        struct Interface {
848#y1Tqlc|            my_peer_id: PeerId,
849#rrAPV3|            peers: Weak<PeerMap>,
850#21pgeS|            foreign_network_client: Weak<ForeignNetworkClient>,
851#8Gbfqs|            foreign_network_manager: Weak<ForeignNetworkManager>,
852#KkQWgx|        }
853#47DEQp|
854#KGY7Sc|        #[async_trait]
855#DkYunO|        impl RouteInterface for Interface {
856#l_VjGY|            async fn list_peers(&self) -> Vec<PeerId> {
857#XDZ2Vk|                let Some(foreign_client) = self.foreign_network_client.upgrade() else {
858#W_IXmF|                    return vec![];
859#JBqzQb|                };
860#47DEQp|
861#PUDAeF|                let Some(peer_map) = self.peers.upgrade() else {
862#W_IXmF|                    return vec![];
863#JBqzQb|                };
864#47DEQp|
865#XQDNH1|                let mut peers = foreign_client.list_public_peers().await;
866#njwKeW|                peers.extend(peer_map.list_peers_with_conn().await);
867#MTa5cE|                peers
868#hq_fcK|            }
869#47DEQp|
870#LFp0oo|            fn my_peer_id(&self) -> PeerId {
871#AE5UEx|                self.my_peer_id
872#hq_fcK|            }
873#47DEQp|
874#uonLKK|            async fn list_foreign_networks(&self) -> ForeignNetworkRouteInfoMap {
875#5THHmO|                let ret = DashMap::new();
876#swf1dg|                let Some(foreign_mgr) = self.foreign_network_manager.upgrade() else {
877#b5Svxi|                    return ret;
878#JBqzQb|                };
879#47DEQp|
880#Y8o-Yf|                let networks = foreign_mgr.list_foreign_networks().await;
881#ioetyj|                for (network_name, info) in networks.foreign_networks.iter() {
882#JV3_bG|                    if info.peers.is_empty() {
883#W98xla|                        continue;
884#uNGRgX|                    }
885#47DEQp|
886#LK7acJ|                    let last_update = foreign_mgr
887#JxC-fm|                        .get_foreign_network_last_update(network_name)
888#v0277T|                        .unwrap_or(SystemTime::now());
889#4cuRRV|                    ret.insert(
890#zuxFX7|                        ForeignNetworkRouteInfoKey {
891#FvjDof|                            peer_id: self.my_peer_id,
892#MBDbjS|                            network_name: network_name.clone(),
893#xt3MAa|                        },
894#WTdtwn|                        ForeignNetworkRouteInfoEntry {
895#nSGc8d|                            foreign_peer_ids: info.peers.iter().map(|x| x.peer_id).collect(),
896#DEQ9Fe|                            last_update: Some(last_update.into()),
897#QBzroT|                            version: 0,
898#rLkInO|                            network_secret_digest: info.network_secret_digest.clone(),
899#5efuR1|                            my_peer_id_for_this_network: info.my_peer_id_for_this_network,
900#xt3MAa|                        },
901#iTFhZ0|                    );
902#C_CgiK|                }
903#0qn6Vo|                ret
904#hq_fcK|            }
905#KkQWgx|        }
906#47DEQp|
907#vnR7aH|        let my_peer_id = self.my_peer_id;
908#T7G1Zk|        let _route_id = route
909#SfQrJo|            .open(Box::new(Interface {
910#hGlxsx|                my_peer_id,
911#0F1sFB|                peers: Arc::downgrade(&self.peers),
912#ER66ww|                foreign_network_client: Arc::downgrade(&self.foreign_network_client),
913#yRkAf3|                foreign_network_manager: Arc::downgrade(&self.foreign_network_manager),
914#LufvaI|            }))
915#icqUnQ|            .await
916#WEGyMY|            .unwrap();
917#47DEQp|
918#ORaZ3y|        let arc_route: ArcRoute = Arc::new(Box::new(route));
919#fY7N7T|        self.peers.add_route(arc_route).await;
920#KNhneG|    }
921#47DEQp|
922#rj8ACA|    pub fn get_route(&self) -> Box<dyn Route + Send + Sync + 'static> {
923#aWnGNt|        match &self.route_algo_inst {
924#ind_vT|            RouteAlgoInst::Ospf(route) => Box::new(route.clone()),
925#3ENyBM|            RouteAlgoInst::None => Box::new(MockRoute {}),
926#KkQWgx|        }
927#KNhneG|    }
928#47DEQp|
929#ggsQcx|    pub async fn list_routes(&self) -> Vec<cli::Route> {
930#buqXa2|        self.get_route().list_routes().await
931#KNhneG|    }
932#47DEQp|
933#SQwiYH|    pub async fn get_route_peer_info_last_update_time(&self) -> Instant {
934#o0Hn44|        self.get_route().get_peer_info_last_update_time().await
935#KNhneG|    }
936#47DEQp|
937#rMN99o|    pub async fn dump_route(&self) -> String {
938#NjIrQ5|        self.get_route().dump().await
939#KNhneG|    }
940#47DEQp|
941#qHMIGl|    pub async fn list_global_foreign_network(&self) -> ListGlobalForeignNetworkResponse {
942#dSN10n|        let mut resp = ListGlobalForeignNetworkResponse::default();
943#0GRtAb|        let ret = self.get_route().list_foreign_network_info().await;
944#PfLspy|        for info in ret.infos.iter() {
945#los8mn|            let entry = resp
946#cAKnz7|                .foreign_networks
947#x4ul2a|                .entry(info.key.as_ref().unwrap().peer_id)
948#ctxgtd|                .or_insert_with(Default::default);
949#MiV-NN|            let Some(route_info) = info.value.as_ref() else {
950#YqjFPd|                continue;
951#huZslw|            };
952#47DEQp|
953#ubAyqu|            let f = OneForeignNetwork {
954#1WNjy7|                network_name: info.key.as_ref().unwrap().network_name.clone(),
955#LogZYQ|                peer_ids: route_info.foreign_peer_ids.clone(),
956#RIHZBu|                last_updated: format!("{}", route_info.last_update.unwrap()),
957#hHIWcX|                version: route_info.version,
958#huZslw|            };
959#47DEQp|
960#TZwkyd|            entry.foreign_networks.push(f);
961#KkQWgx|        }
962#47DEQp|
963#ZudY7T|        resp
964#KNhneG|    }
965#47DEQp|
966#J0LLTy|    pub async fn get_foreign_network_summary(&self) -> RouteForeignNetworkSummary {
967#yFVVTB|        self.get_route().get_foreign_network_summary().await
968#KNhneG|    }
969#47DEQp|
970#WFyYU9|    async fn run_nic_packet_process_pipeline(&self, data: &mut ZCPacket) {
971#81LGkF|        if !self.global_ctx.get_acl_filter().process_packet_with_acl(
972#RSksDL|            data,
973#bBjPxJ|            false,
974#SKXxAq|            None,
975#SKXxAq|            None,
976#HUVukP|            &self.get_route(),
977#HceTsT|        ) {
978#NhdEjk|            return;
979#KkQWgx|        }
980#47DEQp|
981#AzFZTs|        for pipeline in self.nic_packet_process_pipeline.read().await.iter().rev() {
982#iJa1xN|            let _ = pipeline.try_process_packet_from_nic(data).await;
983#KkQWgx|        }
984#KNhneG|    }
985#47DEQp|
986#LNmNxR|    pub async fn remove_nic_packet_process_pipeline(&self, id: String) -> Result<(), Error> {
987#kzaKYh|        let mut pipelines = self.nic_packet_process_pipeline.write().await;
988#aIOBXg|        if let Some(pos) = pipelines.iter().position(|x| x.id() == id) {
989#pyEoMy|            pipelines.remove(pos);
990#Y1ZmOf|            Ok(())
991#d4ha1c|        } else {
992#j9KMDj|            Err(Error::NotFound)
993#KkQWgx|        }
994#KNhneG|    }
995#47DEQp|
996#QwGVbl|    fn get_next_hop_policy(is_first_latency: bool) -> NextHopPolicy {
997#Prtm2m|        if is_first_latency {
998#lrGTD9|            NextHopPolicy::LeastCost
999#d4ha1c|        } else {
1000#0t3aFq|            NextHopPolicy::LeastHop
1001#KkQWgx|        }
1002#KNhneG|    }
1003#47DEQp|
1004#YcV_Zw|    pub async fn send_msg(&self, msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
1005#0Y1ljl|        self.self_tx_counters
1006#7boxeQ|            .self_tx_bytes
1007#OREiCd|            .add(msg.buf_len() as u64);
1008#P1nqM0|        self.self_tx_counters.self_tx_packets.inc();
1009#X4VtRb|        let msg_len = msg.buf_len() as u64;
1010#X_Kl5i|        let result =
1011#Pd9f-1|            Self::send_msg_internal(&self.peers, &self.foreign_network_client, msg, dst_peer_id)
1012#mju4UJ|                .await;
1013#ZxIhWe|        if result.is_ok() {
1014#wEbH0X|            self.self_tx_counters.self_tx_bytes.add(msg_len);
1015#zhexxv|            self.self_tx_counters.self_tx_packets.inc();
1016#KkQWgx|        }
1017#NMBvw3|        result
1018#KNhneG|    }
1019#47DEQp|
1020#zhhnC3|    async fn send_msg_internal(
1021#swj2Tw|        peers: &Arc<PeerMap>,
1022#rQ13r4|        foreign_network_client: &Arc<ForeignNetworkClient>,
1023#i65YQK|        msg: ZCPacket,
1024#LiSCIf|        dst_peer_id: PeerId,
1025#Ew16UP|    ) -> Result<(), Error> {
1026#Vg3OfO|        let policy =
1027#s6Hg6i|            Self::get_next_hop_policy(msg.peer_manager_header().unwrap().is_latency_first());
1028#47DEQp|
1029#S-yTsc|        if let Some(gateway) = peers.get_gateway_peer_id(dst_peer_id, policy.clone()).await {
1030#CVigK7|            if peers.has_peer(gateway) {
1031#5Mp0l-|                peers.send_msg_directly(msg, gateway).await
1032#JP80iE|            } else if foreign_network_client.has_next_hop(gateway) {
1033#znEXzS|                foreign_network_client.send_msg(msg, gateway).await
1034#CE4VEs|            } else {
1035#tw81bP|                tracing::warn!(
1036#kk6F7t|                    ?gateway,
1037#_vSdBB|                    ?dst_peer_id,
1038#5aLh6k|                    "cannot send msg to peer through gateway"
1039#iyW0eL|                );
1040#rw6i9c|                Err(Error::RouteError(None))
1041#hq_fcK|            }
1042#OEJCrt|        } else if foreign_network_client.has_next_hop(dst_peer_id) {
1043#hzCk3v|            // check foreign network again. so in happy path we can avoid extra check
1044#YLJ9ty|            foreign_network_client.send_msg(msg, dst_peer_id).await
1045#d4ha1c|        } else {
1046#kVR-Se|            tracing::debug!(?dst_peer_id, "no gateway for peer");
1047#NuwARt|            Err(Error::RouteError(None))
1048#KkQWgx|        }
1049#KNhneG|    }
1050#47DEQp|
1051#n2tvl8|    pub async fn get_msg_dst_peer(&self, ipv4_addr: &Ipv4Addr) -> (Vec<PeerId>, bool) {
1052#YR8kEa|        let mut is_exit_node = false;
1053#5eR7LF|        let mut dst_peers = vec![];
1054#FZMhE_|        let network_length = self
1055#7t5vZi|            .global_ctx
1056#THImhK|            .get_ipv4()
1057#8fe7wM|            .map(|x| x.network_length())
1058#y5RfWx|            .unwrap_or(24);
1059#D0sue1|        let ipv4_inet = cidr::Ipv4Inet::new(*ipv4_addr, network_length).unwrap();
1060#vpDCah|        if ipv4_addr.is_broadcast()
1061#lwUXjr|            || ipv4_addr.is_multicast()
1062#QttO4f|            || *ipv4_addr == ipv4_inet.last_address()
1063#qWZHcZ|        {
1064#poVYD0|            dst_peers.extend(self.peers.list_routes().await.iter().filter_map(|x| {
1065#r49BH8|                if *x.key() != self.my_peer_id {
1066#RZUfe_|                    Some(*x.key())
1067#ZtcDW9|                } else {
1068#JqjnWy|                    None
1069#C_CgiK|                }
1070#Hsxh2D|            }));
1071#vflSHi|        } else if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(ipv4_addr).await {
1072#otf58d|            dst_peers.push(peer_id);
1073#pegr-g|        } else if !self
1074#7t5vZi|            .global_ctx
1075#VvsySw|            .is_ip_in_same_network(&std::net::IpAddr::V4(*ipv4_addr))
1076#qWZHcZ|        {
1077#jQL-Kg|            for exit_node in &self.exit_nodes {
1078#yDUUhE|                let IpAddr::V4(exit_node) = exit_node else {
1079#_01Y-5|                    continue;
1080#JBqzQb|                };
1081#t182g9|                if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(exit_node).await {

...35440 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3c4ea4001hjD2g5sktUcQAb
Use Grep to search the full content or Read with offset/limit to view specific sections.
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_map.rs","offset":275,"limit":30}) [completed]
1#JkvAPI|use std::{
2#8mazTT|    net::{Ipv4Addr, Ipv6Addr},
3#WxoiW7|    sync::Arc,
4#3zmC_W|};
5#47DEQp|
6#woeOSu|use anyhow::Context;
7#a1CEkS|use dashmap::{DashMap, DashSet};
8#D2xR6v|use tokio::sync::RwLock;
9#47DEQp|
10#8h8Z46|use crate::{
11#hIkMG-|    common::{
12#Qez2zE|        error::Error,
13#d2v0pW|        global_ctx::{ArcGlobalCtx, GlobalCtxEvent, NetworkIdentity},
14#yr1lub|        PeerId,
15#WGNt2R|    },
16#Js7gIl|    proto::{
17#kWa7Vc|        cli::{self, PeerConnInfo},
18#7GkkPH|        peer_rpc::RoutePeerInfo,
19#WGNt2R|    },
20#ClWcxR|    tunnel::{packet_def::ZCPacket, TunnelError},
21#3zmC_W|};
22#47DEQp|
23#Z2Y0cv|use super::{
24#UmlIjl|    peer::Peer,
25#JWQyPm|    peer_conn::{PeerConn, PeerConnId},
26#pLB8L1|    route_trait::{ArcRoute, NextHopPolicy},
27#Hao9WQ|    PacketRecvChan,
28#3zmC_W|};
29#47DEQp|
30#_pwnmT|pub struct PeerMap {
31#sfM3Lj|    global_ctx: ArcGlobalCtx,
32#ug-G5V|    my_peer_id: PeerId,
33#eecNlm|    peer_map: DashMap<PeerId, Arc<Peer>>,
34#9ZkHk2|    packet_send: PacketRecvChan,
35#klRo0P|    routes: RwLock<Vec<ArcRoute>>,
36#PM4hWk|    alive_conns: Arc<DashMap<(PeerId, PeerConnId), PeerConnInfo>>,
37#0Qs2qn|}
38#47DEQp|
39#c-pqth|impl PeerMap {
40#OUGVNh|    pub fn new(packet_send: PacketRecvChan, global_ctx: ArcGlobalCtx, my_peer_id: PeerId) -> Self {
41#1N1OyG|        PeerMap {
42#PvFYEj|            global_ctx,
43#907ngS|            my_peer_id,
44#IruKsh|            peer_map: DashMap::new(),
45#X4qo8z|            packet_send,
46#3Dh2XX|            routes: RwLock::new(Vec::new()),
47#2f2SEz|            alive_conns: Arc::new(DashMap::new()),
48#KkQWgx|        }
49#KNhneG|    }
50#47DEQp|
51#XlJMiK|    async fn add_new_peer(&self, peer: Peer) {
52#n4b96r|        let peer_id = peer.peer_node_id;
53#De22W5|        self.peer_map.insert(peer_id, Arc::new(peer));
54#G6v1RW|        self.global_ctx
55#tbIbEA|            .issue_event(GlobalCtxEvent::PeerAdded(peer_id));
56#KNhneG|    }
57#47DEQp|
58#2myMP0|    pub async fn add_new_peer_conn(&self, peer_conn: PeerConn) {
59#8Mb-z5|        self.maintain_alive_conns(&peer_conn);
60#ofM7n0|        let peer_id = peer_conn.get_peer_id();
61#m06fVf|        let no_entry = self.peer_map.get(&peer_id).is_none();
62#rdpO-Y|        if no_entry {
63#HLoY0I|            let new_peer = Peer::new(peer_id, self.packet_send.clone(), self.global_ctx.clone());
64#H7EFGW|            new_peer.add_peer_conn(peer_conn).await;
65#AKGZrs|            self.add_new_peer(new_peer).await;
66#d4ha1c|        } else {
67#8JTLQp|            let peer = self.peer_map.get(&peer_id).unwrap().clone();
68#GIkF7F|            peer.add_peer_conn(peer_conn).await;
69#KkQWgx|        }
70#KNhneG|    }
71#47DEQp|
72#hrdaXh|    fn maintain_alive_conns(&self, peer_conn: &PeerConn) {
73#NG_Pae|        let close_notifier = peer_conn.get_close_notifier();
74#i6ecPU|        let alive_conns_weak = Arc::downgrade(&self.alive_conns);
75#cYRlAh|        let conn_id = close_notifier.get_conn_id();
76#vUFqU6|        let conn_info = peer_conn.get_conn_info();
77#w8127h|        self.alive_conns
78#5ZoiQa|            .insert((conn_info.peer_id, conn_id), conn_info.clone());
79#b9LDnJ|        tokio::spawn(async move {
80#5ztD8Z|            if let Some(mut waiter) = close_notifier.get_waiter().await {
81#7Ax_0q|                let _ = waiter.recv().await;
82#hq_fcK|            }
83#dII6hH|            let mut alive_conn_count = 0;
84#dTEELD|            if let Some(alive_conns) = alive_conns_weak.upgrade() {
85#hlBFL2|                alive_conns.remove(&(conn_info.peer_id, conn_id)).unwrap();
86#wrDnNu|                alive_conn_count = alive_conns.len();
87#hq_fcK|            }
88#fcB9Dy|            tracing::debug!(
89#6TGP0z|                ?conn_id,
90#u9L1zN|                "peer conn is closed, current alive conns: {}",
91#eivXZv|                alive_conn_count
92#TP8Q3D|            );
93#NQ-BLD|        });
94#KNhneG|    }
95#47DEQp|
96#sfcqu7|    pub fn get_peer_by_id(&self, peer_id: PeerId) -> Option<Arc<Peer>> {
97#fZuGoE|        self.peer_map.get(&peer_id).map(|v| v.clone())
98#KNhneG|    }
99#47DEQp|
100#CnMOHa|    pub fn get_directly_connections_by_peer_id(&self, peer_id: PeerId) -> DashSet<uuid::Uuid> {
101#9oBa3H|        if let Some(peer) = self.get_peer_by_id(peer_id) {
102#VKiljz|            return peer.get_directly_connections();
103#KkQWgx|        }
104#47DEQp|
105#kJRvWY|        DashSet::new()
106#KNhneG|    }
107#47DEQp|
108#VEN8Zw|    pub fn has_peer(&self, peer_id: PeerId) -> bool {
109#B63652|        peer_id == self.my_peer_id || self.peer_map.contains_key(&peer_id)
110#KNhneG|    }
111#47DEQp|
112#DbfmS4|    pub async fn send_msg_directly(&self, msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
113#RfRj_s|        if dst_peer_id == self.my_peer_id {
114#8ejvsg|            let packet_send = self.packet_send.clone();
115#gNYhpy|            tokio::spawn(async move {
116#uSsj-w|                let ret = packet_send
117#246MjH|                    .send(msg)
118#lYNw0O|                    .await
119#6OvsUe|                    .with_context(|| "send msg to self failed");
120#r2cOm0|                if ret.is_err() {
121#z187zC|                    tracing::error!("send msg to self failed: {:?}", ret);
122#C_CgiK|                }
123#-JdJBA|            });
124#6WCzfY|            return Ok(());
125#KkQWgx|        }
126#47DEQp|
127#7sRcBZ|        match self.get_peer_by_id(dst_peer_id) {
128#Pf5nYY|            Some(peer) => {
129#o8UaIa|                peer.send_msg(msg).await?;
130#hq_fcK|            }
131#p2SdXN|            None => {
132#BsWMan|                tracing::error!("no peer for dst_peer_id: {}", dst_peer_id);
133#yvU-od|                return Err(Error::RouteError(Some(format!(
134#oiLCNU|                    "peer map sengmsg directly no connected dst_peer_id: {}",
135#PlqiUY|                    dst_peer_id
136#JIIC1E|                ))));
137#hq_fcK|            }
138#KkQWgx|        }
139#47DEQp|
140#sX41Gp|        Ok(())
141#KNhneG|    }
142#47DEQp|
143#Bzhlvk|    pub async fn get_gateway_peer_id(
144#7BRTWY|        &self,
145#LiSCIf|        dst_peer_id: PeerId,
146#F4Rim3|        policy: NextHopPolicy,
147#NNNbmV|    ) -> Option<PeerId> {
148#RfRj_s|        if dst_peer_id == self.my_peer_id {
149#tEXxAh|            return Some(dst_peer_id);
150#KkQWgx|        }
151#47DEQp|
152#p3zRTy|        if self.has_peer(dst_peer_id) && matches!(policy, NextHopPolicy::LeastHop) {
153#tEXxAh|            return Some(dst_peer_id);
154#KkQWgx|        }
155#47DEQp|
156#FBO1Ge|        // get route info
157#O2RPx6|        for route in self.routes.read().await.iter() {
158#GkeXvY|            if let Some(gateway_peer_id) = route
159#34XONE|                .get_next_hop_with_policy(dst_peer_id, policy.clone())
160#PcpKA_|                .await
161#eP03rc|            {
162#aJtdz4|                // NOTIC: for foreign network, gateway_peer_id may not connect to me
163#OtE5rc|                return Some(gateway_peer_id);
164#hq_fcK|            }
165#KkQWgx|        }
166#47DEQp|
167#jan1Ge|        None
168#KNhneG|    }
169#47DEQp|
170#omLDWw|    pub async fn list_peers_own_foreign_network(
171#7BRTWY|        &self,
172#G_5pTW|        network_identity: &NetworkIdentity,
173#PhPQkT|    ) -> Vec<PeerId> {
174#EkbaIO|        let mut ret = Vec::new();
175#O2RPx6|        for route in self.routes.read().await.iter() {
176#2ardLg|            let peers = route.list_peers_own_foreign_network(network_identity).await;
177#WkfWPM|            ret.extend(peers);
178#KkQWgx|        }
179#eL4xpv|        ret
180#KNhneG|    }
181#47DEQp|
182#uDpS7p|    pub async fn send_msg(
183#7BRTWY|        &self,
184#i65YQK|        msg: ZCPacket,
185#LiSCIf|        dst_peer_id: PeerId,
186#F4Rim3|        policy: NextHopPolicy,
187#Ew16UP|    ) -> Result<(), Error> {
188#9jdNdP|        let Some(gateway_peer_id) = self.get_gateway_peer_id(dst_peer_id, policy).await else {
189#0NHhsG|            return Err(Error::RouteError(Some(format!(
190#N87KnY|                "peer map sengmsg no gateway for dst_peer_id: {}",
191#iQcdbn|                dst_peer_id
192#hs1NVi|            ))));
193#8OSK9L|        };
194#47DEQp|
195#yLRbC3|        self.send_msg_directly(msg, gateway_peer_id).await?;
196#sX41Gp|        Ok(())
197#KNhneG|    }
198#47DEQp|
199#Q8m5ig|    pub async fn get_peer_id_by_ipv4(&self, ipv4: &Ipv4Addr) -> Option<PeerId> {
200#O2RPx6|        for route in self.routes.read().await.iter() {
201#uCJ5LM|            let peer_id = route.get_peer_id_by_ipv4(ipv4).await;
202#kmr7Q2|            if peer_id.is_some() {
203#xfJXec|                return peer_id;
204#hq_fcK|            }
205#KkQWgx|        }
206#jan1Ge|        None
207#KNhneG|    }
208#47DEQp|
209#OK9c6v|    pub async fn get_peer_id_by_ipv6(&self, ipv6: &Ipv6Addr) -> Option<PeerId> {
210#O2RPx6|        for route in self.routes.read().await.iter() {
211#xH-yuw|            let peer_id = route.get_peer_id_by_ipv6(ipv6).await;
212#kmr7Q2|            if peer_id.is_some() {
213#xfJXec|                return peer_id;
214#hq_fcK|            }
215#KkQWgx|        }
216#jan1Ge|        None
217#KNhneG|    }
218#47DEQp|
219#TkCM6O|    pub async fn get_route_peer_info(&self, peer_id: PeerId) -> Option<RoutePeerInfo> {
220#O2RPx6|        for route in self.routes.read().await.iter() {
221#pPaZWV|            if let Some(info) = route.get_peer_info(peer_id).await {
222#TaNBcm|                return Some(info);
223#hq_fcK|            }
224#KkQWgx|        }
225#jan1Ge|        None
226#KNhneG|    }
227#47DEQp|
228#9k-kk_|    pub async fn get_origin_my_peer_id(
229#7BRTWY|        &self,
230#Sc-Kyu|        network_name: &str,
231#NIn2qp|        foreign_my_peer_id: PeerId,
232#NNNbmV|    ) -> Option<PeerId> {
233#O2RPx6|        for route in self.routes.read().await.iter() {
234#-Ls4k0|            let origin_peer_id = route
235#ctTOzc|                .get_origin_my_peer_id(network_name, foreign_my_peer_id)
236#mju4UJ|                .await;
237#thUoth|            if origin_peer_id.is_some() {
238#Wfdb8J|                return origin_peer_id;
239#hq_fcK|            }
240#KkQWgx|        }
241#jan1Ge|        None
242#KNhneG|    }
243#47DEQp|
244#EMdGLb|    pub fn is_empty(&self) -> bool {
245#mKC9wT|        self.peer_map.is_empty()
246#KNhneG|    }
247#47DEQp|
248#706Kgj|    pub async fn list_peers(&self) -> Vec<PeerId> {
249#EkbaIO|        let mut ret = Vec::new();
250#VLB-93|        for item in self.peer_map.iter() {
251#9BGsKy|            let peer_id = item.key();
252#yizItv|            ret.push(*peer_id);
253#KkQWgx|        }
254#eL4xpv|        ret
255#KNhneG|    }
256#47DEQp|
257#ixG8Ha|    pub async fn list_peers_with_conn(&self) -> Vec<PeerId> {
258#EkbaIO|        let mut ret = Vec::new();
259#j6sxVE|        let peers = self.list_peers().await;
260#tcGoOF|        for peer_id in peers.iter() {
261#FgwH3H|            let Some(peer) = self.get_peer_by_id(*peer_id) else {
262#YqjFPd|                continue;
263#huZslw|            };
264#HxSj1d|            if !peer.list_peer_conns().await.is_empty() {
265#HzqYmJ|                ret.push(*peer_id);
266#hq_fcK|            }
267#KkQWgx|        }
268#eL4xpv|        ret
269#KNhneG|    }
270#47DEQp|
271#CTyMWs|    pub async fn list_peer_conns(&self, peer_id: PeerId) -> Option<Vec<PeerConnInfo>> {
272#amn0ap|        if let Some(p) = self.get_peer_by_id(peer_id) {
273#ft7nGL|            Some(p.list_peer_conns().await)
274#d4ha1c|        } else {
275#T3LJbU|            None
276#KkQWgx|        }
277#KNhneG|    }
278#47DEQp|
279#zktFhz|    pub async fn get_peer_default_conn_id(&self, peer_id: PeerId) -> Option<PeerConnId> {
280#W2qZ4u|        self.get_peer_by_id(peer_id)
281#uOL2SF|            .map(|p| p.get_default_conn_id())
282#KNhneG|    }
283#47DEQp|
284#FiOS6u|    pub async fn close_peer_conn(
285#7BRTWY|        &self,
286#ECN-1O|        peer_id: PeerId,
287#uPXWFD|        conn_id: &PeerConnId,
288#Ew16UP|    ) -> Result<(), Error> {
289#amn0ap|        if let Some(p) = self.get_peer_by_id(peer_id) {
290#KXrkJc|            p.close_peer_conn(conn_id).await
291#d4ha1c|        } else {
292#j9KMDj|            Err(Error::NotFound)
293#KkQWgx|        }
294#KNhneG|    }
295#47DEQp|
296#9nM7J3|    pub async fn close_peer(&self, peer_id: PeerId) -> Result<(), TunnelError> {
297#OjxU4G|        let remove_ret = self.peer_map.remove(&peer_id);
298#G6v1RW|        self.global_ctx
299#Ma6T8Y|            .issue_event(GlobalCtxEvent::PeerRemoved(peer_id));
300#3_Ehij|        tracing::info!(
301#rFb__q|            ?peer_id,
302#vOpqn5|            has_old_value = ?remove_ret.is_some(),
303#2BLqPI|            peer_ref_counter = ?remove_ret.map(|v| Arc::strong_count(&v.1)),
304#1TksaD|            "peer is closed"
305#bq_4ei|        );
306#sX41Gp|        Ok(())
307#KNhneG|    }
308#47DEQp|
309#7pE0Uo|    pub async fn add_route(&self, route: ArcRoute) {
310#Tyw88e|        let mut routes = self.routes.write().await;
311#685_dX|        routes.insert(0, route);
312#KNhneG|    }
313#47DEQp|
314#eu0JRP|    pub async fn clean_peer_without_conn(&self) {
315#FCfSl7|        let mut to_remove = vec![];
316#47DEQp|
317#qK4cHe|        for peer_id in self.list_peers().await {
318#7IUH18|            let conns = self.list_peer_conns(peer_id).await;
319#itsQwD|            if conns.is_none() || conns.as_ref().unwrap().is_empty() {
320#vf3wU-|                to_remove.push(peer_id);
321#hq_fcK|            }
322#KkQWgx|        }
323#47DEQp|
324#moJ6Bu|        for peer_id in to_remove {
325#CTQjBm|            self.close_peer(peer_id).await.unwrap();
326#KkQWgx|        }
327#KNhneG|    }
328#47DEQp|
329#ZaAC5P|    pub async fn list_routes(&self) -> DashMap<PeerId, PeerId> {
330#1c3AKw|        let route_map = DashMap::new();
331#O2RPx6|        for route in self.routes.read().await.iter() {
332#N-TnLL|            for item in route.list_routes().await.iter() {
333#BJHg0c|                route_map.insert(item.peer_id, item.next_hop_peer_id);
334#hq_fcK|            }
335#KkQWgx|        }
336#IpyYbY|        route_map
337#KNhneG|    }
338#47DEQp|
339#auhBCq|    pub async fn list_route_infos(&self) -> Vec<cli::Route> {
340#QjF4-A|        if let Some(route) = self.routes.read().await.iter().next() {
341#0NG1Eo|            return route.list_routes().await;
342#KkQWgx|        }
343#8nXhhK|        vec![]
344#KNhneG|    }
345#47DEQp|
346#yEyLTz|    pub async fn need_relay_by_foreign_network(&self, dst_peer_id: PeerId) -> Result<bool, Error> {
347#mSaKrp|        // if gateway_peer_id is not connected to me, means need relay by foreign network
348#f5SNIk|        let gateway_id = self
349#dyP6ry|            .get_gateway_peer_id(dst_peer_id, NextHopPolicy::LeastHop)
350#icqUnQ|            .await
351#vsaN4Z|            .ok_or(Error::RouteError(Some(format!(
352#smrLh1|                "peer map need_relay_by_foreign_network no gateway for dst_peer_id: {}",
353#iQcdbn|                dst_peer_id
354#UHPxHs|            ))))?;
355#47DEQp|
356#jWYGH8|        Ok(!self.has_peer(gateway_id))
357#KNhneG|    }
358#47DEQp|
359#I7po7u|    pub fn get_alive_conns(&self) -> DashMap<(PeerId, PeerConnId), PeerConnInfo> {
360#w8127h|        self.alive_conns
361#mGJIm4|            .iter()
362#BrkX1y|            .map(|v| (*v.key(), v.value().clone()))
363#b1dGJO|            .collect()
364#KNhneG|    }
365#47DEQp|
366#qHnbXo|    pub fn my_peer_id(&self) -> PeerId {
367#t_Rp3F|        self.my_peer_id
368#KNhneG|    }
369#47DEQp|
370#l4qVgh|    pub fn get_global_ctx(&self) -> ArcGlobalCtx {
371#am8HJx|        self.global_ctx.clone()
372#KNhneG|    }
373#0Qs2qn|}
374#47DEQp|
375#6nQnI2|impl Drop for PeerMap {
376#mDnVS2|    fn drop(&mut self) {
377#JQLjXO|        tracing::debug!(
378#a_mCaH|            self.my_peer_id,
379#E_7E54|            network = ?self.global_ctx.get_network_identity(),
380#WFkGmR|            "PeerMap is dropped"
381#bq_4ei|        );
382#KNhneG|    }
383#0Qs2qn|}
384#47DEQp|
--- grep({"pattern":"add_tunnel_as_server","path":"/mnt/nvme2t_2/easytier/easytier/src"}) [completed]
Found 7 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/tests.rs:
  Line 48:         b_mgr_copy.add_tunnel_as_server(b_ring, true).await.unwrap();


/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs:
  Line 469:     pub async fn add_tunnel_as_server(

  Line 1718:             b_mgr_copy.add_tunnel_as_server(b_ring, true).await.unwrap();


/mnt/nvme2t_2/easytier/easytier/src/peers/foreign_network_manager.rs:
  Line 840:             tokio::spawn(async move { b_mgr_copy.add_tunnel_as_server(b_ring, true).await });

  Line 1327:         assert!(b_mgr_copy.add_tunnel_as_server(b_ring, true).await.is_err());


/mnt/nvme2t_2/easytier/easytier/src/instance/listeners.rs:
  Line 79:         self.add_tunnel_as_server(tunnel, true).await


/mnt/nvme2t_2/easytier/easytier/src/connector/udp_hole_punch/common.rs:
  Line 409:                     if let Err(e) = peer_mgr.add_tunnel_as_server(conn, false).await {


## [assistant] +058391-05-10T20:44:51.000Z
[thinking] Now I have a much clearer picture. Let me trace the full flow:

## Connection Flow
1. `listeners.rs:79` or `udp_hole_punch/common.rs:409` → calls `peer_manager.add_tunnel_as_server(tunnel, ...)`
2. `add_tunnel_as_server` → `add_new_peer_conn`
3. `add_new_peer_conn` → checks `is_peer_blocked(peer_id)` → if blocked, returns error

## Blocking Flow (periodic & event-driven)
1. Both periodic and event-driven tasks:
   - Get whitelist entries
   - List routes (get peer IPs)
   - For each peer:
     - If NOT in whitelist: `block_peer(peer_id)` + `close_peer_conn(peer_id, &conn_id)`
     - Where `conn_id` = `get_peer_default_conn_id(peer_id)`

## Key Issue Identified

**`close_peer_conn` only closes the DEFAULT connection.** 

Looking at peer_map.rs:279-282:
```rust
pub async fn get_peer_default_conn_id(&self, peer_id: PeerId) -> Option<PeerConnId> {
    self.get_peer_by_id(peer_id)
        .map(|p| p.get_default_conn_id())
}
```

The peer might have **multiple concurrent connections**. For example, after a WebSocket (TCP) connection is established and then a UDP hole punching connection is also established (or the other way around). The non-default connection could still be alive after closing the default one.

Even more importantly, if **the peer reconnects before the periodic task** (30s), the new connection is accepted because `is_peer_blocked` returns true (the peer IS in blocked_peers)... wait, that's the opposite. Let me re-examine.

Actually, if `is_peer_blocked` returns true, `add_new_peer_conn` returns an error (line 357-361). So the question is whether a new connection from a blocked peer can be established at all.

So the blocking looks correct at the connection acceptance level. Let me think about what could go wrong...

Wait - the `add_tunnel_as_server` in `udp_hole_punch/common.rs:409` is called when a UDP hole punch connection is established. But what if the peer already has a WebSocket connection that was established before the whitelist was loaded?

Here'
Let me check when `clean_peer_without_conn` is called and how the listener accepts connections:
--- grep({"pattern":"clean_peer_without_conn","path":"/mnt/nvme2t_2/easytier/easytier/src"}) [completed]
Found 6 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/peer_map.rs:
  Line 314:     pub async fn clean_peer_without_conn(&self) {


/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs:
  Line 1235:     async fn run_clean_peer_without_conn_routine(&self) {

  Line 1239:                 peer_map.clean_peer_without_conn().await;

  Line 1265:         self.run_clean_peer_without_conn_routine().await;


/mnt/nvme2t_2/easytier/easytier/src/peers/foreign_network_manager.rs:
  Line 461:         peer_map.clean_peer_without_conn().await;


/mnt/nvme2t_2/easytier/easytier/src/peers/foreign_network_client.rs:
  Line 92:                     peer_map.clean_peer_without_conn().await;

--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/instance/listeners.rs","limit":120}) [completed]
1#JkvAPI|use std::{
2#n-mlx5|    fmt::Debug,
3#BXiknu|    net::IpAddr,
4#fyKbzj|    str::FromStr,
5#RZrM0R|    sync::{Arc, Weak},
6#3zmC_W|};
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#7mQoz2|use async_trait::async_trait;
10#GMgIVE|use tokio::task::JoinSet;
11#47DEQp|
12#DrrMzg|#[cfg(feature = "quic")]
13#OdcYfz|use crate::tunnel::quic::QUICTunnelListener;
14#xmlr0I|#[cfg(feature = "wireguard")]
15#GtSFRM|use crate::tunnel::wireguard::{WgConfig, WgTunnelListener};
16#8h8Z46|use crate::{
17#hIkMG-|    common::{
18#Qez2zE|        error::Error,
19#wM7JAa|        global_ctx::{ArcGlobalCtx, GlobalCtxEvent},
20#mz8mPX|        netns::NetNS,
21#WGNt2R|    },
22#uEmZXP|    peers::peer_manager::PeerManager,
23#5fAXpE|    tunnel::{
24#hjtBMg|        ring::RingTunnelListener, tcp::TcpTunnelListener, udp::UdpTunnelListener, Tunnel,
25#bAdNf6|        TunnelListener,
26#WGNt2R|    },
27#3zmC_W|};
28#47DEQp|
29#0m71ZA|pub fn get_listener_by_url(
30#voZz18|    l: &url::Url,
31#wbUZWF|    _ctx: ArcGlobalCtx,
32#uulT7Z|) -> Result<Box<dyn TunnelListener>, Error> {
33#nHKCcx|    Ok(match l.scheme() {
34#Ir7aKp|        "tcp" => Box::new(TcpTunnelListener::new(l.clone())),
35#LwnJp_|        "udp" => Box::new(UdpTunnelListener::new(l.clone())),
36#yrddWf|        #[cfg(feature = "wireguard")]
37#C2GUz5|        "wg" => {
38#SV2NY6|            let nid = _ctx.get_network_identity();
39#IJ54pb|            let wg_config = WgConfig::new_from_network_identity(
40#Y0azkD|                &nid.network_name,
41#GeG8tM|                &nid.network_secret.unwrap_or_default(),
42#TP8Q3D|            );
43#7Cnm-d|            Box::new(WgTunnelListener::new(l.clone(), wg_config))
44#KkQWgx|        }
45#iihhy1|        #[cfg(feature = "quic")]
46#CuWRAT|        "quic" => Box::new(QUICTunnelListener::new(l.clone())),
47#HaM8ug|        #[cfg(feature = "websocket")]
48#44dzes|        "ws" | "wss" => {
49#DyO8uy|            use crate::tunnel::websocket::WSTunnelListener;
50#NjEdaq|            Box::new(WSTunnelListener::new(l.clone()))
51#KkQWgx|        }
52#hSo-Cy|        _ => {
53#sQV_7X|            return Err(Error::InvalidUrl(l.to_string()));
54#KkQWgx|        }
55#odBdVY|    })
56#0Qs2qn|}
57#47DEQp|
58#lt2EZG|pub fn is_url_host_ipv6(l: &url::Url) -> bool {
59#NiSlpg|    l.host_str().is_some_and(|h| h.contains(':'))
60#0Qs2qn|}
61#47DEQp|
62#0bJ51W|pub fn is_url_host_unspecified(l: &url::Url) -> bool {
63#rjNNRI|    if let Ok(ip) = IpAddr::from_str(l.host_str().unwrap_or_default()) {
64#8YCKWr|        ip.is_unspecified()
65#1iXXq0|    } else {
66#L_AeCK|        false
67#KNhneG|    }
68#0Qs2qn|}
69#47DEQp|
70#mJwy3F|#[async_trait]
71#2Yapj3|pub trait TunnelHandlerForListener {
72#4BGTSm|    async fn handle_tunnel(&self, tunnel: Box<dyn Tunnel>) -> Result<(), Error>;
73#0Qs2qn|}
74#47DEQp|
75#mJwy3F|#[async_trait]
76#z0dTe5|impl TunnelHandlerForListener for PeerManager {
77#pwuqRZ|    #[tracing::instrument]
78#ybdLT7|    async fn handle_tunnel(&self, tunnel: Box<dyn Tunnel>) -> Result<(), Error> {
79#6eA1IE|        self.add_tunnel_as_server(tunnel, true).await
80#KNhneG|    }
81#0Qs2qn|}
82#47DEQp|
83#VyA0J0|pub trait ListenerCreatorTrait: Fn() -> Box<dyn TunnelListener> + Send + Sync {}
84#l0dXD_|impl<T: Send + Sync> ListenerCreatorTrait for T where T: Fn() -> Box<dyn TunnelListener> + Send {}
85#zAk_6M|pub type ListenerCreator = Box<dyn ListenerCreatorTrait>;
86#47DEQp|
87#fG4NjB|#[derive(Clone)]
88#nvruvN|struct ListenerFactory {
89#nD8TCO|    creator_fn: Arc<ListenerCreator>,
90#obAtRW|    must_succ: bool,
91#0Qs2qn|}
92#47DEQp|
93#irV5pT|pub struct ListenerManager<H> {
94#sfM3Lj|    global_ctx: ArcGlobalCtx,
95#C2GBHl|    net_ns: NetNS,
96#f0vmZu|    listeners: Vec<ListenerFactory>,
97#4MQCrc|    peer_manager: Weak<H>,
98#47DEQp|
99#FgvfeJ|    tasks: JoinSet<()>,
100#0Qs2qn|}
101#47DEQp|
102#H5OWa5|impl<H: TunnelHandlerForListener + Send + Sync + 'static + Debug> ListenerManager<H> {
103#FJr2Ma|    pub fn new(global_ctx: ArcGlobalCtx, peer_manager: Arc<H>) -> Self {
104#qMx6zU|        Self {
105#ADZfdQ|            global_ctx: global_ctx.clone(),
106#xA6uX-|            net_ns: global_ctx.net_ns.clone(),
107#zVwJ1e|            listeners: Vec::new(),
108#qUWE76|            peer_manager: Arc::downgrade(&peer_manager),
109#Voyy5y|            tasks: JoinSet::new(),
110#KkQWgx|        }
111#KNhneG|    }
112#47DEQp|
113#0V2hb0|    pub async fn prepare_listeners(&mut self) -> Result<(), Error> {
114#xYqwzu|        let self_id = self.global_ctx.get_id();
115#yubKcG|        self.add_listener(
116#9XZZqf|            move || {
117#dFF1GD|                Box::new(RingTunnelListener::new(
118#5dFEUP|                    format!("ring://{}", self_id).parse().unwrap(),
119#gHZtJ7|                ))
120#P3fk1A|            },
121#JSMKnu|            true,
122#qzjo1m|        )
123#GGG_xT|        .await?;
124#47DEQp|
125#l1S9WL|        for l in self.global_ctx.config.get_listener_uris().iter() {
126#MQDqp4|            let l = l.clone();
127#f1fYb7|            let Ok(_) = get_listener_by_url(&l, self.global_ctx.clone()) else {
128#0TF2rT|                let msg = format!("failed to get listener by url: {}, maybe not supported", l);
129#GNk9wH|                self.global_ctx
130#3bwR7q|                    .issue_event(GlobalCtxEvent::ListenerAddFailed(l.clone(), msg));
131#YqjFPd|                continue;
132#huZslw|            };
133#fPLcQM|            let ctx = self.global_ctx.clone();
134#47DEQp|
135#QeWOD_|            let listener = l.clone();
136#SOGPPO|            self.add_listener(
137#HQnKWV|                move || get_listener_by_url(&listener, ctx.clone()).unwrap(),
138#oAfmk6|                true,
139#jOhom-|            )
140#hmaiLi|            .await?;
141#47DEQp|
142#SX_5qQ|            if self.global_ctx.config.get_flags().enable_ipv6
143#P0rDSn|                && !is_url_host_ipv6(&l)
144#yFPVyH|                && is_url_host_unspecified(&l)
145#N4E_k6|                // quic enables dual-stack by default, may conflict with v4 listener
146#ZW1Wxa|                && l.scheme() != "quic"
147#eP03rc|            {
148#1B81Pd|                let mut ipv6_listener = l.clone();
149#VDhR-L|                ipv6_listener
150#X_AsUz|                    .set_host(Some("[::]".to_string().as_str()))
151#GwBalw|                    .with_context(|| format!("failed to set ipv6 host for listener: {}", l))?;
152#OppBq-|                let ctx = self.global_ctx.clone();
153#QD4C14|                self.add_listener(
154#NgUqL6|                    move || get_listener_by_url(&ipv6_listener, ctx.clone()).unwrap(),
155#E4eG5r|                    false,
156#cN3xS-|                )
157#DJQPO4|                .await?;
158#hq_fcK|            }
159#KkQWgx|        }
160#47DEQp|
161#sX41Gp|        Ok(())
162#KNhneG|    }
163#47DEQp|
164#aaFvC0|    pub async fn add_listener<C: ListenerCreatorTrait + 'static>(
165#HmXHKy|        &mut self,
166#EHypm1|        creator: C,
167#fWbw87|        must_succ: bool,
168#Ew16UP|    ) -> Result<(), Error> {
169#0aX97G|        self.listeners.push(ListenerFactory {
170#9Kw9-L|            creator_fn: Arc::new(Box::new(creator)),
171#dJKhhK|            must_succ,
172#NQ-BLD|        });
173#sX41Gp|        Ok(())
174#KNhneG|    }
175#47DEQp|
176#58E_mF|    #[tracing::instrument(skip(creator))]
177#louk6l|    async fn run_listener(
178#dlzVRC|        creator: Arc<ListenerCreator>,
179#DUVCMN|        peer_manager: Weak<H>,
180#V5_-NZ|        global_ctx: ArcGlobalCtx,
181#5PWdJw|    ) {
182#MGaxfm|        let mut err_count = 0;
183#oDRlU0|        loop {
184#6Tdof4|            let mut l = (creator)();
185#3wkIjy|            let _g = global_ctx.net_ns.guard();
186#sgtr_l|            match l.listen().await {
187#iuWsUw|                Ok(_) => {
188#WodoVn|                    err_count = 0;
189#8b5lYE|                    global_ctx.add_running_listener(l.local_url());
190#8tfqKm|                    global_ctx.issue_event(GlobalCtxEvent::ListenerAdded(l.local_url()));
191#C_CgiK|                }
192#R7DtDC|                Err(e) => {
193#vHWqVj|                    tracing::error!(?e, ?l, "listener listen error");
194#8q2NAX|                    global_ctx.issue_event(GlobalCtxEvent::ListenerAddFailed(
195#t1V2r9|                        l.local_url(),
196#Blpz-1|                        format!("error: {:?}, retry listen later...", e),
197#Xx4KDD|                    ));
198#JTuMyB|                    err_count += 1;
199#yxLOF_|                    if err_count > 5 {
200#QEZAyt|                        return;
201#uNGRgX|                    }
202#car5RF|                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
203#C_CgiK|                }
204#hq_fcK|            }
205#elI2Dp|            loop {
206#3quUK7|                let ret = match l.accept().await {
207#Zg0vtr|                    Ok(ret) => ret,
208#2UzB-B|                    Err(e) => {
209#emROz3|                        global_ctx.issue_event(GlobalCtxEvent::ListenerAcceptFailed(
210#7Haobo|                            l.local_url(),
211#YTMicG|                            format!("error: {:?}, retry listen later...", e),
212#MAyVQN|                        ));
213#5meMuZ|                        tracing::error!(?e, ?l, "listener accept error");
214#gMlMHh|                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
215#oLntjS|                        break;
216#uNGRgX|                    }
217#JBqzQb|                };
218#47DEQp|
219#XNvwO3|                let tunnel_info = ret.info().unwrap();
220#rsKGHo|                global_ctx.issue_event(GlobalCtxEvent::ConnectionAccepted(
221#w_CakG|                    tunnel_info
222#rTANte|                        .local_addr
223#0BvaKw|                        .clone()
224#1wrCE_|                        .unwrap_or_default()
225#35wbGl|                        .to_string(),
226#w_CakG|                    tunnel_info
227#jkJ0vt|                        .remote_addr
228#0BvaKw|                        .clone()
229#1wrCE_|                        .unwrap_or_default()
230#35wbGl|                        .to_string(),
231#uRtctN|                ));
232#7Q2a77|                tracing::info!(ret = ?ret, "conn accepted");
233#1iLyQL|                let peer_manager = peer_manager.clone();
234#uiu-uQ|                let global_ctx = global_ctx.clone();
235#Wx0fj1|                tokio::spawn(async move {
236#LOehrU|                    let Some(peer_manager) = peer_manager.upgrade() else {
237#Sq6_PP|                        tracing::error!("peer manager is gone, cannot handle tunnel");
238#QEZAyt|                        return;
239#cAQ1mR|                    };
240#ZtJrpG|                    let server_ret = peer_manager.handle_tunnel(ret).await;
241#GJnUeg|                    if let Err(e) = &server_ret {
242#hs3qbh|                        global_ctx.issue_event(GlobalCtxEvent::ConnectionError(
243#YNLH1Z|                            tunnel_info.local_addr.unwrap_or_default().to_string(),
244#Pfptkn|                            tunnel_info.remote_addr.unwrap_or_default().to_string(),
245#O1BFLi|                            e.to_string(),
246#MAyVQN|                        ));
247#lOofzZ|                        tracing::error!(error = ?e, "handle conn error");
248#uNGRgX|                    }
249#RDMkP3|                });
250#hq_fcK|            }
251#KkQWgx|        }
252#KNhneG|    }
253#47DEQp|
254#xZKdNb|    pub async fn run(&mut self) -> Result<(), Error> {
255#MNT2y5|        for listener in &self.listeners {
256#1Foa9e|            if listener.must_succ {
257#N4gBeu|                // try listen once
258#IjxuiW|                let mut l = (listener.creator_fn)();
259#5QKx-u|                let _g = self.net_ns.guard();
260#rufvTM|                l.listen()
261#lYNw0O|                    .await
262#eE68Hb|                    .with_context(|| format!("failed to listen on {}", l.local_url()))?;
263#hq_fcK|            }
264#47DEQp|
265#vBkZS1|            self.tasks.spawn(Self::run_listener(
266#nGPe4l|                listener.creator_fn.clone(),
267#zXvc2N|                self.peer_manager.clone(),
268#iyxrEk|                self.global_ctx.clone(),
269#1oJHlw|            ));
270#KkQWgx|        }
271#47DEQp|
272#sX41Gp|        Ok(())
273#KNhneG|    }
274#0Qs2qn|}
275#47DEQp|
276#O6JvtA|#[cfg(test)]
277#FQleaY|mod tests {
278#7JaNvg|    use std::sync::atomic::{AtomicI32, Ordering};
279#47DEQp|
280#FKiUiH|    use futures::{SinkExt, StreamExt};
281#aVc3iO|    use tokio::time::timeout;
282#47DEQp|
283#iA5wSn|    use crate::{
284#AA2hIL|        common::global_ctx::tests::get_mock_global_ctx,
285#wsX6iG|        tunnel::{packet_def::ZCPacket, ring::RingTunnelConnector, TunnelConnector, TunnelError},
286#7em-_b|    };
287#47DEQp|
288#6ffN3e|    use super::*;
289#47DEQp|
290#ZrfMg9|    #[derive(Debug)]
291#2EnXli|    struct MockListenerHandler {}
292#47DEQp|
293#jPfT8Q|    #[async_trait]
294#iAm86o|    impl TunnelHandlerForListener for MockListenerHandler {
295#If4f2q|        async fn handle_tunnel(&self, tunnel: Box<dyn Tunnel>) -> Result<(), Error> {
296#CZEob4|            let data = "abc";
297#HVIClf|            let (_recv, mut send) = tunnel.split();
298#47DEQp|
299#ha964d|            let zc_packet = ZCPacket::new_with_payload(data.as_bytes());
300#LgqMsg|            send.send(zc_packet).await.unwrap();
301#7mEL_z|            Err(Error::Unknown)
302#KkQWgx|        }
303#KNhneG|    }
304#47DEQp|
305#y2wAzC|    #[tokio::test]
306#cCuZ8q|    async fn handle_error_in_accept() {
307#ysGEl-|        let handler = Arc::new(MockListenerHandler {});
308#ev4ilD|        let mut listener_mgr = ListenerManager::new(get_mock_global_ctx(), handler.clone());
309#47DEQp|
310#-Pl1g1|        let ring_id = format!("ring://{}", uuid::Uuid::new_v4());
311#47DEQp|
312#M_IxUj|        let ring_id_clone = ring_id.clone();
313#Zg6yIW|        listener_mgr
314#2rbPal|            .add_listener(
315#GXQwrR|                move || Box::new(RingTunnelListener::new(ring_id_clone.parse().unwrap())),
316#oAfmk6|                true,
317#jOhom-|            )
318#icqUnQ|            .await
319#WEGyMY|            .unwrap();
320#4kPZ06|        listener_mgr.run().await.unwrap();
321#47DEQp|
322#3rA1kU|        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
323#47DEQp|
324#jSuEwP|        let connect_once = |ring_id| async move {
325#S7uW0G|            let tunnel = RingTunnelConnector::new(ring_id).connect().await.unwrap();
326#vCHNic|            let (mut recv, _send) = tunnel.split();
327#SQ2FV8|            assert_eq!(
328#3SFQtC|                recv.next().await.unwrap().unwrap().payload(),
329#HJnFlX|                "abc".as_bytes()
330#TP8Q3D|            );
331#vAwOvC|            tunnel
332#8OSK9L|        };
333#47DEQp|
334#rbKboZ|        timeout(std::time::Duration::from_secs(1), async move {
335#vfkyXX|            connect_once(ring_id.parse().unwrap()).await;
336#-u5pJg|            // handle tunnel fail should not impact the second connect
337#vfkyXX|            connect_once(ring_id.parse().unwrap()).await;
338#9I500R|        })
339#g6MTzE|        .await
340#Du2ckX|        .unwrap();
341#KNhneG|    }
342#47DEQp|
343#y2wAzC|    #[tokio::test]
344#LyjS4D|    async fn retry_listen() {
345#_XqrsD|        let counter = Arc::new(AtomicI32::new(0));
346#AeP5ov|        let drop_counter = Arc::new(AtomicI32::new(0));
347#2Ts6ua|        struct MockListener {
348#Xw7Klv|            counter: Arc<AtomicI32>,
349#9BBmJQ|            drop_counter: Arc<AtomicI32>,
350#KkQWgx|        }
351#47DEQp|
352#kD-9ea|        #[async_trait::async_trait]
353#7CKb0p|        impl TunnelListener for MockListener {
354#tU-jQp|            fn local_url(&self) -> url::Url {
355#6K2CFq|                "mock://".parse().unwrap()
356#hq_fcK|            }
357#47DEQp|
358#cW0bbp|            async fn listen(&mut self) -> Result<(), TunnelError> {
359#8TvcSc|                self.counter.fetch_add(1, Ordering::Relaxed);
360#ySE9H0|                Ok(())
361#hq_fcK|            }
362#47DEQp|
363#Y0qQhu|            async fn accept(&mut self) -> Result<Box<dyn Tunnel>, TunnelError> {
364#oU0lRM|                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
365#YaEo4q|                Err(TunnelError::BufferFull)
366#hq_fcK|            }
367#KkQWgx|        }
368#47DEQp|
369#1xhL9z|        impl Drop for MockListener {
370#qTblpN|            fn drop(&mut self) {
371#pSD8ne|                self.drop_counter.fetch_add(1, Ordering::Relaxed);
372#hq_fcK|            }
373#KkQWgx|        }
374#47DEQp|
375#ysGEl-|        let handler = Arc::new(MockListenerHandler {});
376#ev4ilD|        let mut listener_mgr = ListenerManager::new(get_mock_global_ctx(), handler.clone());
377#kIX2Vd|        let counter_clone = counter.clone();
378#abd34W|        let drop_counter_clone = drop_counter.clone();
379#Zg6yIW|        listener_mgr
380#2rbPal|            .add_listener(
381#1fF26W|                move || {
382#ExcWMZ|                    Box::new(MockListener {
383#WTQ7AL|                        counter: counter_clone.clone(),
384#2jqLpl|                        drop_counter: drop_counter_clone.clone(),
385#O1Rl7p|                    })
386#K3VK__|                },
387#oAfmk6|                true,
388#jOhom-|            )
389#icqUnQ|            .await
390#WEGyMY|            .unwrap();
391#4kPZ06|        listener_mgr.run().await.unwrap();
392#47DEQp|
393#6oOLLF|        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
394#47DEQp|
395#9SY3My|        assert!(counter.load(Ordering::Relaxed) >= 2);
396#VWZq0O|        assert!(drop_counter.load(Ordering::Relaxed) >= 1);
397#KNhneG|    }
398#0Qs2qn|}
399#47DEQp|
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/connector/udp_hole_punch/common.rs","offset":390,"limit":50}) [completed]
1#JkvAPI|use std::{
2#t_CKfh|    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
3#WxoiW7|    sync::Arc,
4#iMLr6L|    time::Duration,
5#3zmC_W|};
6#47DEQp|
7#yfXgtS|use crossbeam::atomic::AtomicCell;
8#a1CEkS|use dashmap::{DashMap, DashSet};
9#SiFgqJ|use rand::seq::SliceRandom as _;
10#FVvSo4|use tokio::{net::UdpSocket, sync::Mutex, task::JoinSet};
11#e7OtNQ|use tracing::{instrument, Instrument, Level};
12#L8r6Zh|use zerocopy::FromBytes as _;
13#47DEQp|
14#8h8Z46|use crate::{
15#hIkMG-|    common::{
16#voDGlj|        error::Error, global_ctx::ArcGlobalCtx, join_joinset_background, netns::NetNS,
17#oAdUUI|        stun::StunInfoCollectorTrait as _, PeerId,
18#WGNt2R|    },
19#Qlr_QH|    defer,
20#uEmZXP|    peers::peer_manager::PeerManager,
21#23O5hq|    proto::common::NatType,
22#5fAXpE|    tunnel::{
23#BQuY9U|        packet_def::{UDPTunnelHeader, UdpPacketType, UDP_TUNNEL_HEADER_SIZE},
24#sWTCzf|        udp::{new_hole_punch_packet, UdpTunnelConnector, UdpTunnelListener},
25#blqorN|        Tunnel, TunnelConnCounter, TunnelListener as _,
26#WGNt2R|    },
27#3zmC_W|};
28#47DEQp|
29#d6TTeW|pub(crate) const HOLE_PUNCH_PACKET_BODY_LEN: u16 = 16;
30#47DEQp|
31#eaPl3H|fn generate_shuffled_port_vec() -> Vec<u16> {
32#c4V8RO|    let mut rng = rand::thread_rng();
33#SrsMmG|    let mut port_vec: Vec<u16> = (1..=65535).collect();
34#ntg6rG|    port_vec.shuffle(&mut rng);
35#2K6cOT|    port_vec
36#0Qs2qn|}
37#47DEQp|
38#uRHaLX|pub(crate) enum UdpPunchClientMethod {
39#gW4I1O|    None,
40#4Etf_v|    ConeToCone,
41#pYHdmD|    SymToCone,
42#6nwgRv|    EasySymToEasySym,
43#0Qs2qn|}
44#47DEQp|
45#sJvaq7|#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
46#ojAFRv|pub(crate) enum UdpNatType {
47#y4ZN3V|    Unknown,
48#4Pn6IZ|    Open(NatType),
49#LaGEZr|    Cone(NatType),
50#H1mAnZ|    // bool means if it is incremental
51#d4DpR6|    EasySymmetric(NatType, bool),
52#iaCgKS|    HardSymmetric(NatType),
53#0Qs2qn|}
54#47DEQp|
55#RWn6Vm|impl From<NatType> for UdpNatType {
56#y31IPI|    fn from(nat_type: NatType) -> Self {
57#LNIRvw|        match nat_type {
58#y5hxVU|            NatType::Unknown => UdpNatType::Unknown,
59#_l_WPO|            NatType::OpenInternet => UdpNatType::Open(nat_type),
60#bbDn3K|            NatType::NoPat | NatType::FullCone | NatType::Restricted | NatType::PortRestricted => {
61#qPLhx7|                UdpNatType::Cone(nat_type)
62#hq_fcK|            }
63#aTMS2l|            NatType::Symmetric | NatType::SymUdpFirewall => UdpNatType::HardSymmetric(nat_type),
64#G1zkO7|            NatType::SymmetricEasyInc => UdpNatType::EasySymmetric(nat_type, true),
65#rLJXso|            NatType::SymmetricEasyDec => UdpNatType::EasySymmetric(nat_type, false),
66#KkQWgx|        }
67#KNhneG|    }
68#0Qs2qn|}
69#47DEQp|
70#ZRktpV|impl From<UdpNatType> for NatType {
71#kWbgVr|    fn from(val: UdpNatType) -> Self {
72#DJ1swk|        match val {
73#d8eTi2|            UdpNatType::Unknown => NatType::Unknown,
74#PzpaXc|            UdpNatType::Open(nat_type) => nat_type,
75#h3wMgn|            UdpNatType::Cone(nat_type) => nat_type,
76#XtuLOH|            UdpNatType::EasySymmetric(nat_type, _) => nat_type,
77#6ln7uM|            UdpNatType::HardSymmetric(nat_type) => nat_type,
78#KkQWgx|        }
79#KNhneG|    }
80#0Qs2qn|}
81#47DEQp|
82#jYGo2g|impl UdpNatType {
83#_NmXAb|    pub(crate) fn is_open(&self) -> bool {
84#j3fXFx|        matches!(self, UdpNatType::Open(_))
85#KNhneG|    }
86#47DEQp|
87#k4sqnw|    pub(crate) fn is_unknown(&self) -> bool {
88#mmQWKY|        matches!(self, UdpNatType::Unknown)
89#KNhneG|    }
90#47DEQp|
91#RBn7UO|    pub(crate) fn is_sym(&self) -> bool {
92#oPnYGK|        self.is_hard_sym() || self.is_easy_sym()
93#KNhneG|    }
94#47DEQp|
95#tAbL3X|    pub(crate) fn is_hard_sym(&self) -> bool {
96#rwp1ft|        matches!(self, UdpNatType::HardSymmetric(_))
97#KNhneG|    }
98#47DEQp|
99#9DNwK4|    pub(crate) fn is_easy_sym(&self) -> bool {
100#f_qDNR|        matches!(self, UdpNatType::EasySymmetric(_, _))
101#KNhneG|    }
102#47DEQp|
103#LncYX9|    pub(crate) fn is_cone(&self) -> bool {
104#mNO4fg|        matches!(self, UdpNatType::Cone(_))
105#KNhneG|    }
106#47DEQp|
107#9bCOYE|    pub(crate) fn get_inc_of_easy_sym(&self) -> Option<bool> {
108#CnbSep|        match self {
109#bLhmQ0|            UdpNatType::EasySymmetric(_, inc) => Some(*inc),
110#1DFhGn|            _ => None,
111#KkQWgx|        }
112#KNhneG|    }
113#47DEQp|
114#j5sV5F|    pub(crate) fn get_punch_hole_method(
115#7BRTWY|        &self,
116#owRA8j|        other: Self,
117#V5_-NZ|        global_ctx: ArcGlobalCtx,
118#tDO53f|    ) -> UdpPunchClientMethod {
119#2BW9r1|        // Check if symmetric NAT hole punching is disabled
120#7Ieyeu|        let disable_sym_hole_punching = global_ctx.get_flags().disable_sym_hole_punching;
121#47DEQp|
122#a5FCaz|        // If symmetric NAT hole punching is disabled, treat symmetric as cone
123#w9EzzC|        if disable_sym_hole_punching && self.is_sym() {
124#oNBI10|            // Convert symmetric to cone type for hole punching logic
125#jhcxQk|            if other.is_sym() {
126#-PuRxn|                return UdpPunchClientMethod::None;
127#CE4VEs|            } else {
128#dBvRst|                return UdpPunchClientMethod::ConeToCone;
129#hq_fcK|            }
130#KkQWgx|        }
131#47DEQp|
132#UoDfDn|        if other.is_unknown() {
133#wNH_Am|            if self.is_sym() {
134#0CHj97|                return UdpPunchClientMethod::SymToCone;
135#CE4VEs|            } else {
136#dBvRst|                return UdpPunchClientMethod::ConeToCone;
137#hq_fcK|            }
138#KkQWgx|        }
139#47DEQp|
140#11bOJL|        if self.is_unknown() {
141#jhcxQk|            if other.is_sym() {
142#-PuRxn|                return UdpPunchClientMethod::None;
143#CE4VEs|            } else {
144#dBvRst|                return UdpPunchClientMethod::ConeToCone;
145#hq_fcK|            }
146#KkQWgx|        }
147#47DEQp|
148#R6doId|        if self.is_open() || other.is_open() {
149#aCK30U|            // open nat does not need to punch hole
150#eJ9jd2|            return UdpPunchClientMethod::None;
151#KkQWgx|        }
152#47DEQp|
153#Enc1ab|        if self.is_cone() {
154#jhcxQk|            if other.is_sym() {
155#-PuRxn|                return UdpPunchClientMethod::None;
156#CE4VEs|            } else {
157#dBvRst|                return UdpPunchClientMethod::ConeToCone;
158#hq_fcK|            }
159#Rnbyi1|        } else if self.is_easy_sym() {
160#vUQRA7|            if other.is_hard_sym() {
161#-PuRxn|                return UdpPunchClientMethod::None;
162#48yCzR|            } else if other.is_easy_sym() {
163#FMeDtt|                return UdpPunchClientMethod::EasySymToEasySym;
164#CE4VEs|            } else {
165#0CHj97|                return UdpPunchClientMethod::SymToCone;
166#hq_fcK|            }
167#ZR7d19|        } else if self.is_hard_sym() {
168#jhcxQk|            if other.is_sym() {
169#-PuRxn|                return UdpPunchClientMethod::None;
170#CE4VEs|            } else {
171#0CHj97|                return UdpPunchClientMethod::SymToCone;
172#hq_fcK|            }
173#KkQWgx|        }
174#47DEQp|
175#P5uRwR|        unreachable!("invalid nat type");
176#KNhneG|    }
177#47DEQp|
178#FQ_IdQ|    pub(crate) fn can_punch_hole_as_client(
179#7BRTWY|        &self,
180#owRA8j|        other: Self,
181#kIT2Ch|        my_peer_id: PeerId,
182#LiSCIf|        dst_peer_id: PeerId,
183#V5_-NZ|        global_ctx: ArcGlobalCtx,
184#wPF74x|    ) -> bool {
185#Kafruw|        match self.get_punch_hole_method(other, global_ctx) {
186#z7gpSP|            UdpPunchClientMethod::None => false,
187#KwLPmA|            UdpPunchClientMethod::ConeToCone | UdpPunchClientMethod::SymToCone => true,
188#1nGqyI|            UdpPunchClientMethod::EasySymToEasySym => my_peer_id < dst_peer_id,
189#KkQWgx|        }
190#KNhneG|    }
191#0Qs2qn|}
192#47DEQp|
193#4a8kjR|#[derive(Debug)]
194#9by-02|pub(crate) struct PunchedUdpSocket {
195#0CeEYC|    pub(crate) socket: Arc<UdpSocket>,
196#KRtMia|    pub(crate) tid: u32,
197#85ImX_|    pub(crate) remote_addr: SocketAddr,
198#0Qs2qn|}
199#47DEQp|
200#EkYLiH|// used for symmetric hole punching, binding to multiple ports to increase the chance of success
201#YdL0_n|pub(crate) struct UdpSocketArray {
202#yhtSPF|    sockets: Arc<DashMap<SocketAddr, Arc<UdpSocket>>>,
203#--uow3|    max_socket_count: usize,
204#C2GBHl|    net_ns: NetNS,
205#Arwk7i|    tasks: Arc<std::sync::Mutex<JoinSet<()>>>,
206#47DEQp|
207#INupI0|    intreast_tids: Arc<DashSet<u32>>,
208#jU8NJE|    tid_to_socket: Arc<DashMap<u32, Vec<PunchedUdpSocket>>>,
209#0Qs2qn|}
210#47DEQp|
211#JSvFq_|impl UdpSocketArray {
212#J3-3w5|    pub fn new(max_socket_count: usize, net_ns: NetNS) -> Self {
213#K-7JAZ|        let tasks = Arc::new(std::sync::Mutex::new(JoinSet::new()));
214#ZTEWwO|        join_joinset_background(tasks.clone(), "UdpSocketArray".to_owned());
215#47DEQp|
216#qMx6zU|        Self {
217#8qoL6u|            sockets: Arc::new(DashMap::new()),
218#MV3_EB|            max_socket_count,
219#cmAMGr|            net_ns,
220#OB-5xI|            tasks,
221#47DEQp|
222#Xp_RSP|            intreast_tids: Arc::new(DashSet::new()),
223#x6Rpwl|            tid_to_socket: Arc::new(DashMap::new()),
224#KkQWgx|        }
225#KNhneG|    }
226#47DEQp|
227#TjACHc|    pub fn started(&self) -> bool {
228#I83eBY|        !self.sockets.is_empty()
229#KNhneG|    }
230#47DEQp|
231#Nkjdpl|    pub async fn add_new_socket(&self, socket: Arc<UdpSocket>) -> Result<(), anyhow::Error> {
232#cCepv5|        let socket_map = self.sockets.clone();
233#Io-9Pk|        let local_addr = socket.local_addr()?;
234#gxmfwE|        let intreast_tids = self.intreast_tids.clone();
235#K3vVhQ|        let tid_to_socket = self.tid_to_socket.clone();
236#A5RuiA|        socket_map.insert(local_addr, socket.clone());
237#aqIQlX|        self.tasks.lock().unwrap().spawn(
238#5taq5Y|            async move {
239#0j6wOW|                defer!(socket_map.remove(&local_addr););
240#c6bVKt|                let mut buf = [0u8; UDP_TUNNEL_HEADER_SIZE + HOLE_PUNCH_PACKET_BODY_LEN as usize];
241#ZTuhmm|                tracing::trace!(?local_addr, "udp socket added");
242#nZx_YA|                loop {
243#TUdcgj|                    let Ok((len, addr)) = socket.recv_from(&mut buf).await else {
244#oLntjS|                        break;
245#cAQ1mR|                    };
246#47DEQp|
247#DjbzvP|                    tracing::debug!(?len, ?addr, "got raw packet");
248#47DEQp|
249#aZzklB|                    if len != UDP_TUNNEL_HEADER_SIZE + HOLE_PUNCH_PACKET_BODY_LEN as usize {
250#W98xla|                        continue;
251#uNGRgX|                    }
252#47DEQp|
253#Y2VDo9|                    let Some(p) = UDPTunnelHeader::ref_from_prefix(&buf) else {
254#W98xla|                        continue;
255#cAQ1mR|                    };
256#47DEQp|
257#lXgiqL|                    let tid = p.conn_id.get();
258#vCngAX|                    let valid = p.msg_type == UdpPacketType::HolePunch as u8
259#LsVgCC|                        && p.len.get() == HOLE_PUNCH_PACKET_BODY_LEN;
260#PNt9bH|                    tracing::debug!(?p, ?addr, ?tid, ?valid, ?p, "got udp hole punch packet");
261#47DEQp|
262#Dx9uXK|                    if !valid {
263#W98xla|                        continue;
264#uNGRgX|                    }
265#47DEQp|
266#D7MOKO|                    if intreast_tids.contains(&tid) {
267#jGRC4w|                        tracing::info!(?addr, ?tid, "got hole punching packet with intreast tid");
268#cRed4r|                        tid_to_socket
269#dQxWie|                            .entry(tid)
270#4-K8Ho|                            .or_default()
271#pRz0iE|                            .push(PunchedUdpSocket {
272#8k1rF2|                                socket: socket.clone(),
273#1yAtd5|                                tid,
274#i4Ll1g|                                remote_addr: addr,
275#eMXesW|                            });
276#oLntjS|                        break;
277#uNGRgX|                    }
278#C_CgiK|                }
279#wfrheA|                tracing::debug!(?local_addr, "udp socket recv loop end");
280#hq_fcK|            }
281#SCb0ug|            .instrument(tracing::info_span!("udp array socket recv loop")),
282#bq_4ei|        );
283#sX41Gp|        Ok(())
284#KNhneG|    }
285#47DEQp|
286#_ZE7gG|    #[instrument(err)]
287#TZXkNa|    pub async fn start(&self) -> Result<(), anyhow::Error> {
288#c-6zLe|        tracing::info!("starting udp socket array");
289#47DEQp|
290#oWmcMo|        while self.sockets.len() < self.max_socket_count {
291#Sv5RHG|            let socket = {
292#5QKx-u|                let _g = self.net_ns.guard();
293#y3pjod|                Arc::new(UdpSocket::bind("0.0.0.0:0").await?)
294#huZslw|            };
295#47DEQp|
296#EaKH1n|            self.add_new_socket(socket).await?;
297#KkQWgx|        }
298#47DEQp|
299#sX41Gp|        Ok(())
300#KNhneG|    }
301#47DEQp|
302#_ZE7gG|    #[instrument(err)]
303#0vi6-I|    pub async fn send_with_all(&self, data: &[u8], addr: SocketAddr) -> Result<(), anyhow::Error> {
304#DML_cs|        tracing::info!(?addr, "sending hole punching packet");
305#47DEQp|
306#ViLVzT|        let sockets = self
307#YR99ch|            .sockets
308#mGJIm4|            .iter()
309#q_UGnT|            .map(|s| s.value().clone())
310#wiKOts|            .collect::<Vec<_>>();
311#47DEQp|
312#eZkHdD|        for socket in sockets.iter() {
313#crwV8G|            for _ in 0..3 {
314#homHt9|                socket.send_to(data, addr).await?;
315#hq_fcK|            }
316#KkQWgx|        }
317#47DEQp|
318#sX41Gp|        Ok(())
319#KNhneG|    }
320#47DEQp|
321#JDVA0v|    #[instrument(ret(level = Level::DEBUG))]
322#17FnOK|    pub fn try_fetch_punched_socket(&self, tid: u32) -> Option<PunchedUdpSocket> {
323#_sLVMd|        tracing::debug!(?tid, "try fetch punched socket");
324#qJ5lw-|        self.tid_to_socket.get_mut(&tid)?.value_mut().pop()
325#KNhneG|    }
326#47DEQp|
327#xeskgN|    pub fn add_intreast_tid(&self, tid: u32) {
328#ppWeAx|        self.intreast_tids.insert(tid);
329#KNhneG|    }
330#47DEQp|
331#N2VlgP|    pub fn remove_intreast_tid(&self, tid: u32) {
332#AL1Cx-|        self.intreast_tids.remove(&tid);
333#4-islO|        self.tid_to_socket.remove(&tid);
334#KNhneG|    }
335#0Qs2qn|}
336#47DEQp|
337#JmgOWJ|impl std::fmt::Debug for UdpSocketArray {
338#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
339#6kP3Cu|        f.debug_struct("UdpSocketArray")
340#GCQYVX|            .field("sockets", &self.sockets.len())
341#_iKMXY|            .field("max_socket_count", &self.max_socket_count)
342#THP04k|            .field("started", &self.started())
343#mCWg4U|            .field("intreast_tids", &self.intreast_tids.len())
344#Obp7I_|            .field("tid_to_socket", &self.tid_to_socket.len())
345#uPgCZz|            .finish()
346#KNhneG|    }
347#0Qs2qn|}
348#47DEQp|
349#4a8kjR|#[derive(Debug)]
350#S4I2e_|pub(crate) struct UdpHolePunchListener {
351#jL2Jja|    socket: Arc<UdpSocket>,
352#FgvfeJ|    tasks: JoinSet<()>,
353#c9wdS4|    running: Arc<AtomicCell<bool>>,
354#4az_gz|    mapped_addr: SocketAddr,
355#Oczjuw|    conn_counter: Arc<Box<dyn TunnelConnCounter>>,
356#47DEQp|
357#dvju6y|    listen_time: std::time::Instant,
358#OjH2PD|    last_select_time: AtomicCell<std::time::Instant>,
359#-VhvAR|    last_active_time: Arc<AtomicCell<std::time::Instant>>,
360#0Qs2qn|}
361#47DEQp|
362#esOzX-|impl UdpHolePunchListener {
363#T2K4Fy|    async fn get_avail_port() -> Result<u16, Error> {
364#KnJhFD|        let socket = UdpSocket::bind("0.0.0.0:0").await?;
365#ba55pu|        Ok(socket.local_addr()?.port())
366#KNhneG|    }
367#47DEQp|
368#_ZE7gG|    #[instrument(err)]
369#JV5bzj|    pub async fn new(peer_mgr: Arc<PeerManager>) -> Result<Self, Error> {
370#xx2Y7r|        Self::new_ext(peer_mgr, true, None).await
371#KNhneG|    }
372#47DEQp|
373#_ZE7gG|    #[instrument(err)]
374#tsJWvJ|    pub async fn new_ext(
375#dE-qy1|        peer_mgr: Arc<PeerManager>,
376#8ehwNP|        with_mapped_addr: bool,
377#-uMIkd|        port: Option<u16>,
378#ax67nL|    ) -> Result<Self, Error> {
379#X7F9ru|        let port = port.unwrap_or(Self::get_avail_port().await?);
380#1IWleH|        let listen_url = format!("udp://0.0.0.0:{}", port);
381#47DEQp|
382#az3wIG|        let mapped_addr = if with_mapped_addr {
383#VJF0Mv|            let gctx = peer_mgr.get_global_ctx();
384#UJVNpa|            let stun_info_collect = gctx.get_stun_info_collector();
385#pDyQW4|            stun_info_collect.get_udp_port_mapping(port).await?
386#d4ha1c|        } else {
387#N5mDaP|            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
388#8OSK9L|        };
389#47DEQp|
390#Hv-Gzx|        let mut listener = UdpTunnelListener::new(listen_url.parse().unwrap());
391#47DEQp|
392#qWZHcZ|        {
393#9Ze1nH|            let _g = peer_mgr.get_global_ctx().net_ns.guard();
394#KTlR0b|            listener.listen().await?;
395#KkQWgx|        }
396#2iyoTB|        let socket = listener.get_socket().unwrap();
397#47DEQp|
398#HPsr3i|        let running = Arc::new(AtomicCell::new(true));
399#GOB5uR|        let running_clone = running.clone();
400#47DEQp|
401#1f2fsi|        let conn_counter = listener.get_conn_counter();
402#W8HYf4|        let mut tasks = JoinSet::new();
403#47DEQp|
404#2pyWyL|        tasks.spawn(async move {
405#SwASo3|            while let Ok(conn) = listener.accept().await {
406#J_Opd5|                tracing::warn!(?conn, "udp hole punching listener got peer connection");
407#RZMD0S|                let peer_mgr = peer_mgr.clone();
408#Wx0fj1|                tokio::spawn(async move {
409#Dj7Gqy|                    if let Err(e) = peer_mgr.add_tunnel_as_server(conn, false).await {
410#gb55OK|                        tracing::error!(
411#hlQDxN|                            ?e,
412#wBMb5A|                            "failed to add tunnel as server in hole punch listener"
413#OatpD-|                        );
414#uNGRgX|                    }
415#RDMkP3|                });
416#hq_fcK|            }
417#47DEQp|
418#N64jTF|            running_clone.store(false);
419#NQ-BLD|        });
420#47DEQp|
421#CmaOfW|        let last_active_time = Arc::new(AtomicCell::new(std::time::Instant::now()));
422#S0T6iL|        let conn_counter_clone = conn_counter.clone();
423#6gFVmX|        let last_active_time_clone = last_active_time.clone();
424#2pyWyL|        tasks.spawn(async move {
425#elI2Dp|            loop {
426#T2kLuB|                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
427#q5esTf|                if conn_counter_clone.get().unwrap_or(0) != 0 {
428#4dr5kw|                    last_active_time_clone.store(std::time::Instant::now());
429#C_CgiK|                }
430#hq_fcK|            }
431#NQ-BLD|        });
432#47DEQp|
433#xlZFS5|        tracing::warn!(?mapped_addr, ?socket, "udp hole punching listener started");
434#47DEQp|
435#Nb2CQh|        Ok(Self {
436#OB-5xI|            tasks,
437#amaGmo|            socket,
438#62bTEF|            running,
439#sDaG1A|            mapped_addr,
440#yFEkJR|            conn_counter,
441#47DEQp|
442#PGqxhI|            listen_time: std::time::Instant::now(),
443#0kWUB4|            last_select_time: AtomicCell::new(std::time::Instant::now()),
444#Pwz_Nl|            last_active_time,
445#9I500R|        })
446#KNhneG|    }
447#47DEQp|
448#dZ79yx|    pub async fn get_socket(&self) -> Arc<UdpSocket> {
449#GxftgD|        self.last_select_time.store(std::time::Instant::now());
450#kIW_FO|        self.socket.clone()
451#KNhneG|    }
452#47DEQp|
453#ABkwDY|    pub async fn get_conn_count(&self) -> usize {
454#14FkpS|        self.conn_counter.get().unwrap_or(0) as usize
455#KNhneG|    }
456#0Qs2qn|}
457#47DEQp|
458#zS9qJx|pub(crate) struct PunchHoleServerCommon {
459#ulwTIt|    peer_mgr: Arc<PeerManager>,
460#47DEQp|
461#WESkGg|    listeners: Arc<Mutex<Vec<UdpHolePunchListener>>>,
462#Arwk7i|    tasks: Arc<std::sync::Mutex<JoinSet<()>>>,
463#0Qs2qn|}
464#47DEQp|
465#mGPbXT|impl PunchHoleServerCommon {
466#d_aIOu|    pub(crate) fn new(peer_mgr: Arc<PeerManager>) -> Self {
467#K-7JAZ|        let tasks = Arc::new(std::sync::Mutex::new(JoinSet::new()));
468#8LN6hT|        join_joinset_background(tasks.clone(), "PunchHoleServerCommon".to_owned());
469#47DEQp|
470#y_wX2s|        let listeners = Arc::new(Mutex::new(Vec::<UdpHolePunchListener>::new()));
471#47DEQp|
472#FtNgXB|        let l = listeners.clone();
473#t5x288|        tasks.lock().unwrap().spawn(async move {
474#elI2Dp|            loop {
475#ylb1dk|                tokio::time::sleep(Duration::from_secs(5)).await;
476#NV9RY5|                {
477#igaQb1|                    // remove listener that is not active for 40 seconds but keep listeners that are selected less than 30 seconds
478#aBgFxE|                    l.lock().await.retain(|listener| {
479#UQMkzX|                        listener.last_active_time.load().elapsed().as_secs() < 40
480#XGFnpg|                            || listener.last_select_time.load().elapsed().as_secs() < 30
481#x8RDhW|                    });
482#C_CgiK|                }
483#hq_fcK|            }
484#NQ-BLD|        });
485#47DEQp|
486#qMx6zU|        Self {
487#paGBW0|            peer_mgr,
488#47DEQp|
489#rLHNxC|            listeners,
490#OB-5xI|            tasks,
491#KkQWgx|        }
492#KNhneG|    }
493#47DEQp|
494#XCv9KA|    pub(crate) async fn add_listener(&self, listener: UdpHolePunchListener) {
495#_ubi-G|        self.listeners.lock().await.push(listener);
496#KNhneG|    }
497#47DEQp|
498#QBJF1q|    pub(crate) async fn find_listener(&self, addr: &SocketAddr) -> Option<Arc<UdpSocket>> {
499#BNkxz-|        let all_listener_sockets = self.listeners.lock().await;
500#47DEQp|
501#ezmKaP|        let listener = all_listener_sockets
502#mGJIm4|            .iter()
503#dfLXME|            .find(|listener| listener.mapped_addr == *addr && listener.running.load())?;
504#47DEQp|
505#oFA9Ks|        Some(listener.get_socket().await)
506#KNhneG|    }
507#47DEQp|
508#r6Pa0b|    pub(crate) async fn my_udp_nat_type(&self) -> i32 {
509#3dtVFc|        self.peer_mgr
510#KLHIlU|            .get_global_ctx()
511#IIYWeN|            .get_stun_info_collector()
512#mznjfv|            .get_stun_info()
513#GXrcOX|            .udp_nat_type
514#KNhneG|    }
515#47DEQp|
516#AAYlcM|    #[async_recursion::async_recursion]
517#83zK5Z|    pub(crate) async fn select_listener(
518#7BRTWY|        &self,
519#OIiM6z|        use_new_listener: bool,
520#7Bi2cA|    ) -> Option<(Arc<UdpSocket>, SocketAddr)> {
521#PNamCt|        let all_listener_sockets = &self.listeners;
522#47DEQp|
523#BQgK9L|        let mut use_last = false;
524#v5jJLv|        if all_listener_sockets.lock().await.len() < 16 || use_new_listener {
525#k2_U_i|            tracing::warn!("creating new udp hole punching listener");
526#A_YryN|            all_listener_sockets.lock().await.push(
527#aknZdY|                UdpHolePunchListener::new(self.peer_mgr.clone())
528#lYNw0O|                    .await
529#jzbb-L|                    .ok()?,
530#TP8Q3D|            );
531#lbxR7z|            use_last = true;
532#KkQWgx|        }
533#47DEQp|
534#nMGNKD|        let mut locked = all_listener_sockets.lock().await;
535#47DEQp|
536#H5--YG|        let listener = if use_last {
537#Yd1gKj|            Some(locked.last_mut()?)
538#d4ha1c|        } else {
539#8HvpjQ|            // use the listener that is active most recently
540#zLFkI3|            locked
541#VbdVXw|                .iter_mut()
542#z7M1kt|                .filter(|l| !l.mapped_addr.ip().is_unspecified())
543#uMDWPe|                .max_by_key(|listener| listener.last_active_time.load())
544#8OSK9L|        };
545#47DEQp|
546#Rb_4g3|        if listener.is_none() || listener.as_ref().unwrap().mapped_addr.ip().is_unspecified() {
547#eQBeMq|            tracing::warn!(
548#NbAfbb|                ?use_new_listener,
549#Pr2EFw|                "no available udp hole punching listener with mapped address"
550#TP8Q3D|            );
551#sw2XhV|            if !use_new_listener {
552#4wTuDo|                return self.select_listener(true).await;
553#CE4VEs|            } else {
554#Q6EONn|                return None;
555#hq_fcK|            }
556#KkQWgx|        }
557#47DEQp|
558#LvpynK|        let listener = listener.unwrap();
559#pnaQFP|        Some((listener.get_socket().await, listener.mapped_addr))
560#KNhneG|    }
561#47DEQp|
562#UnYym_|    pub(crate) fn get_joinset(&self) -> Arc<std::sync::Mutex<JoinSet<()>>> {
563#ZrjNTA|        self.tasks.clone()
564#KNhneG|    }
565#47DEQp|
566#Bs3aF-|    pub(crate) fn get_global_ctx(&self) -> ArcGlobalCtx {
567#9KgMJB|        self.peer_mgr.get_global_ctx()
568#KNhneG|    }
569#47DEQp|
570#qalw2L|    pub(crate) fn get_peer_mgr(&self) -> Arc<PeerManager> {
571#WuB0dA|        self.peer_mgr.clone()
572#KNhneG|    }
573#0Qs2qn|}
574#47DEQp|
575#QwqlEI|#[tracing::instrument(err, ret(level=Level::DEBUG), skip(ports))]
576#rt0rjR|pub(crate) async fn send_symmetric_hole_punch_packet(
577#oooqm9|    ports: &[u16],
578#zVWisB|    udp: Arc<UdpSocket>,
579#ErbSKB|    transaction_id: u32,
580#xSn7O0|    public_ips: &Vec<Ipv4Addr>,
581#QNwm2K|    port_start_idx: usize,
582#r5ex7-|    max_packets: usize,
583#Omwdj8|) -> Result<usize, Error> {
584#nAB1KD|    tracing::debug!("sending hard symmetric hole punching packet");
585#H47FOF|    let mut sent_packets = 0;
586#_vIImp|    let mut cur_port_idx = port_start_idx;
587#ylrr07|    while sent_packets < max_packets {
588#Hx-Fem|        let port = ports[cur_port_idx % ports.len()];
589#wyBwnK|        for pub_ip in public_ips {
590#fjpsWA|            let addr = SocketAddr::V4(SocketAddrV4::new(*pub_ip, port));
591#crwV8G|            for _ in 0..3 {
592#sVouCe|                let packet = new_hole_punch_packet(transaction_id, HOLE_PUNCH_PACKET_BODY_LEN);
593#d5if18|                udp.send_to(&packet.into_bytes(), addr).await?;
594#hq_fcK|            }
595#2BzaT-|            sent_packets += 1;
596#KkQWgx|        }
597#nQMO8R|        cur_port_idx = cur_port_idx.wrapping_add(1);
598#Baz6JL|        tokio::time::sleep(Duration::from_millis(1)).await;
599#KNhneG|    }
600#mOeKR9|    Ok(cur_port_idx % ports.len())
601#0Qs2qn|}
602#47DEQp|
603#JoRXZ3|async fn check_udp_socket_local_addr(
604#sfM3Lj|    global_ctx: ArcGlobalCtx,
605#AAfNq_|    remote_mapped_addr: SocketAddr,
606#4bwfND|) -> Result<(), Error> {
607#IVbzKz|    let socket = UdpSocket::bind("0.0.0.0:0").await?;
608#xz-8la|    socket.connect(remote_mapped_addr).await?;
609#0IpnTQ|    if let Ok(local_addr) = socket.local_addr() {
610#w9cbRx|        // local_addr should not be equal to virtual ipv4 or virtual ipv6
611#6JcZY-|        match local_addr.ip() {
612#s4Q_jk|            IpAddr::V4(ip) => {
613#Fna4t8|                if global_ctx.get_ipv4().map(|ip| ip.address()) == Some(ip) {
614#Pw0W1h|                    return Err(anyhow::anyhow!("local address is virtual ipv4").into());
615#C_CgiK|                }
616#hq_fcK|            }
617#H0eP3c|            IpAddr::V6(ip) => {
618#gMr0_6|                if global_ctx.get_ipv6().map(|ip| ip.address()) == Some(ip) {
619#xi0-jn|                    return Err(anyhow::anyhow!("local address is virtual ipv6").into());
620#C_CgiK|                }
621#hq_fcK|            }
622#KkQWgx|        }
623#KNhneG|    }
624#47DEQp|
625#rAUqQA|    Ok(())
626#0Qs2qn|}
627#47DEQp|
628#LOeuxa|pub(crate) async fn try_connect_with_socket(
629#sfM3Lj|    global_ctx: ArcGlobalCtx,
630#jL2Jja|    socket: Arc<UdpSocket>,
631#AAfNq_|    remote_mapped_addr: SocketAddr,
632#jiNZKS|) -> Result<Box<dyn Tunnel>, Error> {
633#MKk6Ab|    let connector = UdpTunnelConnector::new(
634#AiEH2b|        format!(
635#Kt5gx_|            "udp://{}:{}",
636#qGwhlw|            remote_mapped_addr.ip(),
637#F4dQwv|            remote_mapped_addr.port()
638#qzjo1m|        )
639#egczhJ|        .to_string()
640#IRdAtj|        .parse()
641#vQqmt1|        .unwrap(),
642#Z2lMEl|    );
643#47DEQp|
644#gaaf-n|    check_udp_socket_local_addr(global_ctx, remote_mapped_addr).await?;
645#47DEQp|
646#1-RFDn|    connector
647#grjW-v|        .try_connect_with_socket(socket, remote_mapped_addr)
648#g6MTzE|        .await
649#ivrjNV|        .map_err(Error::from)
650#0Qs2qn|}
651#47DEQp|

## [assistant] +058391-05-10T23:47:44.000Z
[thinking] Now let me understand the full picture:

## Connection Acceptance Flow

1. **WebSocket/TCP listener** (listeners.rs): Accepts a connection → `peer_manager.handle_tunnel()` → `add_tunnel_as_server(tunnel, true)` → `add_new_peer_conn` → checks `is_peer_blocked(peer_id)` → if blocked, returns error.

2. **UDP hole punch listener** (common.rs:404-416): Accepts a connection → `peer_mgr.add_tunnel_as_server(conn, false)` (note: `is_directly_connected=false`) → `add_new_peer_conn` → same check.

## The Race Condition

Now here's a crucial point. The `add_tunnel_as_server` for UDP hole punch is always `is_directly_connected=false`. This means the connection is considered "hole punched" (not directly connected). 

But there's something more important. Let me look at the **event-driven task vs periodic task**:

### Timeline of events:

1. Agent starts with `RUST_LOG=easytier=warn`
2. Periodic task and event-driven task start
3. Peer connects via WebSocket → `add_tunnel_as_server(tunnel, true)` → `add_new_peer_conn` → if not blocked yet, **accepted** (whitelist may or may not be loaded)
4. Periodic task runs (after up to 30s):
   - Loads whitelist from file
   - Lists routes
   - For peer not in whitelist: `block_peer(peer_id)` + `get_peer_default_conn_id` → closes **default connection only**

### The Critical Bug: Multiple Connections per Peer

The peer has **multiple connections** (e.g., WebSocket + UDP hole punch). When `close_peer_conn` is called with only the **default connection ID**, the **other connections remain open**. The peer can still send/receive data through the non-default connection.

Let me verify: what is the default connection? 

peer.rs:174 `close_peer_conn` - takes a specific conn_id. And `get_peer_default_conn_id` returns just one conn_id.

But more importantly, even if ALL connections are closed and the peer is in `blocked_peers`:
- When the peer tries to reconnect, `add_new_peer_conn` checks `is_peer_blocked` → returns true → rejects the connection. Go
--- grep({"pattern":"pub struct Route","path":"/mnt/nvme2t_2/easytier/easytier/src/proto"}) [completed]
No files found
--- grep({"pattern":"fn list_routes","path":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_ospf_route.rs"}) [completed]
Found 1 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/peer_ospf_route.rs:
  Line 2364:     async fn list_routes(&self) -> Vec<crate::proto::cli::Route> {


## [assistant] +058391-05-11T04:23:54.000Z
[thinking] Let me find the `cli::Route` definition.
--- grep({"pattern":"struct Route","path":"/mnt/nvme2t_2/easytier/easytier/src/proto/cli"}) [completed]
No files found
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_ospf_route.rs","offset":2360,"limit":60}) [completed]
1#JkvAPI|use std::{
2#N7ICCU|    collections::{
3#WRRGnr|        HashMap, {BTreeMap, BTreeSet},
4#WGNt2R|    },
5#n-mlx5|    fmt::Debug,
6#8mazTT|    net::{Ipv4Addr, Ipv6Addr},
7#6SaBei|    sync::{
8#0RnncT|        atomic::{AtomicBool, AtomicU32, Ordering},
9#c99N8I|        Arc, Weak,
10#WGNt2R|    },
11#W-_bJY|    time::{Duration, Instant, SystemTime},
12#3zmC_W|};
13#47DEQp|
14#yfXgtS|use crossbeam::atomic::AtomicCell;
15#pF_Tl6|use dashmap::DashMap;
16#YaAB9Q|use petgraph::{
17#ShsONE|    algo::dijkstra,
18#IdzLZB|    graph::{Graph, NodeIndex},
19#SajCAV|    visit::{EdgeRef, IntoNodeReferences},
20#Bbytbb|    Directed,
21#3zmC_W|};
22#zLRrCT|use prost::Message;
23#pBJmQW|use prost_reflect::{DynamicMessage, ReflectMessage};
24#8eJef2|use serde::{Deserialize, Serialize};
25#McK12P|use tokio::{
26#su4K1l|    select,
27#BmN-RE|    sync::Mutex,
28#kC3W3h|    task::{JoinHandle, JoinSet},
29#3zmC_W|};
30#47DEQp|
31#8h8Z46|use crate::{
32#hIkMG-|    common::{
33#CBH1EJ|        config::NetworkIdentity, constants::EASYTIER_VERSION, global_ctx::ArcGlobalCtx,
34#enzbxG|        stun::StunInfoCollectorTrait, PeerId,
35#WGNt2R|    },
36#i7d1Oh|    peers::route_trait::{Route, RouteInterfaceBox},
37#Js7gIl|    proto::{
38#DPw-Tb|        acl::GroupIdentity,
39#iZTh84|        common::{Ipv4Inet, NatType, StunInfo},
40#GHt5hR|        peer_rpc::{
41#JBU1E0|            route_foreign_network_infos, route_foreign_network_summary,
42#8VrnFX|            ForeignNetworkRouteInfoEntry, ForeignNetworkRouteInfoKey, OspfRouteRpc,
43#45J3Er|            OspfRouteRpcClientFactory, OspfRouteRpcServer, PeerIdVersion, RouteForeignNetworkInfos,
44#RdXrlU|            RouteForeignNetworkSummary, RoutePeerInfo, RoutePeerInfos, SyncRouteInfoError,
45#9O1BqE|            SyncRouteInfoRequest, SyncRouteInfoResponse,
46#Vev0I6|        },
47#tDYluf|        rpc_types::{
48#Ah3VYI|            self,
49#HgzIkz|            controller::{BaseController, Controller},
50#Vev0I6|        },
51#WGNt2R|    },
52#6Jquyp|    use_global_var,
53#3zmC_W|};
54#47DEQp|
55#Z2Y0cv|use super::{
56#tSear3|    graph_algo::dijkstra_with_first_hop,
57#s2iwev|    peer_rpc::PeerRpcManager,
58#AkXBT2|    route_trait::{
59#zzB2Jm|        DefaultRouteCostCalculator, ForeignNetworkRouteInfoMap, NextHopPolicy, RouteCostCalculator,
60#5s2nDK|        RouteCostCalculatorInterface,
61#WGNt2R|    },
62#bgU-OT|    PeerPacketFilter,
63#3zmC_W|};
64#47DEQp|
65#hyzZ_C|static SERVICE_ID: u32 = 7;
66#L4n4JD|static UPDATE_PEER_INFO_PERIOD: Duration = Duration::from_secs(3600);
67#cwN7DS|static REMOVE_DEAD_PEER_INFO_AFTER: Duration = Duration::from_secs(3660);
68#y40jYM|// the cost (latency between two peers) is i32, i32::MAX is large enough.
69#jywoBo|static AVOID_RELAY_COST: usize = i32::MAX as usize;
70#47DEQp|
71#wMS2YG|type Version = u32;
72#47DEQp|
73#P-GtIR|#[derive(Debug, Clone)]
74#FCzBdK|struct AtomicVersion(Arc<AtomicU32>);
75#47DEQp|
76#1J8CDg|impl AtomicVersion {
77#3CEZ2W|    fn new() -> Self {
78#Ascl-Q|        AtomicVersion(Arc::new(AtomicU32::new(0)))
79#KNhneG|    }
80#47DEQp|
81#0gR366|    fn get(&self) -> Version {
82#PTr_xr|        self.0.load(Ordering::Relaxed)
83#KNhneG|    }
84#47DEQp|
85#ZKTfbj|    fn set(&self, version: Version) {
86#FV_jDo|        self.0.store(version, Ordering::Relaxed);
87#KNhneG|    }
88#47DEQp|
89#yI9ikv|    fn inc(&self) -> Version {
90#0GLO09|        self.0.fetch_add(1, Ordering::Relaxed) + 1
91#KNhneG|    }
92#47DEQp|
93#5kDGst|    fn set_if_larger(&self, version: Version) -> bool {
94#ugKr6K|        // return true if the version is set.
95#Rz1KsH|        self.0.fetch_max(version, Ordering::Relaxed) < version
96#KNhneG|    }
97#0Qs2qn|}
98#47DEQp|
99#2Y6TJe|impl From<Version> for AtomicVersion {
100#yDeiYl|    fn from(version: Version) -> Self {
101#sLuqtp|        AtomicVersion(Arc::new(AtomicU32::new(version)))
102#KNhneG|    }
103#0Qs2qn|}
104#47DEQp|
105#y9d29q|fn is_foreign_network_info_newer(
106#Ef_aLQ|    next: &ForeignNetworkRouteInfoEntry,
107#0Bz96W|    prev: &ForeignNetworkRouteInfoEntry,
108#jw172r|) -> Option<bool> {
109#7EBKN6|    Some(
110#mypSLS|        SystemTime::try_from(next.last_update?).ok()?
111#bR_YKW|            > SystemTime::try_from(prev.last_update?).ok()?,
112#_xlEZ1|    )
113#0Qs2qn|}
114#47DEQp|
115#_2iiu_|impl RoutePeerInfo {
116#RzU1eb|    pub fn new() -> Self {
117#qMx6zU|        Self {
118#a1cC2j|            peer_id: 0,
119#a_AOR6|            inst_id: Some(uuid::Uuid::nil().into()),
120#VOi2up|            cost: 0,
121#DDjSM-|            ipv4_addr: None,
122#Rywkhx|            proxy_cidrs: Vec::new(),
123#_njHPm|            hostname: None,
124#KefYw3|            udp_stun_info: 0,
125#HZQYUU|            last_update: Some(SystemTime::now().into()),
126#TNoYMz|            version: 0,
127#rH6Si4|            easytier_version: EASYTIER_VERSION.to_string(),
128#zi_ikx|            feature_flag: None,
129#9v4FlB|            peer_route_id: 0,
130#3xQ3u6|            network_length: 24,
131#hqXNjg|            quic_port: None,
132#1-Pal1|            ipv6_addr: None,
133#5mPbu3|            groups: Vec::new(),
134#KkQWgx|        }
135#KNhneG|    }
136#47DEQp|
137#wf-nAU|    pub fn update_self(
138#7BRTWY|        &self,
139#kIT2Ch|        my_peer_id: PeerId,
140#_QXgjK|        peer_route_id: u64,
141#Rj5Vrp|        global_ctx: &ArcGlobalCtx,
142#P1yxtn|    ) -> Self {
143#p_Cn3N|        let mut new = Self {
144#HSQr35|            peer_id: my_peer_id,
145#UihkLo|            inst_id: Some(global_ctx.get_id().into()),
146#VOi2up|            cost: 0,
147#EFCy7I|            ipv4_addr: global_ctx.get_ipv4().map(|x| x.address().into()),
148#SHv0xq|            proxy_cidrs: global_ctx
149#t8A9zt|                .config
150#LLCbGN|                .get_proxy_cidrs()
151#KP6Two|                .iter()
152#QNUYIC|                .map(|x| x.mapped_cidr.unwrap_or(x.cidr))
153#gylNQ1|                .chain(global_ctx.get_vpn_portal_cidr())
154#Onds5C|                .map(|x| x.to_string())
155#FMB8yn|                .collect(),
156#qQ9WIk|            hostname: Some(global_ctx.get_hostname()),
157#m_B86V|            udp_stun_info: global_ctx
158#YMTkWV|                .get_stun_info_collector()
159#gVN7B1|                .get_stun_info()
160#cBdi9f|                .udp_nat_type,
161#NRvZcU|            // following fields do not participate in comparison.
162#xfYgWc|            last_update: self.last_update,
163#hZHkWU|            version: self.version,
164#47DEQp|
165#rH6Si4|            easytier_version: EASYTIER_VERSION.to_string(),
166#qcMVjT|            feature_flag: Some(global_ctx.get_feature_flags()),
167#EamJBM|            peer_route_id,
168#tJmlcn|            network_length: global_ctx
169#BVaY5f|                .get_ipv4()
170#USR1jS|                .map(|x| x.network_length() as u32)
171#tQ-wRt|                .unwrap_or(24),
172#47DEQp|
173#f8QvZm|            quic_port: global_ctx.get_quic_proxy_port().map(|x| x as u32),
174#jrEJed|            ipv6_addr: global_ctx.get_ipv6().map(|x| x.into()),
175#47DEQp|
176#_G2FTE|            groups: global_ctx.get_acl_groups(my_peer_id),
177#8OSK9L|        };
178#47DEQp|
179#5tMmOa|        let need_update_periodically = if let Ok(Ok(d)) =
180#FliEoK|            SystemTime::try_from(new.last_update.unwrap_or_default()).map(|x| x.elapsed())
181#qWZHcZ|        {
182#E-H4mV|            d > UPDATE_PEER_INFO_PERIOD
183#d4ha1c|        } else {
184#AqqhgY|            true
185#8OSK9L|        };
186#47DEQp|
187#v9ooOR|        if new != *self || need_update_periodically {
188#pd6cpD|            new.last_update = Some(SystemTime::now().into());
189#eox31T|            new.version += 1;
190#KkQWgx|        }
191#47DEQp|
192#UktWGU|        new
193#KNhneG|    }
194#0Qs2qn|}
195#47DEQp|
196#-ypo4c|impl From<RoutePeerInfo> for crate::proto::cli::Route {
197#8nIF-d|    fn from(val: RoutePeerInfo) -> Self {
198#7O7Pyw|        let network_length = if val.network_length == 0 {
199#JcZe37|            24
200#d4ha1c|        } else {
201#cyTw5D|            val.network_length
202#8OSK9L|        };
203#47DEQp|
204#aGeupJ|        crate::proto::cli::Route {
205#QDkH5b|            peer_id: val.peer_id,
206#WTIsrD|            ipv4_addr: val.ipv4_addr.map(|ipv4_addr| Ipv4Inet {
207#81NKli|                address: Some(ipv4_addr),
208#OdVv3E|                network_length,
209#_Wy84G|            }),
210#Sr1oUt|            next_hop_peer_id: 0, // next_hop_peer_id is calculated in RouteTable.
211#PI8GCS|            cost: 0,             // cost is calculated in RouteTable.
212#8aWo-W|            path_latency: 0,     // path_latency is calculated in RouteTable.
213#dmYUoz|            proxy_cidrs: val.proxy_cidrs.clone(),
214#9J9tUD|            hostname: val.hostname.unwrap_or_default(),
215#nr83lT|            stun_info: {
216#qJOa2y|                let mut stun_info = StunInfo::default();
217#E30Buk|                if let Ok(udp_nat_type) = NatType::try_from(val.udp_stun_info) {
218#5VCVd7|                    stun_info.set_udp_nat_type(udp_nat_type);
219#C_CgiK|                }
220#D9voD-|                Some(stun_info)
221#P3fk1A|            },
222#3u1OvH|            inst_id: val.inst_id.map(|x| x.to_string()).unwrap_or_default(),
223#COEaqc|            version: val.easytier_version,
224#TtAZlP|            feature_flag: val.feature_flag,
225#47DEQp|
226#OqkpY5|            next_hop_peer_id_latency_first: None,
227#D1iW3Z|            cost_latency_first: None,
228#ozQPei|            path_latency_latency_first: None,
229#47DEQp|
230#I_DJTk|            ipv6_addr: val.ipv6_addr,
231#KkQWgx|        }
232#KNhneG|    }
233#0Qs2qn|}
234#47DEQp|
235#tLgTP7|#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
236#pxkxSt|struct RouteConnBitmap {
237#nnipcK|    peer_ids: Vec<(PeerId, Version)>,
238#BQeEXY|    bitmap: Vec<u8>,
239#0Qs2qn|}
240#47DEQp|
241#lGZHsJ|impl From<RouteConnBitmap> for crate::proto::peer_rpc::RouteConnBitmap {
242#z9vpki|    fn from(val: RouteConnBitmap) -> Self {
243#vemeUk|        crate::proto::peer_rpc::RouteConnBitmap {
244#PPXYN-|            peer_ids: val
245#58ZEbr|                .peer_ids
246#nqpqUq|                .into_iter()
247#WDkoGO|                .map(|x| PeerIdVersion {
248#nnR0qF|                    peer_id: x.0,
249#dLchk0|                    version: x.1,
250#9F-IDi|                })
251#FMB8yn|                .collect(),
252#Qbt4yV|            bitmap: val.bitmap,
253#KkQWgx|        }
254#KNhneG|    }
255#0Qs2qn|}
256#47DEQp|
257#9_bow7|impl From<crate::proto::peer_rpc::RouteConnBitmap> for RouteConnBitmap {
258#hVzjh8|    fn from(v: crate::proto::peer_rpc::RouteConnBitmap) -> Self {
259#tMuXx8|        RouteConnBitmap {
260#l-PgU5|            peer_ids: v
261#58ZEbr|                .peer_ids
262#nqpqUq|                .into_iter()
263#tANRoy|                .map(|x| (x.peer_id, x.version))
264#FMB8yn|                .collect(),
265#mBfroH|            bitmap: v.bitmap,
266#KkQWgx|        }
267#KNhneG|    }
268#0Qs2qn|}
269#47DEQp|
270#iiNscs|impl RouteConnBitmap {
271#3CEZ2W|    fn new() -> Self {
272#tMuXx8|        RouteConnBitmap {
273#gDiNaK|            peer_ids: Vec::new(),
274#rZWPkC|            bitmap: Vec::new(),
275#KkQWgx|        }
276#KNhneG|    }
277#47DEQp|
278#rFUrVW|    fn get_bit(&self, idx: usize) -> bool {
279#xPkBkB|        let byte_idx = idx / 8;
280#YTirBh|        let bit_idx = idx % 8;
281#RbAbqa|        let byte = self.bitmap[byte_idx];
282#wg7PaJ|        (byte >> bit_idx) & 1 == 1
283#KNhneG|    }
284#47DEQp|
285#DP8zLV|    fn get_connected_peers(&self, peer_idx: usize) -> BTreeSet<PeerId> {
286#bz2Hc7|        let mut connected_peers = BTreeSet::new();
287#8erQJu|        for (idx, (peer_id, _)) in self.peer_ids.iter().enumerate() {
288#CyEio4|            if self.get_bit(peer_idx * self.peer_ids.len() + idx) {
289#XIx5sQ|                connected_peers.insert(*peer_id);
290#hq_fcK|            }
291#KkQWgx|        }
292#iyGohD|        connected_peers
293#KNhneG|    }
294#0Qs2qn|}
295#47DEQp|
296#QB3XnS|type Error = SyncRouteInfoError;
297#47DEQp|
298#BGNtOx|// constructed with all infos synced from all peers.
299#0uxRaX|struct SyncedRouteInfo {
300#eXddqg|    peer_infos: DashMap<PeerId, RoutePeerInfo>,
301#aqwXbg|    // prost doesn't support unknown fields, so we use DynamicMessage to store raw infos and progate them to other peers.
302#uPHhTe|    raw_peer_infos: DashMap<PeerId, DynamicMessage>,
303#Rn5DZ0|    conn_map: DashMap<PeerId, (BTreeSet<PeerId>, AtomicVersion)>,
304#-k6LSX|    foreign_network: DashMap<ForeignNetworkRouteInfoKey, ForeignNetworkRouteInfoEntry>,
305#KQ-uu2|    group_trust_map: DashMap<PeerId, HashMap<String, Vec<u8>>>,
306#O3AuY1|    group_trust_map_cache: DashMap<PeerId, Arc<Vec<String>>>, // cache for group trust map, should sync with group_trust_map
307#47DEQp|
308#4fxGAU|    version: AtomicVersion,
309#0Qs2qn|}
310#47DEQp|
311#SVY9fp|impl Debug for SyncedRouteInfo {
312#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
313#nR2-uK|        f.debug_struct("SyncedRouteInfo")
314#htgAzI|            .field("peer_infos", &self.peer_infos)
315#HkQy1U|            .field("conn_map", &self.conn_map)
316#0o7p_D|            .field("foreign_network", &self.foreign_network)
317#83EScM|            .field("group_trust_map", &self.group_trust_map)
318#O-2q3K|            .field("version", &self.version.get())
319#uPgCZz|            .finish()
320#KNhneG|    }
321#0Qs2qn|}
322#47DEQp|
323#w-wbCM|impl SyncedRouteInfo {
324#Rb6nKd|    fn get_connected_peers<T: FromIterator<PeerId>>(&self, peer_id: PeerId) -> Option<T> {
325#E-DwyO|        self.conn_map
326#ehozMf|            .get(&peer_id)
327#MShhIo|            .map(|x| x.0.clone().iter().copied().collect())
328#KNhneG|    }
329#47DEQp|
330#laFAF0|    fn remove_peer(&self, peer_id: PeerId) {
331#qb43Fs|        tracing::warn!(?peer_id, "remove_peer from synced_route_info");
332#3XfH9M|        self.peer_infos.remove(&peer_id);
333#qGSgve|        self.raw_peer_infos.remove(&peer_id);
334#mvvgjU|        self.conn_map.remove(&peer_id);
335#rsNnoE|        self.foreign_network.retain(|k, _| k.peer_id != peer_id);
336#zGe3Xm|        self.group_trust_map.remove(&peer_id);
337#oXfQJO|        self.group_trust_map_cache.remove(&peer_id);
338#1a7mJQ|        self.version.inc();
339#KNhneG|    }
340#47DEQp|
341#L3o5DJ|    fn fill_empty_peer_info(&self, peer_ids: &BTreeSet<PeerId>) {
342#DKfkXB|        let mut need_inc_version = false;
343#2BF2M3|        for peer_id in peer_ids {
344#27TQeJ|            self.peer_infos.entry(*peer_id).or_insert_with(|| {
345#tTL9WB|                need_inc_version = true;
346#R6CP7D|                RoutePeerInfo::new()
347#-JdJBA|            });
348#47DEQp|
349#A1iexY|            self.conn_map.entry(*peer_id).or_insert_with(|| {
350#tTL9WB|                need_inc_version = true;
351#rUmfi8|                (BTreeSet::new(), AtomicVersion::new())
352#-JdJBA|            });
353#KkQWgx|        }
354#b4mR_I|        if need_inc_version {
355#6mastK|            self.version.inc();
356#KkQWgx|        }
357#KNhneG|    }
358#47DEQp|
359#PJ6PA6|    fn get_peer_info_version_with_default(&self, peer_id: PeerId) -> Version {
360#Bl4Rlm|        self.peer_infos
361#ehozMf|            .get(&peer_id)
362#fs4ujp|            .map(|x| x.version)
363#34rP-R|            .unwrap_or(0)
364#KNhneG|    }
365#47DEQp|
366#vYrw7Z|    fn get_avoid_relay_data(&self, peer_id: PeerId) -> bool {
367#dhc1ap|        // if avoid relay, just set all outgoing edges to a large value: AVOID_RELAY_COST.
368#Bl4Rlm|        self.peer_infos
369#ehozMf|            .get(&peer_id)
370#hGn8mf|            .and_then(|x| x.value().feature_flag)
371#Z89Irw|            .map(|x| x.avoid_relay_data)
372#77Xzxy|            .unwrap_or_default()
373#KNhneG|    }
374#47DEQp|
375#Kvqsev|    fn check_duplicate_peer_id(
376#7BRTWY|        &self,
377#kIT2Ch|        my_peer_id: PeerId,
378#8dL5jX|        my_peer_route_id: u64,
379#LiSCIf|        dst_peer_id: PeerId,
380#si0mSl|        dst_peer_route_id: Option<u64>,
381#CF75GV|        info: &RoutePeerInfo,
382#Ew16UP|    ) -> Result<(), Error> {
383#_kFt68|        // 1. check if we are duplicated.
384#4cFj56|        if info.peer_id == my_peer_id {
385#EEepv5|            if info.peer_route_id != my_peer_route_id
386#JsohOU|                && info.version > self.get_peer_info_version_with_default(info.peer_id)
387#eP03rc|            {
388#inO0z-|                // if dst peer send to us with higher version info of my peer, our peer id is duplicated
389#JapeP0|                // TODO: handle this better. restart peer manager?
390#rO-r0-|                panic!("my peer id is duplicated");
391#tAT2rA|                // return Err(Error::DuplicatePeerId);
392#hq_fcK|            }
393#K50aSa|        } else if info.peer_id == dst_peer_id {
394#aJoFyY|            let Some(dst_peer_route_id) = dst_peer_route_id else {
395#LEBeyD|                return Ok(());
396#huZslw|            };
397#47DEQp|
398#1LCsNI|            if dst_peer_route_id != info.peer_route_id
399#Q1NU3a|                && info.version < self.get_peer_info_version_with_default(info.peer_id)
400#eP03rc|            {
401#kWusK4|                // if dst peer send to us with lower version info of dst peer, dst peer id is duplicated
402#LFx5Qi|                return Err(Error::DuplicatePeerId);
403#hq_fcK|            }
404#KkQWgx|        }
405#47DEQp|
406#sX41Gp|        Ok(())
407#KNhneG|    }
408#47DEQp|
409#3GTTNA|    fn update_peer_infos(
410#7BRTWY|        &self,
411#kIT2Ch|        my_peer_id: PeerId,
412#8dL5jX|        my_peer_route_id: u64,
413#LiSCIf|        dst_peer_id: PeerId,
414#Gg6oIF|        peer_infos: &[RoutePeerInfo],
415#DQucjp|        raw_peer_infos: &[DynamicMessage],
416#Ew16UP|    ) -> Result<(), Error> {
417#DKfkXB|        let mut need_inc_version = false;
418#VBehrj|        for (idx, route_info) in peer_infos.iter().enumerate() {
419#nq5I9K|            let mut route_info = route_info.clone();
420#CBscNO|            let raw_route_info = &raw_peer_infos[idx];
421#tDWzEZ|            self.check_duplicate_peer_id(
422#hGlxsx|                my_peer_id,
423#OFPjDO|                my_peer_route_id,
424#GeyYzE|                dst_peer_id,
425#Ved3_D|                if route_info.peer_id == dst_peer_id {
426#0I_wsG|                    self.peer_infos.get(&dst_peer_id).map(|x| x.peer_route_id)
427#ZtcDW9|                } else {
428#JqjnWy|                    None
429#K3VK__|                },
430#omjjgO|                &route_info,
431#DKRHWs|            )?;
432#47DEQp|
433#AO_-4V|            let peer_id_raw = raw_route_info
434#2tldwz|                .get_field_by_name("peer_id")
435#gQk2jP|                .unwrap()
436#Bfk0o_|                .as_u32()
437#kaMlII|                .unwrap();
438#mSbm9S|            assert_eq!(peer_id_raw, route_info.peer_id);
439#47DEQp|
440#hUstq3|            // time between peers may not be synchronized, so update last_update to local now.
441#_hnv1n|            // note only last_update with larger version will be updated to local saved peer info.
442#iCLdzg|            route_info.last_update = Some(SystemTime::now().into());
443#47DEQp|
444#vKIgkh|            self.peer_infos
445#RGeH8C|                .entry(route_info.peer_id)
446#D_ehho|                .and_modify(|old_entry| {
447#h9y1Nt|                    if route_info.version > old_entry.version {
448#FsEb8R|                        self.raw_peer_infos
449#r-AKH3|                            .insert(route_info.peer_id, raw_route_info.clone());
450#OH2fy9|                        *old_entry = route_info.clone();
451#4UAOgc|                        need_inc_version = true;
452#uNGRgX|                    }
453#9F-IDi|                })
454#yY1QDQ|                .or_insert_with(|| {
455#z21c5O|                    need_inc_version = true;
456#etrrtj|                    self.raw_peer_infos
457#Up8va8|                        .insert(route_info.peer_id, raw_route_info.clone());
458#2MdpZx|                    route_info.clone()
459#RDMkP3|                });
460#KkQWgx|        }
461#b4mR_I|        if need_inc_version {
462#6mastK|            self.version.inc();
463#KkQWgx|        }
464#sX41Gp|        Ok(())
465#KNhneG|    }
466#47DEQp|
467#1sKyfi|    fn update_conn_map(&self, conn_bitmap: &RouteConnBitmap) {
468#Kkha8x|        self.fill_empty_peer_info(&conn_bitmap.peer_ids.iter().map(|x| x.0).collect());
469#47DEQp|
470#DKfkXB|        let mut need_inc_version = false;
471#47DEQp|
472#EAizJR|        for (peer_idx, (peer_id, version)) in conn_bitmap.peer_ids.iter().enumerate() {
473#Xb_s8U|            let connceted_peers = conn_bitmap.get_connected_peers(peer_idx);
474#h4W_4h|            self.fill_empty_peer_info(&connceted_peers);
475#47DEQp|
476#S5KcJN|            self.conn_map
477#NqD1s-|                .entry(*peer_id)
478#DQANGm|                .and_modify(|(old_conn_bitmap, old_version)| {
479#fPkF1_|                    if *version > old_version.get() {
480#pAwCVv|                        *old_conn_bitmap = connceted_peers.clone();
481#4UAOgc|                        need_inc_version = true;
482#vyit5o|                        old_version.set(*version);
483#uNGRgX|                    }
484#9F-IDi|                })
485#yY1QDQ|                .or_insert_with(|| {
486#z21c5O|                    need_inc_version = true;
487#b85Mi1|                    (connceted_peers, (*version).into())
488#RDMkP3|                });
489#KkQWgx|        }
490#b4mR_I|        if need_inc_version {
491#6mastK|            self.version.inc();
492#KkQWgx|        }
493#KNhneG|    }
494#47DEQp|
495#tKPk7J|    fn update_foreign_network(&self, foreign_network: &RouteForeignNetworkInfos) {
496#rwROM7|        for item in foreign_network.infos.iter().map(Clone::clone) {
497#nYbUvS|            let Some(key) = item.key else {
498#YqjFPd|                continue;
499#huZslw|            };
500#DcaP-d|            let Some(mut entry) = item.value else {
501#YqjFPd|                continue;
502#huZslw|            };
503#47DEQp|
504#6nZxUD|            entry.last_update = Some(SystemTime::now().into());
505#47DEQp|
506#KxlxAv|            self.foreign_network
507#64gePI|                .entry(key.clone())
508#D_ehho|                .and_modify(|old_entry| {
509#_iiR-e|                    if entry.version > old_entry.version {
510#lmL5lu|                        *old_entry = entry.clone();
511#uNGRgX|                    }
512#9F-IDi|                })
513#lsJTNh|                .or_insert_with(|| entry.clone());
514#KkQWgx|        }
515#KNhneG|    }
516#47DEQp|
517#SGObZ_|    fn update_my_peer_info(
518#7BRTWY|        &self,
519#kIT2Ch|        my_peer_id: PeerId,
520#8dL5jX|        my_peer_route_id: u64,
521#Rj5Vrp|        global_ctx: &ArcGlobalCtx,
522#wPF74x|    ) -> bool {
523#aCyIGf|        let mut old = self.peer_infos.entry(my_peer_id).or_default();
524#8LSMRP|        let new = old.update_self(my_peer_id, my_peer_route_id, global_ctx);
525#6e6JDp|        let new_version = new.version;
526#AkF81l|        let old_version = old.version;
527#JjU0Rg|        *old = new;
528#47DEQp|
529#pyHxzb|        if new_version != old_version {
530#6mastK|            self.version.inc();
531#AqqhgY|            true
532#d4ha1c|        } else {
533#MLhjDD|            false
534#KkQWgx|        }
535#KNhneG|    }
536#47DEQp|
537#bm7NzA|    fn update_my_conn_info(&self, my_peer_id: PeerId, connected_peers: BTreeSet<PeerId>) -> bool {
538#tJy0SW|        self.fill_empty_peer_info(&connected_peers);
539#47DEQp|
540#9fCJof|        let mut my_conn_info = self
541#b-f-R8|            .conn_map
542#YGcp1h|            .entry(my_peer_id)
543#kg_O2m|            .or_insert((BTreeSet::new(), AtomicVersion::new()));
544#47DEQp|
545#qhVpVw|        if connected_peers == my_conn_info.value().0 {
546#MLhjDD|            false
547#d4ha1c|        } else {
548#2Ix6jh|            let _ = std::mem::replace(&mut my_conn_info.value_mut().0, connected_peers);
549#JvTX2c|            my_conn_info.value().1.inc();
550#6mastK|            self.version.inc();
551#AqqhgY|            true
552#KkQWgx|        }
553#KNhneG|    }
554#47DEQp|
555#8JH7mc|    fn update_my_foreign_network(
556#7BRTWY|        &self,
557#kIT2Ch|        my_peer_id: PeerId,
558#hmXa2c|        foreign_networks: ForeignNetworkRouteInfoMap,
559#wPF74x|    ) -> bool {
560#-elbSO|        let now = SystemTime::now();
561#pFOY0r|        let now_version = now
562#6XWOr_|            .duration_since(SystemTime::UNIX_EPOCH)
563#fdrLG-|            .unwrap()
564#txIaXl|            .as_secs() as Version;
565#uiWGpd|        let mut updated = false;
566#A4dgMB|        for mut item in self
567#IiSK0Z|            .foreign_network
568#uCW-vN|            .iter_mut()
569#hrQMJJ|            .filter(|x| x.key().peer_id == my_peer_id)
570#qWZHcZ|        {
571#5XZStb|            let (key, entry) = item.pair_mut();
572#wsbjQ5|            if let Some(mut new_entry) = foreign_networks.get_mut(key) {
573#84_QLQ|                assert!(!new_entry.foreign_peer_ids.is_empty());
574#5UT3fN|                if let Some(is_newer) = is_foreign_network_info_newer(&new_entry, entry) {
575#1WwSs6|                    let need_renew = is_newer
576#H-VDiV|                        || now
577#oz2sYq|                            .duration_since(entry.last_update.unwrap().try_into().unwrap())
578#znkbR4|                            .unwrap_or(Duration::from_secs(0))
579#Kc4ZCJ|                            > UPDATE_PEER_INFO_PERIOD;
580#z-BIIy|                    if need_renew {
581#5kTSZa|                        new_entry.version = std::cmp::max(new_entry.version + 1, now_version);
582#oHCD1b|                        *entry = new_entry.clone();
583#WdiOsW|                        updated = true;
584#uNGRgX|                    }
585#C_CgiK|                }
586#9MvROQ|                drop(new_entry);
587#nz0mGf|                foreign_networks.remove(key).unwrap();
588#6ofDQN|            } else if !item.foreign_peer_ids.is_empty() {
589#dz4tV9|                item.foreign_peer_ids.clear();
590#5KRkMY|                item.last_update = Some(SystemTime::now().into());
591#8ImU6G|                item.version = std::cmp::max(item.version + 1, now_version);
592#kHmik9|                updated = true;
593#hq_fcK|            }
594#KkQWgx|        }
595#47DEQp|
596#p_VY_K|        for item in foreign_networks.iter() {
597#ycdEgA|            assert!(!item.value().foreign_peer_ids.is_empty());
598#KxlxAv|            self.foreign_network
599#uk9_Du|                .entry(item.key().clone())
600#D_ehho|                .and_modify(|old_entry| {
601#qpALyn|                    if item.value().version > old_entry.version {
602#x_1E8j|                        *old_entry = item.value().clone();
603#uNGRgX|                    }
604#9F-IDi|                })
605#yY1QDQ|                .or_insert_with(|| {
606#2_3osr|                    let mut v = item.value().clone();
607#alHA2g|                    v.version = now_version;
608#RdxbGj|                    v
609#RDMkP3|                });
610#tm8rOe|            updated = true;
611#KkQWgx|        }
612#47DEQp|
613#bftboN|        if updated {
614#6mastK|            self.version.inc();
615#KkQWgx|        }
616#47DEQp|
617#4ebUi8|        updated
618#KNhneG|    }
619#47DEQp|
620#fcfH4H|    fn is_peer_bidirectly_connected(&self, src_peer_id: PeerId, dst_peer_id: PeerId) -> bool {
621#E-DwyO|        self.conn_map
622#yqDWY8|            .get(&src_peer_id)
623#eZvxdA|            .map(|x| x.0.contains(&dst_peer_id))
624#twEnen|            .unwrap_or(false)
625#KNhneG|    }
626#47DEQp|
627#9wOiFj|    fn is_peer_directly_connected(&self, src_peer_id: PeerId, dst_peer_id: PeerId) -> bool {
628#z4zsrC|        self.is_peer_bidirectly_connected(src_peer_id, dst_peer_id)
629#lcUqiw|            || self.is_peer_bidirectly_connected(dst_peer_id, src_peer_id)
630#KNhneG|    }
631#47DEQp|
632#B7_f2_|    fn verify_and_update_group_trusts(
633#7BRTWY|        &self,
634#Gg6oIF|        peer_infos: &[RoutePeerInfo],
635#Uk6UlF|        local_group_declarations: &[GroupIdentity],
636#5PWdJw|    ) {
637#_P-2Wt|        let local_group_declarations = local_group_declarations
638#mGJIm4|            .iter()
639#mjU_Io|            .map(|g| (g.group_name.as_str(), g.group_secret.as_str()))
640#yXpNYm|            .collect::<std::collections::HashMap<&str, &str>>();
641#47DEQp|
642#MCFS8v|        let verify_groups = |old_trusted_groups: Option<&HashMap<String, Vec<u8>>>,
643#UfKwcH|                             info: &RoutePeerInfo|
644#Cz8o-2|         -> HashMap<String, Vec<u8>> {
645#e_HFgK|            let mut trusted_groups_for_peer: HashMap<String, Vec<u8>> = HashMap::new();
646#47DEQp|
647#LukH3S|            for group_proof in &info.groups {
648#VIcvbR|                let name = &group_proof.group_name;
649#4vdkcB|                let proof_bytes = group_proof.group_proof.clone();
650#47DEQp|
651#nPe5Ja|                // If we already trusted this group and the proof hasn't changed, reuse it.
652#g0NPnq|                if old_trusted_groups
653#LB3Zyr|                    .and_then(|g| g.get(name))
654#j2sEED|                    .map(|old| old == &proof_bytes)
655#D1zkNj|                    .unwrap_or(false)
656#NV9RY5|                {
657#VNkVib|                    trusted_groups_for_peer.insert(name.clone(), proof_bytes);
658#_01Y-5|                    continue;
659#C_CgiK|                }
660#47DEQp|
661#-cBHY8|                if let Some(&local_secret) =
662#BOASX_|                    local_group_declarations.get(group_proof.group_name.as_str())
663#NV9RY5|                {
664#NwWifC|                    if group_proof.verify(local_secret, info.peer_id) {
665#7Dpt8L|                        trusted_groups_for_peer.insert(name.clone(), proof_bytes);
666#HyGTCY|                    } else {
667#4MW_re|                        tracing::warn!(
668#5I-7ga|                            peer_id = info.peer_id,
669#YRAEMR|                            group = %group_proof.group_name,
670#RMnEQf|                            "Group proof verification failed"
671#OatpD-|                        );
672#uNGRgX|                    }
673#C_CgiK|                }
674#hq_fcK|            }
675#47DEQp|
676#-Px8mn|            trusted_groups_for_peer
677#8OSK9L|        };
678#47DEQp|
679#ZXgDVr|        for info in peer_infos {
680#aRZnVE|            match self.group_trust_map.entry(info.peer_id) {
681#aETxR0|                dashmap::mapref::entry::Entry::Occupied(mut entry) => {
682#qAzz8a|                    let old_trusted_groups = entry.get().clone();
683#jk6QEx|                    let trusted_groups_for_peer = verify_groups(Some(&old_trusted_groups), info);
684#47DEQp|
685#r_HQnQ|                    if trusted_groups_for_peer.is_empty() {
686#J4bD2p|                        entry.remove();
687#hlmdt9|                        self.group_trust_map_cache.remove(&info.peer_id);
688#HyGTCY|                    } else {
689#UQ6KpQ|                        self.group_trust_map_cache.insert(
690#Dbi9TS|                            info.peer_id,
691#8Rx1Gw|                            Arc::new(trusted_groups_for_peer.keys().cloned().collect()),
692#OatpD-|                        );
693#qcfNGO|                        *entry.get_mut() = trusted_groups_for_peer;
694#uNGRgX|                    }
695#C_CgiK|                }
696#FoywQr|                dashmap::mapref::entry::Entry::Vacant(entry) => {
697#GKmI-1|                    let trusted_groups_for_peer = verify_groups(None, info);
698#47DEQp|
699#l6P2Fe|                    if !trusted_groups_for_peer.is_empty() {
700#UQ6KpQ|                        self.group_trust_map_cache.insert(
701#Dbi9TS|                            info.peer_id,
702#8Rx1Gw|                            Arc::new(trusted_groups_for_peer.keys().cloned().collect()),
703#OatpD-|                        );
704#WbO682|                        entry.insert(trusted_groups_for_peer);
705#uNGRgX|                    }
706#C_CgiK|                }
707#hq_fcK|            }
708#KkQWgx|        }
709#KNhneG|    }
710#0Qs2qn|}
711#47DEQp|
712#UKblUJ|type PeerGraph = Graph<PeerId, usize, Directed>;
713#4MF2ej|type PeerIdToNodexIdxMap = DashMap<PeerId, NodeIndex>;
714#A-uuFf|#[derive(Debug, Clone, Copy)]
715#MvV72p|struct NextHopInfo {
716#KzyAT5|    next_hop_peer_id: PeerId,
717#pGvE1K|    path_latency: i32,
718#eRsSiP|    path_len: usize, // path includes src and dst.
719#9rYsyc|    version: Version,
720#0Qs2qn|}
721#Onc87_|// dst_peer_id -> (next_hop_peer_id, cost, path_len)
722#zXSSUL|type NextHopMap = DashMap<PeerId, NextHopInfo>;
723#A-uuFf|#[derive(Debug, Clone, Copy)]
724#KlrkO1|struct PeerIdAndVersion {
725#OfEt-I|    peer_id: PeerId,
726#9rYsyc|    version: Version,
727#0Qs2qn|}
728#47DEQp|
729#cnm757|// computed with SyncedRouteInfo. used to get next hop.
730#4a8kjR|#[derive(Debug)]
731#6PIRnM|struct RouteTable {
732#eXddqg|    peer_infos: DashMap<PeerId, RoutePeerInfo>,
733#DS-a2D|    next_hop_map: NextHopMap,
734#aZISt7|    ipv4_peer_id_map: DashMap<Ipv4Addr, PeerIdAndVersion>,
735#AlPCrF|    ipv6_peer_id_map: DashMap<Ipv6Addr, PeerIdAndVersion>,
736#wb_M5c|    cidr_peer_id_map: DashMap<cidr::IpCidr, PeerIdAndVersion>,
737#uqFRpc|    next_hop_map_version: AtomicVersion,
738#0Qs2qn|}
739#47DEQp|
740#f3y_py|impl RouteTable {
741#3CEZ2W|    fn new() -> Self {
742#ErAYJT|        RouteTable {
743#3QcW6G|            peer_infos: DashMap::new(),
744#bML_gS|            next_hop_map: DashMap::new(),
745#NW6Ar8|            ipv4_peer_id_map: DashMap::new(),
746#QU7nhO|            ipv6_peer_id_map: DashMap::new(),
747#pfUj3a|            cidr_peer_id_map: DashMap::new(),
748#mYIhh8|            next_hop_map_version: AtomicVersion::new(),
749#KkQWgx|        }
750#KNhneG|    }
751#47DEQp|
752#v0_xZR|    fn get_next_hop(&self, dst_peer_id: PeerId) -> Option<NextHopInfo> {
753#1Eqny0|        let cur_version = self.next_hop_map_version.get();
754#gKW82b|        self.next_hop_map.get(&dst_peer_id).and_then(|x| {
755#01UpTv|            if x.version >= cur_version {
756#8TNqTV|                Some(*x)
757#CE4VEs|            } else {
758#DauEWu|                None
759#hq_fcK|            }
760#9I500R|        })
761#KNhneG|    }
762#47DEQp|
763#CqpwAy|    fn peer_reachable(&self, peer_id: PeerId) -> bool {
764#y1EV2s|        self.get_next_hop(peer_id).is_some()
765#KNhneG|    }
766#47DEQp|
767#-lK-Fr|    fn get_nat_type(&self, peer_id: PeerId) -> Option<NatType> {
768#Bl4Rlm|        self.peer_infos
769#ehozMf|            .get(&peer_id)
770#V2t6H4|            .map(|x| NatType::try_from(x.udp_stun_info).unwrap_or_default())
771#KNhneG|    }
772#47DEQp|
773#K17Ndz|    // return graph and start node index (node of my peer id).
774#2U--Nr|    fn build_peer_graph_from_synced_info<T: RouteCostCalculatorInterface>(
775#kIT2Ch|        my_peer_id: PeerId,
776#ZKW5A2|        synced_info: &SyncedRouteInfo,
777#HrCv2i|        cost_calc: &T,
778#0IEJTn|    ) -> (PeerGraph, NodeIndex) {
779#FTpW3Y|        let mut graph: PeerGraph = PeerGraph::new();
780#47DEQp|
781#nREaST|        let mut start_node_idx = None;
782#WsgMzT|        let peer_id_to_node_index: PeerIdToNodexIdxMap = DashMap::new();
783#8dBeiU|        for item in synced_info.peer_infos.iter() {
784#9BGsKy|            let peer_id = item.key();
785#Hlzqzj|            let info = item.value();
786#47DEQp|
787#wGbJQx|            if info.version == 0 {
788#YqjFPd|                continue;
789#hq_fcK|            }
790#47DEQp|
791#MVkhYZ|            let node_idx = graph.add_node(*peer_id);
792#47DEQp|
793#iMDykR|            peer_id_to_node_index.insert(*peer_id, node_idx);
794#_x-INT|            if *peer_id == my_peer_id {
795#Q9oDD1|                start_node_idx = Some(node_idx);
796#hq_fcK|            }
797#KkQWgx|        }
798#47DEQp|
799#8gJ0zQ|        if start_node_idx.is_none() {
800#JRauSD|            return (graph, NodeIndex::end());
801#KkQWgx|        }
802#47DEQp|
803#73fH-2|        for item in peer_id_to_node_index.iter() {
804#T4Ap1U|            let src_peer_id = item.key();
805#Z4FM0h|            let src_node_idx = item.value();
806#10jDGz|            let connected_peers: BTreeSet<_> = synced_info
807#Uv0Qim|                .get_connected_peers(*src_peer_id)
808#adm-2S|                .unwrap_or_default();
809#47DEQp|
810#j0QugZ|            // if avoid relay, just set all outgoing edges to a large value: AVOID_RELAY_COST.
811#0p-xrr|            let peer_avoid_relay_data = synced_info.get_avoid_relay_data(*src_peer_id);
812#47DEQp|
813#90jVbL|            for dst_peer_id in connected_peers.iter() {
814#PCRJgD|                let Some(dst_node_idx) = peer_id_to_node_index.get(dst_peer_id) else {
815#_01Y-5|                    continue;
816#JBqzQb|                };
817#47DEQp|
818#oVR1Ap|                let mut cost = cost_calc.calculate_cost(*src_peer_id, *dst_peer_id) as usize;
819#UvCYvv|                if peer_avoid_relay_data {
820#pRZ3cb|                    cost += AVOID_RELAY_COST;
821#C_CgiK|                }
822#47DEQp|
823#FsCp-K|                graph.add_edge(*src_node_idx, *dst_node_idx, cost);
824#hq_fcK|            }
825#KkQWgx|        }
826#47DEQp|
827#D1t2k3|        (graph, start_node_idx.unwrap())
828#KNhneG|    }
829#47DEQp|
830#6qXLmr|    fn clean_expired_route_info(&self) {
831#1Eqny0|        let cur_version = self.next_hop_map_version.get();
832#Onj6Qf|        self.next_hop_map.retain(|_, v| {
833#Wyvrwa|            // remove next hop map for peers we cannot reach.
834#rqJW2T|            v.version >= cur_version
835#NQ-BLD|        });
836#M3-bqp|        self.peer_infos.retain(|k, _| {
837#t-5UC-|            // remove peer info for peers we cannot reach.
838#44CCoS|            self.next_hop_map.contains_key(k)
839#NQ-BLD|        });
840#24Gb8w|        self.ipv4_peer_id_map.retain(|_, v| {
841#QyepMc|            // remove ipv4 map for peers we cannot reach.
842#bfMfWH|            self.next_hop_map.contains_key(&v.peer_id)
843#NQ-BLD|        });
844#FA64EG|        self.ipv6_peer_id_map.retain(|_, v| {
845#Mxa7RP|            // remove ipv6 map for peers we cannot reach.
846#bfMfWH|            self.next_hop_map.contains_key(&v.peer_id)
847#NQ-BLD|        });
848#MCFJzw|        self.cidr_peer_id_map.retain(|_, v| {
849#zhUXdW|            // remove cidr map for peers we cannot reach.
850#bfMfWH|            self.next_hop_map.contains_key(&v.peer_id)
851#NQ-BLD|        });
852#KNhneG|    }
853#47DEQp|
854#W0usb4|    fn gen_next_hop_map_with_least_hop(
855#7BRTWY|        &self,
856#Kb8LHL|        graph: &PeerGraph,
857#JBvkzL|        start_node: &NodeIndex,
858#Gi2I3B|        version: Version,
859#5PWdJw|    ) {
860#3tMJqS|        let normalize_edge_cost = |e: petgraph::graph::EdgeReference<usize>| {
861#DVk2Vf|            if *e.weight() >= AVOID_RELAY_COST {
862#Ru00b3|                AVOID_RELAY_COST + 1
863#CE4VEs|            } else {
864#2rmWkO|                1
865#hq_fcK|            }
866#8OSK9L|        };
867#3UQRem|        // Step 1: 第一次 Dijkstra - 计算最短跳数
868#wJfjQc|        let path_len_map = dijkstra(&graph, *start_node, None, normalize_edge_cost);
869#47DEQp|
870#QSkYIx|        // Step 2: 构建最短跳数子图（只保留属于最短路径和 AVOID RELAY 的边）
871#FakpiJ|        let mut subgraph: PeerGraph = PeerGraph::new();
872#nREaST|        let mut start_node_idx = None;
873#mbbkj9|        for (node_idx, peer_id) in graph.node_references() {
874#ziNI2-|            let new_node_idx = subgraph.add_node(*peer_id);
875#BPZkq0|            if node_idx == *start_node {
876#oEbA8T|                start_node_idx = Some(new_node_idx);
877#hq_fcK|            }
878#KkQWgx|        }
879#47DEQp|
880#v1gyk_|        for edge in graph.edge_references() {
881#dD9rcW|            let (src, tgt) = graph.edge_endpoints(edge.id()).unwrap();
882#OrwlD8|            let Some(src_path_len) = path_len_map.get(&src) else {
883#YqjFPd|                continue;
884#huZslw|            };
885#Oe08lu|            let Some(tgt_path_len) = path_len_map.get(&tgt) else {
886#YqjFPd|                continue;
887#huZslw|            };
888#C9TBtv|            if *src_path_len + normalize_edge_cost(edge) == *tgt_path_len {
889#st8heK|                subgraph.add_edge(src, tgt, *edge.weight());
890#hq_fcK|            }
891#KkQWgx|        }
892#47DEQp|
893#UmfCjQ|        // Step 3: 第二次 Dijkstra - 在子图上找代价最小的路径
894#lpJR_8|        self.gen_next_hop_map_with_least_cost(&subgraph, &start_node_idx.unwrap(), version);
895#KNhneG|    }
896#47DEQp|
897#Geu1Ji|    fn gen_next_hop_map_with_least_cost(
898#7BRTWY|        &self,
899#Kb8LHL|        graph: &PeerGraph,
900#JBvkzL|        start_node: &NodeIndex,
901#Gi2I3B|        version: Version,
902#5PWdJw|    ) {
903#88nJ0O|        let (costs, next_hops) = dijkstra_with_first_hop(&graph, *start_node, |e| *e.weight());
904#47DEQp|
905#8kzf1r|        for (dst, (next_hop, path_len)) in next_hops.iter() {
906#9gEFkj|            let info = NextHopInfo {
907#uRhjrO|                next_hop_peer_id: *graph.node_weight(*next_hop).unwrap(),
908#jTTO3b|                path_latency: (*costs.get(dst).unwrap() % AVOID_RELAY_COST) as i32,
909#ogAKIR|                path_len: { *path_len },
910#alEFbl|                version,
911#huZslw|            };
912#UA85kj|            let dst_peer_id = *graph.node_weight(*dst).unwrap();
913#Nt0x1G|            self.next_hop_map
914#8EDs-s|                .entry(dst_peer_id)
915#GzF3v0|                .and_modify(|x| {
916#YcIjyK|                    if x.version < version {
917#QafkCZ|                        *x = info;
918#uNGRgX|                    }
919#9F-IDi|                })
920#nmdBTU|                .or_insert(info);
921#KkQWgx|        }
922#47DEQp|
923#VUVvH3|        self.next_hop_map_version.set_if_larger(version);
924#KNhneG|    }
925#47DEQp|
926#yoYRiK|    fn build_from_synced_info<T: RouteCostCalculatorInterface>(
927#7BRTWY|        &self,
928#kIT2Ch|        my_peer_id: PeerId,
929#ZKW5A2|        synced_info: &SyncedRouteInfo,
930#F4Rim3|        policy: NextHopPolicy,
931#HrCv2i|        cost_calc: &T,
932#5PWdJw|    ) {
933#FIAS4g|        let version = synced_info.version.get();
934#47DEQp|
935#HK7-V5|        // build next hop map
936#jGWIMl|        let (graph, start_node) =
937#vMj7L0|            Self::build_peer_graph_from_synced_info(my_peer_id, synced_info, cost_calc);
938#47DEQp|
939#9PEDw4|        if graph.node_count() == 0 {
940#n9KoTd|            tracing::warn!("no peer in graph, cannot build next hop map");
941#NhdEjk|            return;
942#KkQWgx|        }
943#47DEQp|
944#6j1rPl|        if matches!(policy, NextHopPolicy::LeastHop) {
945#Sr_Iua|            self.gen_next_hop_map_with_least_hop(&graph, &start_node, version);
946#d4ha1c|        } else {
947#ra8FfP|            self.gen_next_hop_map_with_least_cost(&graph, &start_node, version);
948#8OSK9L|        };
949#47DEQp|
950#sZ9up8|        // build peer_infos, ipv4_peer_id_map, cidr_peer_id_map
951#oJXRSS|        // only set map for peers we can reach.
952#iV7haV|        for item in self.next_hop_map.iter() {
953#WuKzRZ|            if item.version < version {
954#XNlC7y|                // skip if the next hop entry is outdated. (peer is unreachable)
955#YqjFPd|                continue;
956#hq_fcK|            }
957#47DEQp|
958#9BGsKy|            let peer_id = item.key();
959#ZE4Jj3|            let Some(info) = synced_info.peer_infos.get(peer_id) else {
960#YqjFPd|                continue;
961#huZslw|            };
962#47DEQp|
963#u4_gA3|            self.peer_infos.insert(*peer_id, info.clone());
964#47DEQp|
965#AnpfkF|            let peer_id_and_version = PeerIdAndVersion {
966#mOXQQU|                peer_id: *peer_id,
967#alEFbl|                version,
968#huZslw|            };
969#47DEQp|
970#z8Q1B1|            let is_new_peer_better = |old_peer: &PeerIdAndVersion| -> bool {
971#Qf-DyC|                if peer_id_and_version.version > old_peer.version {
972#jGA1In|                    return true;
973#C_CgiK|                }
974#KY_SKH|                if peer_id_and_version.peer_id == old_peer.peer_id {
975#ZKW5rA|                    return false;
976#C_CgiK|                }
977#yPpLeC|                let old_next_hop = self.get_next_hop(old_peer.peer_id);
978#aRPThd|                let new_next_hop = item.value();
979#TtcIXY|                old_next_hop.is_none() || new_next_hop.path_len < old_next_hop.unwrap().path_len
980#huZslw|            };
981#47DEQp|
982#1jt8C0|            if let Some(ipv4_addr) = info.ipv4_addr {
983#wFoEsq|                self.ipv4_peer_id_map
984#nMix7P|                    .entry(ipv4_addr.into())
985#8IFVrs|                    .and_modify(|v| {
986#dkcRIP|                        if is_new_peer_better(v) {
987#OFW87r|                            *v = peer_id_and_version;
988#qrgf36|                        }
989#O1Rl7p|                    })
990#a2UBLI|                    .or_insert(peer_id_and_version);
991#hq_fcK|            }
992#47DEQp|
993#Tormu7|            if let Some(ipv6_addr) = info.ipv6_addr.and_then(|x| x.address) {
994#W-pzuv|                self.ipv6_peer_id_map
995#BubTAo|                    .entry(ipv6_addr.into())
996#8IFVrs|                    .and_modify(|v| {
997#dkcRIP|                        if is_new_peer_better(v) {
998#OFW87r|                            *v = peer_id_and_version;
999#qrgf36|                        }
1000#O1Rl7p|                    })
1001#a2UBLI|                    .or_insert(peer_id_and_version);
1002#hq_fcK|            }
1003#47DEQp|
1004#dNBRUm|            for cidr in info.proxy_cidrs.iter() {
1005#uZLlOe|                self.cidr_peer_id_map
1006#4QNC_t|                    .entry(cidr.parse().unwrap())
1007#8IFVrs|                    .and_modify(|v| {
1008#dkcRIP|                        if is_new_peer_better(v) {
1009#CDRdd7|                            // if the next hop is not set or the new next hop is better, update it.
1010#OFW87r|                            *v = peer_id_and_version;
1011#qrgf36|                        }
1012#O1Rl7p|                    })
1013#a2UBLI|                    .or_insert(peer_id_and_version);
1014#hq_fcK|            }
1015#KkQWgx|        }
1016#KNhneG|    }
1017#47DEQp|
1018#OcrH4e|    fn get_peer_id_for_proxy(&self, ipv4: &Ipv4Addr) -> Option<PeerId> {
1019#sMUCJp|        let ipv4 = std::net::IpAddr::V4(*ipv4);
1020#x67ur7|        for item in self.cidr_peer_id_map.iter() {
1021#v3fiH4|            let (k, v) = item.pair();
1022#v0oUm-|            if k.contains(&ipv4) {
1023#638CoQ|                return Some(v.peer_id);
1024#hq_fcK|            }
1025#KkQWgx|        }
1026#jan1Ge|        None
1027#KNhneG|    }
1028#0Qs2qn|}
1029#47DEQp|
1030#yax8_Q|type SessionId = u64;
1031#47DEQp|
1032#sx6p3c|type AtomicSessionId = atomic_shim::AtomicU64;
1033#47DEQp|
1034#MWH1xw|struct SessionTask {
1035#ug-G5V|    my_peer_id: PeerId,
1036#5khUz5|    task: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
1037#0Qs2qn|}
1038#47DEQp|
1039#l_2H1J|impl SessionTask {
1040#Qxkxir|    fn new(my_peer_id: PeerId) -> Self {
1041#ISK0N5|        SessionTask {
1042#907ngS|            my_peer_id,
1043#EpnHwO|            task: Arc::new(std::sync::Mutex::new(None)),
1044#KkQWgx|        }
1045#KNhneG|    }
1046#47DEQp|
1047#ueyQ6-|    fn set_task(&self, task: JoinHandle<()>) {
1048#IyFRQY|        if let Some(old) = self.task.lock().unwrap().replace(task) {
1049#56lFE2|            old.abort();
1050#KkQWgx|        }
1051#KNhneG|    }
1052#47DEQp|
1053#-LKLCo|    fn is_running(&self) -> bool {
1054#j7LeFh|        if let Some(task) = self.task.lock().unwrap().as_ref() {
1055#74or6e|            !task.is_finished()
1056#d4ha1c|        } else {
1057#MLhjDD|            false
1058#KkQWgx|        }
1059#KNhneG|    }
1060#0Qs2qn|}
1061#47DEQp|
1062#xYlpZi|impl Drop for SessionTask {
1063#mDnVS2|    fn drop(&mut self) {
1064#_Oy5SH|        if let Some(task) = self.task.lock().unwrap().take() {
1065#fnHkO_|            task.abort();
1066#KkQWgx|        }
1067#w4kGBI|        tracing::debug!(my_peer_id = self.my_peer_id, "drop SessionTask");
1068#KNhneG|    }
1069#0Qs2qn|}
1070#47DEQp|
1071#1bT2a8|impl Debug for SessionTask {
1072#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
1073#Am3Ysl|        f.debug_struct("SessionTask")
1074#9lQx57|            .field("is_running", &self.is_running())
1075#uPgCZz|            .finish()
1076#KNhneG|    }
1077#0Qs2qn|}
1078#47DEQp|
1079#wDfRhl|// if we need to sync route info with one peer, we create a SyncRouteSession with that peer.
1080#4a8kjR|#[derive(Debug)]
1081#0_dUiy|struct SyncRouteSession {
1082#ug-G5V|    my_peer_id: PeerId,
1083#y4jKvi|    dst_peer_id: PeerId,
1084#mJQbhP|    dst_saved_peer_info_versions: DashMap<PeerId, AtomicVersion>,
1085#JnA86U|    dst_saved_conn_bitmap_version: DashMap<PeerId, AtomicVersion>,
1086#sBurT8|    dst_saved_foreign_network_versions: DashMap<ForeignNetworkRouteInfoKey, AtomicVersion>,
1087#47DEQp|
1088#1Wq1Oc|    my_session_id: AtomicSessionId,
1089#_nwlKK|    dst_session_id: AtomicSessionId,
1090#47DEQp|
1091#1M6Sg4|    // every node should have exactly one initator session to one other non-initiator peer.
1092#9dwGvl|    we_are_initiator: AtomicBool,
1093#-3qqeF|    dst_is_initiator: AtomicBool,
1094#47DEQp|
1095#FiVn6t|    need_sync_initiator_info: AtomicBool,
1096#47DEQp|
1097#X_03ai|    rpc_tx_count: AtomicU32,
1098#_I2JCo|    rpc_rx_count: AtomicU32,
1099#47DEQp|
1100#ZvfM9l|    task: SessionTask,
1101#0Qs2qn|}
1102#47DEQp|
1103#Hlpr4G|impl SyncRouteSession {
1104#igRZlz|    fn new(my_peer_id: PeerId, dst_peer_id: PeerId) -> Self {
1105#KWPlFC|        SyncRouteSession {
1106#907ngS|            my_peer_id,
1107#EL8syC|            dst_peer_id,
1108#jcf4s3|            dst_saved_peer_info_versions: DashMap::new(),
1109#TKZCj_|            dst_saved_conn_bitmap_version: DashMap::new(),
1110#iVKm83|            dst_saved_foreign_network_versions: DashMap::new(),
1111#47DEQp|
1112#6bHYo9|            my_session_id: AtomicSessionId::new(rand::random()),
1113#6OvbvX|            dst_session_id: AtomicSessionId::new(0),
1114#47DEQp|
1115#HTzVyu|            we_are_initiator: AtomicBool::new(false),
1116#9SbFPc|            dst_is_initiator: AtomicBool::new(false),
1117#47DEQp|
1118#UG95LM|            need_sync_initiator_info: AtomicBool::new(false),
1119#47DEQp|
1120#BnuHRm|            rpc_tx_count: AtomicU32::new(0),
1121#PPXpdO|            rpc_rx_count: AtomicU32::new(0),
1122#47DEQp|
1123#ZtjE4j|            task: SessionTask::new(my_peer_id),
1124#KkQWgx|        }
1125#KNhneG|    }
1126#47DEQp|
1127#2MOuSV|    fn check_saved_peer_info_update_to_date(&self, peer_id: PeerId, version: Version) -> bool {
1128#VS7f_S|        if version == 0 || peer_id == self.dst_peer_id {
1129#nZMMkb|            // never send version 0 peer info to dst peer.
1130#KXQCUd|            return true;
1131#KkQWgx|        }
1132#L9eJ6o|        self.dst_saved_peer_info_versions
1133#ehozMf|            .get(&peer_id)
1134#sL1Vcw|            .map(|v| v.get() >= version)
1135#twEnen|            .unwrap_or(false)
1136#KNhneG|    }
1137#47DEQp|
1138#XGfsmJ|    fn update_dst_saved_peer_info_version(&self, infos: &[RoutePeerInfo]) {
1139#_KW523|        for info in infos.iter() {
1140#FR15-q|            self.dst_saved_peer_info_versions

...88458 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3cc963001rhbW21fwhJgZ2w
Use Grep to search the full content or Read with offset/limit to view specific sections.

## [assistant] +058391-05-11T05:19:10.000Z
[thinking] I found the critical piece. Looking at `RoutePeerInfo::update_self`:
```rust
ipv4_addr: global_ctx.get_ipv4().map(|x| x.address().into()),
```

And the conversion to `cli::Route`:
```rust
ipv4_addr: val.ipv4_addr.map(|ipv4_addr| Ipv4Inet {
    address: Some(ipv4_addr),
    network_length,
}),
```

So `ipv4_addr` in `cli::Route` is `Option<Ipv4Inet>` where `address` is `Option<Ipv4Addr>` and `network_length` is a u32.

But more importantly, `route.ipv4_addr` is of type `Option<Ipv4Inet>` (from proto). And `route.ipv4_addr.as_ref().and_then(|ip| ip.address.as_ref()).map(|a| format!("{}", a))` gives `Option<String>` with just the IP.

Actually wait, I know that there was a bug previously - the periodic task was using `format!("{}", ip)` where `ip` was `Ipv4Inet` (with subnet). But that was fixed. Let me check again... The periodic task now uses:
```rust
let ip_str = route.ipv4_addr.as_ref().and_then(|ip| ip.address.as_ref()).map(|a| format!("{}", a));
```

And the event-driven task:
```rust
let ip_str = format!("{}", ip);
```
where `ip` is from `info.ipv4_addr` which is `RoutePeerInfo.ipv4_addr` of type `Option<Ipv4Addr>` (from the prost-generated code). Wait, let me check again.

Actually, looking at `RoutePeerInfo` (from the prost-generated peer_rpc):
```
ipv4_addr: Option<Ipv4Addr>,
```

But in event-driven task (line 397-398):
```rust
if let Some(ref ip) = info.ipv4_addr {
    let ip_str = format!("{}", ip);
```

And `ip` here is an `Ipv4Addr` (from prost). `format!("{}", Ipv4Addr)` outputs just the IP without subnet. So that should be fine.

But wait, the periodic task uses `cli::Route` (from `list_routes()`), while the event-driven task uses `RoutePeerInfo` (from `get_route_peer_info()`). These have different types for `ipv4_addr`:
- `cli::Route.ipv4_addr`: `Option<Ipv4Inet>` where `Ipv4Inet` has `address: Option<Ipv4Addr>` and `network_length`
- `RoutePeerInfo.ipv4_addr`: `Option<Ipv4Addr>` (just the address)

So both should output pure IP strings. The earlier b
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer.rs"}) [completed]
1#Ss0OAx|use std::sync::Arc;
2#47DEQp|
3#yfXgtS|use crossbeam::atomic::AtomicCell;
4#a1CEkS|use dashmap::{DashMap, DashSet};
5#47DEQp|
6#L-bqq2|use tokio::{select, sync::mpsc};
7#47DEQp|
8#6jp-sY|use tracing::Instrument;
9#47DEQp|
10#Z2Y0cv|use super::{
11#JWQyPm|    peer_conn::{PeerConn, PeerConnId},
12#Hao9WQ|    PacketRecvChan,
13#3zmC_W|};
14#lq22mU|use crate::{common::scoped_task::ScopedTask, proto::cli::PeerConnInfo};
15#8h8Z46|use crate::{
16#hIkMG-|    common::{
17#Qez2zE|        error::Error,
18#wM7JAa|        global_ctx::{ArcGlobalCtx, GlobalCtxEvent},
19#yr1lub|        PeerId,
20#WGNt2R|    },
21#daFjPN|    tunnel::packet_def::ZCPacket,
22#3zmC_W|};
23#47DEQp|
24#pXEtkb|type ArcPeerConn = Arc<PeerConn>;
25#1OrYM0|type ConnMap = Arc<DashMap<PeerConnId, ArcPeerConn>>;
26#47DEQp|
27#KsEbVg|pub struct Peer {
28#WTvAAc|    pub peer_node_id: PeerId,
29#ij7mKO|    conns: ConnMap,
30#sfM3Lj|    global_ctx: ArcGlobalCtx,
31#47DEQp|
32#VAbqTO|    packet_recv_chan: PacketRecvChan,
33#47DEQp|
34#-fmM1d|    close_event_sender: mpsc::Sender<PeerConnId>,
35#h3kRUc|    close_event_listener: ScopedTask<()>,
36#47DEQp|
37#5dBR_6|    shutdown_notifier: Arc<tokio::sync::Notify>,
38#47DEQp|
39#4kOg11|    default_conn_id: Arc<AtomicCell<PeerConnId>>,
40#fILERX|    default_conn_id_clear_task: ScopedTask<()>,
41#0Qs2qn|}
42#47DEQp|
43#dAavnZ|impl Peer {
44#4SQmJP|    pub fn new(
45#RLAjwd|        peer_node_id: PeerId,
46#hyn7vu|        packet_recv_chan: PacketRecvChan,
47#V5_-NZ|        global_ctx: ArcGlobalCtx,
48#P1yxtn|    ) -> Self {
49#CwI0rI|        let conns: ConnMap = Arc::new(DashMap::new());
50#aQf4o2|        let (close_event_sender, mut close_event_receiver) = mpsc::channel(10);
51#Uvoo2p|        let shutdown_notifier = Arc::new(tokio::sync::Notify::new());
52#47DEQp|
53#r6Db5q|        let conns_copy = conns.clone();
54#NNJtLY|        let shutdown_notifier_copy = shutdown_notifier.clone();
55#YJfuJL|        let global_ctx_copy = global_ctx.clone();
56#A2u0zi|        let close_event_listener = tokio::spawn(
57#5taq5Y|            async move {
58#nZx_YA|                loop {
59#DESOsf|                    select! {
60#eyJkgS|                        ret = close_event_receiver.recv() => {
61#1I_WGs|                            if ret.is_none() {
62#oF2gnx|                                break;
63#ELKDGY|                            }
64#AhzUu3|                            let ret = ret.unwrap();
65#TnAwyk|                            tracing::warn!(
66#XlYi7g|                                ?peer_node_id,
67#e-GtVN|                                ?ret,
68#a1VEMa|                                "notified that peer conn is closed",
69#LPL0Oo|                            );
70#47DEQp|
71#bDPz9g|                            if let Some((_, conn)) = conns_copy.remove(&ret) {
72#vgpMTb|                                global_ctx_copy.issue_event(GlobalCtxEvent::PeerConnRemoved(
73#yRZD0u|                                    conn.get_conn_info(),
74#D5Ktgw|                                ));
75#ELKDGY|                            }
76#qrgf36|                        }
77#47DEQp|
78#aHgtsj|                        _ = shutdown_notifier_copy.notified() => {
79#bN4l5u|                            close_event_receiver.close();
80#qD1bZw|                            tracing::warn!(?peer_node_id, "peer close event listener notified");
81#qrgf36|                        }
82#uNGRgX|                    }
83#C_CgiK|                }
84#QnEoLG|                tracing::info!("peer {} close event listener exit", peer_node_id);
85#hq_fcK|            }
86#IBPyAb|            .instrument(tracing::info_span!(
87#LAX9Lb|                "peer_close_event_listener",
88#KXyaYo|                ?peer_node_id,
89#bt9-a0|            )),
90#qzjo1m|        )
91#I-oJyM|        .into();
92#47DEQp|
93#qkUrcT|        let default_conn_id = Arc::new(AtomicCell::new(PeerConnId::default()));
94#47DEQp|
95#r6Db5q|        let conns_copy = conns.clone();
96#5TM4GS|        let default_conn_id_copy = default_conn_id.clone();
97#zScP-1|        let default_conn_id_clear_task = ScopedTask::from(tokio::spawn(async move {
98#elI2Dp|            loop {
99#T2kLuB|                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
100#YFks8M|                if conns_copy.len() > 1 {
101#6Venhu|                    default_conn_id_copy.store(PeerConnId::default());
102#C_CgiK|                }
103#hq_fcK|            }
104#5u6gdO|        }));
105#47DEQp|
106#__EKyP|        Peer {
107#Y0ynKQ|            peer_node_id,
108#EOslZK|            conns: conns.clone(),
109#mXWJJ7|            packet_recv_chan,
110#PvFYEj|            global_ctx,
111#47DEQp|
112#qlA5Pl|            close_event_sender,
113#g7BDDN|            close_event_listener,
114#47DEQp|
115#_20raj|            shutdown_notifier,
116#f7aiOd|            default_conn_id,
117#MQAL0s|            default_conn_id_clear_task,
118#KkQWgx|        }
119#KNhneG|    }
120#47DEQp|
121#2qEkA3|    pub async fn add_peer_conn(&self, mut conn: PeerConn) {
122#zsGJPr|        let close_notifier = conn.get_close_notifier();
123#jCWgsN|        let conn_info = conn.get_conn_info();
124#47DEQp|
125#nB5VhN|        conn.start_recv_loop(self.packet_recv_chan.clone()).await;
126#z8rjF1|        conn.start_pingpong();
127#YqG3pH|        self.conns.insert(conn.get_conn_id(), Arc::new(conn));
128#47DEQp|
129#zapqyK|        let close_event_sender = self.close_event_sender.clone();
130#b9LDnJ|        tokio::spawn(async move {
131#HW6qrZ|            let conn_id = close_notifier.get_conn_id();
132#5ztD8Z|            if let Some(mut waiter) = close_notifier.get_waiter().await {
133#7Ax_0q|                let _ = waiter.recv().await;
134#hq_fcK|            }
135#32Vs6e|            if let Err(e) = close_event_sender.send(conn_id).await {
136#ucywu5|                tracing::warn!(?conn_id, "failed to send close event: {}", e);
137#hq_fcK|            }
138#NQ-BLD|        });
139#47DEQp|
140#G6v1RW|        self.global_ctx
141#j3ckLS|            .issue_event(GlobalCtxEvent::PeerConnAdded(conn_info));
142#KNhneG|    }
143#47DEQp|
144#RswJmf|    async fn select_conn(&self) -> Option<ArcPeerConn> {
145#sI3yp0|        let default_conn_id = self.default_conn_id.load();
146#nkcgsD|        if let Some(conn) = self.conns.get(&default_conn_id) {
147#m8-6rT|            return Some(conn.clone());
148#KkQWgx|        }
149#47DEQp|
150#GAlHqX|        // find a conn with the smallest latency
151#Vf4JXc|        let mut min_latency = u64::MAX;
152#aySRDp|        for conn in self.conns.iter() {
153#NPWYoe|            let latency = conn.value().get_stats().latency_us;
154#yf8XkS|            if latency < min_latency {
155#FBjzZj|                min_latency = latency;
156#tEVD_G|                self.default_conn_id.store(conn.get_conn_id());
157#hq_fcK|            }
158#KkQWgx|        }
159#47DEQp|
160#mq3b2V|        self.conns
161#_ZFddr|            .get(&self.default_conn_id.load())
162#Jt39_k|            .map(|conn| conn.clone())
163#KNhneG|    }
164#47DEQp|
165#ZHJ_Ud|    pub async fn send_msg(&self, msg: ZCPacket) -> Result<(), Error> {
166#MwJkZa|        let Some(conn) = self.select_conn().await else {
167#L0yUrc|            return Err(Error::PeerNoConnectionError(self.peer_node_id));
168#8OSK9L|        };
169#CmWy0i|        conn.send_msg(msg).await?;
170#47DEQp|
171#sX41Gp|        Ok(())
172#KNhneG|    }
173#47DEQp|
174#kF7-pG|    pub async fn close_peer_conn(&self, conn_id: &PeerConnId) -> Result<(), Error> {
175#7rrMGW|        let has_key = self.conns.contains_key(conn_id);
176#inq76c|        if !has_key {
177#5l5r9r|            return Err(Error::NotFound);
178#KkQWgx|        }
179#bfPNTs|        self.close_event_sender.send(*conn_id).await.unwrap();
180#sX41Gp|        Ok(())
181#KNhneG|    }
182#47DEQp|
183#eqAO5n|    pub async fn list_peer_conns(&self) -> Vec<PeerConnInfo> {
184#Vx3zu-|        let mut conns = vec![];
185#aySRDp|        for conn in self.conns.iter() {
186#urQVFX|            // do not lock here, otherwise it will cause dashmap deadlock
187#Rn0L3g|            conns.push(conn.clone());
188#KkQWgx|        }
189#47DEQp|
190#EkbaIO|        let mut ret = Vec::new();
191#O-hZgs|        for conn in conns {
192#VyaacC|            let info = conn.get_conn_info();
193#aWQuBH|            if !info.is_closed {
194#Hw-kH0|                ret.push(info);
195#CE4VEs|            } else {
196#Uwj3YD|                let conn_id = info.conn_id.parse().unwrap();
197#xa9vDt|                let _ = self.close_peer_conn(&conn_id).await;
198#hq_fcK|            }
199#KkQWgx|        }
200#eL4xpv|        ret
201#KNhneG|    }
202#47DEQp|
203#XqweCW|    pub fn has_directly_connected_conn(&self) -> bool {
204#mq3b2V|        self.conns
205#mGJIm4|            .iter()
206#ulofO6|            .any(|entry| !(entry.value()).is_hole_punched())
207#KNhneG|    }
208#47DEQp|
209#EbKrmt|    pub fn get_directly_connections(&self) -> DashSet<uuid::Uuid> {
210#mq3b2V|        self.conns
211#mGJIm4|            .iter()
212#5OLdK_|            .filter(|entry| !(entry.value()).is_hole_punched())
213#A3kw2l|            .map(|entry| (entry.value()).get_conn_id())
214#b1dGJO|            .collect()
215#KNhneG|    }
216#47DEQp|
217#U9dIl4|    pub fn get_default_conn_id(&self) -> PeerConnId {
218#mY_1tE|        self.default_conn_id.load()
219#KNhneG|    }
220#0Qs2qn|}
221#47DEQp|
222#_y7ihF|// pritn on drop
223#TUrODa|impl Drop for Peer {
224#mDnVS2|    fn drop(&mut self) {
225#lXYnl0|        self.shutdown_notifier.notify_one();
226#TUJwKf|        tracing::info!("peer {} drop", self.peer_node_id);
227#KNhneG|    }
228#0Qs2qn|}
229#47DEQp|
230#O6JvtA|#[cfg(test)]
231#FQleaY|mod tests {
232#47DEQp|
233#aVc3iO|    use tokio::time::timeout;
234#47DEQp|
235#iA5wSn|    use crate::{
236#gDJ50X|        common::{global_ctx::tests::get_mock_global_ctx, new_peer_id},
237#uvzRmo|        peers::{create_packet_recv_chan, peer_conn::PeerConn},
238#j8L9UI|        tunnel::ring::create_ring_tunnel_pair,
239#7em-_b|    };
240#47DEQp|
241#vs-2I_|    use super::Peer;
242#47DEQp|
243#y2wAzC|    #[tokio::test]
244#rD0D9S|    async fn close_peer() {
245#J-T8bk|        let (local_packet_send, _local_packet_recv) = create_packet_recv_chan();
246#h2YJPO|        let (remote_packet_send, _remote_packet_recv) = create_packet_recv_chan();
247#ABGv-R|        let global_ctx = get_mock_global_ctx();
248#JEGBs2|        let local_peer = Peer::new(new_peer_id(), local_packet_send, global_ctx.clone());
249#yAGLy0|        let remote_peer = Peer::new(new_peer_id(), remote_packet_send, global_ctx.clone());
250#47DEQp|
251#LOmfN5|        let (local_tunnel, remote_tunnel) = create_ring_tunnel_pair();
252#fDxP1k|        let mut local_peer_conn =
253#HV-pBF|            PeerConn::new(local_peer.peer_node_id, global_ctx.clone(), local_tunnel);
254#JHP0--|        let mut remote_peer_conn =
255#ghDlqj|            PeerConn::new(remote_peer.peer_node_id, global_ctx.clone(), remote_tunnel);
256#47DEQp|
257#lKwiiM|        assert!(!local_peer_conn.handshake_done());
258#iOedqH|        assert!(!remote_peer_conn.handshake_done());
259#47DEQp|
260#Xegkgg|        let (a, b) = tokio::join!(
261#PyzUFV|            local_peer_conn.do_handshake_as_client(),
262#fv60Zg|            remote_peer_conn.do_handshake_as_server()
263#bq_4ei|        );
264#Z2XVPK|        a.unwrap();
265#cxcUiZ|        b.unwrap();
266#47DEQp|
267#CG-04G|        let local_conn_id = local_peer_conn.get_conn_id();
268#47DEQp|
269#Ts5SkL|        local_peer.add_peer_conn(local_peer_conn).await;
270#goJxX9|        remote_peer.add_peer_conn(remote_peer_conn).await;
271#47DEQp|
272#S7W-66|        assert_eq!(local_peer.list_peer_conns().await.len(), 1);
273#eGNETb|        assert_eq!(remote_peer.list_peer_conns().await.len(), 1);
274#47DEQp|
275#plb2Di|        let close_handler =
276#iA1MlU|            tokio::spawn(async move { local_peer.close_peer_conn(&local_conn_id).await });
277#47DEQp|
278#DLetsy|        // wait for remote peer conn close
279#F-I7v2|        timeout(std::time::Duration::from_secs(5), async {
280#y-aDZ4|            while !remote_peer.list_peer_conns().await.is_empty() {
281#ZwCPnB|                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
282#hq_fcK|            }
283#9I500R|        })
284#g6MTzE|        .await
285#Du2ckX|        .unwrap();
286#47DEQp|
287#cCawc-|        println!("wait for close handler");
288#HX7e8w|        close_handler.await.unwrap().unwrap();
289#KNhneG|    }
290#0Qs2qn|}
291#47DEQp|

## [assistant] +058391-05-11T10:29:04.000Z
[thinking] NOW I found the critical bug! Look at this in `peer.rs`:

```rust
let default_conn_id_clear_task = ScopedTask::from(tokio::spawn(async move {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        if conns_copy.len() > 1 {
            default_conn_id_copy.store(PeerConnId::default());
        }
    }
}));
```

When there are multiple connections (len > 1), the `default_conn_id` is set to `PeerConnId::default()`. And the default `PeerConnId` is likely all zeros.

Now in `select_conn`:
```rust
async fn select_conn(&self) -> Option<ArcPeerConn> {
    let default_conn_id = self.default_conn_id.load();
    if let Some(conn) = self.conns.get(&default_conn_id) {
        return Some(conn.clone());
    }
    // find a conn with the smallest latency
    ...
}
```

So when `default_conn_id` is the default (all zeros), it won't match any real conn, so it falls through to finding the lowest latency conn. This then sets the `default_conn_id` to that conn.

But here's the key question: what does `get_peer_default_conn_id` return? It returns `p.get_default_conn_id()` which is `self.default_conn_id.load()`.

So when:
1. Peer has 2 connections (WebSocket + UDP)
2. At the 5-second check, `conns_copy.len() > 1` → `default_conn_id` is set to default (zeros)
3. Periodic task runs: `get_peer_default_conn_id` returns zeros
4. `close_peer_conn(peer_id, &zeros)` → `self.conns.contains_key(&zeros)` → FALSE (no conn with id zeros)
5. So `close_peer_conn` returns `Err(NotFound)`

BUT the periodic task doesn't check the error! It does:
```rust
if let Some(conn_id) = pm.get_peer_map().get_peer_default_conn_id(peer_id).await {
    let _ = pm.close_peer_conn(peer_id, &conn_id).await;
```

`get_peer_default_conn_id` returns `Option<PeerConnId>` - it always returns the default conn id (even if it's zeros). So `Some(zeros)` is returned, and `close_peer_conn` is called with zeros, which fails because there's no connection with that ID.

So **`close_peer_conn` fails sile
Now I see the critical issue. Let me verify the default connection ID behavior:
--- read_with_hash({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs","offset":1230,"limit":50}) [completed]
1#JkvAPI|use std::{
2#n-mlx5|    fmt::Debug,
3#0miUK5|    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
4#2oOvB1|    sync::{atomic::AtomicBool, Arc, Weak},
5#nkq1zO|    time::{Instant, SystemTime},
6#3zmC_W|};
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#7mQoz2|use async_trait::async_trait;
10#47DEQp|
11#pF_Tl6|use dashmap::DashMap;
12#47DEQp|
13#McK12P|use tokio::{
14#6SaBei|    sync::{
15#yXmr9W|        mpsc::{self, UnboundedReceiver, UnboundedSender},
16#keoZXw|        Mutex, RwLock,
17#WGNt2R|    },
18#8wmH-9|    task::JoinSet,
19#3zmC_W|};
20#47DEQp|
21#8h8Z46|use crate::{
22#hIkMG-|    common::{
23#7LA2Ap|        compressor::{Compressor as _, DefaultCompressor},
24#yGOl1X|        constants::EASYTIER_VERSION,
25#Qez2zE|        error::Error,
26#IOwpvn|        global_ctx::{ArcGlobalCtx, NetworkIdentity},
27#-CQTtT|        stats_manager::{CounterHandle, LabelSet, LabelType, MetricName},
28#TN8nTI|        stun::StunInfoCollectorTrait,
29#yr1lub|        PeerId,
30#WGNt2R|    },
31#bBQPjc|    peers::{
32#r7_WyB|        peer_conn::PeerConn,
33#ag2bHw|        peer_rpc::PeerRpcManagerTransport,
34#Fkytt7|        recv_packet_from_chan,
35#wGxRrF|        route_trait::{ForeignNetworkRouteInfoMap, MockRoute, NextHopPolicy, RouteInterface},
36#DlpQe-|        PeerPacketFilter,
37#WGNt2R|    },
38#Js7gIl|    proto::{
39#2Obpu4|        cli::{
40#jjjzE7|            self, list_global_foreign_network_response::OneForeignNetwork,
41#XNpGX2|            ListGlobalForeignNetworkResponse,
42#Vev0I6|        },
43#GHt5hR|        peer_rpc::{
44#LNbBxd|            ForeignNetworkRouteInfoEntry, ForeignNetworkRouteInfoKey, RouteForeignNetworkSummary,
45#Vev0I6|        },
46#WGNt2R|    },
47#5fAXpE|    tunnel::{
48#tnN8Km|        self,
49#Ce4DhK|        packet_def::{CompressorAlgo, PacketType, ZCPacket},
50#BHnU6g|        Tunnel, TunnelConnector,
51#WGNt2R|    },
52#3zmC_W|};
53#47DEQp|
54#Z2Y0cv|use super::{
55#xTI2Qn|    create_packet_recv_chan,
56#TV_3YQ|    encrypt::{Encryptor, NullCipher},
57#nXLxyb|    foreign_network_client::ForeignNetworkClient,
58#oGbRre|    foreign_network_manager::{ForeignNetworkManager, GlobalForeignNetworkAccessor},
59#rXyyCw|    peer_conn::PeerConnId,
60#mZdjXu|    peer_map::PeerMap,
61#XRTSKE|    peer_ospf_route::PeerRoute,
62#s2iwev|    peer_rpc::PeerRpcManager,
63#LWqQ_9|    route_trait::{ArcRoute, Route},
64#_s6yp8|    BoxNicPacketFilter, BoxPeerPacketFilter, PacketRecvChan, PacketRecvChanReceiver,
65#3zmC_W|};
66#47DEQp|
67#XQRd2S|struct RpcTransport {
68#ug-G5V|    my_peer_id: PeerId,
69#3hRIgc|    peers: Weak<PeerMap>,
70#4FpEM7|    // TODO: this seems can be removed
71#IiaVCa|    foreign_peers: Mutex<Option<Weak<ForeignNetworkClient>>>,
72#47DEQp|
73#Ta7SHq|    packet_recv: Mutex<UnboundedReceiver<ZCPacket>>,
74#JTTivI|    peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
75#47DEQp|
76#HzqH-Y|    encryptor: Arc<dyn Encryptor>,
77#0Qs2qn|}
78#47DEQp|
79#31qlYx|#[async_trait::async_trait]
80#bWsjuU|impl PeerRpcManagerTransport for RpcTransport {
81#2-s-Hc|    fn my_peer_id(&self) -> PeerId {
82#t_Rp3F|        self.my_peer_id
83#KNhneG|    }
84#47DEQp|
85#iLULEg|    async fn send(&self, mut msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
86#5WJPcd|        let peers = self.peers.upgrade().ok_or(Error::Unknown)?;
87#B8dSEy|        // NOTE: if route info is not exchanged, this will return None. treat it as public server.
88#vGxGid|        let is_dst_peer_public_server = peers
89#cg69RI|            .get_route_peer_info(dst_peer_id)
90#icqUnQ|            .await
91#5_bb6B|            .and_then(|x| x.feature_flag.map(|x| x.is_public_server))
92#7cxG-C|            // if dst is directly connected, it's must not public server
93#7Oh1uK|            .unwrap_or(!peers.has_peer(dst_peer_id));
94#PqGi0l|        if !is_dst_peer_public_server {
95#r_wdly|            self.encryptor
96#nsRc7J|                .encrypt(&mut msg)
97#lPh9vD|                .with_context(|| "encrypt failed")?;
98#KkQWgx|        }
99#EV3zNG|        // send to self and this packet will be forwarded in peer_recv loop
100#FaOb0U|        peers.send_msg_directly(msg, self.my_peer_id).await
101#KNhneG|    }
102#47DEQp|
103#wYJVUI|    async fn recv(&self) -> Result<ZCPacket, Error> {
104#EhjLK5|        if let Some(o) = self.packet_recv.lock().await.recv().await {
105#2qgdDw|            Ok(o)
106#d4ha1c|        } else {
107#7mEL_z|            Err(Error::Unknown)
108#KkQWgx|        }
109#KNhneG|    }
110#0Qs2qn|}
111#47DEQp|
112#e7IyZm|pub enum RouteAlgoType {
113#Ai27Qg|    Ospf,
114#gW4I1O|    None,
115#0Qs2qn|}
116#47DEQp|
117#Vim1EE|enum RouteAlgoInst {
118#cMn152|    Ospf(Arc<PeerRoute>),
119#gW4I1O|    None,
120#0Qs2qn|}
121#47DEQp|
122#RYRGuJ|struct SelfTxCounters {
123#D-V-tR|    self_tx_packets: CounterHandle,
124#6V0uMP|    self_tx_bytes: CounterHandle,
125#zGXsep|    compress_tx_bytes_before: CounterHandle,
126#DukShZ|    compress_tx_bytes_after: CounterHandle,
127#0Qs2qn|}
128#47DEQp|
129#A-4Ewh|pub struct PeerManager {
130#ug-G5V|    my_peer_id: PeerId,
131#47DEQp|
132#sfM3Lj|    global_ctx: ArcGlobalCtx,
133#kfhNSA|    nic_channel: PacketRecvChan,
134#47DEQp|
135#FwTrUD|    tasks: Mutex<JoinSet<()>>,
136#47DEQp|
137#_j4kdf|    packet_recv: Arc<Mutex<Option<PacketRecvChanReceiver>>>,
138#47DEQp|
139#tGvkRl|    peers: Arc<PeerMap>,
140#47DEQp|
141#8VMx0Q|    peer_rpc_mgr: Arc<PeerRpcManager>,
142#FeSjdm|    peer_rpc_tspt: Arc<RpcTransport>,
143#47DEQp|
144#YPcpo0|    peer_packet_process_pipeline: Arc<RwLock<Vec<BoxPeerPacketFilter>>>,
145#bNnuK9|    nic_packet_process_pipeline: Arc<RwLock<Vec<BoxNicPacketFilter>>>,
146#47DEQp|
147#65LL3T|    route_algo_inst: RouteAlgoInst,
148#47DEQp|
149#KBjRVM|    foreign_network_manager: Arc<ForeignNetworkManager>,
150#HP62aE|    foreign_network_client: Arc<ForeignNetworkClient>,
151#47DEQp|
152#JTtOhW|    encryptor: Arc<dyn Encryptor + 'static>,
153#ajfTlp|    data_compress_algo: CompressorAlgo,
154#47DEQp|
155#MzW6ir|    exit_nodes: Vec<IpAddr>,
156#47DEQp|
157#-OFllv|    reserved_my_peer_id_map: DashMap<String, PeerId>,
158#47DEQp|
159#yPvr-Q|    allow_loopback_tunnel: AtomicBool,
160#47DEQp|
161#EB2QlC|    self_tx_counters: SelfTxCounters,
162#0Qs2qn|}
163#47DEQp|
164#VEJ-Nv|impl Debug for PeerManager {
165#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
166#GfxoC0|        f.debug_struct("PeerManager")
167#yUbTEK|            .field("my_peer_id", &self.my_peer_id())
168#pZnIWM|            .field("instance_name", &self.global_ctx.inst_name)
169#jwchZi|            .field("net_ns", &self.global_ctx.net_ns.name())
170#uPgCZz|            .finish()
171#KNhneG|    }
172#0Qs2qn|}
173#47DEQp|
174#WaSJgC|impl PeerManager {
175#4SQmJP|    pub fn new(
176#84qYcS|        route_algo: RouteAlgoType,
177#V5_-NZ|        global_ctx: ArcGlobalCtx,
178#itxN3d|        nic_channel: PacketRecvChan,
179#P1yxtn|    ) -> Self {
180#Ei27Hm|        let my_peer_id = rand::random();
181#47DEQp|
182#bF21tz|        let (packet_send, packet_recv) = create_packet_recv_chan();
183#-k9OTl|        let peers = Arc::new(PeerMap::new(
184#TcJi4O|            packet_send.clone(),
185#2rw4tz|            global_ctx.clone(),
186#907ngS|            my_peer_id,
187#XNrhe2|        ));
188#47DEQp|
189#BCJiZI|        let encryptor = if global_ctx.get_flags().enable_encryption {
190#TrvYkF|            // 只有在启用加密时才使用工厂函数选择算法
191#kL1789|            let algorithm = &global_ctx.get_flags().encryption_algorithm;
192#dtVNcK|            super::encrypt::create_encryptor(
193#LTVAb5|                algorithm,
194#xEkbYr|                global_ctx.get_128_key(),
195#tezR8V|                global_ctx.get_256_key(),
196#jOhom-|            )
197#d4ha1c|        } else {
198#DlGJ0c|            // disable_encryption = true 时使用 NullCipher
199#P1gqj_|            Arc::new(NullCipher)
200#8OSK9L|        };
201#47DEQp|
202#cnaP6d|        if global_ctx
203#gL1TGg|            .check_network_in_whitelist(&global_ctx.get_network_name())
204#UZAafT|            .is_err()
205#qWZHcZ|        {
206#9cFfax|            // if local network is not in whitelist, avoid relay data when exist any other route path
207#55FUIS|            let mut f = global_ctx.get_feature_flags();
208#7NEeJZ|            f.avoid_relay_data = true;
209#ghRxLc|            global_ctx.set_feature_flags(f);
210#KkQWgx|        }
211#47DEQp|
212#eC7ab7|        // TODO: remove these because we have impl pipeline processor.
213#4QNvTE|        let (peer_rpc_tspt_sender, peer_rpc_tspt_recv) = mpsc::unbounded_channel();
214#odkilt|        let rpc_tspt = Arc::new(RpcTransport {
215#907ngS|            my_peer_id,
216#irHp49|            peers: Arc::downgrade(&peers),
217#XFt7Gs|            foreign_peers: Mutex::new(None),
218#-q3GY9|            packet_recv: Mutex::new(peer_rpc_tspt_recv),
219#rnjVTo|            peer_rpc_tspt_sender,
220#QCQqiA|            encryptor: encryptor.clone(),
221#NQ-BLD|        });
222#dTWeBi|        let peer_rpc_mgr = Arc::new(PeerRpcManager::new_with_stats_manager(
223#cL_fh1|            rpc_tspt.clone(),
224#UXBsZE|            global_ctx.stats_manager().clone(),
225#XNrhe2|        ));
226#47DEQp|
227#YyP1mU|        let route_algo_inst = match route_algo {
228#Y9u96Z|            RouteAlgoType::Ospf => RouteAlgoInst::Ospf(PeerRoute::new(
229#hGlxsx|                my_peer_id,
230#_bhH_r|                global_ctx.clone(),
231#LQuUWT|                peer_rpc_mgr.clone(),
232#bt9-a0|            )),
233#DFHyU5|            RouteAlgoType::None => RouteAlgoInst::None,
234#8OSK9L|        };
235#47DEQp|
236#cHpfI6|        let foreign_network_manager = Arc::new(ForeignNetworkManager::new(
237#907ngS|            my_peer_id,
238#2rw4tz|            global_ctx.clone(),
239#TcJi4O|            packet_send.clone(),
240#6wEwdz|            Self::build_foreign_network_manager_accessor(&peers),
241#XNrhe2|        ));
242#IqUQio|        let foreign_network_client = Arc::new(ForeignNetworkClient::new(
243#2rw4tz|            global_ctx.clone(),
244#TcJi4O|            packet_send.clone(),
245#DdNEnS|            peer_rpc_mgr.clone(),
246#907ngS|            my_peer_id,
247#XNrhe2|        ));
248#47DEQp|
249#JOS7nS|        let data_compress_algo = global_ctx
250#w9YV8E|            .get_flags()
251#CMuCL3|            .data_compress_algo()
252#aJofO-|            .try_into()
253#_byGCt|            .expect("invalid data compress algo, maybe some features not enabled");
254#47DEQp|
255#dA-spH|        let exit_nodes = global_ctx.config.get_exit_nodes();
256#47DEQp|
257#dvFkte|        let stats_manager = global_ctx.stats_manager();
258#ZcR7Qb|        let self_tx_counters = SelfTxCounters {
259#UwSzpA|            self_tx_packets: stats_manager.get_counter(
260#I7V2Ec|                MetricName::TrafficPacketsSelfTx,
261#qRrZQ1|                LabelSet::new()
262#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
263#XsBKuC|            ),
264#HPPGMJ|            self_tx_bytes: stats_manager.get_counter(
265#zOOjtD|                MetricName::TrafficBytesSelfTx,
266#qRrZQ1|                LabelSet::new()
267#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
268#XsBKuC|            ),
269#bc-v55|            compress_tx_bytes_before: stats_manager.get_counter(
270#xCAqxm|                MetricName::CompressionBytesTxBefore,
271#qRrZQ1|                LabelSet::new()
272#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
273#XsBKuC|            ),
274#u85wqm|            compress_tx_bytes_after: stats_manager.get_counter(
275#Frn6d_|                MetricName::CompressionBytesTxAfter,
276#qRrZQ1|                LabelSet::new()
277#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
278#XsBKuC|            ),
279#8OSK9L|        };
280#47DEQp|
281#eSauPW|        PeerManager {
282#907ngS|            my_peer_id,
283#47DEQp|
284#PvFYEj|            global_ctx,
285#3JnIN-|            nic_channel,
286#47DEQp|
287#V0fnw8|            tasks: Mutex::new(JoinSet::new()),
288#47DEQp|
289#chi1qM|            packet_recv: Arc::new(Mutex::new(Some(packet_recv))),
290#47DEQp|
291#N9VvX9|            peers: peers.clone(),
292#47DEQp|
293#V4bq0M|            peer_rpc_mgr,
294#AkAixq|            peer_rpc_tspt: rpc_tspt,
295#47DEQp|
296#RDG8zn|            peer_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
297#AVkW1r|            nic_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
298#47DEQp|
299#vtQn2d|            route_algo_inst,
300#47DEQp|
301#Sj49Bi|            foreign_network_manager,
302#mDTqnb|            foreign_network_client,
303#47DEQp|
304#QC8bmd|            encryptor,
305#Pu-vJW|            data_compress_algo,
306#47DEQp|
307#4PpgxG|            exit_nodes,
308#47DEQp|
309#MFZYOb|            reserved_my_peer_id_map: DashMap::new(),
310#47DEQp|
311#jn9OJ3|            allow_loopback_tunnel: AtomicBool::new(true),
312#47DEQp|
313#qTdfR3|            self_tx_counters,
314#KkQWgx|        }
315#KNhneG|    }
316#47DEQp|
317#XCaz50|    pub fn set_allow_loopback_tunnel(&self, allow_loopback_tunnel: bool) {
318#9uXnfd|        self.allow_loopback_tunnel
319#5i63Ur|            .store(allow_loopback_tunnel, std::sync::atomic::Ordering::Relaxed);
320#KNhneG|    }
321#47DEQp|
322#bSQhio|    fn build_foreign_network_manager_accessor(
323#dEPfKJ|        peer_map: &Arc<PeerMap>,
324#f_qBdl|    ) -> Box<dyn GlobalForeignNetworkAccessor> {
325#O4kh_j|        struct T {
326#mqjLcH|            peer_map: Weak<PeerMap>,
327#KkQWgx|        }
328#47DEQp|
329#kD-9ea|        #[async_trait::async_trait]
330#cwfbLX|        impl GlobalForeignNetworkAccessor for T {
331#-FwRBf|            async fn list_global_foreign_peer(
332#Xibb9x|                &self,
333#HOFVLU|                network_identity: &NetworkIdentity,
334#b6Lxu3|            ) -> Vec<PeerId> {
335#1C2Yt4|                let Some(peer_map) = self.peer_map.upgrade() else {
336#W_IXmF|                    return vec![];
337#JBqzQb|                };
338#47DEQp|
339#aWxCLN|                peer_map
340#lEVlUJ|                    .list_peers_own_foreign_network(network_identity)
341#lYNw0O|                    .await
342#hq_fcK|            }
343#KkQWgx|        }
344#47DEQp|
345#D2hsWL|        Box::new(T {
346#jYzaNh|            peer_map: Arc::downgrade(peer_map),
347#9I500R|        })
348#KNhneG|    }
349#47DEQp|
350#2lSRhY|    async fn add_new_peer_conn(&self, peer_conn: PeerConn) -> Result<(), Error> {
351#mxf9Cw|        if !self.global_ctx.is_valid_network_identity(&peer_conn.get_network_identity()) {
352#DvThUD|            return Err(Error::SecretKeyError(
353#cEWqcp|                "network identity not match".to_string(),
354#1oJHlw|            ));
355#KkQWgx|        }
356#ofM7n0|        let peer_id = peer_conn.get_peer_id();
357#Rar11V|        if self.global_ctx.is_peer_blocked(peer_id) {
358#DvThUD|            return Err(Error::SecretKeyError(
359#YTPquW|                "peer is blocked by IP whitelist".to_string(),
360#1oJHlw|            ));
361#KkQWgx|        }
362#aoqkXN|        self.peers.add_new_peer_conn(peer_conn).await;
363#sX41Gp|        Ok(())
364#KNhneG|    }
365#47DEQp|
366#14lYQ5|    pub async fn add_client_tunnel(
367#7BRTWY|        &self,
368#TMFJR1|        tunnel: Box<dyn Tunnel>,
369#GkW2PL|        is_directly_connected: bool,
370#QfKvkA|    ) -> Result<(PeerId, PeerConnId), Error> {
371#c0IlOn|        let mut peer = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
372#_IfWok|        peer.set_is_hole_punched(!is_directly_connected);
373#5yUaks|        peer.do_handshake_as_client().await?;
374#7JL2LP|        let conn_id = peer.get_conn_id();
375#RU1WKB|        let peer_id = peer.get_peer_id();
376#kTrXVm|        if peer.get_network_identity().network_name
377#ZrhvYf|            == self.global_ctx.get_network_identity().network_name
378#qWZHcZ|        {
379#f-Qeac|            self.add_new_peer_conn(peer).await?;
380#d4ha1c|        } else {
381#A7ODMm|            self.foreign_network_client.add_new_peer_conn(peer).await;
382#KkQWgx|        }
383#EPLuW8|        Ok((peer_id, conn_id))
384#KNhneG|    }
385#47DEQp|
386#jJc1fo|    pub fn has_directly_connected_conn(&self, peer_id: PeerId) -> bool {
387#3bpcwG|        if let Some(peer) = self.peers.get_peer_by_id(peer_id) {
388#Zq_A7q|            peer.has_directly_connected_conn()
389#d4ha1c|        } else {
390#kjtVBH|            self.foreign_network_client.get_peer_map().has_peer(peer_id)
391#KkQWgx|        }
392#KNhneG|    }
393#47DEQp|
394#pwuqRZ|    #[tracing::instrument]
395#o0DshP|    pub async fn try_direct_connect<C>(
396#7BRTWY|        &self,
397#i1Ryys|        mut connector: C,
398#EvgalJ|    ) -> Result<(PeerId, PeerConnId), Error>
399#JSvlCU|    where
400#rdvGRT|        C: TunnelConnector + Debug,
401#DT0vWo|    {
402#FQRxlN|        let ns = self.global_ctx.net_ns.clone();
403#09Wskl|        let t = ns
404#s0FDIJ|            .run_async(|| async move { connector.connect().await })
405#hmaiLi|            .await?;
406#9oIpNF|        self.add_client_tunnel(t, true).await
407#KNhneG|    }
408#47DEQp|
409#k9IM0H|    // avoid loop back to virtual network
410#4-_0cQ|    fn check_remote_addr_not_from_virtual_network(
411#7BRTWY|        &self,
412#6OnkCQ|        tunnel: &dyn Tunnel,
413#UnHDae|    ) -> Result<(), anyhow::Error> {
414#sd1I2C|        tracing::info!("check remote addr not from virtual network");
415#ClcXQQ|        let Some(tunnel_info) = tunnel.info() else {
416#PdNkBX|            anyhow::bail!("tunnel info is not set");
417#8OSK9L|        };
418#W0tgbh|        let Some(src) = tunnel_info.remote_addr.map(url::Url::from) else {
419#ol3jOX|            anyhow::bail!("tunnel info remote addr is not set");
420#8OSK9L|        };
421#u-6BoE|        if src.scheme() == "ring" {
422#6WCzfY|            return Ok(());
423#KkQWgx|        }
424#u9Qbc0|        let src_host = match src.socket_addrs(|| Some(1)) {
425#elIV8e|            Ok(addrs) => addrs,
426#gJq-2M|            Err(_) => {
427#PnbuE3|                // if the tunnel is not rely on ip address, skip check
428#LEBeyD|                return Ok(());
429#hq_fcK|            }
430#8OSK9L|        };
431#YrPT6Z|        let virtual_ipv4 = self.global_ctx.get_ipv4().map(|ip| ip.network());
432#MnnRwK|        let virtual_ipv6 = self.global_ctx.get_ipv6().map(|ip| ip.network());
433#3_Ehij|        tracing::info!(
434#5owbgP|            ?virtual_ipv4,
435#P0zdad|            ?virtual_ipv6,
436#jKufAz|            "check remote addr not from virtual network"
437#bq_4ei|        );
438#H79IuK|        for addr in src_host {
439#5YpsUk|            // if no-tun is enabled, the src ip of packet in virtual network is converted to loopback address
440#uqyRvu|            if addr.ip().is_loopback()
441#aSe1bi|                && !self
442#GwaNY6|                    .allow_loopback_tunnel
443#wg0XBg|                    .load(std::sync::atomic::Ordering::Relaxed)
444#eP03rc|            {
445#-2-r1E|                anyhow::bail!("tunnel src host is loopback address");
446#hq_fcK|            }
447#47DEQp|
448#Vs2UUY|            match addr {
449#Fkp5F3|                SocketAddr::V4(addr) => {
450#u2nxdD|                    if let Some(virtual_ipv4) = virtual_ipv4 {
451#JYPb6X|                        if virtual_ipv4.contains(addr.ip()) {
452#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
453#qrgf36|                        }
454#uNGRgX|                    }
455#C_CgiK|                }
456#Yj08jy|                SocketAddr::V6(addr) => {
457#qo9J_v|                    if let Some(virtual_ipv6) = virtual_ipv6 {
458#U0F33o|                        if virtual_ipv6.contains(addr.ip()) {
459#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
460#qrgf36|                        }
461#uNGRgX|                    }
462#C_CgiK|                }
463#hq_fcK|            }
464#KkQWgx|        }
465#sX41Gp|        Ok(())
466#KNhneG|    }
467#47DEQp|
468#U7Knbg|    #[tracing::instrument(ret)]
469#pg0BwE|    pub async fn add_tunnel_as_server(
470#7BRTWY|        &self,
471#TMFJR1|        tunnel: Box<dyn Tunnel>,
472#GkW2PL|        is_directly_connected: bool,
473#Ew16UP|    ) -> Result<(), Error> {
474#c2-UuG|        tracing::info!("add tunnel as server start");
475#6b66k9|        self.check_remote_addr_not_from_virtual_network(&tunnel)?;
476#47DEQp|
477#c3DuKs|        let mut conn = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
478#1CaMg8|        conn.do_handshake_as_server_ext(|peer, msg| {
479#2MbFGD|            if msg.network_name
480#z_HzRK|                == self.global_ctx.get_network_identity().network_name
481#eP03rc|            {
482#LEBeyD|                return Ok(());
483#hq_fcK|            }
484#47DEQp|
485#HMq8Dh|            if self.global_ctx.config.get_flags().private_mode {
486#Pvb-m7|                return Err(Error::SecretKeyError(
487#rq_jVx|                    "private mode is turned on, network identity not match".to_string(),
488#uRtctN|                ));
489#hq_fcK|            }
490#47DEQp|
491#Wvjjso|            let mut peer_id = self
492#WhSa3T|                .foreign_network_manager
493#ujWxtI|                .get_network_peer_id(&msg.network_name);
494#r-UKkg|            if peer_id.is_none() {
495#f8G6ks|                peer_id = Some(*self.reserved_my_peer_id_map.entry(msg.network_name.clone()).or_insert_with(|| {
496#cUHymN|                    rand::random::<PeerId>()
497#JxOJH_|                }).value());
498#hq_fcK|            }
499#OyZvBS|            peer.set_peer_id(peer_id.unwrap());
500#47DEQp|
501#8JJIOp|            tracing::info!(
502#QAVlTO|                ?peer_id,
503#XOCU_s|                ?msg.network_name,
504#YBX1Va|                "handshake as server with foreign network, new peer id: {}, peer id in foreign manager: {:?}",
505#jsth7q|                peer.get_my_peer_id(), peer_id
506#TP8Q3D|            );
507#47DEQp|
508#Y1ZmOf|            Ok(())
509#9I500R|        })
510#GGG_xT|        .await?;
511#47DEQp|
512#xVC9Et|        let peer_network_name = conn.get_network_identity().network_name.clone();
513#47DEQp|
514#Srsdme|        conn.set_is_hole_punched(!is_directly_connected);
515#47DEQp|
516#ia8Sn0|        if peer_network_name == self.global_ctx.get_network_identity().network_name {
517#32dI9Z|            self.add_new_peer_conn(conn).await?;
518#d4ha1c|        } else {
519#MIGy32|            self.foreign_network_manager.add_peer_conn(conn).await?;
520#KkQWgx|        }
521#47DEQp|
522#eakY8y|        self.reserved_my_peer_id_map.remove(&peer_network_name);
523#47DEQp|
524#093W8_|        tracing::info!("add tunnel as server done");
525#sX41Gp|        Ok(())
526#KNhneG|    }
527#47DEQp|
528#yBJw-S|    async fn try_handle_foreign_network_packet(
529#sCE1BR|        mut packet: ZCPacket,
530#kIT2Ch|        my_peer_id: PeerId,
531#huG0PK|        peer_map: &PeerMap,
532#ZvH2o7|        foreign_network_mgr: &ForeignNetworkManager,
533#u88bzm|    ) -> Result<(), ZCPacket> {
534#bhZeMH|        let pm_header = packet.peer_manager_header().unwrap();
535#fDsfht|        if pm_header.packet_type != PacketType::ForeignNetworkPacket as u8 {
536#KmX_9i|            return Err(packet);
537#KkQWgx|        }
538#47DEQp|
539#flN_K6|        let from_peer_id = pm_header.from_peer_id.get();
540#FRT-lF|        let to_peer_id = pm_header.to_peer_id.get();
541#47DEQp|
542#y4SHAG|        let foreign_hdr = packet.foreign_network_hdr().unwrap();
543#aXbqEm|        let foreign_network_name = foreign_hdr.get_network_name(packet.payload());
544#8quVoo|        let foreign_peer_id = foreign_hdr.get_dst_peer_id();
545#47DEQp|
546#IxuZAU|        let foreign_network_my_peer_id =
547#QNcXt2|            foreign_network_mgr.get_network_peer_id(&foreign_network_name);
548#47DEQp|
549#6MHiqH|        let buf_len = packet.buf_len();
550#-vAIvE|        let stats_manager = peer_map.get_global_ctx().stats_manager().clone();
551#5HN3PJ|        let label_set =
552#8VYKuw|            LabelSet::new().with_label_type(LabelType::NetworkName(foreign_network_name.clone()));
553#GxRWuo|        let add_counter = move |bytes_metric, packets_metric| {
554#S_bDbN|            stats_manager
555#XTcotD|                .get_counter(bytes_metric, label_set.clone())
556#nLeZsi|                .add(buf_len as u64);
557#9fhBOM|            stats_manager.get_counter(packets_metric, label_set).inc();
558#8OSK9L|        };
559#47DEQp|
560#YbVzBx|        // NOTICE: the to peer id is modified by the src from foreign network my peer id to the origin my peer id
561#W0MF1w|        if to_peer_id == my_peer_id {
562#viIycQ|            // packet sent from other peer to me, extract the inner packet and forward it
563#YUb47C|            add_counter(
564#mPzK_l|                MetricName::TrafficBytesForeignForwardRx,
565#h0W-Nu|                MetricName::TrafficPacketsForeignForwardRx,
566#TP8Q3D|            );
567#fdRDE9|            if let Err(e) = foreign_network_mgr
568#z4HPvh|                .send_msg_to_peer(
569#KnDqPt|                    &foreign_network_name,
570#5wz9iB|                    foreign_peer_id,
571#HFocUC|                    packet.foreign_network_packet(),
572#cN3xS-|                )
573#PcpKA_|                .await
574#eP03rc|            {
575#CW4xPB|                tracing::debug!(
576#2EUTrQ|                    ?e,
577#EwHmim|                    ?foreign_network_name,
578#P4MAxf|                    ?foreign_peer_id,
579#H3knNr|                    "foreign network mgr send_msg_to_peer failed"
580#iyW0eL|                );
581#hq_fcK|            }
582#Y1ZmOf|            Ok(())
583#YIUK0N|        } else if Some(from_peer_id) == foreign_network_my_peer_id {
584#mwhHwz|            // to_peer_id is my peer id for the foreign network, need to convert to the origin my_peer_id of dst
585#1Cllfa|            let Some(to_peer_id) = peer_map
586#vOHCEe|                .get_origin_my_peer_id(&foreign_network_name, to_peer_id)
587#PcpKA_|                .await
588#HO4mJ1|            else {
589#CW4xPB|                tracing::debug!(
590#EwHmim|                    ?foreign_network_name,
591#64bVg4|                    ?to_peer_id,
592#Fa_5NR|                    "cannot find origin my peer id for foreign network."
593#iyW0eL|                );
594#1QMLIr|                return Err(packet);
595#huZslw|            };
596#47DEQp|
597#YUb47C|            add_counter(
598#isNHJA|                MetricName::TrafficBytesForeignForwardTx,
599#Yut8ka|                MetricName::TrafficPacketsForeignForwardTx,
600#TP8Q3D|            );
601#47DEQp|
602#qFCK98|            // modify the to_peer id from foreign network my peer id to the origin my peer id
603#1qxWD1|            packet
604#0lMrqe|                .mut_peer_manager_header()
605#gQk2jP|                .unwrap()
606#2oYuph|                .to_peer_id
607#1FcmIL|                .set(to_peer_id);
608#47DEQp|
609#WWRUyf|            // packet is generated from foreign network mgr and should be forward to other peer
610#GjxVjo|            if let Err(e) = peer_map
611#pWezG5|                .send_msg(packet, to_peer_id, NextHopPolicy::LeastHop)
612#PcpKA_|                .await
613#eP03rc|            {
614#CW4xPB|                tracing::debug!(
615#2EUTrQ|                    ?e,
616#64bVg4|                    ?to_peer_id,
617#0xHh2s|                    "send_msg_directly failed when forward local generated foreign network packet"
618#iyW0eL|                );
619#hq_fcK|            }
620#Y1ZmOf|            Ok(())
621#d4ha1c|        } else {
622#b_Rzck|            // target is not me, forward it. try get origin peer id
623#YUb47C|            add_counter(
624#iZmaoy|                MetricName::TrafficBytesForeignForwardForwarded,
625#1fxT_q|                MetricName::TrafficPacketsForeignForwardForwarded,
626#TP8Q3D|            );
627#BKXjm4|            Err(packet)
628#KkQWgx|        }
629#KNhneG|    }
630#47DEQp|
631#FHuK_1|    async fn start_peer_recv(&self) {
632#37sTF-|        let mut recv = self.packet_recv.lock().await.take().unwrap();
633#vnR7aH|        let my_peer_id = self.my_peer_id;
634#nmPmgi|        let peers = self.peers.clone();
635#97eb5s|        let pipe_line = self.peer_packet_process_pipeline.clone();
636#8aTJMr|        let foreign_client = self.foreign_network_client.clone();
637#C546Zb|        let foreign_mgr = self.foreign_network_manager.clone();
638#k_2-Ol|        let encryptor = self.encryptor.clone();
639#mPlkH9|        let compress_algo = self.data_compress_algo;
640#GRlbc0|        let acl_filter = self.global_ctx.get_acl_filter().clone();
641#ZGLiLv|        let global_ctx = self.global_ctx.clone();
642#UZrWMo|        let stats_mgr = self.global_ctx.stats_manager().clone();
643#9sGRWS|        let route = self.get_route();
644#47DEQp|
645#5HN3PJ|        let label_set =
646#aDVoKb|            LabelSet::new().with_label_type(LabelType::NetworkName(global_ctx.get_network_name()));
647#47DEQp|
648#mEGOr9|        let self_tx_bytes = self.self_tx_counters.self_tx_bytes.clone();
649#T7N9pS|        let self_tx_packets = self.self_tx_counters.self_tx_packets.clone();
650#hCUNVJ|        let self_rx_bytes =
651#80PMSq|            stats_mgr.get_counter(MetricName::TrafficBytesSelfRx, label_set.clone());
652#DmhFt-|        let self_rx_packets =
653#DmhvAH|            stats_mgr.get_counter(MetricName::TrafficPacketsSelfRx, label_set.clone());
654#FP31iZ|        let forward_tx_bytes =
655#mIb08E|            stats_mgr.get_counter(MetricName::TrafficBytesForwarded, label_set.clone());
656#N6gzzb|        let forward_tx_packets =
657#nCT_oP|            stats_mgr.get_counter(MetricName::TrafficPacketsForwarded, label_set.clone());
658#47DEQp|
659#9nvSTI|        let compress_tx_bytes_before = self.self_tx_counters.compress_tx_bytes_before.clone();
660#8Ehyj-|        let compress_tx_bytes_after = self.self_tx_counters.compress_tx_bytes_after.clone();
661#dMW85W|        let compress_rx_bytes_before =
662#Jryksd|            stats_mgr.get_counter(MetricName::CompressionBytesRxBefore, label_set.clone());
663#CJs0ZN|        let compress_rx_bytes_after =
664#OkiaLh|            stats_mgr.get_counter(MetricName::CompressionBytesRxAfter, label_set.clone());
665#47DEQp|
666#DDENzd|        self.tasks.lock().await.spawn(async move {
667#s1vPO9|            tracing::trace!("start_peer_recv");
668#z3Goih|            while let Ok(ret) = recv_packet_from_chan(&mut recv).await {
669#w5bSOZ|                let Err(mut ret) =
670#E9vP_K|                    Self::try_handle_foreign_network_packet(ret, my_peer_id, &peers, &foreign_mgr)
671#o6c0n-|                        .await
672#QIIDCb|                else {
673#_01Y-5|                    continue;
674#JBqzQb|                };
675#47DEQp|
676#z4xbBM|                let buf_len = ret.buf_len();
677#sLsb9g|                let Some(hdr) = ret.mut_peer_manager_header() else {
678#eP2t8R|                    tracing::warn!(?ret, "invalid packet, skip");
679#_01Y-5|                    continue;
680#JBqzQb|                };
681#47DEQp|
682#ax0YAR|                tracing::trace!(?hdr, "peer recv a packet...");
683#G1tVuV|                let from_peer_id = hdr.from_peer_id.get();
684#i2EURd|                let to_peer_id = hdr.to_peer_id.get();
685#Cuycos|                if to_peer_id != my_peer_id {
686#JRJHei|                    if hdr.forward_counter > 7 {
687#F8mduZ|                        tracing::warn!(?hdr, "forward counter exceed, drop packet");
688#W98xla|                        continue;
689#uNGRgX|                    }
690#47DEQp|
691#pwyl0-|                    if hdr.forward_counter > 2 && hdr.is_latency_first() {
692#uVS30W|                        tracing::trace!(?hdr, "set_latency_first false because too many hop");
693#9ndRqd|                        hdr.set_latency_first(false);
694#uNGRgX|                    }
695#47DEQp|
696#VdVaow|                    hdr.forward_counter += 1;
697#47DEQp|
698#ZT0lS_|                    if from_peer_id == my_peer_id {
699#uroO3I|                        compress_tx_bytes_before.add(buf_len as u64);
700#47DEQp|
701#iGD8zw|                        if hdr.packet_type == PacketType::Data as u8
702#GABpVR|                            || hdr.packet_type == PacketType::KcpSrc as u8
703#Z6rbMg|                            || hdr.packet_type == PacketType::KcpDst as u8
704#Fzqoyl|                        {
705#1LI0Vo|                            let _ =
706#JUJFc0|                                Self::try_compress_and_encrypt(compress_algo, &encryptor, &mut ret)
707#1yUlMs|                                    .await;
708#qrgf36|                        }
709#47DEQp|
710#T0tOoJ|                        compress_tx_bytes_after.add(ret.buf_len() as u64);
711#tANDt7|                        self_tx_bytes.add(ret.buf_len() as u64);
712#f8O7M7|                        self_tx_packets.inc();
713#HyGTCY|                    } else {
714#Pguetm|                        forward_tx_bytes.add(buf_len as u64);
715#gmyQh8|                        forward_tx_packets.inc();
716#uNGRgX|                    }
717#47DEQp|
718#kSLxh3|                    tracing::trace!(?to_peer_id, ?my_peer_id, "need forward");
719#b8Js0t|                    let ret =
720#EZHc3h|                        Self::send_msg_internal(&peers, &foreign_client, ret, to_peer_id).await;
721#LXf7AG|                    if ret.is_err() {
722#RK5a5R|                        tracing::error!(?ret, ?to_peer_id, ?from_peer_id, "forward packet error");
723#uNGRgX|                    }
724#ZtcDW9|                } else {
725#fn_fyy|                    if let Err(e) = encryptor.decrypt(&mut ret) {
726#HZCGH0|                        tracing::error!(?e, "decrypt failed");
727#W98xla|                        continue;
728#uNGRgX|                    }
729#47DEQp|
730#Ln7UBJ|                    self_rx_bytes.add(buf_len as u64);
731#XNjtrJ|                    self_rx_packets.inc();
732#BHaA-C|                    compress_rx_bytes_before.add(buf_len as u64);
733#47DEQp|
734#QblX98|                    let compressor = DefaultCompressor {};
735#GMZW0O|                    if let Err(e) = compressor.decompress(&mut ret).await {
736#gbQ6SV|                        tracing::error!(?e, "decompress failed");
737#W98xla|                        continue;
738#uNGRgX|                    }
739#47DEQp|
740#Hjej0y|                    compress_rx_bytes_after.add(ret.buf_len() as u64);
741#47DEQp|
742#2dQ5pH|                    if !acl_filter.process_packet_with_acl(
743#NQJi05|                        &ret,
744#VaXc6J|                        true,
745#NFzhEu|                        global_ctx.get_ipv4().map(|x| x.address()),
746#NvLih8|                        global_ctx.get_ipv6().map(|x| x.address()),
747#5sWIyd|                        &route,
748#SwK0Oa|                    ) {
749#W98xla|                        continue;
750#uNGRgX|                    }
751#47DEQp|
752#cqHypd|                    let mut processed = false;
753#fYLeTn|                    let mut zc_packet = Some(ret);
754#IhX6Kq|                    for (idx, pipeline) in pipe_line.read().await.iter().rev().enumerate() {
755#r-rtuq|                        tracing::trace!(?zc_packet, ?idx, "try_process_packet_from_peer");
756#0x6iPo|                        zc_packet = pipeline
757#D_qrXs|                            .try_process_packet_from_peer(zc_packet.unwrap())
758#IY999y|                            .await;
759#Ew5lXA|                        if zc_packet.is_none() {
760#10AbF1|                            processed = true;
761#Ko0WZT|                            break;
762#qrgf36|                        }
763#uNGRgX|                    }
764#R2opY7|                    if !processed {
765#BN52j-|                        tracing::error!(?zc_packet, "unhandled packet");
766#uNGRgX|                    }
767#C_CgiK|                }
768#hq_fcK|            }
769#H99HIN|            panic!("done_peer_recv");
770#NQ-BLD|        });
771#KNhneG|    }
772#47DEQp|
773#q_JTCC|    pub async fn add_packet_process_pipeline(&self, pipeline: BoxPeerPacketFilter) {
774#5xAyFb|        // newest pipeline will be executed first
775#9k-qPG|        self.peer_packet_process_pipeline
776#jtBdos|            .write()
777#icqUnQ|            .await
778#gbkGf8|            .push(pipeline);
779#KNhneG|    }
780#47DEQp|
781#ekn-0q|    pub async fn add_nic_packet_process_pipeline(&self, pipeline: BoxNicPacketFilter) {
782#5xAyFb|        // newest pipeline will be executed first
783#lgMoPV|        self.nic_packet_process_pipeline
784#jtBdos|            .write()
785#icqUnQ|            .await
786#gbkGf8|            .push(pipeline);
787#KNhneG|    }
788#47DEQp|
789#rwu4HQ|    async fn init_packet_process_pipeline(&self) {
790#LJfv5W|        // for tun/tap ip/eth packet.
791#hwmngc|        struct NicPacketProcessor {
792#My7Qci|            nic_channel: PacketRecvChan,
793#KkQWgx|        }
794#kD-9ea|        #[async_trait::async_trait]
795#OtOTlR|        impl PeerPacketFilter for NicPacketProcessor {
796#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
797#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
798#L2pk7f|                if hdr.packet_type == PacketType::Data as u8 {
799#dx_3aF|                    tracing::trace!(?packet, "send packet to nic channel");
800#9RlVs2|                    // TODO: use a function to get the body ref directly for zero copy
801#D1yJAP|                    let _ = self.nic_channel.send(packet).await;
802#JqjnWy|                    None
803#ZtcDW9|                } else {
804#jLWmFO|                    Some(packet)
805#C_CgiK|                }
806#hq_fcK|            }
807#KkQWgx|        }
808#zvD7Bg|        self.add_packet_process_pipeline(Box::new(NicPacketProcessor {
809#PTKjfM|            nic_channel: self.nic_channel.clone(),
810#AVIbAB|        }))
811#10YU6B|        .await;
812#47DEQp|
813#s7MHfd|        // for peer rpc packet
814#N1Iaxj|        struct PeerRpcPacketProcessor {
815#Lx--BG|            peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
816#KkQWgx|        }
817#47DEQp|
818#kD-9ea|        #[async_trait::async_trait]
819#D1zWOn|        impl PeerPacketFilter for PeerRpcPacketProcessor {
820#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
821#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
822#CrfTXZ|                if hdr.packet_type == PacketType::TaRpc as u8
823#TBQp0z|                    || hdr.packet_type == PacketType::RpcReq as u8
824#6aiYqm|                    || hdr.packet_type == PacketType::RpcResp as u8
825#NV9RY5|                {
826#DWqudi|                    self.peer_rpc_tspt_sender.send(packet).unwrap();
827#JqjnWy|                    None
828#ZtcDW9|                } else {
829#jLWmFO|                    Some(packet)
830#C_CgiK|                }
831#hq_fcK|            }
832#KkQWgx|        }
833#4XfYK9|        self.add_packet_process_pipeline(Box::new(PeerRpcPacketProcessor {
834#Nc02qB|            peer_rpc_tspt_sender: self.peer_rpc_tspt.peer_rpc_tspt_sender.clone(),
835#AVIbAB|        }))
836#10YU6B|        .await;
837#KNhneG|    }
838#47DEQp|
839#qnv61H|    pub async fn add_route<T>(&self, route: T)
840#JSvlCU|    where
841#s0mP8g|        T: Route + PeerPacketFilter + Send + Sync + Clone + 'static,
842#DT0vWo|    {
843#YR42TL|        // for route
844#DOPJKV|        self.add_packet_process_pipeline(Box::new(route.clone()))
845#agFGm5|            .await;
846#47DEQp|
847#hY75HP|        struct Interface {
848#y1Tqlc|            my_peer_id: PeerId,
849#rrAPV3|            peers: Weak<PeerMap>,
850#21pgeS|            foreign_network_client: Weak<ForeignNetworkClient>,
851#8Gbfqs|            foreign_network_manager: Weak<ForeignNetworkManager>,
852#KkQWgx|        }
853#47DEQp|
854#KGY7Sc|        #[async_trait]
855#DkYunO|        impl RouteInterface for Interface {
856#l_VjGY|            async fn list_peers(&self) -> Vec<PeerId> {
857#XDZ2Vk|                let Some(foreign_client) = self.foreign_network_client.upgrade() else {
858#W_IXmF|                    return vec![];
859#JBqzQb|                };
860#47DEQp|
861#PUDAeF|                let Some(peer_map) = self.peers.upgrade() else {
862#W_IXmF|                    return vec![];
863#JBqzQb|                };
864#47DEQp|
865#XQDNH1|                let mut peers = foreign_client.list_public_peers().await;
866#njwKeW|                peers.extend(peer_map.list_peers_with_conn().await);
867#MTa5cE|                peers
868#hq_fcK|            }
869#47DEQp|
870#LFp0oo|            fn my_peer_id(&self) -> PeerId {
871#AE5UEx|                self.my_peer_id
872#hq_fcK|            }
873#47DEQp|
874#uonLKK|            async fn list_foreign_networks(&self) -> ForeignNetworkRouteInfoMap {
875#5THHmO|                let ret = DashMap::new();
876#swf1dg|                let Some(foreign_mgr) = self.foreign_network_manager.upgrade() else {
877#b5Svxi|                    return ret;
878#JBqzQb|                };
879#47DEQp|
880#Y8o-Yf|                let networks = foreign_mgr.list_foreign_networks().await;
881#ioetyj|                for (network_name, info) in networks.foreign_networks.iter() {
882#JV3_bG|                    if info.peers.is_empty() {
883#W98xla|                        continue;
884#uNGRgX|                    }
885#47DEQp|
886#LK7acJ|                    let last_update = foreign_mgr
887#JxC-fm|                        .get_foreign_network_last_update(network_name)
888#v0277T|                        .unwrap_or(SystemTime::now());
889#4cuRRV|                    ret.insert(
890#zuxFX7|                        ForeignNetworkRouteInfoKey {
891#FvjDof|                            peer_id: self.my_peer_id,
892#MBDbjS|                            network_name: network_name.clone(),
893#xt3MAa|                        },
894#WTdtwn|                        ForeignNetworkRouteInfoEntry {
895#nSGc8d|                            foreign_peer_ids: info.peers.iter().map(|x| x.peer_id).collect(),
896#DEQ9Fe|                            last_update: Some(last_update.into()),
897#QBzroT|                            version: 0,
898#rLkInO|                            network_secret_digest: info.network_secret_digest.clone(),
899#5efuR1|                            my_peer_id_for_this_network: info.my_peer_id_for_this_network,
900#xt3MAa|                        },
901#iTFhZ0|                    );
902#C_CgiK|                }
903#0qn6Vo|                ret
904#hq_fcK|            }
905#KkQWgx|        }
906#47DEQp|
907#vnR7aH|        let my_peer_id = self.my_peer_id;
908#T7G1Zk|        let _route_id = route
909#SfQrJo|            .open(Box::new(Interface {
910#hGlxsx|                my_peer_id,
911#0F1sFB|                peers: Arc::downgrade(&self.peers),
912#ER66ww|                foreign_network_client: Arc::downgrade(&self.foreign_network_client),
913#yRkAf3|                foreign_network_manager: Arc::downgrade(&self.foreign_network_manager),
914#LufvaI|            }))
915#icqUnQ|            .await
916#WEGyMY|            .unwrap();
917#47DEQp|
918#ORaZ3y|        let arc_route: ArcRoute = Arc::new(Box::new(route));
919#fY7N7T|        self.peers.add_route(arc_route).await;
920#KNhneG|    }
921#47DEQp|
922#rj8ACA|    pub fn get_route(&self) -> Box<dyn Route + Send + Sync + 'static> {
923#aWnGNt|        match &self.route_algo_inst {
924#ind_vT|            RouteAlgoInst::Ospf(route) => Box::new(route.clone()),
925#3ENyBM|            RouteAlgoInst::None => Box::new(MockRoute {}),
926#KkQWgx|        }
927#KNhneG|    }
928#47DEQp|
929#ggsQcx|    pub async fn list_routes(&self) -> Vec<cli::Route> {
930#buqXa2|        self.get_route().list_routes().await
931#KNhneG|    }
932#47DEQp|
933#SQwiYH|    pub async fn get_route_peer_info_last_update_time(&self) -> Instant {
934#o0Hn44|        self.get_route().get_peer_info_last_update_time().await
935#KNhneG|    }
936#47DEQp|
937#rMN99o|    pub async fn dump_route(&self) -> String {
938#NjIrQ5|        self.get_route().dump().await
939#KNhneG|    }
940#47DEQp|
941#qHMIGl|    pub async fn list_global_foreign_network(&self) -> ListGlobalForeignNetworkResponse {
942#dSN10n|        let mut resp = ListGlobalForeignNetworkResponse::default();
943#0GRtAb|        let ret = self.get_route().list_foreign_network_info().await;
944#PfLspy|        for info in ret.infos.iter() {
945#los8mn|            let entry = resp
946#cAKnz7|                .foreign_networks
947#x4ul2a|                .entry(info.key.as_ref().unwrap().peer_id)
948#ctxgtd|                .or_insert_with(Default::default);
949#MiV-NN|            let Some(route_info) = info.value.as_ref() else {
950#YqjFPd|                continue;
951#huZslw|            };
952#47DEQp|
953#ubAyqu|            let f = OneForeignNetwork {
954#1WNjy7|                network_name: info.key.as_ref().unwrap().network_name.clone(),
955#LogZYQ|                peer_ids: route_info.foreign_peer_ids.clone(),
956#RIHZBu|                last_updated: format!("{}", route_info.last_update.unwrap()),
957#hHIWcX|                version: route_info.version,
958#huZslw|            };
959#47DEQp|
960#TZwkyd|            entry.foreign_networks.push(f);
961#KkQWgx|        }
962#47DEQp|
963#ZudY7T|        resp
964#KNhneG|    }
965#47DEQp|
966#J0LLTy|    pub async fn get_foreign_network_summary(&self) -> RouteForeignNetworkSummary {
967#yFVVTB|        self.get_route().get_foreign_network_summary().await
968#KNhneG|    }
969#47DEQp|
970#WFyYU9|    async fn run_nic_packet_process_pipeline(&self, data: &mut ZCPacket) {
971#81LGkF|        if !self.global_ctx.get_acl_filter().process_packet_with_acl(
972#RSksDL|            data,
973#bBjPxJ|            false,
974#SKXxAq|            None,
975#SKXxAq|            None,
976#HUVukP|            &self.get_route(),
977#HceTsT|        ) {
978#NhdEjk|            return;
979#KkQWgx|        }
980#47DEQp|
981#AzFZTs|        for pipeline in self.nic_packet_process_pipeline.read().await.iter().rev() {
982#iJa1xN|            let _ = pipeline.try_process_packet_from_nic(data).await;
983#KkQWgx|        }
984#KNhneG|    }
985#47DEQp|
986#LNmNxR|    pub async fn remove_nic_packet_process_pipeline(&self, id: String) -> Result<(), Error> {
987#kzaKYh|        let mut pipelines = self.nic_packet_process_pipeline.write().await;
988#aIOBXg|        if let Some(pos) = pipelines.iter().position(|x| x.id() == id) {
989#pyEoMy|            pipelines.remove(pos);
990#Y1ZmOf|            Ok(())
991#d4ha1c|        } else {
992#j9KMDj|            Err(Error::NotFound)
993#KkQWgx|        }
994#KNhneG|    }
995#47DEQp|
996#QwGVbl|    fn get_next_hop_policy(is_first_latency: bool) -> NextHopPolicy {
997#Prtm2m|        if is_first_latency {
998#lrGTD9|            NextHopPolicy::LeastCost
999#d4ha1c|        } else {
1000#0t3aFq|            NextHopPolicy::LeastHop
1001#KkQWgx|        }
1002#KNhneG|    }
1003#47DEQp|
1004#YcV_Zw|    pub async fn send_msg(&self, msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
1005#0Y1ljl|        self.self_tx_counters
1006#7boxeQ|            .self_tx_bytes
1007#OREiCd|            .add(msg.buf_len() as u64);
1008#P1nqM0|        self.self_tx_counters.self_tx_packets.inc();
1009#X4VtRb|        let msg_len = msg.buf_len() as u64;
1010#X_Kl5i|        let result =
1011#Pd9f-1|            Self::send_msg_internal(&self.peers, &self.foreign_network_client, msg, dst_peer_id)
1012#mju4UJ|                .await;
1013#ZxIhWe|        if result.is_ok() {
1014#wEbH0X|            self.self_tx_counters.self_tx_bytes.add(msg_len);
1015#zhexxv|            self.self_tx_counters.self_tx_packets.inc();
1016#KkQWgx|        }
1017#NMBvw3|        result
1018#KNhneG|    }
1019#47DEQp|
1020#zhhnC3|    async fn send_msg_internal(
1021#swj2Tw|        peers: &Arc<PeerMap>,
1022#rQ13r4|        foreign_network_client: &Arc<ForeignNetworkClient>,
1023#i65YQK|        msg: ZCPacket,
1024#LiSCIf|        dst_peer_id: PeerId,
1025#Ew16UP|    ) -> Result<(), Error> {
1026#Vg3OfO|        let policy =
1027#s6Hg6i|            Self::get_next_hop_policy(msg.peer_manager_header().unwrap().is_latency_first());
1028#47DEQp|
1029#S-yTsc|        if let Some(gateway) = peers.get_gateway_peer_id(dst_peer_id, policy.clone()).await {
1030#CVigK7|            if peers.has_peer(gateway) {
1031#5Mp0l-|                peers.send_msg_directly(msg, gateway).await
1032#JP80iE|            } else if foreign_network_client.has_next_hop(gateway) {
1033#znEXzS|                foreign_network_client.send_msg(msg, gateway).await
1034#CE4VEs|            } else {
1035#tw81bP|                tracing::warn!(
1036#kk6F7t|                    ?gateway,
1037#_vSdBB|                    ?dst_peer_id,
1038#5aLh6k|                    "cannot send msg to peer through gateway"
1039#iyW0eL|                );
1040#rw6i9c|                Err(Error::RouteError(None))
1041#hq_fcK|            }
1042#OEJCrt|        } else if foreign_network_client.has_next_hop(dst_peer_id) {
1043#hzCk3v|            // check foreign network again. so in happy path we can avoid extra check
1044#YLJ9ty|            foreign_network_client.send_msg(msg, dst_peer_id).await
1045#d4ha1c|        } else {
1046#kVR-Se|            tracing::debug!(?dst_peer_id, "no gateway for peer");
1047#NuwARt|            Err(Error::RouteError(None))
1048#KkQWgx|        }
1049#KNhneG|    }
1050#47DEQp|
1051#n2tvl8|    pub async fn get_msg_dst_peer(&self, ipv4_addr: &Ipv4Addr) -> (Vec<PeerId>, bool) {
1052#YR8kEa|        let mut is_exit_node = false;
1053#5eR7LF|        let mut dst_peers = vec![];
1054#FZMhE_|        let network_length = self
1055#7t5vZi|            .global_ctx
1056#THImhK|            .get_ipv4()
1057#8fe7wM|            .map(|x| x.network_length())
1058#y5RfWx|            .unwrap_or(24);
1059#D0sue1|        let ipv4_inet = cidr::Ipv4Inet::new(*ipv4_addr, network_length).unwrap();
1060#vpDCah|        if ipv4_addr.is_broadcast()
1061#lwUXjr|            || ipv4_addr.is_multicast()
1062#QttO4f|            || *ipv4_addr == ipv4_inet.last_address()
1063#qWZHcZ|        {
1064#poVYD0|            dst_peers.extend(self.peers.list_routes().await.iter().filter_map(|x| {
1065#r49BH8|                if *x.key() != self.my_peer_id {
1066#RZUfe_|                    Some(*x.key())
1067#ZtcDW9|                } else {
1068#JqjnWy|                    None
1069#C_CgiK|                }
1070#Hsxh2D|            }));
1071#vflSHi|        } else if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(ipv4_addr).await {
1072#otf58d|            dst_peers.push(peer_id);
1073#pegr-g|        } else if !self
1074#7t5vZi|            .global_ctx
1075#VvsySw|            .is_ip_in_same_network(&std::net::IpAddr::V4(*ipv4_addr))
1076#qWZHcZ|        {
1077#jQL-Kg|            for exit_node in &self.exit_nodes {
1078#yDUUhE|                let IpAddr::V4(exit_node) = exit_node else {
1079#_01Y-5|                    continue;
1080#JBqzQb|                };
1081#t182g9|                if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(exit_node).await {

...35440 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3d5289001rX7YXsvDqhcX19
Use Grep to search the full content or Read with offset/limit to view specific sections.
--- grep({"pattern":"fn run_clean_peer_without_conn_routine","path":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs"}) [completed]
Found 1 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs:
  Line 1235:     async fn run_clean_peer_without_conn_routine(&self) {


## [assistant] +058391-05-11T15:09:20.000Z
[thinking] Now I have a much clearer picture. Let me look at the `run_clean_peer_without_conn_routine` and the `run` method to understand the periodic cleanup:
--- read_with_hash({"offset":1235,"filePath":"/mnt/nvme2t_2/easytier/easytier/src/peers/peer_manager.rs","limit":60}) [completed]
1#JkvAPI|use std::{
2#n-mlx5|    fmt::Debug,
3#0miUK5|    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
4#2oOvB1|    sync::{atomic::AtomicBool, Arc, Weak},
5#nkq1zO|    time::{Instant, SystemTime},
6#3zmC_W|};
7#47DEQp|
8#woeOSu|use anyhow::Context;
9#7mQoz2|use async_trait::async_trait;
10#47DEQp|
11#pF_Tl6|use dashmap::DashMap;
12#47DEQp|
13#McK12P|use tokio::{
14#6SaBei|    sync::{
15#yXmr9W|        mpsc::{self, UnboundedReceiver, UnboundedSender},
16#keoZXw|        Mutex, RwLock,
17#WGNt2R|    },
18#8wmH-9|    task::JoinSet,
19#3zmC_W|};
20#47DEQp|
21#8h8Z46|use crate::{
22#hIkMG-|    common::{
23#7LA2Ap|        compressor::{Compressor as _, DefaultCompressor},
24#yGOl1X|        constants::EASYTIER_VERSION,
25#Qez2zE|        error::Error,
26#IOwpvn|        global_ctx::{ArcGlobalCtx, NetworkIdentity},
27#-CQTtT|        stats_manager::{CounterHandle, LabelSet, LabelType, MetricName},
28#TN8nTI|        stun::StunInfoCollectorTrait,
29#yr1lub|        PeerId,
30#WGNt2R|    },
31#bBQPjc|    peers::{
32#r7_WyB|        peer_conn::PeerConn,
33#ag2bHw|        peer_rpc::PeerRpcManagerTransport,
34#Fkytt7|        recv_packet_from_chan,
35#wGxRrF|        route_trait::{ForeignNetworkRouteInfoMap, MockRoute, NextHopPolicy, RouteInterface},
36#DlpQe-|        PeerPacketFilter,
37#WGNt2R|    },
38#Js7gIl|    proto::{
39#2Obpu4|        cli::{
40#jjjzE7|            self, list_global_foreign_network_response::OneForeignNetwork,
41#XNpGX2|            ListGlobalForeignNetworkResponse,
42#Vev0I6|        },
43#GHt5hR|        peer_rpc::{
44#LNbBxd|            ForeignNetworkRouteInfoEntry, ForeignNetworkRouteInfoKey, RouteForeignNetworkSummary,
45#Vev0I6|        },
46#WGNt2R|    },
47#5fAXpE|    tunnel::{
48#tnN8Km|        self,
49#Ce4DhK|        packet_def::{CompressorAlgo, PacketType, ZCPacket},
50#BHnU6g|        Tunnel, TunnelConnector,
51#WGNt2R|    },
52#3zmC_W|};
53#47DEQp|
54#Z2Y0cv|use super::{
55#xTI2Qn|    create_packet_recv_chan,
56#TV_3YQ|    encrypt::{Encryptor, NullCipher},
57#nXLxyb|    foreign_network_client::ForeignNetworkClient,
58#oGbRre|    foreign_network_manager::{ForeignNetworkManager, GlobalForeignNetworkAccessor},
59#rXyyCw|    peer_conn::PeerConnId,
60#mZdjXu|    peer_map::PeerMap,
61#XRTSKE|    peer_ospf_route::PeerRoute,
62#s2iwev|    peer_rpc::PeerRpcManager,
63#LWqQ_9|    route_trait::{ArcRoute, Route},
64#_s6yp8|    BoxNicPacketFilter, BoxPeerPacketFilter, PacketRecvChan, PacketRecvChanReceiver,
65#3zmC_W|};
66#47DEQp|
67#XQRd2S|struct RpcTransport {
68#ug-G5V|    my_peer_id: PeerId,
69#3hRIgc|    peers: Weak<PeerMap>,
70#4FpEM7|    // TODO: this seems can be removed
71#IiaVCa|    foreign_peers: Mutex<Option<Weak<ForeignNetworkClient>>>,
72#47DEQp|
73#Ta7SHq|    packet_recv: Mutex<UnboundedReceiver<ZCPacket>>,
74#JTTivI|    peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
75#47DEQp|
76#HzqH-Y|    encryptor: Arc<dyn Encryptor>,
77#0Qs2qn|}
78#47DEQp|
79#31qlYx|#[async_trait::async_trait]
80#bWsjuU|impl PeerRpcManagerTransport for RpcTransport {
81#2-s-Hc|    fn my_peer_id(&self) -> PeerId {
82#t_Rp3F|        self.my_peer_id
83#KNhneG|    }
84#47DEQp|
85#iLULEg|    async fn send(&self, mut msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
86#5WJPcd|        let peers = self.peers.upgrade().ok_or(Error::Unknown)?;
87#B8dSEy|        // NOTE: if route info is not exchanged, this will return None. treat it as public server.
88#vGxGid|        let is_dst_peer_public_server = peers
89#cg69RI|            .get_route_peer_info(dst_peer_id)
90#icqUnQ|            .await
91#5_bb6B|            .and_then(|x| x.feature_flag.map(|x| x.is_public_server))
92#7cxG-C|            // if dst is directly connected, it's must not public server
93#7Oh1uK|            .unwrap_or(!peers.has_peer(dst_peer_id));
94#PqGi0l|        if !is_dst_peer_public_server {
95#r_wdly|            self.encryptor
96#nsRc7J|                .encrypt(&mut msg)
97#lPh9vD|                .with_context(|| "encrypt failed")?;
98#KkQWgx|        }
99#EV3zNG|        // send to self and this packet will be forwarded in peer_recv loop
100#FaOb0U|        peers.send_msg_directly(msg, self.my_peer_id).await
101#KNhneG|    }
102#47DEQp|
103#wYJVUI|    async fn recv(&self) -> Result<ZCPacket, Error> {
104#EhjLK5|        if let Some(o) = self.packet_recv.lock().await.recv().await {
105#2qgdDw|            Ok(o)
106#d4ha1c|        } else {
107#7mEL_z|            Err(Error::Unknown)
108#KkQWgx|        }
109#KNhneG|    }
110#0Qs2qn|}
111#47DEQp|
112#e7IyZm|pub enum RouteAlgoType {
113#Ai27Qg|    Ospf,
114#gW4I1O|    None,
115#0Qs2qn|}
116#47DEQp|
117#Vim1EE|enum RouteAlgoInst {
118#cMn152|    Ospf(Arc<PeerRoute>),
119#gW4I1O|    None,
120#0Qs2qn|}
121#47DEQp|
122#RYRGuJ|struct SelfTxCounters {
123#D-V-tR|    self_tx_packets: CounterHandle,
124#6V0uMP|    self_tx_bytes: CounterHandle,
125#zGXsep|    compress_tx_bytes_before: CounterHandle,
126#DukShZ|    compress_tx_bytes_after: CounterHandle,
127#0Qs2qn|}
128#47DEQp|
129#A-4Ewh|pub struct PeerManager {
130#ug-G5V|    my_peer_id: PeerId,
131#47DEQp|
132#sfM3Lj|    global_ctx: ArcGlobalCtx,
133#kfhNSA|    nic_channel: PacketRecvChan,
134#47DEQp|
135#FwTrUD|    tasks: Mutex<JoinSet<()>>,
136#47DEQp|
137#_j4kdf|    packet_recv: Arc<Mutex<Option<PacketRecvChanReceiver>>>,
138#47DEQp|
139#tGvkRl|    peers: Arc<PeerMap>,
140#47DEQp|
141#8VMx0Q|    peer_rpc_mgr: Arc<PeerRpcManager>,
142#FeSjdm|    peer_rpc_tspt: Arc<RpcTransport>,
143#47DEQp|
144#YPcpo0|    peer_packet_process_pipeline: Arc<RwLock<Vec<BoxPeerPacketFilter>>>,
145#bNnuK9|    nic_packet_process_pipeline: Arc<RwLock<Vec<BoxNicPacketFilter>>>,
146#47DEQp|
147#65LL3T|    route_algo_inst: RouteAlgoInst,
148#47DEQp|
149#KBjRVM|    foreign_network_manager: Arc<ForeignNetworkManager>,
150#HP62aE|    foreign_network_client: Arc<ForeignNetworkClient>,
151#47DEQp|
152#JTtOhW|    encryptor: Arc<dyn Encryptor + 'static>,
153#ajfTlp|    data_compress_algo: CompressorAlgo,
154#47DEQp|
155#MzW6ir|    exit_nodes: Vec<IpAddr>,
156#47DEQp|
157#-OFllv|    reserved_my_peer_id_map: DashMap<String, PeerId>,
158#47DEQp|
159#yPvr-Q|    allow_loopback_tunnel: AtomicBool,
160#47DEQp|
161#EB2QlC|    self_tx_counters: SelfTxCounters,
162#0Qs2qn|}
163#47DEQp|
164#VEJ-Nv|impl Debug for PeerManager {
165#GxWs9C|    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
166#GfxoC0|        f.debug_struct("PeerManager")
167#yUbTEK|            .field("my_peer_id", &self.my_peer_id())
168#pZnIWM|            .field("instance_name", &self.global_ctx.inst_name)
169#jwchZi|            .field("net_ns", &self.global_ctx.net_ns.name())
170#uPgCZz|            .finish()
171#KNhneG|    }
172#0Qs2qn|}
173#47DEQp|
174#WaSJgC|impl PeerManager {
175#4SQmJP|    pub fn new(
176#84qYcS|        route_algo: RouteAlgoType,
177#V5_-NZ|        global_ctx: ArcGlobalCtx,
178#itxN3d|        nic_channel: PacketRecvChan,
179#P1yxtn|    ) -> Self {
180#Ei27Hm|        let my_peer_id = rand::random();
181#47DEQp|
182#bF21tz|        let (packet_send, packet_recv) = create_packet_recv_chan();
183#-k9OTl|        let peers = Arc::new(PeerMap::new(
184#TcJi4O|            packet_send.clone(),
185#2rw4tz|            global_ctx.clone(),
186#907ngS|            my_peer_id,
187#XNrhe2|        ));
188#47DEQp|
189#BCJiZI|        let encryptor = if global_ctx.get_flags().enable_encryption {
190#TrvYkF|            // 只有在启用加密时才使用工厂函数选择算法
191#kL1789|            let algorithm = &global_ctx.get_flags().encryption_algorithm;
192#dtVNcK|            super::encrypt::create_encryptor(
193#LTVAb5|                algorithm,
194#xEkbYr|                global_ctx.get_128_key(),
195#tezR8V|                global_ctx.get_256_key(),
196#jOhom-|            )
197#d4ha1c|        } else {
198#DlGJ0c|            // disable_encryption = true 时使用 NullCipher
199#P1gqj_|            Arc::new(NullCipher)
200#8OSK9L|        };
201#47DEQp|
202#cnaP6d|        if global_ctx
203#gL1TGg|            .check_network_in_whitelist(&global_ctx.get_network_name())
204#UZAafT|            .is_err()
205#qWZHcZ|        {
206#9cFfax|            // if local network is not in whitelist, avoid relay data when exist any other route path
207#55FUIS|            let mut f = global_ctx.get_feature_flags();
208#7NEeJZ|            f.avoid_relay_data = true;
209#ghRxLc|            global_ctx.set_feature_flags(f);
210#KkQWgx|        }
211#47DEQp|
212#eC7ab7|        // TODO: remove these because we have impl pipeline processor.
213#4QNvTE|        let (peer_rpc_tspt_sender, peer_rpc_tspt_recv) = mpsc::unbounded_channel();
214#odkilt|        let rpc_tspt = Arc::new(RpcTransport {
215#907ngS|            my_peer_id,
216#irHp49|            peers: Arc::downgrade(&peers),
217#XFt7Gs|            foreign_peers: Mutex::new(None),
218#-q3GY9|            packet_recv: Mutex::new(peer_rpc_tspt_recv),
219#rnjVTo|            peer_rpc_tspt_sender,
220#QCQqiA|            encryptor: encryptor.clone(),
221#NQ-BLD|        });
222#dTWeBi|        let peer_rpc_mgr = Arc::new(PeerRpcManager::new_with_stats_manager(
223#cL_fh1|            rpc_tspt.clone(),
224#UXBsZE|            global_ctx.stats_manager().clone(),
225#XNrhe2|        ));
226#47DEQp|
227#YyP1mU|        let route_algo_inst = match route_algo {
228#Y9u96Z|            RouteAlgoType::Ospf => RouteAlgoInst::Ospf(PeerRoute::new(
229#hGlxsx|                my_peer_id,
230#_bhH_r|                global_ctx.clone(),
231#LQuUWT|                peer_rpc_mgr.clone(),
232#bt9-a0|            )),
233#DFHyU5|            RouteAlgoType::None => RouteAlgoInst::None,
234#8OSK9L|        };
235#47DEQp|
236#cHpfI6|        let foreign_network_manager = Arc::new(ForeignNetworkManager::new(
237#907ngS|            my_peer_id,
238#2rw4tz|            global_ctx.clone(),
239#TcJi4O|            packet_send.clone(),
240#6wEwdz|            Self::build_foreign_network_manager_accessor(&peers),
241#XNrhe2|        ));
242#IqUQio|        let foreign_network_client = Arc::new(ForeignNetworkClient::new(
243#2rw4tz|            global_ctx.clone(),
244#TcJi4O|            packet_send.clone(),
245#DdNEnS|            peer_rpc_mgr.clone(),
246#907ngS|            my_peer_id,
247#XNrhe2|        ));
248#47DEQp|
249#JOS7nS|        let data_compress_algo = global_ctx
250#w9YV8E|            .get_flags()
251#CMuCL3|            .data_compress_algo()
252#aJofO-|            .try_into()
253#_byGCt|            .expect("invalid data compress algo, maybe some features not enabled");
254#47DEQp|
255#dA-spH|        let exit_nodes = global_ctx.config.get_exit_nodes();
256#47DEQp|
257#dvFkte|        let stats_manager = global_ctx.stats_manager();
258#ZcR7Qb|        let self_tx_counters = SelfTxCounters {
259#UwSzpA|            self_tx_packets: stats_manager.get_counter(
260#I7V2Ec|                MetricName::TrafficPacketsSelfTx,
261#qRrZQ1|                LabelSet::new()
262#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
263#XsBKuC|            ),
264#HPPGMJ|            self_tx_bytes: stats_manager.get_counter(
265#zOOjtD|                MetricName::TrafficBytesSelfTx,
266#qRrZQ1|                LabelSet::new()
267#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
268#XsBKuC|            ),
269#bc-v55|            compress_tx_bytes_before: stats_manager.get_counter(
270#xCAqxm|                MetricName::CompressionBytesTxBefore,
271#qRrZQ1|                LabelSet::new()
272#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
273#XsBKuC|            ),
274#u85wqm|            compress_tx_bytes_after: stats_manager.get_counter(
275#Frn6d_|                MetricName::CompressionBytesTxAfter,
276#qRrZQ1|                LabelSet::new()
277#UqYpUH|                    .with_label_type(LabelType::NetworkName(global_ctx.get_network_name())),
278#XsBKuC|            ),
279#8OSK9L|        };
280#47DEQp|
281#eSauPW|        PeerManager {
282#907ngS|            my_peer_id,
283#47DEQp|
284#PvFYEj|            global_ctx,
285#3JnIN-|            nic_channel,
286#47DEQp|
287#V0fnw8|            tasks: Mutex::new(JoinSet::new()),
288#47DEQp|
289#chi1qM|            packet_recv: Arc::new(Mutex::new(Some(packet_recv))),
290#47DEQp|
291#N9VvX9|            peers: peers.clone(),
292#47DEQp|
293#V4bq0M|            peer_rpc_mgr,
294#AkAixq|            peer_rpc_tspt: rpc_tspt,
295#47DEQp|
296#RDG8zn|            peer_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
297#AVkW1r|            nic_packet_process_pipeline: Arc::new(RwLock::new(Vec::new())),
298#47DEQp|
299#vtQn2d|            route_algo_inst,
300#47DEQp|
301#Sj49Bi|            foreign_network_manager,
302#mDTqnb|            foreign_network_client,
303#47DEQp|
304#QC8bmd|            encryptor,
305#Pu-vJW|            data_compress_algo,
306#47DEQp|
307#4PpgxG|            exit_nodes,
308#47DEQp|
309#MFZYOb|            reserved_my_peer_id_map: DashMap::new(),
310#47DEQp|
311#jn9OJ3|            allow_loopback_tunnel: AtomicBool::new(true),
312#47DEQp|
313#qTdfR3|            self_tx_counters,
314#KkQWgx|        }
315#KNhneG|    }
316#47DEQp|
317#XCaz50|    pub fn set_allow_loopback_tunnel(&self, allow_loopback_tunnel: bool) {
318#9uXnfd|        self.allow_loopback_tunnel
319#5i63Ur|            .store(allow_loopback_tunnel, std::sync::atomic::Ordering::Relaxed);
320#KNhneG|    }
321#47DEQp|
322#bSQhio|    fn build_foreign_network_manager_accessor(
323#dEPfKJ|        peer_map: &Arc<PeerMap>,
324#f_qBdl|    ) -> Box<dyn GlobalForeignNetworkAccessor> {
325#O4kh_j|        struct T {
326#mqjLcH|            peer_map: Weak<PeerMap>,
327#KkQWgx|        }
328#47DEQp|
329#kD-9ea|        #[async_trait::async_trait]
330#cwfbLX|        impl GlobalForeignNetworkAccessor for T {
331#-FwRBf|            async fn list_global_foreign_peer(
332#Xibb9x|                &self,
333#HOFVLU|                network_identity: &NetworkIdentity,
334#b6Lxu3|            ) -> Vec<PeerId> {
335#1C2Yt4|                let Some(peer_map) = self.peer_map.upgrade() else {
336#W_IXmF|                    return vec![];
337#JBqzQb|                };
338#47DEQp|
339#aWxCLN|                peer_map
340#lEVlUJ|                    .list_peers_own_foreign_network(network_identity)
341#lYNw0O|                    .await
342#hq_fcK|            }
343#KkQWgx|        }
344#47DEQp|
345#D2hsWL|        Box::new(T {
346#jYzaNh|            peer_map: Arc::downgrade(peer_map),
347#9I500R|        })
348#KNhneG|    }
349#47DEQp|
350#2lSRhY|    async fn add_new_peer_conn(&self, peer_conn: PeerConn) -> Result<(), Error> {
351#mxf9Cw|        if !self.global_ctx.is_valid_network_identity(&peer_conn.get_network_identity()) {
352#DvThUD|            return Err(Error::SecretKeyError(
353#cEWqcp|                "network identity not match".to_string(),
354#1oJHlw|            ));
355#KkQWgx|        }
356#ofM7n0|        let peer_id = peer_conn.get_peer_id();
357#Rar11V|        if self.global_ctx.is_peer_blocked(peer_id) {
358#DvThUD|            return Err(Error::SecretKeyError(
359#YTPquW|                "peer is blocked by IP whitelist".to_string(),
360#1oJHlw|            ));
361#KkQWgx|        }
362#aoqkXN|        self.peers.add_new_peer_conn(peer_conn).await;
363#sX41Gp|        Ok(())
364#KNhneG|    }
365#47DEQp|
366#14lYQ5|    pub async fn add_client_tunnel(
367#7BRTWY|        &self,
368#TMFJR1|        tunnel: Box<dyn Tunnel>,
369#GkW2PL|        is_directly_connected: bool,
370#QfKvkA|    ) -> Result<(PeerId, PeerConnId), Error> {
371#c0IlOn|        let mut peer = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
372#_IfWok|        peer.set_is_hole_punched(!is_directly_connected);
373#5yUaks|        peer.do_handshake_as_client().await?;
374#7JL2LP|        let conn_id = peer.get_conn_id();
375#RU1WKB|        let peer_id = peer.get_peer_id();
376#kTrXVm|        if peer.get_network_identity().network_name
377#ZrhvYf|            == self.global_ctx.get_network_identity().network_name
378#qWZHcZ|        {
379#f-Qeac|            self.add_new_peer_conn(peer).await?;
380#d4ha1c|        } else {
381#A7ODMm|            self.foreign_network_client.add_new_peer_conn(peer).await;
382#KkQWgx|        }
383#EPLuW8|        Ok((peer_id, conn_id))
384#KNhneG|    }
385#47DEQp|
386#jJc1fo|    pub fn has_directly_connected_conn(&self, peer_id: PeerId) -> bool {
387#3bpcwG|        if let Some(peer) = self.peers.get_peer_by_id(peer_id) {
388#Zq_A7q|            peer.has_directly_connected_conn()
389#d4ha1c|        } else {
390#kjtVBH|            self.foreign_network_client.get_peer_map().has_peer(peer_id)
391#KkQWgx|        }
392#KNhneG|    }
393#47DEQp|
394#pwuqRZ|    #[tracing::instrument]
395#o0DshP|    pub async fn try_direct_connect<C>(
396#7BRTWY|        &self,
397#i1Ryys|        mut connector: C,
398#EvgalJ|    ) -> Result<(PeerId, PeerConnId), Error>
399#JSvlCU|    where
400#rdvGRT|        C: TunnelConnector + Debug,
401#DT0vWo|    {
402#FQRxlN|        let ns = self.global_ctx.net_ns.clone();
403#09Wskl|        let t = ns
404#s0FDIJ|            .run_async(|| async move { connector.connect().await })
405#hmaiLi|            .await?;
406#9oIpNF|        self.add_client_tunnel(t, true).await
407#KNhneG|    }
408#47DEQp|
409#k9IM0H|    // avoid loop back to virtual network
410#4-_0cQ|    fn check_remote_addr_not_from_virtual_network(
411#7BRTWY|        &self,
412#6OnkCQ|        tunnel: &dyn Tunnel,
413#UnHDae|    ) -> Result<(), anyhow::Error> {
414#sd1I2C|        tracing::info!("check remote addr not from virtual network");
415#ClcXQQ|        let Some(tunnel_info) = tunnel.info() else {
416#PdNkBX|            anyhow::bail!("tunnel info is not set");
417#8OSK9L|        };
418#W0tgbh|        let Some(src) = tunnel_info.remote_addr.map(url::Url::from) else {
419#ol3jOX|            anyhow::bail!("tunnel info remote addr is not set");
420#8OSK9L|        };
421#u-6BoE|        if src.scheme() == "ring" {
422#6WCzfY|            return Ok(());
423#KkQWgx|        }
424#u9Qbc0|        let src_host = match src.socket_addrs(|| Some(1)) {
425#elIV8e|            Ok(addrs) => addrs,
426#gJq-2M|            Err(_) => {
427#PnbuE3|                // if the tunnel is not rely on ip address, skip check
428#LEBeyD|                return Ok(());
429#hq_fcK|            }
430#8OSK9L|        };
431#YrPT6Z|        let virtual_ipv4 = self.global_ctx.get_ipv4().map(|ip| ip.network());
432#MnnRwK|        let virtual_ipv6 = self.global_ctx.get_ipv6().map(|ip| ip.network());
433#3_Ehij|        tracing::info!(
434#5owbgP|            ?virtual_ipv4,
435#P0zdad|            ?virtual_ipv6,
436#jKufAz|            "check remote addr not from virtual network"
437#bq_4ei|        );
438#H79IuK|        for addr in src_host {
439#5YpsUk|            // if no-tun is enabled, the src ip of packet in virtual network is converted to loopback address
440#uqyRvu|            if addr.ip().is_loopback()
441#aSe1bi|                && !self
442#GwaNY6|                    .allow_loopback_tunnel
443#wg0XBg|                    .load(std::sync::atomic::Ordering::Relaxed)
444#eP03rc|            {
445#-2-r1E|                anyhow::bail!("tunnel src host is loopback address");
446#hq_fcK|            }
447#47DEQp|
448#Vs2UUY|            match addr {
449#Fkp5F3|                SocketAddr::V4(addr) => {
450#u2nxdD|                    if let Some(virtual_ipv4) = virtual_ipv4 {
451#JYPb6X|                        if virtual_ipv4.contains(addr.ip()) {
452#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
453#qrgf36|                        }
454#uNGRgX|                    }
455#C_CgiK|                }
456#Yj08jy|                SocketAddr::V6(addr) => {
457#qo9J_v|                    if let Some(virtual_ipv6) = virtual_ipv6 {
458#U0F33o|                        if virtual_ipv6.contains(addr.ip()) {
459#DGEFMM|                            anyhow::bail!("tunnel src host is from the virtual network (ignore this error please)");
460#qrgf36|                        }
461#uNGRgX|                    }
462#C_CgiK|                }
463#hq_fcK|            }
464#KkQWgx|        }
465#sX41Gp|        Ok(())
466#KNhneG|    }
467#47DEQp|
468#U7Knbg|    #[tracing::instrument(ret)]
469#pg0BwE|    pub async fn add_tunnel_as_server(
470#7BRTWY|        &self,
471#TMFJR1|        tunnel: Box<dyn Tunnel>,
472#GkW2PL|        is_directly_connected: bool,
473#Ew16UP|    ) -> Result<(), Error> {
474#c2-UuG|        tracing::info!("add tunnel as server start");
475#6b66k9|        self.check_remote_addr_not_from_virtual_network(&tunnel)?;
476#47DEQp|
477#c3DuKs|        let mut conn = PeerConn::new(self.my_peer_id, self.global_ctx.clone(), tunnel);
478#1CaMg8|        conn.do_handshake_as_server_ext(|peer, msg| {
479#2MbFGD|            if msg.network_name
480#z_HzRK|                == self.global_ctx.get_network_identity().network_name
481#eP03rc|            {
482#LEBeyD|                return Ok(());
483#hq_fcK|            }
484#47DEQp|
485#HMq8Dh|            if self.global_ctx.config.get_flags().private_mode {
486#Pvb-m7|                return Err(Error::SecretKeyError(
487#rq_jVx|                    "private mode is turned on, network identity not match".to_string(),
488#uRtctN|                ));
489#hq_fcK|            }
490#47DEQp|
491#Wvjjso|            let mut peer_id = self
492#WhSa3T|                .foreign_network_manager
493#ujWxtI|                .get_network_peer_id(&msg.network_name);
494#r-UKkg|            if peer_id.is_none() {
495#f8G6ks|                peer_id = Some(*self.reserved_my_peer_id_map.entry(msg.network_name.clone()).or_insert_with(|| {
496#cUHymN|                    rand::random::<PeerId>()
497#JxOJH_|                }).value());
498#hq_fcK|            }
499#OyZvBS|            peer.set_peer_id(peer_id.unwrap());
500#47DEQp|
501#8JJIOp|            tracing::info!(
502#QAVlTO|                ?peer_id,
503#XOCU_s|                ?msg.network_name,
504#YBX1Va|                "handshake as server with foreign network, new peer id: {}, peer id in foreign manager: {:?}",
505#jsth7q|                peer.get_my_peer_id(), peer_id
506#TP8Q3D|            );
507#47DEQp|
508#Y1ZmOf|            Ok(())
509#9I500R|        })
510#GGG_xT|        .await?;
511#47DEQp|
512#xVC9Et|        let peer_network_name = conn.get_network_identity().network_name.clone();
513#47DEQp|
514#Srsdme|        conn.set_is_hole_punched(!is_directly_connected);
515#47DEQp|
516#ia8Sn0|        if peer_network_name == self.global_ctx.get_network_identity().network_name {
517#32dI9Z|            self.add_new_peer_conn(conn).await?;
518#d4ha1c|        } else {
519#MIGy32|            self.foreign_network_manager.add_peer_conn(conn).await?;
520#KkQWgx|        }
521#47DEQp|
522#eakY8y|        self.reserved_my_peer_id_map.remove(&peer_network_name);
523#47DEQp|
524#093W8_|        tracing::info!("add tunnel as server done");
525#sX41Gp|        Ok(())
526#KNhneG|    }
527#47DEQp|
528#yBJw-S|    async fn try_handle_foreign_network_packet(
529#sCE1BR|        mut packet: ZCPacket,
530#kIT2Ch|        my_peer_id: PeerId,
531#huG0PK|        peer_map: &PeerMap,
532#ZvH2o7|        foreign_network_mgr: &ForeignNetworkManager,
533#u88bzm|    ) -> Result<(), ZCPacket> {
534#bhZeMH|        let pm_header = packet.peer_manager_header().unwrap();
535#fDsfht|        if pm_header.packet_type != PacketType::ForeignNetworkPacket as u8 {
536#KmX_9i|            return Err(packet);
537#KkQWgx|        }
538#47DEQp|
539#flN_K6|        let from_peer_id = pm_header.from_peer_id.get();
540#FRT-lF|        let to_peer_id = pm_header.to_peer_id.get();
541#47DEQp|
542#y4SHAG|        let foreign_hdr = packet.foreign_network_hdr().unwrap();
543#aXbqEm|        let foreign_network_name = foreign_hdr.get_network_name(packet.payload());
544#8quVoo|        let foreign_peer_id = foreign_hdr.get_dst_peer_id();
545#47DEQp|
546#IxuZAU|        let foreign_network_my_peer_id =
547#QNcXt2|            foreign_network_mgr.get_network_peer_id(&foreign_network_name);
548#47DEQp|
549#6MHiqH|        let buf_len = packet.buf_len();
550#-vAIvE|        let stats_manager = peer_map.get_global_ctx().stats_manager().clone();
551#5HN3PJ|        let label_set =
552#8VYKuw|            LabelSet::new().with_label_type(LabelType::NetworkName(foreign_network_name.clone()));
553#GxRWuo|        let add_counter = move |bytes_metric, packets_metric| {
554#S_bDbN|            stats_manager
555#XTcotD|                .get_counter(bytes_metric, label_set.clone())
556#nLeZsi|                .add(buf_len as u64);
557#9fhBOM|            stats_manager.get_counter(packets_metric, label_set).inc();
558#8OSK9L|        };
559#47DEQp|
560#YbVzBx|        // NOTICE: the to peer id is modified by the src from foreign network my peer id to the origin my peer id
561#W0MF1w|        if to_peer_id == my_peer_id {
562#viIycQ|            // packet sent from other peer to me, extract the inner packet and forward it
563#YUb47C|            add_counter(
564#mPzK_l|                MetricName::TrafficBytesForeignForwardRx,
565#h0W-Nu|                MetricName::TrafficPacketsForeignForwardRx,
566#TP8Q3D|            );
567#fdRDE9|            if let Err(e) = foreign_network_mgr
568#z4HPvh|                .send_msg_to_peer(
569#KnDqPt|                    &foreign_network_name,
570#5wz9iB|                    foreign_peer_id,
571#HFocUC|                    packet.foreign_network_packet(),
572#cN3xS-|                )
573#PcpKA_|                .await
574#eP03rc|            {
575#CW4xPB|                tracing::debug!(
576#2EUTrQ|                    ?e,
577#EwHmim|                    ?foreign_network_name,
578#P4MAxf|                    ?foreign_peer_id,
579#H3knNr|                    "foreign network mgr send_msg_to_peer failed"
580#iyW0eL|                );
581#hq_fcK|            }
582#Y1ZmOf|            Ok(())
583#YIUK0N|        } else if Some(from_peer_id) == foreign_network_my_peer_id {
584#mwhHwz|            // to_peer_id is my peer id for the foreign network, need to convert to the origin my_peer_id of dst
585#1Cllfa|            let Some(to_peer_id) = peer_map
586#vOHCEe|                .get_origin_my_peer_id(&foreign_network_name, to_peer_id)
587#PcpKA_|                .await
588#HO4mJ1|            else {
589#CW4xPB|                tracing::debug!(
590#EwHmim|                    ?foreign_network_name,
591#64bVg4|                    ?to_peer_id,
592#Fa_5NR|                    "cannot find origin my peer id for foreign network."
593#iyW0eL|                );
594#1QMLIr|                return Err(packet);
595#huZslw|            };
596#47DEQp|
597#YUb47C|            add_counter(
598#isNHJA|                MetricName::TrafficBytesForeignForwardTx,
599#Yut8ka|                MetricName::TrafficPacketsForeignForwardTx,
600#TP8Q3D|            );
601#47DEQp|
602#qFCK98|            // modify the to_peer id from foreign network my peer id to the origin my peer id
603#1qxWD1|            packet
604#0lMrqe|                .mut_peer_manager_header()
605#gQk2jP|                .unwrap()
606#2oYuph|                .to_peer_id
607#1FcmIL|                .set(to_peer_id);
608#47DEQp|
609#WWRUyf|            // packet is generated from foreign network mgr and should be forward to other peer
610#GjxVjo|            if let Err(e) = peer_map
611#pWezG5|                .send_msg(packet, to_peer_id, NextHopPolicy::LeastHop)
612#PcpKA_|                .await
613#eP03rc|            {
614#CW4xPB|                tracing::debug!(
615#2EUTrQ|                    ?e,
616#64bVg4|                    ?to_peer_id,
617#0xHh2s|                    "send_msg_directly failed when forward local generated foreign network packet"
618#iyW0eL|                );
619#hq_fcK|            }
620#Y1ZmOf|            Ok(())
621#d4ha1c|        } else {
622#b_Rzck|            // target is not me, forward it. try get origin peer id
623#YUb47C|            add_counter(
624#iZmaoy|                MetricName::TrafficBytesForeignForwardForwarded,
625#1fxT_q|                MetricName::TrafficPacketsForeignForwardForwarded,
626#TP8Q3D|            );
627#BKXjm4|            Err(packet)
628#KkQWgx|        }
629#KNhneG|    }
630#47DEQp|
631#FHuK_1|    async fn start_peer_recv(&self) {
632#37sTF-|        let mut recv = self.packet_recv.lock().await.take().unwrap();
633#vnR7aH|        let my_peer_id = self.my_peer_id;
634#nmPmgi|        let peers = self.peers.clone();
635#97eb5s|        let pipe_line = self.peer_packet_process_pipeline.clone();
636#8aTJMr|        let foreign_client = self.foreign_network_client.clone();
637#C546Zb|        let foreign_mgr = self.foreign_network_manager.clone();
638#k_2-Ol|        let encryptor = self.encryptor.clone();
639#mPlkH9|        let compress_algo = self.data_compress_algo;
640#GRlbc0|        let acl_filter = self.global_ctx.get_acl_filter().clone();
641#ZGLiLv|        let global_ctx = self.global_ctx.clone();
642#UZrWMo|        let stats_mgr = self.global_ctx.stats_manager().clone();
643#9sGRWS|        let route = self.get_route();
644#47DEQp|
645#5HN3PJ|        let label_set =
646#aDVoKb|            LabelSet::new().with_label_type(LabelType::NetworkName(global_ctx.get_network_name()));
647#47DEQp|
648#mEGOr9|        let self_tx_bytes = self.self_tx_counters.self_tx_bytes.clone();
649#T7N9pS|        let self_tx_packets = self.self_tx_counters.self_tx_packets.clone();
650#hCUNVJ|        let self_rx_bytes =
651#80PMSq|            stats_mgr.get_counter(MetricName::TrafficBytesSelfRx, label_set.clone());
652#DmhFt-|        let self_rx_packets =
653#DmhvAH|            stats_mgr.get_counter(MetricName::TrafficPacketsSelfRx, label_set.clone());
654#FP31iZ|        let forward_tx_bytes =
655#mIb08E|            stats_mgr.get_counter(MetricName::TrafficBytesForwarded, label_set.clone());
656#N6gzzb|        let forward_tx_packets =
657#nCT_oP|            stats_mgr.get_counter(MetricName::TrafficPacketsForwarded, label_set.clone());
658#47DEQp|
659#9nvSTI|        let compress_tx_bytes_before = self.self_tx_counters.compress_tx_bytes_before.clone();
660#8Ehyj-|        let compress_tx_bytes_after = self.self_tx_counters.compress_tx_bytes_after.clone();
661#dMW85W|        let compress_rx_bytes_before =
662#Jryksd|            stats_mgr.get_counter(MetricName::CompressionBytesRxBefore, label_set.clone());
663#CJs0ZN|        let compress_rx_bytes_after =
664#OkiaLh|            stats_mgr.get_counter(MetricName::CompressionBytesRxAfter, label_set.clone());
665#47DEQp|
666#DDENzd|        self.tasks.lock().await.spawn(async move {
667#s1vPO9|            tracing::trace!("start_peer_recv");
668#z3Goih|            while let Ok(ret) = recv_packet_from_chan(&mut recv).await {
669#w5bSOZ|                let Err(mut ret) =
670#E9vP_K|                    Self::try_handle_foreign_network_packet(ret, my_peer_id, &peers, &foreign_mgr)
671#o6c0n-|                        .await
672#QIIDCb|                else {
673#_01Y-5|                    continue;
674#JBqzQb|                };
675#47DEQp|
676#z4xbBM|                let buf_len = ret.buf_len();
677#sLsb9g|                let Some(hdr) = ret.mut_peer_manager_header() else {
678#eP2t8R|                    tracing::warn!(?ret, "invalid packet, skip");
679#_01Y-5|                    continue;
680#JBqzQb|                };
681#47DEQp|
682#ax0YAR|                tracing::trace!(?hdr, "peer recv a packet...");
683#G1tVuV|                let from_peer_id = hdr.from_peer_id.get();
684#i2EURd|                let to_peer_id = hdr.to_peer_id.get();
685#Cuycos|                if to_peer_id != my_peer_id {
686#JRJHei|                    if hdr.forward_counter > 7 {
687#F8mduZ|                        tracing::warn!(?hdr, "forward counter exceed, drop packet");
688#W98xla|                        continue;
689#uNGRgX|                    }
690#47DEQp|
691#pwyl0-|                    if hdr.forward_counter > 2 && hdr.is_latency_first() {
692#uVS30W|                        tracing::trace!(?hdr, "set_latency_first false because too many hop");
693#9ndRqd|                        hdr.set_latency_first(false);
694#uNGRgX|                    }
695#47DEQp|
696#VdVaow|                    hdr.forward_counter += 1;
697#47DEQp|
698#ZT0lS_|                    if from_peer_id == my_peer_id {
699#uroO3I|                        compress_tx_bytes_before.add(buf_len as u64);
700#47DEQp|
701#iGD8zw|                        if hdr.packet_type == PacketType::Data as u8
702#GABpVR|                            || hdr.packet_type == PacketType::KcpSrc as u8
703#Z6rbMg|                            || hdr.packet_type == PacketType::KcpDst as u8
704#Fzqoyl|                        {
705#1LI0Vo|                            let _ =
706#JUJFc0|                                Self::try_compress_and_encrypt(compress_algo, &encryptor, &mut ret)
707#1yUlMs|                                    .await;
708#qrgf36|                        }
709#47DEQp|
710#T0tOoJ|                        compress_tx_bytes_after.add(ret.buf_len() as u64);
711#tANDt7|                        self_tx_bytes.add(ret.buf_len() as u64);
712#f8O7M7|                        self_tx_packets.inc();
713#HyGTCY|                    } else {
714#Pguetm|                        forward_tx_bytes.add(buf_len as u64);
715#gmyQh8|                        forward_tx_packets.inc();
716#uNGRgX|                    }
717#47DEQp|
718#kSLxh3|                    tracing::trace!(?to_peer_id, ?my_peer_id, "need forward");
719#b8Js0t|                    let ret =
720#EZHc3h|                        Self::send_msg_internal(&peers, &foreign_client, ret, to_peer_id).await;
721#LXf7AG|                    if ret.is_err() {
722#RK5a5R|                        tracing::error!(?ret, ?to_peer_id, ?from_peer_id, "forward packet error");
723#uNGRgX|                    }
724#ZtcDW9|                } else {
725#fn_fyy|                    if let Err(e) = encryptor.decrypt(&mut ret) {
726#HZCGH0|                        tracing::error!(?e, "decrypt failed");
727#W98xla|                        continue;
728#uNGRgX|                    }
729#47DEQp|
730#Ln7UBJ|                    self_rx_bytes.add(buf_len as u64);
731#XNjtrJ|                    self_rx_packets.inc();
732#BHaA-C|                    compress_rx_bytes_before.add(buf_len as u64);
733#47DEQp|
734#QblX98|                    let compressor = DefaultCompressor {};
735#GMZW0O|                    if let Err(e) = compressor.decompress(&mut ret).await {
736#gbQ6SV|                        tracing::error!(?e, "decompress failed");
737#W98xla|                        continue;
738#uNGRgX|                    }
739#47DEQp|
740#Hjej0y|                    compress_rx_bytes_after.add(ret.buf_len() as u64);
741#47DEQp|
742#2dQ5pH|                    if !acl_filter.process_packet_with_acl(
743#NQJi05|                        &ret,
744#VaXc6J|                        true,
745#NFzhEu|                        global_ctx.get_ipv4().map(|x| x.address()),
746#NvLih8|                        global_ctx.get_ipv6().map(|x| x.address()),
747#5sWIyd|                        &route,
748#SwK0Oa|                    ) {
749#W98xla|                        continue;
750#uNGRgX|                    }
751#47DEQp|
752#cqHypd|                    let mut processed = false;
753#fYLeTn|                    let mut zc_packet = Some(ret);
754#IhX6Kq|                    for (idx, pipeline) in pipe_line.read().await.iter().rev().enumerate() {
755#r-rtuq|                        tracing::trace!(?zc_packet, ?idx, "try_process_packet_from_peer");
756#0x6iPo|                        zc_packet = pipeline
757#D_qrXs|                            .try_process_packet_from_peer(zc_packet.unwrap())
758#IY999y|                            .await;
759#Ew5lXA|                        if zc_packet.is_none() {
760#10AbF1|                            processed = true;
761#Ko0WZT|                            break;
762#qrgf36|                        }
763#uNGRgX|                    }
764#R2opY7|                    if !processed {
765#BN52j-|                        tracing::error!(?zc_packet, "unhandled packet");
766#uNGRgX|                    }
767#C_CgiK|                }
768#hq_fcK|            }
769#H99HIN|            panic!("done_peer_recv");
770#NQ-BLD|        });
771#KNhneG|    }
772#47DEQp|
773#q_JTCC|    pub async fn add_packet_process_pipeline(&self, pipeline: BoxPeerPacketFilter) {
774#5xAyFb|        // newest pipeline will be executed first
775#9k-qPG|        self.peer_packet_process_pipeline
776#jtBdos|            .write()
777#icqUnQ|            .await
778#gbkGf8|            .push(pipeline);
779#KNhneG|    }
780#47DEQp|
781#ekn-0q|    pub async fn add_nic_packet_process_pipeline(&self, pipeline: BoxNicPacketFilter) {
782#5xAyFb|        // newest pipeline will be executed first
783#lgMoPV|        self.nic_packet_process_pipeline
784#jtBdos|            .write()
785#icqUnQ|            .await
786#gbkGf8|            .push(pipeline);
787#KNhneG|    }
788#47DEQp|
789#rwu4HQ|    async fn init_packet_process_pipeline(&self) {
790#LJfv5W|        // for tun/tap ip/eth packet.
791#hwmngc|        struct NicPacketProcessor {
792#My7Qci|            nic_channel: PacketRecvChan,
793#KkQWgx|        }
794#kD-9ea|        #[async_trait::async_trait]
795#OtOTlR|        impl PeerPacketFilter for NicPacketProcessor {
796#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
797#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
798#L2pk7f|                if hdr.packet_type == PacketType::Data as u8 {
799#dx_3aF|                    tracing::trace!(?packet, "send packet to nic channel");
800#9RlVs2|                    // TODO: use a function to get the body ref directly for zero copy
801#D1yJAP|                    let _ = self.nic_channel.send(packet).await;
802#JqjnWy|                    None
803#ZtcDW9|                } else {
804#jLWmFO|                    Some(packet)
805#C_CgiK|                }
806#hq_fcK|            }
807#KkQWgx|        }
808#zvD7Bg|        self.add_packet_process_pipeline(Box::new(NicPacketProcessor {
809#PTKjfM|            nic_channel: self.nic_channel.clone(),
810#AVIbAB|        }))
811#10YU6B|        .await;
812#47DEQp|
813#s7MHfd|        // for peer rpc packet
814#N1Iaxj|        struct PeerRpcPacketProcessor {
815#Lx--BG|            peer_rpc_tspt_sender: UnboundedSender<ZCPacket>,
816#KkQWgx|        }
817#47DEQp|
818#kD-9ea|        #[async_trait::async_trait]
819#D1zWOn|        impl PeerPacketFilter for PeerRpcPacketProcessor {
820#tjzQdi|            async fn try_process_packet_from_peer(&self, packet: ZCPacket) -> Option<ZCPacket> {
821#z--RkG|                let hdr = packet.peer_manager_header().unwrap();
822#CrfTXZ|                if hdr.packet_type == PacketType::TaRpc as u8
823#TBQp0z|                    || hdr.packet_type == PacketType::RpcReq as u8
824#6aiYqm|                    || hdr.packet_type == PacketType::RpcResp as u8
825#NV9RY5|                {
826#DWqudi|                    self.peer_rpc_tspt_sender.send(packet).unwrap();
827#JqjnWy|                    None
828#ZtcDW9|                } else {
829#jLWmFO|                    Some(packet)
830#C_CgiK|                }
831#hq_fcK|            }
832#KkQWgx|        }
833#4XfYK9|        self.add_packet_process_pipeline(Box::new(PeerRpcPacketProcessor {
834#Nc02qB|            peer_rpc_tspt_sender: self.peer_rpc_tspt.peer_rpc_tspt_sender.clone(),
835#AVIbAB|        }))
836#10YU6B|        .await;
837#KNhneG|    }
838#47DEQp|
839#qnv61H|    pub async fn add_route<T>(&self, route: T)
840#JSvlCU|    where
841#s0mP8g|        T: Route + PeerPacketFilter + Send + Sync + Clone + 'static,
842#DT0vWo|    {
843#YR42TL|        // for route
844#DOPJKV|        self.add_packet_process_pipeline(Box::new(route.clone()))
845#agFGm5|            .await;
846#47DEQp|
847#hY75HP|        struct Interface {
848#y1Tqlc|            my_peer_id: PeerId,
849#rrAPV3|            peers: Weak<PeerMap>,
850#21pgeS|            foreign_network_client: Weak<ForeignNetworkClient>,
851#8Gbfqs|            foreign_network_manager: Weak<ForeignNetworkManager>,
852#KkQWgx|        }
853#47DEQp|
854#KGY7Sc|        #[async_trait]
855#DkYunO|        impl RouteInterface for Interface {
856#l_VjGY|            async fn list_peers(&self) -> Vec<PeerId> {
857#XDZ2Vk|                let Some(foreign_client) = self.foreign_network_client.upgrade() else {
858#W_IXmF|                    return vec![];
859#JBqzQb|                };
860#47DEQp|
861#PUDAeF|                let Some(peer_map) = self.peers.upgrade() else {
862#W_IXmF|                    return vec![];
863#JBqzQb|                };
864#47DEQp|
865#XQDNH1|                let mut peers = foreign_client.list_public_peers().await;
866#njwKeW|                peers.extend(peer_map.list_peers_with_conn().await);
867#MTa5cE|                peers
868#hq_fcK|            }
869#47DEQp|
870#LFp0oo|            fn my_peer_id(&self) -> PeerId {
871#AE5UEx|                self.my_peer_id
872#hq_fcK|            }
873#47DEQp|
874#uonLKK|            async fn list_foreign_networks(&self) -> ForeignNetworkRouteInfoMap {
875#5THHmO|                let ret = DashMap::new();
876#swf1dg|                let Some(foreign_mgr) = self.foreign_network_manager.upgrade() else {
877#b5Svxi|                    return ret;
878#JBqzQb|                };
879#47DEQp|
880#Y8o-Yf|                let networks = foreign_mgr.list_foreign_networks().await;
881#ioetyj|                for (network_name, info) in networks.foreign_networks.iter() {
882#JV3_bG|                    if info.peers.is_empty() {
883#W98xla|                        continue;
884#uNGRgX|                    }
885#47DEQp|
886#LK7acJ|                    let last_update = foreign_mgr
887#JxC-fm|                        .get_foreign_network_last_update(network_name)
888#v0277T|                        .unwrap_or(SystemTime::now());
889#4cuRRV|                    ret.insert(
890#zuxFX7|                        ForeignNetworkRouteInfoKey {
891#FvjDof|                            peer_id: self.my_peer_id,
892#MBDbjS|                            network_name: network_name.clone(),
893#xt3MAa|                        },
894#WTdtwn|                        ForeignNetworkRouteInfoEntry {
895#nSGc8d|                            foreign_peer_ids: info.peers.iter().map(|x| x.peer_id).collect(),
896#DEQ9Fe|                            last_update: Some(last_update.into()),
897#QBzroT|                            version: 0,
898#rLkInO|                            network_secret_digest: info.network_secret_digest.clone(),
899#5efuR1|                            my_peer_id_for_this_network: info.my_peer_id_for_this_network,
900#xt3MAa|                        },
901#iTFhZ0|                    );
902#C_CgiK|                }
903#0qn6Vo|                ret
904#hq_fcK|            }
905#KkQWgx|        }
906#47DEQp|
907#vnR7aH|        let my_peer_id = self.my_peer_id;
908#T7G1Zk|        let _route_id = route
909#SfQrJo|            .open(Box::new(Interface {
910#hGlxsx|                my_peer_id,
911#0F1sFB|                peers: Arc::downgrade(&self.peers),
912#ER66ww|                foreign_network_client: Arc::downgrade(&self.foreign_network_client),
913#yRkAf3|                foreign_network_manager: Arc::downgrade(&self.foreign_network_manager),
914#LufvaI|            }))
915#icqUnQ|            .await
916#WEGyMY|            .unwrap();
917#47DEQp|
918#ORaZ3y|        let arc_route: ArcRoute = Arc::new(Box::new(route));
919#fY7N7T|        self.peers.add_route(arc_route).await;
920#KNhneG|    }
921#47DEQp|
922#rj8ACA|    pub fn get_route(&self) -> Box<dyn Route + Send + Sync + 'static> {
923#aWnGNt|        match &self.route_algo_inst {
924#ind_vT|            RouteAlgoInst::Ospf(route) => Box::new(route.clone()),
925#3ENyBM|            RouteAlgoInst::None => Box::new(MockRoute {}),
926#KkQWgx|        }
927#KNhneG|    }
928#47DEQp|
929#ggsQcx|    pub async fn list_routes(&self) -> Vec<cli::Route> {
930#buqXa2|        self.get_route().list_routes().await
931#KNhneG|    }
932#47DEQp|
933#SQwiYH|    pub async fn get_route_peer_info_last_update_time(&self) -> Instant {
934#o0Hn44|        self.get_route().get_peer_info_last_update_time().await
935#KNhneG|    }
936#47DEQp|
937#rMN99o|    pub async fn dump_route(&self) -> String {
938#NjIrQ5|        self.get_route().dump().await
939#KNhneG|    }
940#47DEQp|
941#qHMIGl|    pub async fn list_global_foreign_network(&self) -> ListGlobalForeignNetworkResponse {
942#dSN10n|        let mut resp = ListGlobalForeignNetworkResponse::default();
943#0GRtAb|        let ret = self.get_route().list_foreign_network_info().await;
944#PfLspy|        for info in ret.infos.iter() {
945#los8mn|            let entry = resp
946#cAKnz7|                .foreign_networks
947#x4ul2a|                .entry(info.key.as_ref().unwrap().peer_id)
948#ctxgtd|                .or_insert_with(Default::default);
949#MiV-NN|            let Some(route_info) = info.value.as_ref() else {
950#YqjFPd|                continue;
951#huZslw|            };
952#47DEQp|
953#ubAyqu|            let f = OneForeignNetwork {
954#1WNjy7|                network_name: info.key.as_ref().unwrap().network_name.clone(),
955#LogZYQ|                peer_ids: route_info.foreign_peer_ids.clone(),
956#RIHZBu|                last_updated: format!("{}", route_info.last_update.unwrap()),
957#hHIWcX|                version: route_info.version,
958#huZslw|            };
959#47DEQp|
960#TZwkyd|            entry.foreign_networks.push(f);
961#KkQWgx|        }
962#47DEQp|
963#ZudY7T|        resp
964#KNhneG|    }
965#47DEQp|
966#J0LLTy|    pub async fn get_foreign_network_summary(&self) -> RouteForeignNetworkSummary {
967#yFVVTB|        self.get_route().get_foreign_network_summary().await
968#KNhneG|    }
969#47DEQp|
970#WFyYU9|    async fn run_nic_packet_process_pipeline(&self, data: &mut ZCPacket) {
971#81LGkF|        if !self.global_ctx.get_acl_filter().process_packet_with_acl(
972#RSksDL|            data,
973#bBjPxJ|            false,
974#SKXxAq|            None,
975#SKXxAq|            None,
976#HUVukP|            &self.get_route(),
977#HceTsT|        ) {
978#NhdEjk|            return;
979#KkQWgx|        }
980#47DEQp|
981#AzFZTs|        for pipeline in self.nic_packet_process_pipeline.read().await.iter().rev() {
982#iJa1xN|            let _ = pipeline.try_process_packet_from_nic(data).await;
983#KkQWgx|        }
984#KNhneG|    }
985#47DEQp|
986#LNmNxR|    pub async fn remove_nic_packet_process_pipeline(&self, id: String) -> Result<(), Error> {
987#kzaKYh|        let mut pipelines = self.nic_packet_process_pipeline.write().await;
988#aIOBXg|        if let Some(pos) = pipelines.iter().position(|x| x.id() == id) {
989#pyEoMy|            pipelines.remove(pos);
990#Y1ZmOf|            Ok(())
991#d4ha1c|        } else {
992#j9KMDj|            Err(Error::NotFound)
993#KkQWgx|        }
994#KNhneG|    }
995#47DEQp|
996#QwGVbl|    fn get_next_hop_policy(is_first_latency: bool) -> NextHopPolicy {
997#Prtm2m|        if is_first_latency {
998#lrGTD9|            NextHopPolicy::LeastCost
999#d4ha1c|        } else {
1000#0t3aFq|            NextHopPolicy::LeastHop
1001#KkQWgx|        }
1002#KNhneG|    }
1003#47DEQp|
1004#YcV_Zw|    pub async fn send_msg(&self, msg: ZCPacket, dst_peer_id: PeerId) -> Result<(), Error> {
1005#0Y1ljl|        self.self_tx_counters
1006#7boxeQ|            .self_tx_bytes
1007#OREiCd|            .add(msg.buf_len() as u64);
1008#P1nqM0|        self.self_tx_counters.self_tx_packets.inc();
1009#X4VtRb|        let msg_len = msg.buf_len() as u64;
1010#X_Kl5i|        let result =
1011#Pd9f-1|            Self::send_msg_internal(&self.peers, &self.foreign_network_client, msg, dst_peer_id)
1012#mju4UJ|                .await;
1013#ZxIhWe|        if result.is_ok() {
1014#wEbH0X|            self.self_tx_counters.self_tx_bytes.add(msg_len);
1015#zhexxv|            self.self_tx_counters.self_tx_packets.inc();
1016#KkQWgx|        }
1017#NMBvw3|        result
1018#KNhneG|    }
1019#47DEQp|
1020#zhhnC3|    async fn send_msg_internal(
1021#swj2Tw|        peers: &Arc<PeerMap>,
1022#rQ13r4|        foreign_network_client: &Arc<ForeignNetworkClient>,
1023#i65YQK|        msg: ZCPacket,
1024#LiSCIf|        dst_peer_id: PeerId,
1025#Ew16UP|    ) -> Result<(), Error> {
1026#Vg3OfO|        let policy =
1027#s6Hg6i|            Self::get_next_hop_policy(msg.peer_manager_header().unwrap().is_latency_first());
1028#47DEQp|
1029#S-yTsc|        if let Some(gateway) = peers.get_gateway_peer_id(dst_peer_id, policy.clone()).await {
1030#CVigK7|            if peers.has_peer(gateway) {
1031#5Mp0l-|                peers.send_msg_directly(msg, gateway).await
1032#JP80iE|            } else if foreign_network_client.has_next_hop(gateway) {
1033#znEXzS|                foreign_network_client.send_msg(msg, gateway).await
1034#CE4VEs|            } else {
1035#tw81bP|                tracing::warn!(
1036#kk6F7t|                    ?gateway,
1037#_vSdBB|                    ?dst_peer_id,
1038#5aLh6k|                    "cannot send msg to peer through gateway"
1039#iyW0eL|                );
1040#rw6i9c|                Err(Error::RouteError(None))
1041#hq_fcK|            }
1042#OEJCrt|        } else if foreign_network_client.has_next_hop(dst_peer_id) {
1043#hzCk3v|            // check foreign network again. so in happy path we can avoid extra check
1044#YLJ9ty|            foreign_network_client.send_msg(msg, dst_peer_id).await
1045#d4ha1c|        } else {
1046#kVR-Se|            tracing::debug!(?dst_peer_id, "no gateway for peer");
1047#NuwARt|            Err(Error::RouteError(None))
1048#KkQWgx|        }
1049#KNhneG|    }
1050#47DEQp|
1051#n2tvl8|    pub async fn get_msg_dst_peer(&self, ipv4_addr: &Ipv4Addr) -> (Vec<PeerId>, bool) {
1052#YR8kEa|        let mut is_exit_node = false;
1053#5eR7LF|        let mut dst_peers = vec![];
1054#FZMhE_|        let network_length = self
1055#7t5vZi|            .global_ctx
1056#THImhK|            .get_ipv4()
1057#8fe7wM|            .map(|x| x.network_length())
1058#y5RfWx|            .unwrap_or(24);
1059#D0sue1|        let ipv4_inet = cidr::Ipv4Inet::new(*ipv4_addr, network_length).unwrap();
1060#vpDCah|        if ipv4_addr.is_broadcast()
1061#lwUXjr|            || ipv4_addr.is_multicast()
1062#QttO4f|            || *ipv4_addr == ipv4_inet.last_address()
1063#qWZHcZ|        {
1064#poVYD0|            dst_peers.extend(self.peers.list_routes().await.iter().filter_map(|x| {
1065#r49BH8|                if *x.key() != self.my_peer_id {
1066#RZUfe_|                    Some(*x.key())
1067#ZtcDW9|                } else {
1068#JqjnWy|                    None
1069#C_CgiK|                }
1070#Hsxh2D|            }));
1071#vflSHi|        } else if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(ipv4_addr).await {
1072#otf58d|            dst_peers.push(peer_id);
1073#pegr-g|        } else if !self
1074#7t5vZi|            .global_ctx
1075#VvsySw|            .is_ip_in_same_network(&std::net::IpAddr::V4(*ipv4_addr))
1076#qWZHcZ|        {
1077#jQL-Kg|            for exit_node in &self.exit_nodes {
1078#yDUUhE|                let IpAddr::V4(exit_node) = exit_node else {
1079#_01Y-5|                    continue;
1080#JBqzQb|                };
1081#t182g9|                if let Some(peer_id) = self.peers.get_peer_id_by_ipv4(exit_node).await {

...35440 bytes truncated...

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d3d620a001sU1oHE22y1sTHz
Use Grep to search the full content or Read with offset/limit to view specific sections.
--- grep({"pattern":"PeerConnId::default","path":"/mnt/nvme2t_2/easytier/easytier/src/peers"}) [completed]
Found 2 matches
/mnt/nvme2t_2/easytier/easytier/src/peers/peer.rs:
  Line 93:         let default_conn_id = Arc::new(AtomicCell::new(PeerConnId::default()));

  Line 101:                     default_conn_id_copy.store(PeerConnId::default());


## [assistant] +058391-05-11T16:13:53.000Z
[thinking] Now I have the complete picture. Let me compile the root cause analysis.

The key finding is in `peer.rs`:
1. `default_conn_id_clear_task` (line 97-103): Every 5 seconds, if `conns_copy.len() > 1`, sets `default_conn_id` to `PeerConnId::default()`.

2. When the periodic task tries to close the peer's default conn:
   - `get_peer_default_conn_id(peer_id)` returns the `default_conn_id` which could be `PeerConnId::default()` (all zeros)
   - `close_peer_conn(peer_id, &conn_id)` with the default (zeros) ID → `self.conns.contains_key(&conn_id)` returns false → returns `Err(NotFound)`
   - But the periodic task does `let _ = pm.close_peer_conn(...).await;` — the error is silently ignored!
   - So the connection is NOT closed.

But there's more. Even if `close_peer_conn` succeeds (when `default_conn_id` is valid), it only closes ONE connection. The peer might have multiple connections (WebSocket + UDP hole punch).

However, the most critical issue is: **even if ALL connections are closed**, the `block_peer` was called, so the peer IS in `blocked_peers`. This means new connections from this peer ARE rejected. So how can traffic still pass through?

The answer lies in the packet routing:
1. When agent receives a packet for the non-whitelisted peer's IP (e.g., the peer is the intended destination), it calls `get_msg_dst_peer` which uses the route table (OSPF) to find the peer
2. The peer is still in the route table even if blocked
3. `peer.send_msg(msg)` → `select_conn()` → finds the oldest/remaining connection and sends through it
4. The `blocked_peers` check is ONLY in `add_new_peer_conn` (connection admission), NOT in message forwarding

BUT wait - there's an even more subtle issue. The `close_peer_conn` sends to `close_event_sender` which then calls `conns_copy.remove(&ret)` to remove the conn from the peer. But what if the conn_id was already reset to default (zeros)? Then `remove` with the zero ID won't match anything, so the connection is never removed from the peer's 
## 根因分析：agent 间歇性放包

### 连接建立流程

```
client → agent listener (WS/TCP/UDP hole punch)
  → add_tunnel_as_server(tunnel, is_directly_connected)
    → add_new_peer_conn(conn)
      → is_peer_blocked(peer_id)? 若 true 则拒绝
      → peers.add_new_peer_conn(conn) → peer.add_peer_conn(conn)
```

### 阻断流程

```
periodic task (每 30s):
  load_ip_whitelist()
  list_routes()
  for each route (peer_id, ip):
    若 ip 不在白名单:
      block_peer(peer_id)                    # blocked_peers.insert(peer_id)
      get_peer_default_conn_id(peer_id)      # 返回 default_conn_id
      close_peer_conn(peer_id, &conn_id)     # 只关闭默认连接
    若 ip 在白名单:
      allowed_peers.push(peer_id)
  unblock allowed_peers
```

### 根因：两个 bug 叠加导致连接未被实际关闭

**Bug 1：`default_conn_id` 被周期性清空（`peer.rs:97-103`）**

```rust
// 每 5 秒运行一次的后台任务
loop {
    sleep(5s);
    if conns_copy.len() > 1 {                 // 当 peer 有多个连接时
        default_conn_id_copy.store(PeerConnId::default());  // 重置为全零
    }
}
```

当 peer 有 > 1 个连接（如 WebSocket + UDP hole punch），`default_conn_id` 每 5 秒被重置为 `PeerConnId::default()`（全零）。

**Bug 2：`close_peer_conn` 用零 ID 调用时静默失败（`peer.rs:174-181` + `instance.rs:363`）**

```rust
// peer.rs:174
pub async fn close_peer_conn(&self, conn_id: &PeerConnId) -> Result<(), Error> {
    let has_key = self.conns.contains_key(conn_id);  // 零 ID → false!
    if !has_key {
        return Err(Error::NotFound);
    }
    self.close_event_sender.send(*conn_id).await.unwrap();
    Ok(())
}

// instance.rs:363 — 静默丢弃错误
let _ = pm.close_peer_conn(peer_id, &conn_id).await;
```

### 完整漏洞链条

```
1. client 连接 agent → WebSocket 建立（conn_id=uuid1）
2. UDP hole punch 完成 → 第二个连接（conn_id=uuid2）
3. peer.rs 后台: conns.len() > 1 → default_conn_id = 零
4. periodic task: block_peer(peer_id) → ✓ 成功
5. periodic task: get_peer_default_conn_id() = 零
6. periodic task: close_peer_conn(零) → Err(NotFound) → 静默丢弃
7. 连接 (uuid1, uuid2) 都保持开放！
8. 数据包通过 peer.send_msg() → select_conn() → 现存连接 → 自由流通
9. 每 30s 重试，block_peer 已存在，close 再次失败
10. 直到 client 自己掉线，否则连接永不关闭
```

| 假设 | 验证结果 |
|------|---------|
| `close_peer_conn` 只关闭默认连接 | ✅ `peer.rs:165-171` `select_conn` 选最低延迟连接，不检查 blocked |
| `default_conn_id` 在多连接时被清空 | ✅ `peer.rs:97-103` `conns_copy.len() > 1` 时每 5s 清空 |
| 清空后 close 失败 | ✅ `peer.rs:174-181` 零 ID 不匹配任何 conn → `Err(NotFound)` |
| 静默丢弃错误 | ✅ `instance.rs:363` `let _ = ...` |
| `block_peer` 只阻止新连接，不影响已有连接 | ✅ `peer_manager.rs:357-361` 仅在 `add_new_peer_conn` 时检查 |
| 包转发不检查 blocked_peers | ✅ `peer.send_msg()` → `select_conn()` 无 blocked 检查 |

### 修复建议

在 periodic/event-driven task 中，`close_peer_conn` 失败后应**关闭 peer 的所有连接**（而非仅默认连接），或直接**从 peer_map 移除整个 peer**：

```rust
// 替代 get_peer_default_conn_id + close_peer_conn:
// 方案 A: 关闭所有连接
if let Some(peer) = pm.get_peer_map().get_peer_by_id(peer_id) {
    for conn_id in peer.list_peer_conns().await {
        let _ = pm.close_peer_conn(peer_id, &conn_id).await;
    }
}

// 方案 B: 直接移除 peer
pm.get_peer_map().close_peer(peer_id).await;
```

方案 B 更简洁——既然已经 `block_peer(peer_id)` 阻止了重连，移除 peer 后它必须重连，重连时会被 blocked 检查拒绝。
