---
comet_change: manual-block-split
role: technical-design
canonical_spec: openspec
---

# 手动分割块 - 技术设计

## 背景

当 shell 集成失败时，终端块模式无法自动分割输出。用户在 active block 中积累了大量输出，但无法利用块模式的结构化功能（块选择、跳转、过滤等）。本设计提供 `Ctrl+Shift+\` 快捷键手动分割块。

## 实现方案

### 1. TerminalAction::SplitBlock

在 `app/src/terminal/view/action.rs` 的 `TerminalAction` 枚举中添加：

```rust
SplitBlock,
```

### 2. BlockList::split_active_block_at_cursor()

在 `app/src/terminal/model/blocks.rs` 的 `BlockList` 中添加方法：

**签名**:
```rust
pub fn split_active_block_at_cursor(&mut self, cursor_line: usize) -> Option<(BlockIndex, BlockIndex)>
```

返回分割后两个 block 的 index，用于 UI 更新。返回 None 表示不执行分割。

**算法**:

```
输入: cursor_line = N (0-based, 在 output_grid 中)
前提: N < active_block.output_grid.lines()

1. 获取 active_block 的 output_grid handler
2. 获取 output_grid 总行数 total_lines
3. 获取第 N 行文本 command_text
4. 如果 command_text.trim().is_empty() → 不分割，返回 None
5. 调用 BlockGrid::split(N) 切分 output_grid:
   - grid_a: 行 0..N-1 留在原地
   - grid_b: 行 N..total_lines-1 移至新 grid
6. 读取 grid_b 的第 0 行文本（即原第 N 行）= command_text
7. 从 grid_b 移除第 0 行（command 行不进入 output）
8. 创建新 Block:
   - command = command_text
   - output = grid_b 的剩余内容
9. 将新 Block 插入到 blocks Vec 中和 SumTree 中
10. 更新块高度: update_block_height_indices()
11. 返回 Some((old_block_index, new_block_index))
```

### 3. Action 处理

在 `app/src/terminal/view/view.rs` 的 `handle_action` 中添加：

```rust
TerminalAction::SplitBlock => {
    let cursor = model.cursor_position();
    let active_idx = model.block_list().active_block_index();
    // 将 cursor 映射到 active block 的 output grid 坐标
    if let Some(cursor_line) = map_cursor_to_output_grid(cursor, active_idx) {
        if model.block_list_mut().split_active_block_at_cursor(cursor_line).is_some() {
            ctx.notify();
        }
    }
}
```

光标映射逻辑：如果光标不在 output grid（如在 header/command grid），返回 None，不分割。

### 4. 快捷键注册

在 `app/src/terminal/view/init.rs` 中，在现有 Ctrl+Shift+V paste 绑定后添加：

```rust
FixedBinding::new(
    "ctrl-shift-\\",
    TerminalAction::SplitBlock,
    id!("Terminal") & !id!("IMEOpen"),
),
```

### 5. 边界情况

| 情况 | 处理 |
|------|------|
| 光标在 header_grid | 不分割（光标不在 output_grid） |
| N = 0（首行） | 第 0 行作为 command，原 block output 为空 |
| N = last row & 空行 | command trim 后空，不分割 |
| N = last row & 非空 | command = 最后一行，新 block output 为空 |
| 无内容 | 不分割（没有可分割的内容） |

### 6. 测试

在 `blocks_tests.rs` 中添加：

- `test_split_active_block_middle`
- `test_split_active_block_first_row`
- `test_split_active_block_last_row_empty`
- `test_split_active_block_cursor_not_in_output`

## 相关风险

- **视图同步**: action handler 返回后需要 `ctx.notify()` 触发渲染更新
- **SumTree 一致性**: 插入 BlockHeightItem 后必须调用 update_block_height_indices()
- **BlockId 唯一性**: `create_new_block()` 自动生成 uuid 作为 BlockId，无冲突
