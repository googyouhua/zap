# Comet Design Handoff

- Change: manual-block-split
- Phase: design
- Mode: compact
- Context hash: 070ed7e00134735409513c7c328e9b03e1e0e06cf6355621fbf45fc229bc09d8

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/manual-block-split/proposal.md

- Source: openspec/changes/manual-block-split/proposal.md
- Lines: 1-26
- SHA256: 59d3aaa69a310a8ec6eb27db940b053cbc9589849c650eefcbdfdd7e0c0cacf4

```md
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
```

## openspec/changes/manual-block-split/design.md

- Source: openspec/changes/manual-block-split/design.md
- Lines: 1-78
- SHA256: 54c9c382887fe4766ef5c6cba6dc024554be9d0cb9f16ae4539a698a11c6aec0

```md
## Context

当 shell 集成失败时，`BlockList` 中的 `active_block` 持续累积输出但不会自动分割。Block 模型包含 `header_grid`（提示符+命令）和 `output_grid`（输出），但已实现的 `BlockGrid::split()`（`blockgrid.rs:135`）和 `BlockList::create_new_block()`（`blocks.rs:2585`）当前没有用户可见的触发方式。

## Goals / Non-Goals

**Goals:**
- 新增 `Ctrl+Shift+\` 快捷键手动分割 active block
- 分割逻辑：以光标行为分界，光标行提取为新块的 command
- 新的块具有完整块结构（header + output）
- 已有单元测试覆盖

**Non-Goals:**
- 用户可配置的快捷键（使用 FixedBinding）
- 选区模式分割（情况 2，留待后续）
- 修改 shell 集成自动分割逻辑

## Decisions

**1. Action 命名：`TerminalAction::SplitBlock`**

不携带参数，分割点由当前光标位置决定。

**2. 分割算法**

输入：
- `active_block` 的 `output_grid`
- 终端光标在 block 中的行号 N（0-based）

输出：
- Block 1（原 active block）：`output_grid` 截断到第 N-1 行
- Block 2（新块）：command = 提取第 N 行文本，output = 第 N+1 行起

步骤：
1. 获取 `active_block.output_grid` 的行数 `total_lines`
2. 验证 N 在 `[0, total_lines)` 范围内
3. 用 `BlockGrid::split(N)` 将 output grid 一分为二
4. 读取分割后第二块的第 0 行文本（即原第 N 行）作为 command
5. 从第二块 output grid 移除第 0 行（command 行不进入 output）
6. 创建新 Block，command = 提取的文本，output = 第二块剩余内容
7. 将新 Block 插入到原 active block 之后

**3. 光标位置获取**

通过 `TerminalModel::active_cursor_position()` 或类似方法获取当前光标在终端网格中的坐标，映射到 active block 的 output grid。

**4. 快捷键注册**

```rust
FixedBinding::new(
    "ctrl-shift-\\",
    TerminalAction::SplitBlock,
    id!("Terminal") & !id!("IMEOpen"),
)
```

使用 FixedBinding（非 EditableBinding），因为这是标准终端操作。

**5. 缺失的 API**

BlockList 新增方法：`split_active_block_at_cursor(cursor_line: usize)` 封装整个分割流程。

**6. 新块拥有完整块能力**

新块通过 `BlockList::create_new_block()` 创建，通过 `BlockList::insert_non_block_item_before_block()` 或直接操作 `blocks_mut()` 插入。由于是完整的 `Block` 结构，天然支持：
- 块选择（鼠标点击 + 键盘 `SelectNextBlock`/`SelectPriorBlock`）
- 块上下文菜单
- 块过滤（`filter_block_output`）
- 复制 command / output
- 块分隔线与块 banner
- AI assistant 关联

## Risks / Trade-offs

- **块高度重算**: 插入新块后需调用 `update_block_height_indices()` 更新 SumTree
- **视口位置**: 分割后视口可能需要保持当前位置，而不是跳转到底部
- **光标在 output_grid 外**: 当光标不在 output_grid 时（如在 header_grid），应无操作
- **空输出行**: 如果光标在输出末尾的空行，分割后新块可能 command 为空——这种情况下不执行分割
```

## openspec/changes/manual-block-split/tasks.md

- Source: openspec/changes/manual-block-split/tasks.md
- Lines: 1-23
- SHA256: d8c6258ff5f8782fe6e7415f6ac5a96fc4a15a5dc986c4c5b44a015d44de4f95

```md
## 1. 新增 SplitBlock Action

- [ ] 1.1 在 `app/src/terminal/view/action.rs` 的 `TerminalAction` 枚举中添加 `SplitBlock` 变体
- [ ] 1.2 在 `action.rs` 的 `fmt::Debug` 实现中添加 `SplitBlock` 分支

## 2. 实现 BlockList 分割方法

- [ ] 2.1 在 `app/src/terminal/model/blocks.rs` 的 `BlockList` 中添加 `split_active_block_at_cursor(cursor_line: usize)` 方法
- [ ] 2.2 方法内部实现：获取 cursor_line 在 output_grid 中的文本、调用 `BlockGrid::split()`、创建新 Block、插入到 BlockList

## 3. Action 处理

- [ ] 3.1 在 `app/src/terminal/view/view.rs` 的 `handle_action` 中添加 `TerminalAction::SplitBlock` 分支
- [ ] 3.2 处理逻辑：获取 active block 和光标位置，调用 `BlockList::split_active_block_at_cursor()`

## 4. 注册快捷键

- [ ] 4.1 在 `app/src/terminal/view/init.rs` 中添加 `ctrl-shift-\` → `TerminalAction::SplitBlock` 的 `FixedBinding`

## 5. 验证

- [ ] 5.1 `cargo check` 通过
- [ ] 5.2 单元测试覆盖
```

## openspec/changes/manual-block-split/specs/manual-block-split/spec.md

- Source: openspec/changes/manual-block-split/specs/manual-block-split/spec.md
- Lines: 1-34
- SHA256: f5cf6da9bae3c31d90ca38b54cc0890b18237cc3fe63dda1c24f61d004166ba0

```md
## ADDED Requirements

### Requirement: 手动分割块

系统在终端中提供一个快捷键，当用户按下时，根据当前终端光标位置分割 active block。

#### Scenario: 在输出行中分割

- **GIVEN** shell 集成未启用，终端显示块模式
- **AND** active block 的 output 包含多行内容
- **WHEN** 用户将光标定位在某输出行并按下 Ctrl+Shift+\
- **THEN** 光标行之前的输出保留在原 block
- **AND** 光标行文本作为新 block 的 command
- **AND** 光标行之后的输出作为新 block 的 output

#### Scenario: 在最后一行分割

- **GIVEN** 光标在 active block output 的最后一行
- **AND** 该行为空行
- **WHEN** 用户按下 Ctrl+Shift+\
- **THEN** 不执行分割（空 command 无意义）

#### Scenario: 在首行分割

- **GIVEN** 光标在 active block output 的第一行
- **WHEN** 用户按下 Ctrl+Shift+\
- **THEN** 第一行文本提取为新 block 的 command
- **AND** 其余行作为新 block 的 output
- **AND** 原 block output 为空

#### Scenario: 快捷键上下文

- **WHEN** 终端失焦或 IME 打开
- **THEN** 快捷键不生效
```

