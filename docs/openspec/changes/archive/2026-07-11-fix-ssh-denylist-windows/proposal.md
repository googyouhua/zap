## Why

SSH denylist hosts UI 在 Windows 上不可见。根因：denylist 渲染代码嵌套在 `use_ssh_tmux_wrapper` 的 `add_setting` 块内，而 `UseSshTmuxWrapper` 的 `supported_platforms` 仅包含 MAC + LINUX，`add_setting` 在非支持平台上跳过整个 block。

## What Changes

- 将 denylist hosts input list 从 `use_ssh_tmux_wrapper` 的 `add_setting` 回调中移到外层 SSHWidget column
- denylist 仅受 `enable_ssh_warpification` 控制

## Capabilities

### New Capabilities
- 无

### Modified Capabilities
- 无

## Impact

- `app/src/settings_view/warpify_page.rs` 一处重构（-22 +14 行）
