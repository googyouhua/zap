## Why

`Ctrl+Shift+\` 与打开左侧面板的快捷键冲突，导致手动分割块快捷键无效。需要改用未占用的 `Ctrl+B`。

## What Changes

- 快捷键从 `ctrl-shift-\` 改为 `ctrl-b`

## Capabilities

### Modified Capabilities
- `manual-block-split`: 快捷键从 Ctrl+Shift+\ 改为 Ctrl+B

## Impact

- `app/src/terminal/view/init.rs`: 修改 FixedBinding 的键名
