---
change: tmux-warpification-fallback
design-doc: docs/superpowers/specs/2026-07-05-tmux-warpification-fallback-design.md
base-ref: 39c17ecdeb4cbc274cf9d4337b8427a8834db34d
---

# TMUX Warpification Fallback — Implementation Plan

## Overview

3 files, ~10 line changes. Tmux warpification failure → auto shell integration.

## Tasks (from tasks.md)

### 1. terminal_manager.rs — 还原守卫

- [ ] 1.1 Restore `&& !use_ssh_tmux_wrapper` at line 620
- [ ] 1.2 `cargo check`

### 2. view.rs — 三个失败事件 → trigger_subshell_bootstrap

- [ ] 2.1 `TmuxNotInstalled` → `trigger_subshell_bootstrap()` with detected shell type
- [ ] 2.2 `UnsupportedTmuxVersion` → `trigger_subshell_bootstrap()` with detected shell type
- [ ] 2.3 `TmuxInstallFailed` → `trigger_subshell_bootstrap()` with `get_shell_type()`
- [ ] 2.4 Confirm installer/error blocks removed
- [ ] 2.5 `cargo check`

### 3. bootstrap.rs — 移除重定向

- [ ] 3.1 Remove `>/dev/null 2>&1` from `init_subshell_command`
- [ ] 3.2 `cargo check`

### 4. 验证

- [ ] 4.1 Build and test SSH to no-tmux host
- [ ] 4.2 Verify `use_ssh_tmux_wrapper=false` → no nested SSH tracking
- [ ] 4.3 Verify no unused import warnings
