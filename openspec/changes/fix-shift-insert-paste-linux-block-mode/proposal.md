## Why

Linux block 模式下 Shift+Insert 粘贴快捷键不生效。

## What Changes

- 修复 Linux block 模式下 Shift+Insert 无法粘贴的问题
- 确保 block 模式的 key 事件路由能正确处理 Shift+Insert 组合键

## Capabilities

### New Capabilities
_(无新能力)_

### Modified Capabilities
_(无 spec 级别变更)_

## Impact

- `app/src/terminal/` — block 模式键盘事件处理
- `crates/warp_terminal/` — block 模式的输入分发
