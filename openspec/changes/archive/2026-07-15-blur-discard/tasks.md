## 1. 核心实现

- [x] 1.1 SubmittableTextInput: 新增 `discard_on_blur` 字段 + builder 方法
- [x] 1.2 SubmittableTextInputEvent: 新增 `Blurred` variant
- [x] 1.3 handle_editor_event 处理 `EditorEvent::Blurred` — 始终 emit Blurred, discard_on_blur 时清空 buffer

## 2. 调用点适配

- [x] 2.1 warpify_page.rs: SSH denylist 编辑器 `.discard_on_blur(true)`
- [x] 2.2 warpify_page.rs: 3 处 handler 添加 `Blurred => {}` 穷尽匹配
- [x] 2.3 ai_page.rs: handler 添加 `Blurred => {}` 穷尽匹配

## 3. 验证

- [x] 3.1 `cargo check` 通过
- [x] 3.2 提交 commit
