## 修复方案

`handle_detected_end_of_ssh_login` 的 `ReadyToWarpify` handler 中，在调用 `evaluate_warpify_ssh_host` 之前检查当前活跃 session 是否已处于 warpified 状态：

```rust
if let Some(active_session_id) = self.model.lock().block_list().active_block().session_id() {
    if self.sessions.as_ref(ctx).get(active_session_id).is_some_and(|s| s.is_subshell_or_ssh()) {
        return;
    }
}
```

`is_subshell_or_ssh()` 覆盖三种已 warpify 的会话类型：
1. `SessionType::WarpifiedRemote` — tmux warpification
2. `is_legacy_ssh_session` — legacy SSH wrapper
3. `subshell_info().is_some()` — 任何已 warpified 的 subshell
