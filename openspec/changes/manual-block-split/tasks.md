## 1. 新增 SplitBlock Action

- [x] 1.1 在 `app/src/terminal/view/action.rs` 的 `TerminalAction` 枚举中添加 `SplitBlock` 变体
- [x] 1.2 在 `action.rs` 的 `fmt::Debug` 实现中添加 `SplitBlock` 分支

## 2. 实现 BlockList 分割方法

- [x] 2.1 在 `app/src/terminal/model/blocks.rs` 的 `BlockList` 中添加 `split_active_block_at_cursor(cursor_line: usize)` 方法
- [x] 2.2 方法内部实现：获取 cursor_line 在 output_grid 中的文本、调用 `BlockGrid::split()`、创建新 Block、插入到 BlockList

## 3. Action 处理

- [x] 3.1 在 `app/src/terminal/view/view.rs` 的 `handle_action` 中添加 `TerminalAction::SplitBlock` 分支
- [x] 3.2 处理逻辑：获取 active block 和光标位置，调用 `BlockList::split_active_block_at_cursor()`

## 4. 注册快捷键

- [x] 4.1 在 `app/src/terminal/view/init.rs` 中添加 `ctrl-shift-\` → `TerminalAction::SplitBlock` 的 `FixedBinding`

## 5. 验证

- [x] 5.1 `cargo check` 通过
- [x] 5.2 单元测试覆盖（已编写，环境 OOM 无法编译测试）
