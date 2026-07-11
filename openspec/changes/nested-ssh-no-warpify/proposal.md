## Why

`use_ssh_tmux_wrapper = false`（关闭 tmux）时，本地终端检测到 SSH 登录完成输出后会触发 `ReadyToWarpify` 事件。如果第一条 SSH 已连接（legacy wrapper 模式），后续嵌套 SSH 的登录输出也会被同一检测路径匹配，导致弹出 tmux warpify 提示。

## What Changes

- `view.rs:handle_detected_end_of_ssh_login` 中 `ReadyToWarpify` 分支：检查当前活跃会话是否是 warpified SSH（`is_subshell_or_ssh`），如果是则跳过 warpification

## Capabilities

### New Capabilities
- 无

### Modified Capabilities
- 无

## Impact

- `app/src/terminal/view.rs` 一处 +7 行
