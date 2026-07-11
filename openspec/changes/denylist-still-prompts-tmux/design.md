## 修复方案

### 问题 1: `ssh_detection.rs` — denylist 检查被绕过

将 denylist 检查从函数末尾移到开头，在 `FeatureDisabled` return 之前执行。即使 `should_prompt_ssh_tmux_wrapper = false`，也先检查 denylist。

### 问题 2: `view.rs` — subshell 路径漏检 SSH denylist

在 `AfterBlockStarted` 的 subshell 检测路径中，增加 `ssh_host_denylisted` 变量，使用 `parse_interactive_ssh_command` 解析 SSH host 后调用 `is_ssh_host_denylisted` 检查。

### 问题 3: Legacy wrapper 不受 Rust denylist 控制

`terminal_manager.rs` 中构建 PTY 环境时，将 denylist 作为 `WARP_SSH_DENY_HOSTS` 逗号分隔列表传给 shell 进程。
`bash_body.sh` 的 `ssh()` 函数覆盖中添加 denylist 检查：解析目标 host，遍历 `WARP_SSH_DENY_HOSTS`，若匹配则跳过 ControlMaster wrapper 直接 `command ssh`。

### 问题 4: Warpify 页面 denylist 在 Windows 不可见

Denylist hosts UI 从 `add_setting(..., &use_ssh_tmux_wrapper, ...)` 闭包内移到外层，仅受 `enable_ssh_warpification` 控制。`UseSshTmuxWrapper` 的 `supported_platforms` 不含 Windows，导致整个 block 被跳过。
