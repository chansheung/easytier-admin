# Agent: coder
# Task: ## 任务：修复 event-driven 任务的 IP 比较格式 bug

### 背景
`easytier/src/instance/instance.rs` 的 event-driven 任务（行 ~397）使用 `format!("{}", ip)` 格式化 `Ipv4Inet`，输出 `"10.0.210.5/24"`（带子网掩码）。但白名单 JSON 存储的是 `"10.0.210.5"`（不带掩码）。比较永远失败，导致所有 peer 被视为非白名单 → 全部断开 → 断开后 peer 多路径重连 → 部分包漏过。

### 修复
修改 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 的 event-driven 任务。

**位置**：event-driven 任务中，`if let Some(ref ip) = info.ipv4_addr` 分支内。

**当前代码**（约行 397-400）：
```rust
if let Some(ref ip) = info.ipv4_addr {
    let ip_str = format!("{}", ip);
    tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
    let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
```

**修改为**：
```rust
if let Some(ref ip) = info.ipv4_addr {
    let ip_str = format!("{}", ip.address());
    tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
    let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
```

**只改一行**：`format!("{}", ip)` → `format!("{}", ip.address())`

### 同一个文件中还有一处类似的格式化需要修改
**位置**：event-driven 任务中 hostname 解绑检查（约行 404-406）：
```rust
let peer_hostname = info.hostname.as_deref().unwrap_or("");
```

这个用的是 `info.hostname`（String），不是 Ipv4Inet，所以没问题。

### 验证
1. 用 `read` 工具读修改后的行，确认是 `ip.address()` 而不是 `ip`
2. 用 `bash -n` 检查语法
3. 确认 periodic task（约行 322）没有同样的问题（periodic task 已经用了 `ip.address()` 修正了，不用改）

### 不要修改其他文件
只修改 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 中的这一行。
# Status: done
# Date: 2026-06-03T11:03:16.058Z

## [user] +058391-04-24T04:31:31.000Z
## Recent context from this session
[2026-06-02T16:40:58.229Z] @analyst: ## 任务：完整审阅 README.md 并返回详细报告

### 文件路径
`/mnt/nvme2t_2/easytier/README.md`

### 任务背景
之前的批量委托被截断了。请**重新完整审阅** README.md，从数据/技术准确性角度给出详细报告。

### 审阅重点

#### 1. 技术准确性
- 项目背景是否清楚说明"为什么二次开发"？
- 半中心化动机（仅 admin 不够）是否讲清楚？
- ASCII 协作流程图是否正确？
- 默认白名单 `.1` 后缀约定是否清楚？
- 鸡生蛋问题（chicken-and-egg）的描述是否清楚？
- sync-daemon 工作流程是否准确？
- admin 端 vs agent 端的 IP 白名单机制是否分开说清楚？

#### 2. 数据/示例一致性
- admin/agent 端 `core.toml` 示例是否
  → Now I have all the data I need. Let me compile the full report.

---

## 总体评分（满分 5 分）：3.8

---

## 必须修复的问题（Blocking）

### 1. 截图文件名与标题**对调**（行 140、146）

- **行 140**：`![IP 白名单](eg/devices.png)` — 标题是「IP 白名单」，但图片文件名是 `devices.png`
- **行 146**：`![设备管理](eg/ip-whitelist.png)` — 标题是「设备管理」，但图片文件名是 `ip-whitelist.png`

建议：将两个文件名交换，使 `ip-whitelist.png` 对应 IP 白名单截图、`devices.png` 对应设备管理截图。

### 2. VPN 端口前后不一致（core.toml 示例 vs 脚本/Dockerfile）

- Admin **core.toml 示例**（行 188–201）使用端口 **`22022`**
- **Dockerfile**（行 69）：`EXPOSE 22020`
- **build-and-run.sh**（行 9）：`VPN_PORT="22020"`
- **一键脚本表格**（行 474）：`--vpn-port

[2026-06-02T16:44:37.700Z] @reviewer: ## 任务：完整审阅 README.md 的隐私、Markdown 语法、可维护性

### 文件路径
`/mnt/nvme2t_2/easytier/README.md`

### 任务背景
之前的批量委托被截断了。请**重新完整审阅** README.md，从隐私、Markdown 语法、可维护性角度给出详细报告。

### 审阅重点

#### 1. 隐私信息检查（Critical）
全文搜索以下敏感字符串，确认**零出现**：
- `cph`、`admincph`
- `64c`、`dell`、`3080ts`
- `172.30.0.3`、`202.104.40.234`、`192.168.168.106/107`
- `3516942721`、`3030205527`、`1137062372`、`1122820763`、`1914331320`

所有示例必须用占位符：
- `
  → 我将并行验证一些交叉引用。
