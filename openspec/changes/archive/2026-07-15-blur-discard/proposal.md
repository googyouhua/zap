## Why

SubmittableTextInput 的 SSH denylist host 输入框在失焦时保留未提交内容，用户需要手动按 Esc 或提交才能清除。期望点击外部区域时自动丢弃未提交的输入。

## What Changes

- `SubmittableTextInputEvent` 新增 `Blurred` variant
- `SubmittableTextInput` 新增 `discard_on_blur` 字段 + builder，失焦时清空 buffer 并 emit `Blurred`
- SSH denylist 编辑器启用 `discard_on_blur(true)`
- 所有现有 `SubmittableTextInputEvent` 的 `match` 添加 `Blurred => {}` 穷尽处理

## Capabilities

- `blur-discard`: SubmittableTextInput 失焦时丢弃未提交内容

## Impact

- `app/src/view_components/submittable_text_input.rs` — 新增字段、builder、Blurred handler 和 event variant
- `app/src/settings_view/warpify_page.rs` — SSH denylist 编辑器启用 discard_on_blur + 穷尽匹配
- `app/src/settings_view/ai_page.rs` — 穷尽匹配
