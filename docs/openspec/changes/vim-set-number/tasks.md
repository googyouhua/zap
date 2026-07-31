# tasks.md

- [x] 1. 在 `EditorView` 中将 `vim_force_insert_mode` 改为 `pub`（`app/src/editor/view/mod.rs:5426`）
- [x] 2. 在终端输入的 `handle_editor_event` 中将 `EditorEvent::ExCommand` 改为插入 `:` 并切换到插入模式（`app/src/terminal/input.rs:9418-9426`）
