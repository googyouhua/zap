## Context

`SubmittableTextInput` delegates focus events from its inner `EditorView` to subscribers. The `Escape` variant follows a pure-event pattern (component emits, subscriber decides). However, `Blurred` had both a `discard_on_blur` flag (component decides) and a `Blurred` event (subscriber also notified). This inconsistency means every new consumer needs to learn about the flag, and adding new blur behaviors requires modifying the component.

## Goals / Non-Goals

**Goals:**
- Remove `discard_on_blur` field and builder from `SubmittableTextInput`
- Make `Blurred` a pure event matching the `Escape` pattern
- Move buffer-clearing decision to the SSH denylist subscriber

**Non-Goals:**
- No changes to other input components
- No changes to the `EditorEvent::Blurred` emission from `EditorView`
- No behavioral changes for existing subscribers that already use `Blurred => {}`

## Decisions

1. **Remove `discard_on_blur` entirely** — the field, builder method, and all references. The subscriber uses `handle.update()` on the received `ViewHandle` to call `clear_buffer`. This keeps the component agnostic of consumer intent.

2. **Match `Escape` exactly** — `EditorEvent::Blurred => ctx.emit(SubmittableTextInputEvent::Blurred)` is a single-line handler, identical in structure to the `Escape` handler.

3. **SSH denylist uses `_handle.update()`** — the subscriber gets the `ViewHandle<SubmittableTextInput>` from the event subscription, uses it to update the input and call `clear_buffer` on the inner editor.

## Risks / Trade-offs

- **Minimal risk**: the change is small (3 files, net -10 lines). The new handler path matches an already-established pattern. No new dependencies.
- **No API breakage for external callers**: `SubmittableTextInputEvent::Blurred` already existed; only the internal field is removed.
