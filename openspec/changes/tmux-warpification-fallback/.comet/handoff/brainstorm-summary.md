# Brainstorm Summary

- Change: tmux-warpification-fallback
- Date: 2026-07-05

## Confirmed Technical Approach

1. `terminal_manager.rs`: Restore `&& !use_ssh_tmux_wrapper` guard so nested SSH is not auto-tracked when `SSHTmuxWrapper` is enabled
2. `view.rs`: `TmuxNotInstalled` / `UnsupportedTmuxVersion` → `trigger_subshell_bootstrap()` with known shell type; `TmuxInstallFailed` → `trigger_subshell_bootstrap()` with `get_shell_type()`
3. `bootstrap.rs`: Remove `>/dev/null 2>&1` from init script so DCS sequences reach Warp

## Key Trade-offs and Risks

- PTY timing: script may be dropped if PTY not ready (but Ctrl+Alt+I path already verified)
- DCS escape: base64 encoding prevents shell interference
- Only works when `use_ssh_tmux_wrapper = true` (when wrapper is off, no SSH DCS → no auto-fallback)

## Testing Strategy

- `cargo check` compilation
- Manual SSH to remote host without tmux
- Verify nested SSH (local→A→B) is not auto-tracked

## Spec Patches

None
