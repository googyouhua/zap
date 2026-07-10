## 修复方案

`ssh_detection.rs:evaluate_warpify_ssh_host` 中，将 `use_ssh_tmux_wrapper` 从 warpification 闸门条件中移除：

```rust
// 修改前:
let should_prompt_ssh_tmux_wrapper = *warpify_settings.enable_ssh_warpification.value()
    && *warpify_settings.use_ssh_tmux_wrapper.value();

// 修改后:
let should_prompt_ssh_tmux_wrapper = *warpify_settings.enable_ssh_warpification.value();
```

当 `use_ssh_tmux_wrapper = false` 时，warpification 流程中 `handle_remote_warpification_is_unavailable` 已经能正确处理：`TmuxNotInstalled` → 若 home writable 则自动回退 shell integration，否则走 install dialog → 取消后露 SshErrorBlock（含 "Warpify Without Tmux" 按钮）。因此仅需修改闸门逻辑一处。
