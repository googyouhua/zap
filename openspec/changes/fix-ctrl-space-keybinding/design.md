# Design: Replace Ctrl+Space Defaults with Ctrl+Alt+Space

## Solution
Replace all `ctrl-space` default keystrokes with `ctrl-alt-space`. This key is:
- Not used by OS IME switching on any major platform
- Already used in the codebase pattern (e.g., `ctrl-alt-t` for new tab)
- Easy to type and remember

## Files to change

| File | Line | Change |
|------|------|--------|
| `app/src/util/bindings.rs` | 407 | `ctrl-space` → `ctrl-alt-space` for `CustomAction::NewAgentModePane` |
| `app/src/editor/accept_autosuggestion_keybinding_view.rs` | 44-48 | `CTRL_SPACE_KEYSTROKE` → `CTRL_ALT_SPACE_KEYSTROKE` with key `ctrl-alt-space` |
| `app/src/settings_view/features_page.rs` | 725-729 | `CTRL_SPACE_KEYSTROKE` → `CTRL_ALT_SPACE_KEYSTROKE` with key `ctrl-alt-space` |

## Not changed
- `ctrl-shift-space` for `AttachSelectionAsAgentModeContext` — does not conflict with IME
- User-editable `keybindings.yaml` — users can rebind as they wish
