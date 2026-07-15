## Context

`SubmittableTextInput` 包装了一个 `EditorView` + submit 按钮。失焦时只有 `EditorView::Blurred` 事件可订阅，但 `SubmittableTextInput` 的 `handle_editor_event` 用 `_ => {}` 吞掉了它。

## Decisions

1. **Blurred 始终 emit，不依赖 discard_on_blur**
   parent 始终要知道失焦事件，即使不清空 buffer。

2. **discard_on_blur 只控制 buffer 清理**
   不影响 `Blurred` 事件的发送。两个独立关心。

3. **默认 false，向后兼容**
   现有 13 处 `SubmittableTextInput` 调用点完全不受影响。

## Risks

- 点击提交按钮时不会误清空：`on_try_submit` 在事件派发阶段同步执行，`Blurred` 在后续 `flush_effects` 中才发生。
