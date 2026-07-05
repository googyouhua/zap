## Why

当用户 SSH 到远程主机时，Warp 会尝试通过 tmux warpification 启用块模式。如果远程主机上没有 tmux（或版本不支持），目前会直接失败并报错。用户需要手动按 Ctrl+Alt+I 来注入 shell integration，体验割裂。

本项目让 tmux warpification 失败后**自动回退**到 shell integration（precmd/preexec），使 SSH 会话即使没有 tmux 也能获得块模式和历史追踪。

同时修复了 `terminal_manager.rs` 中因 `SSHTmuxWrapper` feature flag 意外开启导致嵌套 SSH 也被自动追踪的回归问题。

## What Changes

- **terminal_manager.rs**: 还原 `&& !use_ssh_tmux_wrapper` 守卫，确保嵌套 SSH 不被追踪
- **view.rs**: `TmuxNotInstalled`/`UnsupportedTmuxVersion`/`TmuxInstallFailed` → 自动调用 `trigger_subshell_bootstrap()`
- **bootstrap.rs**: 初始化脚本不再重定向到 `/dev/null`，确保 DCS 序列能到达 Warp
- 不修改 shell 脚本（zsh_body.sh）、不 promotion feature flag、不改 SSH 检测机制

## Capabilities

### New Capabilities

- `ssh-tmux-fallback`: 当 SSH 远程主机上 tmux warpification 因缺少 tmux 或版本不支持而失败时，自动触发 shell integration 注入，无需用户手动干预

### Modified Capabilities

- *无* — 无 spec 级需求变更

## Impact

- `app/src/terminal/local_tty/terminal_manager.rs` — 条件逻辑还原
- `app/src/terminal/view.rs` — 三个失败事件处理分支改为 fallback
- `app/src/terminal/bootstrap.rs` — 移除重定向
- 不涉及 `crates/warp_features/`、`crates/warp_core/`、`app/assets/bundled/`
