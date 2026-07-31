## 方案

在 `app/src/terminal/view/init.rs` 中现有 warpify 绑定块后添加：

```rust
EditableBinding::new(
    "terminal:manual_warpify",
    crate::t!("keybinding-desc-terminal-warpify-subshell"),
    TerminalAction::TriggerSubshellBootstrap,
)
.with_key_binding("ctrl-alt-i")
.with_context_predicate(id!("Terminal") & !id!("IMEOpen")),
```

`TriggerSubshellBootstrap` 的 handler 会调用 `trigger_subshell_bootstrap(None, false, ctx)` → `write_init_subshell_bytes_to_pty()` → 写入 init 脚本到 PTY。无 banner 时 banner dismiss 和 block recording 为空操作。
