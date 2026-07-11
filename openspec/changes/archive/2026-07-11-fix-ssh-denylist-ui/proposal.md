## Why

Warp 的 SSH Hosts Denylist 设置项（`warpify.ssh.ssh_hosts_denylist`）在 UI 中错误地依赖了 `use_ssh_tmux_wrapper` 条件，导致 tmux 包装器未启用时 denylist 不显示。用户无法通过 UI 将堡垒机加入 denylist。

## What Changes

- 移除 `warpify_page.rs:803` 的 `should_prompt_ssh_tmux_wrapper` 条件，SSH hosts denylist 只需 `enable_ssh_warpification` 为 true 即可显示

## Impact

- `app/src/settings_view/warpify_page.rs`: 1 行改动
