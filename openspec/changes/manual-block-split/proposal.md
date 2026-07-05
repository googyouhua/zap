## Why

当终端 shell 集成未启用或失败时，终端的块模式无法自动分割输出内容，所有输出堆积在单个 active block 中，用户无法利用块的结构化视图（块选择、块跳转、块过滤等）管理输出。

## What Changes

- 新增 `Ctrl+Shift+\` 快捷键在终端中手动分割块
- 新增 `TerminalAction::SplitBlock` action 变体
- 实现块分割逻辑：在 active block 的输出网格中，以光标所在行为分割点，提取光标行文本作为新块的 command，光标行后内容作为新块的 output

## Capabilities

### New Capabilities
- `manual-block-split`: 在终端中通过快捷键手动分割块的能力

### Modified Capabilities

无

## Impact

- `app/src/terminal/view/action.rs`: 新增 `TerminalAction::SplitBlock` 变体
- `app/src/terminal/view/view.rs`: 添加 action 处理分支
- `app/src/terminal/view/init.rs`: 注册 `ctrl-shift-\` 快捷键绑定
- `app/src/terminal/model/blocks.rs`: 可能新增 BlockList 分割方法
- `app/src/terminal/model/blockgrid.rs`: 利用已有的 `BlockGrid::split()`
