# Comet Design Handoff

- Change: tmux-warpification-fallback
- Phase: design
- Mode: compact
- Context hash: 6bc70745a80356c56eaadd9b397ec76f941bdd0b747acd14aec96d3729e8f1b1

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/tmux-warpification-fallback/proposal.md

- Source: openspec/changes/tmux-warpification-fallback/proposal.md
- Lines: 1-31
- SHA256: 44a36ea2edb43c8a014d851a822a9beca823d07fcbfdde7896db7d5a82d059da

```md
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
```

## openspec/changes/tmux-warpification-fallback/design.md

- Source: openspec/changes/tmux-warpification-fallback/design.md
- Lines: 1-67
- SHA256: 4832abbbedccca68772a392d11b1ff2af34aacd6579b6a08cb32ffd44df66192

```md
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
```

## openspec/changes/tmux-warpification-fallback/tasks.md

- Source: openspec/changes/tmux-warpification-fallback/tasks.md
- Lines: 1-23
- SHA256: 7aea3b4864907c8ae1e57c9f432f3600748d8ca2308c64137afd9dd6949ea5ac

```md
## 1. terminal_manager.rs 还原原始条件

- [ ] 1.1 在 `terminal_manager.rs` 第 620-626 行恢复 `&& !use_ssh_tmux_wrapper` 守卫
- [ ] 1.2 `cargo check` 编译通过

## 2. view.rs 三个失败事件改为触发 shell integration

- [ ] 2.1 `TmuxNotInstalled` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入检测到的 shell 类型
- [ ] 2.2 `UnsupportedTmuxVersion` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入检测到的 shell 类型
- [ ] 2.3 `TmuxInstallFailed` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入 `get_shell_type()` 结果
- [ ] 2.4 确认移除了对应分支中的 installer 弹框和 error block 逻辑
- [ ] 2.5 `cargo check` 编译通过

## 3. bootstrap.rs 移除初始化脚本重定向

- [ ] 3.1 `init_subshell_command` 中移除 `>/dev/null 2>&1`，确保 DCS 序列不被丢弃
- [ ] 3.2 `cargo check` 编译通过

## 4. 验证

- [ ] 4.1 构建并测试：SSH 到没有 tmux 的主机，确认自动注入 shell integration
- [ ] 4.2 确认 `use_ssh_tmux_wrapper = false` 时嵌套 SSH 不追踪
- [ ] 4.3 确认删除的 `use` 没有引起编译警告
```

## openspec/changes/tmux-warpification-fallback/specs/ssh-tmux-fallback/spec.md

- Source: openspec/changes/tmux-warpification-fallback/specs/ssh-tmux-fallback/spec.md
- Lines: 1-44
- SHA256: ec32edfccc397838c909b49837bd701dfd078467dc8607b37e727d7e70589373

```md
## ADDED Requirements

### Requirement: Auto fallback when tmux is not installed
When an SSH session is warpified and the remote host does not have tmux installed, the system SHALL automatically fall back to shell integration (precmd/preexec) instead of blocking with an error.

#### Scenario: Tmux not installed on remote host
- **WHEN** user SSHs to a host that does not have `tmux` installed
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type
- **THEN** the SSH session SHALL have block mode and command history via shell integration

### Requirement: Auto fallback when tmux version is unsupported
When an SSH session is warpified and the remote host has a tmux version below the minimum required version, the system SHALL automatically fall back to shell integration.

#### Scenario: Unsupported tmux version
- **WHEN** user SSHs to a host with tmux < 3.3
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL NOT block with "unsupported version" error
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type

### Requirement: Auto fallback when tmux installation fails
When an SSH session is warpified and the automatic tmux installation fails, the system SHALL automatically fall back to shell integration.

#### Scenario: Tmux install fails
- **WHEN** user SSHs to a host where tmux auto-install fails
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type

### Requirement: Nested SSH sessions are not auto-fallback targets
The auto-fallback mechanism SHALL only apply to the first SSH hop from the local Warp instance. Nested SSH sessions (ssh from a remote host to another host) SHALL NOT be auto-warpified.

#### Scenario: Nested SSH not auto-tracked
- **WHEN** user SSHs from local Warp to Host A
- **AND** then SSHs from Host A to Host B
- **THEN** Host B SHALL NOT receive auto shell integration injection
- **THEN** Host B SHALL only have block-mode tracking from Host A's shell hooks

### Requirement: No behavior change when tmux wrapper is disabled
When `use_ssh_tmux_wrapper` is disabled, the system SHALL preserve the original SSH session behavior without any auto-fallback.

#### Scenario: Tmux wrapper disabled
- **WHEN** user has `use_ssh_tmux_wrapper` set to false
- **THEN** the auto-fallback mechanism SHALL NOT activate
- **THEN** nested SSH tracking behavior SHALL match the original code (no auto-tracking)
```

