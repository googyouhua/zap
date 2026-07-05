## Context

当用户 SSH 到远程主机时，Warp 的 terminal manager 会发送 SSH DCS 序列触发 warpification 流程。如果 `use_ssh_tmux_wrapper` 开启，Warp 会在远程主机上尝试安装 tmux 并启用块模式。

当前流程：
1. SSH 连接建立 → terminal_manager 发送 DCS hook
2. Warp 检测到 SSH 会话 → 评估是否应 warpify
3. tmux 检测流程：检查远程 `tmux -V` → 不存在或版本过旧 → `TmuxNotInstalled` / `UnsupportedTmuxVersion`
4. 当前：这些事件被当作错误处理（弹 installer 或报错）
5. Tmux 安装失败 → `TmuxInstallFailed` → 同样报错

目标：在步骤 4 和 5 自动回退到 shell integration。

## Goals / Non-Goals

**Goals:**
- tmux 不存在时自动注入 shell integration（等同于 `Ctrl+Alt+I` 的结果）
- tmux 版本不支持时自动回退
- tmux 安装失败时自动回退
- 保持 `ssh_tmux_wrapper` 功能关闭时完全不影响现有行为
- 嵌套 SSH（local→A→B）不作为自动回退目标（仅首跳）

**Non-Goals:**
- 不修改 SSH 自动检测机制
- 不 promotion feature flag
- 不改动 zsh_body.sh 或 bash 脚本
- 不修改 `ssh_detection.rs`
- 不引入新的设置项

## Decisions

### 1. `terminal_manager.rs` 还原原始条件
**决策**: 恢复 `&& !use_ssh_tmux_wrapper` 守卫。

原有代码移除了此守卫（见 `f5d5e862`），导致 `SSHTmuxWrapper` 开启时 legacy 包装器无条件启用，嵌套 SSH 也被追踪。

还原后行为：
```
SSHTmuxWrapper 开启 + use_ssh_tmux_wrapper = true  → wrapper OFF
SSHTmuxWrapper 开启 + use_ssh_tmux_wrapper = false → wrapper ON  (发送 SSH DCS)
SSHTmuxWrapper 关闭                                 → enable_legacy_ssh_wrapper (通常 false)
```

### 2. 三个失败事件改为 trigger_subshell_bootstrap
**决策**: `TmuxNotInstalled`、`UnsupportedTmuxVersion`、`TmuxInstallFailed` 的处理函数中，调用 `trigger_subshell_bootstrap()` 而不是报错。

`trigger_subshell_bootstrap` 写入初始化脚本到 PTY，Warp 收到 precmd/preexec DCS 后自动建立 shell integration，用户获得块模式。

**检测到的 shell 类型**：`TmuxNotInstalled` 和 `UnsupportedTmuxVersion` 时已知 shell 类型（来自 warpification 上下文）；`TmuxInstallFailed` 时通过 `get_shell_type()` 检测。

### 3. 初始化脚本不重定向 stdout/stderr
**决策**: `bootstrap.rs` 中 `init_subshell_command` 移除 `>/dev/null 2>&1`。

DCS 序列（`\eP` ... `\e\`）需要被终端解析器识别。重定向到 `/dev/null` 会使 DCS 被丢弃，shell integration 无法建立。

### 4. 不修改 shell 脚本
**决策**: zsh_body.sh 和 bash 的 `ssh()` 包装器保持不变。

- `ssh()` 包装器由 `WARP_IS_LOCAL_SHELL_SESSION == "1"` 控制（仅本地 shell 安装）
- 远程 shell 上 `WARP_IS_LOCAL_SHELL_SESSION` 未设置 → `ssh()` 不安装 → 嵌套 SSH 不被拦截
- 不需要修改此行为

## Risks / Trade-offs

- **[风险] 初始化脚本写入时机**：如果 PTY 还未完全就绪就写入，可能丢失第一个 shell prompt。→ 缓解：`trigger_subshell_bootstrap` 已在其他路径（Ctrl+Alt+I）验证过，时机恰当。
- **[风险] DCS 序列被 shell 转义**：某些远程 shell 配置可能干扰 DCS 序列。→ 缓解：`bootstrap.rs` 输出采用 base64 编码，shell 不会干扰。
- **[风险] TmuxNotInstalled 在 use_ssh_tmux_wrapper = false 时不触发**：wrapper OFF → 不发送 SSH DCS → warpification 不启动。这是预期行为——用户关闭了 tmux wrapper，需要手动 Ctrl+Alt+I。
