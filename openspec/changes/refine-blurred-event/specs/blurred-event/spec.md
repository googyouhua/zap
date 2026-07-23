# Blurred Event

## Requirements

- `SubmittableTextInput` emits `SubmittableTextInputEvent::Blurred` when the inner `EditorView` fires `EditorEvent::Blurred`
- The emission must happen on every blur, regardless of input state
- The component must NOT perform any side effects (e.g., clearing buffer) — consumers decide
- Consumers subscribe via `ctx.subscribe_to_view()` and match `SubmittableTextInputEvent::Blurred` in their handler

## Consumer Responsibilities

- A consumer that wants to discard input on blur calls `handle.update(ctx, |input, ctx| { input.editor().update(ctx, |editor, ctx| { editor.clear_buffer(ctx); }); });`
- A consumer that wants to ignore blur matches `SubmittableTextInputEvent::Blurred => {}`
