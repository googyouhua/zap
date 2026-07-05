## 方案

将 `app/src/terminal/view/init.rs` 中 `SplitBlock` 的 FixedBinding 键名从 `"ctrl-shift-\\"` 改为 `"ctrl-b"`。

`Ctrl+B` 在终端 context（`id!("Terminal")`）中未被其他绑定占用，仅在编辑器 context 中使用，无冲突。
