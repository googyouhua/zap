---
comet_change: tmux-warpification-fallback
role: technical-design
canonical_spec: openspec
---

# TMUX Warpification Auto-Fallback Design

## Problem

When SSH warpification with tmux fails (tmux not installed, version too old, install fails), Warp currently shows an error. Users must manually press Ctrl+Alt+I to inject shell integration. This creates a poor UX where the first ~10 seconds of an SSH session are wasted on a failed warpification attempt.

## Solution

Auto-fallback to shell integration (precmd/preexec hooks) when tmux warpification fails. This gives users block mode and command history without manual intervention.

## Files Changed

### 1. `app/src/terminal/local_tty/terminal_manager.rs`

**Change**: Restore `&& !use_ssh_tmux_wrapper` guard at line 620.

**Why**: The original code inadvertently removed this guard, causing the legacy SSH wrapper to be unconditionally active when `SSHTmuxWrapper` was enabled. This broke nested SSH behavior by auto-tracking every SSH hop.

**Before (current)**:
```rust
let enable_ssh_wrapper = if FeatureFlag::SSHTmuxWrapper.is_enabled() {
    enable_ssh_warpification
} else {
    enable_legacy_ssh_wrapper
};
```

**After**:
```rust
let enable_ssh_wrapper = if FeatureFlag::SSHTmuxWrapper.is_enabled() {
    enable_ssh_warpification && !use_ssh_tmux_wrapper
} else {
    enable_legacy_ssh_wrapper
};
```

**Behavior matrix**:

| SSHTmuxWrapper | use_ssh_tmux_wrapper | Result |
|----------------|---------------------|--------|
| enabled | true | wrapper OFF (only manual Ctrl+Alt+I) |
| enabled | false | wrapper ON (SSH DCS sent) |
| disabled | — | legacy wrapper (usually OFF) |

### 2. `app/src/terminal/view.rs`

**Three handlers to modify**:

| Event | Current behavior | New behavior |
|-------|-----------------|--------------|
| `TmuxNotInstalled` | Show install-tmux dialog | Call `trigger_subshell_bootstrap()` with detected shell type |
| `UnsupportedTmuxVersion` | Show version error | Call `trigger_subshell_bootstrap()` with detected shell type |
| `TmuxInstallFailed` | Show error block | Call `trigger_subshell_bootstrap()` with `get_shell_type()` |

`trigger_subshell_bootstrap()` writes the init script to the PTY, which installs precmd/preexec hooks on the remote shell. Once the hooks fire, Warp receives DCS sequences and establishes shell integration (block mode + history tracking).

### 3. `app/src/terminal/bootstrap.rs`

**Change**: Remove `>/dev/null 2>&1` from `init_subshell_command`.

**Why**: The DCS sequences (`\eP...\e\`) must reach the terminal parser. Redirecting to `/dev/null` silently drops them, making shell integration impossible.

## Non-Goals

- No changes to `zsh_body.sh`, `bash` scripts
- No feature flag promotion
- No changes to SSH auto-detection (`ssh_detection.rs`)
- No new settings or configuration items
- No changes to `crates/warp_features/` or `crates/warp_core/`

## Risks

| Risk | Mitigation |
|------|------------|
| Init script dropped if PTY not ready | `trigger_subshell_bootstrap()` is already verified via Ctrl+Alt+I path |
| DCS sequence escaped by remote shell | Script output is base64-encoded, shell-safe |
| `TmuxNotInstalled` never fires when wrapper OFF | Expected — user needs Ctrl+Alt+I manually |
