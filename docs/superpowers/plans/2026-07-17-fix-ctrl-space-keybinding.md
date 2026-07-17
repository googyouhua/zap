---
change: fix-ctrl-space-keybinding
design-doc: (hotfix, no design doc needed)
base-ref: 8a8ae0f1964769638a5edf643c54fe47b826df2d
---

# Plan: Fix Ctrl+Space Keybinding Conflict with IME

## Changes
Trivial search-and-replace: change `ctrl-space` to `ctrl-alt-space` in 3 files.

| # | File | Change |
|---|------|--------|
| 1 | `app/src/util/bindings.rs:407` | `ctrl-space` → `ctrl-alt-space` for `NewAgentModePane` |
| 2 | `app/src/editor/accept_autosuggestion_keybinding_view.rs:44-48` | `CTRL_SPACE_KEYSTROKE` → `CTRL_ALT_SPACE_KEYSTROKE`, value `ctrl-alt-space` |
| 3 | `app/src/settings_view/features_page.rs:725-729` | `CTRL_SPACE_KEYSTROKE` → `CTRL_ALT_SPACE_KEYSTROKE`, value `ctrl-alt-space` |
| 4 | Run `cargo check` | Verify compilation |
| 5 | Commit | `fix: change ctrl-space to ctrl-alt-space to avoid IME conflict` |
