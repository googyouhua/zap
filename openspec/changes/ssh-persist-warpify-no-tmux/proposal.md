## Why

当前 `evaluate_warpify_ssh_host` 要求 `enable_ssh_warpification` 和 `use_ssh_tmux_wrapper` 同时为 true 才会弹出 warpify 提示。用户关闭 tmux 后，SSH warpification 完全被禁用，即使不依赖 tmux 的 shell integration 方案也无法使用。

## What Changes

- `ssh_detection.rs`: 将 `use_ssh_tmux_wrapper` 从 warpification 的启停闸门中分离。`enable_ssh_warpification` 单独控制是否允许 warpify SSH 会话；`use_ssh_tmux_wrapper` 仅控制是否优先尝试 tmux 安装，不阻断 warpification 流程
- 当 `use_ssh_tmux_wrapper = false` 时，跳过 tmux 检测/安装，直接进入 shell integration bootstrap

## Capabilities

### New Capabilities
- `ssh-warpify-without-tmux`: 允许用户禁用 tmux wrapper 后仍能通过 shell integration warpify SSH 会话

### Modified Capabilities
- 无

## Impact

- `app/src/terminal/ssh/ssh_detection.rs` — 逻辑修改
- `app/src/terminal/view.rs` — `handle_remote_warpification_is_unavailable` 可能需处理「主动跳过 tmux」的情况
- 仅 1-2 个文件改动
