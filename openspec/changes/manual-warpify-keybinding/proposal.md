## Why

多跳 SSH 场景中（ssh → 跳板机 → ssh → 目标机），Warp 无法自动检测内层 shell 并显示 warpify banner。`Ctrl+I` 快捷键因缺少 `SubshellBanner` context 条件而不生效。需要一个无条件写入 shell init 脚本的快捷键。

## What Changes

- 在 `app/src/terminal/view/init.rs` 中注册 `Ctrl+Alt+I` → `TerminalAction::TriggerSubshellBootstrap`
- Context predicate 不加 `SubshellBanner` 或 `LongRunningCommand` 条件，在任何终端上下文中均可触发

## Impact

- `app/src/terminal/view/init.rs`: 新增一行 EditableBinding
