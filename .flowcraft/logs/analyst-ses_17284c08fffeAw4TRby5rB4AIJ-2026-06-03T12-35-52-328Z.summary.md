# Agent: analyst
# Status: done

## Final output
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

## Tool calls summary
- read: 1 calls
- read_with_hash: 5 calls
- grep: 1 calls