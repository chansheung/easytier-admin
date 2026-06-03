# Agent: analyst
# Status: done

## Final output
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
10. 直到 client 自己掉线，否则连

## Tool calls summary
- read_with_hash: 11 calls
- grep: 10 calls