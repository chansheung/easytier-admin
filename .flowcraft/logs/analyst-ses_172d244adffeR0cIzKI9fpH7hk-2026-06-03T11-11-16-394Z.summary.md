# Agent: analyst
# Status: done

## Final output
以下为分析报告：

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
        echo "[entrypoint] auto-adding admin self ($ADMIN_IP) to whitelist with hostname=$HOSTNAME"
        sqlite3 "$DB_PATH" \
            "INSERT OR IGNORE INTO ip_whitelist (ip, hostname) VALUES ('$ADMIN_IP', '$HOSTNAME')" \
            2>/dev/null || true
    fi
fi

# 然后用这里开始原有 while true 循环
```

- **是否需要新增环境变量**：否。hostname 用 `$(hostname)` 自动获取，IP 从 `$CORE_CONFIG` 解析。
- **对现有逻辑的影响**：

| 方面 | 影响 |
|------|------|
| 首次启动 | `INSERT OR IGNORE` 插入 admin 自身条目，白名单 JSON 从首个周期就包含 admin IP |
| 容器重启 | `OR IGNORE` 防止重复插入（已有则跳过） |
| hostname 变更 | 如果 hostname 改变了，旧条目保留旧 hostname；但 `INSERT OR IGNORE` 不会更新已有行。如果期望更新，可用 `INSERT OR REPLACE` 或先 `UPDATE` 再 `INSERT OR IGNORE` |
| admin IP 变更 | 同上，旧条目保留旧 IP。但 admin 的 core.toml 通常不变 |
| 无副作用 | 仅在第 1 个 sync 周期之前多一次 sqlite3 调用 |

**原理**：agent-entrypoint.sh（行 16-25）已有类似逻辑（读取 core.toml 的 ipv4，计算 `.1`）。admin 容器的 core.toml 的 ipv4 已经是 admin 自身 IP（如 `10.0.210.1/24`），无需计算 `.1`，只需提取 IP 即可。

---

## 问题 2 修复方案

- **修改文件**：`/mnt/nvme2t_2/easytier/easytier/src/utils.rs`
- **修改位置**：行 120-124（console_layer 初始化）
- **具体修改内容**：在行 122 `with_timer(get_logger_timer_rfc3339())` 替换为 `without_time()`

```rust
// 修改前（行 120-124）：
let console_layer = tracing_subscriber::fmt::layer()
    .pretty()
    .with_timer(get_logger_timer_rfc3339())   // 行 122：导致 RFC3339 时间戳
    .with_writer(std::io::stderr)
    .with_filter(console_filter);

// 修改后：
let console_layer = tracing_subscriber::fmt::layer()
    .pretty()
    .without_time()                         

## Tool calls summary
- read: 6 calls
- grep: 3 calls