Both image files exist but are referenced with swapped captions (already known issue).

---

## 总体评分（满分 5 分）：4.2

## 🔒 隐私问题（Critical）

无。全文零敏感字符串泄漏。所有示例均使用占位符（`your-network-name`、`change-me-to-a-random-string`、`<admin公网IP>`、`your-password`、`your-strong-password`）。

## ⚠️ Markdown 语法问题

1. **行 140/146：截图 alt 文字与文件名对调**。`![IP 白名单](eg/devices.png)` 应为 `eg/ip-whitelist.png`；`![设备管理](eg/ip-whitelist.png)` 应为 `eg/devices.png`。文件本身均存在，仅引用交叉。

2. **行 3–36：目录缺少 `为什么仅有 admin 端白名单不够`（#### 级，行 55）**。该小节是架构设计的核心动机段落，但目录中跳过了它，导致用户无法从目录直达。

3. **行 188–201 vs 行 325–341：core.toml 示例端口与 Dockerfil

[2026-06-02T16:48:33.205Z] @reviewer2: ## 任务：完整审阅 README.md 的结构、表达、用户体验

### 文件路径
`/mnt/nvme2t_2/easytier/README.md`

### 任务背景
之前的批量委托被截断了。请**重新完整审阅** README.md，从结构、表达、用户体验角度给出详细报告。

### 审阅重点

#### 1. 文档结构
- 开头钩子（标题下方 1-2 句）是否吸引人？
- 目录是否便于导航？
- 章节顺序是否符合"从抽象到具体"的学习曲线？
- 每个章节信息密度是否合适？

#### 2. 表达质量
- 是否简练？有无冗长句子？
- 术语是否一致？
- 标题层级是否合理？
- 代码块注释是否足够？

#### 3. 用户体验
新用户按顺序读完 README 能否：
1. 理解项目解决什么问题？
2. 理解为什么需要 admin + agent？
3. 知道 admin/agen
  → Now let me also check the actual screenshot files and a few cross-references:
## 总体评分（满分 5 分）：3.6

## 🎯 结构问题

1. **行 3–36：目录缺少 `为什么仅有 admin 端白名单不够`**（行 55）。该小节是架构动机的核心段落，目录跳过了它，用户无法从目录直达。建议在行 7 `半中心化协作架构` 下方补充 `- [为什么仅有 admin 端白名单不够](#为什么仅有-admin-端白名单不够)`。

2. **行 140/146：截图 alt 文字与文件名对调**。`![IP 白名单](eg/devices.png)` 应引用 `eg/ip-whitelist.png`；`![设备管理](eg/ip-whitelist.png)` 应引用 `eg/devices.png`。两个文件均存在，仅交叉引用。

3. **行 293–297 与行 507–525：Admin "常见问题" 与全局 FAQ 重复**。Admin 章节内有 2 条 Q&A（行 295–296），全局 FAQ 有 7 条（行 509–525）。内容高度重叠（空白名单行为、生效时间），用户会在两处找到不同详细度的同类信息。建议删掉行 293–297 的子节 FAQ，统一到全局 FAQ 并按 admin/

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

## 任务：修复 event-driven 任务的 IP 比较格式 bug

### 背景
`easytier/src/instance/instance.rs` 的 event-driven 任务（行 ~397）使用 `format!("{}", ip)` 格式化 `Ipv4Inet`，输出 `"10.0.210.5/24"`（带子网掩码）。但白名单 JSON 存储的是 `"10.0.210.5"`（不带掩码）。比较永远失败，导致所有 peer 被视为非白名单 → 全部断开 → 断开后 peer 多路径重连 → 部分包漏过。

### 修复
修改 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 的 event-driven 任务。

**位置**：event-driven 任务中，`if let Some(ref ip) = info.ipv4_addr` 分支内。

**当前代码**（约行 397-400）：
```rust
if let Some(ref ip) = info.ipv4_addr {
    let ip_str = format!("{}", ip);
    tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
    let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
```

**修改为**：
```rust
if let Some(ref ip) = info.ipv4_addr {
    let ip_str = format!("{}", ip.address());
    tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
    let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
```

**只改一行**：`format!("{}", ip)` → `format!("{}", ip.address())`

