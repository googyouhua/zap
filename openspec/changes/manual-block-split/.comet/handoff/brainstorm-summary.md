# Brainstorm Summary

- Change: manual-block-split
- Date: 2026-07-03

## Confirmed Technical Approach

**方案 A：BlockList 层完整分割**

1. 新增 `TerminalAction::SplitBlock` 变体
2. `BlockList::split_active_block_at_cursor(cursor_line: usize)` 封装分割流程
   - 用 `BlockGrid::split(N)` 切分 output grid
   - 提取第 N 行文本（trim 判空）作为新 block command
   - 通过 `create_new_block()` + `blocks_mut()` 插入新块
   - 调用 `update_block_height_indices()` 更新 SumTree
3. view.rs handler 获取光标位置后调用上述方法，完成后 `ctx.notify()`
4. init.rs 注册 `ctrl-shift-\` → `TerminalAction::SplitBlock` 的 FixedBinding

## Key Trade-offs and Risks

- 内容不重复：`BlockGrid::split()` 物理移动行，非 copy
- 渲染正确：SumTree 更新 + ctx.notify() 触发重渲染
- 空行判断：command text trim 后判空
- 光标在 header_grid 时不分割

## Testing Strategy

- BlockList 单元测试：正常分割、首行分割、末行空行跳过、光标不在 output_grid
- 无需集成测试

## Spec Patches

None（spec 已经在 open 阶段写好了）
