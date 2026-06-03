# Agent: analyst
# Status: done

## Final output
## 方案对比

| 方案 | RUST_LOG 值 | `tracing::warn!` 日志 | `tracing::error!` 日志 | sync-daemon 日志 | Format 2 日志 | 推荐度 |
|------|-------------|----------------------|----------------------|------------------|---------------|--------|
| A | `easytier=error` | ❌ 全部移除 (~126处) | ✅ 保留 (~162处) | ✅ error 保留 | ✅ 不受影响 | ⭐⭐⭐ |
| B | `easytier=off` | ❌ 全部移除 | ❌ 全部移除 | ❌ 全部移除 | ✅ 不受影响 | ⭐⭐ |
| C | `off` | ❌ 全部移除 | ❌ 全部移除 | ❌ 全部移除 | ✅ 不受影响 | ⭐ |
| 当前 | `easytier=warn` | ✅ 显示 (~126处) | ✅ 显示 (~162处) | ✅ warn+error 显示 | ✅ 不受影响 | — |

## 推荐方案：方案 A（`easytier=error`）

**理由**：
1. **安全底线**：`tracing::error!` 共 ~162 处调用，涵盖关键故障（连接失败、解密错误、包转发异常、虚拟网卡错误等）。完全关闭 (`off`) 意味着生产环境中 whitelist sync 失败、wireguard 握手失败等无声无息——用户无从知晓 agent 是否在正常工作。
2. **噪音消除**：`tracing::warn!` 共 ~126 处，包括大量日常告警（"set_nodelay fail"、"packet is already encrypted"、"bind failed, do not return error for ipv6"、"udp send syn" 等）。这些对 agent 运维人员无实际价值，只会污染 stdout/stderr。
3. **sync-daemon 兼容性**：`easytier=error` 保留 whitelist_sync 的 `tracing::error!("Sync error: {}")`（行 41），运维仍能看到 sync 失败。`info!`/`warn!` 被过滤，但 sync-daemon 的 info（行 76 "Whitelist synced: N entries"）和生产故障诊断关系不大。
4. **Format 2 完全不受影响**：`print_event` 用 `println!` 直接写入 stdout，不受 `RUST_LOG` 控制。

**回滚 utils.rs**：是。回滚 `.without_time()` → `.with_timer(get_logger_timer_rfc3339())`。因为 Format 1 日志被过滤后，时间戳格式已不可见；但保持 console_layer 与 file_layer 一致（两者都用 rfc3339）是正确的设计。

## 具体修改

**文件 1**：`easytier/scripts/agent-entrypoint.sh` 第 8 行
- 修改前：`: "${RUST_LOG:=easytier=warn}"`
- 修改后：`: "${RUST_LOG:=easytier=error}"`

**文件 2**：`easytier/src/utils.rs` 第 122 行
- 修改前：`.without_time()`
- 修改后：`.with_timer(get_logger_timer_rfc3339())`

---

**补充数据**：`whitelist_sync.rs` 日志使用分布
| 级别 | 数量 | 示例 |
|------|------|------|
| `info!` | 3 | "Whitelist synced: N entries" |
| `warn!` | 4 | "Admin returned non-success status", "Failed to parse admin JSON" |
| `error!` | 1 | "Sync error: ..." |

方案 A 仍保留 `error!` 行，运维人员可在 sync 完全挂掉时看到错误。

## Tool calls summary
- read: 4 calls
- grep: 6 calls