### 同一个文件中还有一处类似的格式化需要修改
**位置**：event-driven 任务中 hostname 解绑检查（约行 404-406）：
```rust
let peer_hostname = info.hostname.as_deref().unwrap_or("");
```

这个用的是 `info.hostname`（String），不是 Ipv4Inet，所以没问题。

### 验证
1. 用 `read` 工具读修改后的行，确认是 `ip.address()` 而不是 `ip`
2. 用 `bash -n` 检查语法
3. 确认 periodic task（约行 322）没有同样的问题（periodic task 已经用了 `ip.address()` 修正了，不用改）

### 不要修改其他文件
只修改 `/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs` 中的这一行。

[PROGRESS] After each sub-step, output: [PROGRESS: what you just did]. Keep it under 120 chars.

## [assistant] +058391-04-24T04:33:49.000Z
[thinking] The task is clear: I need to fix a bug in `instance.rs` where `format!("{}", ip)` should be `format!("{}", ip.address())` to strip the subnet mask when comparing against whitelist IPs.

Let me read the file first to find the exact location.
--- grep({"pattern":"format!\\(\"\\{\\}\", ip\\)","path":"/mnt/nvme2t_2/easytier/easytier/src/instance"}) [completed]
Found 1 matches
/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs:
  Line 398:                                                 let ip_str = format!("{}", ip);


## [assistant] +058391-04-24T06:18:31.000Z
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

The tool call succeeded but the output was truncated. Full output saved to: /home/cph/.local/share/opencode/tool-output/tool_e8d267ee7001A1qui9SJCZXWdO
Use Grep to search the full content or Read with offset/limit to view specific sections.

## [assistant] +058391-04-24T07:29:02.000Z
Found it at line 398. The `ip` is of type `Ipv4Inet` (from the `cidr` crate), and `format!("{}", ip)` outputs `"10.0.210.5/24"` with subnet mask. Need to use `ip.address()` to get just the IP.
--- hashline_edit({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs","edits":[{"line":398,"hash":"iDSegq","newContent":"                                                let ip_str = format!(\"{}\", ip.address());"}]}) [completed]
Applied 1 hash-verified edit(s) to /mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs.

## [assistant] +058391-04-24T09:00:49.000Z
[PROGRESS: Fixed line 398: `format!("{}", ip)` → `format!("{}", ip.address())` to strip /24 subnet mask]

Now let me verify the periodic task (line 322) already uses `.address()`:
--- read({"filePath":"/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs","offset":395,"limit":10}) [completed]
<path>/mnt/nvme2t_2/easytier/easytier/src/instance/instance.rs</path>
<type>file</type>
<content>
395:                                     match pm.get_peer_map().get_route_peer_info(peer_id).await {
396:                                         Some(info) => {
397:                                             if let Some(ref ip) = info.ipv4_addr {
398:                                                 let ip_str = format!("{}", ip.address());
399:                                                 tracing::info!("Event-driven: peer {} has ipv4={}, checking whitelist", peer_id, ip_str);
400:                                                 let whitelist_ip_match = whitelist.iter().any(|e| e.ip == ip_str);
401:                                                 if whitelist_ip_match {
402:                                                     let entry = whitelist.iter().find(|e| e.ip == ip_str).unwrap();
403:                                                     let peer_hostname = info.hostname.as_deref().unwrap_or("");
404:                                                     match &entry.hostname {

(Showing lines 395-404 of 1525. Use offset=405 to continue.)
</content>

## [assistant] +058391-04-24T09:51:58.000Z
Fix verified. Line 398 now reads `format!("{}", ip.address())`.

The periodic task (line 322) already uses `.address()` — confirmed from the earlier grep which only found one match for `format!("{}", ip)` (the one we just fixed).
--- bash({"command":"rg 'format!\\(\"\\{\\}\", ip\\)' easytier/src/instance/instance.rs","description":"Verify no remaining bare ip format"}) [completed]
/usr/bin/bash: 行 1: rg: 未找到命令


## [assistant] +058391-04-24T11:11:40.000Z
--- grep({"path":"/mnt/nvme2t_2/easytier/easytier/src/instance","pattern":"format!\\(\"\\{\\}\", ip\\)"}) [completed]
No files found

## [assistant] +058391-04-24T12:13:42.000Z
Zero remaining instances of the bug. The fix is complete:

- **Line 398**: `format!("{}", ip)` → `format!("{}", ip.address())` — strips the `/24` subnet mask so the IP string matches whitelist entries like `"10.0.210.5"` instead of `"10.0.210.5/24"`.
