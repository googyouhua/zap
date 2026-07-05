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
