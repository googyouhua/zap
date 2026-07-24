---
comet_change: refine-blurred-event
role: technical-design
canonical_spec: openspec
---

# Blurred Event Refinement — Design Doc

## Problem

`SubmittableTextInput` has a `discard_on_blur` flag that mixes two concerns: the component emits the blur event, AND decides whether to clear the buffer. This breaks the pattern established by `Escape` (pure event — component emits, subscriber decides).

## Solution

Remove `discard_on_blur` entirely. Make `Blurred` a pure event, matching `Escape`:

```rust
// Before
EditorEvent::Blurred => {
    if self.discard_on_blur {
        self.editor.update(ctx, |e, ctx| e.clear_buffer(ctx));
    }
    ctx.emit(SubmittableTextInputEvent::Blurred);
}

// After
EditorEvent::Blurred => ctx.emit(SubmittableTextInputEvent::Blurred),
```

Consumers that need to discard on blur do so explicitly:

```rust
SubmittableTextInputEvent::Blurred => {
    handle.update(ctx, |input, ctx| {
        input.editor().update(ctx, |editor, ctx| {
            editor.clear_buffer(ctx);
        });
    });
}
```

## Files Changed

| File | Change |
|------|--------|
| `app/src/view_components/submittable_text_input.rs` | Remove `discard_on_blur` field + builder; simplify handler |
| `app/src/settings_view/warpify_page.rs` | Remove `.discard_on_blur(true)`; add clear_buffer in `Blurred` handler |
| `app/src/settings_view/ai_page.rs` | No change (already `Blurred => {}`) |

## Testing

- `cargo check` passes with no warnings in the changed code
- No behavioral change: SSH denylist still clears on blur via subscriber
- AI settings page still ignores blur (empty handler)
