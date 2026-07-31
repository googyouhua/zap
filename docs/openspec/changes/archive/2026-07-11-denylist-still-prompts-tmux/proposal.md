## Why

当 `use_ssh_tmux_wrapper = false`（tmux 关闭）时，SSH host denylist 未生效。添加 denylist 的主机仍然会弹出 warpification 提示。原因：

1. `evaluate_warpify_ssh_host` 中的 denylist 检查在 `FeatureDisabled` return 之后，永远不会执行
2. SSH 被 `is_compatible_subshell_command` 匹配为 subshell，走 subshell 路径，该路径没检查 SSH host denylist
3. Legacy SSH wrapper (ControlMaster) 不受 Rust 层 denylist 控制

## What Changes

- `ssh_detection.rs`: denylist 检查移到 `FeatureDisabled` return 之前
- `view.rs`: subshell 路径增加 `ssh_host_denylisted` 检查
- `terminal_manager.rs`: 把 denylist 作为 `WARP_SSH_DENY_HOSTS` 环境变量传给 shell
- `bash_body.sh`: `ssh()` 函数检查目标 host 是否在 denylist 中，跳过 wrapper
- `warpify_page.rs`: denylist hosts UI 移出 `use_ssh_tmux_wrapper` 的 `add_setting` 块（Windows 可见）

## Capabilities

### New Capabilities
- 无

### Modified Capabilities
- 无

## Impact

- `app/src/terminal/ssh/ssh_detection.rs`: -7 +7
- `app/src/terminal/view.rs`: +4
- `app/src/terminal/local_tty/terminal_manager.rs`: +8
- `app/assets/bundled/bootstrap/bash_body.sh`: +30
- `app/src/settings_view/warpify_page.rs`: -22 +14
