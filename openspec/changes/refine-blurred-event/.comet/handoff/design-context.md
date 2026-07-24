# Comet Design Handoff

- Change: refine-blurred-event
- Phase: design
- Mode: compact
- Context hash: 40cd8bfcf4fc21d531c02a0da1e62e2aa60bc4fac4e036f25488de3b91283bf3

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/refine-blurred-event/proposal.md

- Source: openspec/changes/refine-blurred-event/proposal.md
- Lines: 1-28
- SHA256: c531b8b7c2cf72db77e7e65cf13524369d64f2003f41bf8caaa280a0e93f468b

```md
## Why

`SubmittableTextInput` has a `discard_on_blur` flag that duplicates the "consumer decides behavior" pattern already established by `Escape`. This inconsistency adds unnecessary API surface: every new consumer must learn about the flag, and the component itself takes on behavioral responsibility that belongs in the subscriber. Removing the flag makes the component simpler and follows the existing event-handling idiom.

## What Changes

- **Remove** `discard_on_blur` field from `SubmittableTextInput` struct
- **Remove** `discard_on_blur()` builder method
- **Change** `EditorEvent::Blurred` handler to only emit `SubmittableTextInputEvent::Blurred` (no buffer clearing)
- **Add** buffer-clearing logic to the SSH denylist subscriber's `Blurred` handler
- No behavior change for AI settings subscriber (already had empty `Blurred => {}`)

## Capabilities

### New Capabilities

- `blurred-event`: SubmittableTextInput emits a `Blurred` event when its inner editor loses focus. Consumers subscribe to decide how to respond (e.g., discard input, refocus parent).

### Modified Capabilities

*(none — no existing spec-level capability has its requirements changed)*

## Impact

3 files in `app/src/`:
- `view_components/submittable_text_input.rs` — remove field + builder, simplify handler
- `settings_view/warpify_page.rs` — SSH denylist subscriber clears buffer manually
- `settings_view/ai_page.rs` — no change (already matches)
```

## openspec/changes/refine-blurred-event/design.md

- Source: openspec/changes/refine-blurred-event/design.md
- Lines: 1-28
- SHA256: 72f1e00c256a07776f873e8a664b45c5c1becde1de83d61705ee69fcc2601d33

```md
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
```

## openspec/changes/refine-blurred-event/tasks.md

- Source: openspec/changes/refine-blurred-event/tasks.md
- Lines: 1-14
- SHA256: b2531f4df520dab6d8f54de2a73d1342967ac5c5a2bd3873824dd6abdb165d65

```md
## 1. Remove discard_on_blur from SubmittableTextInput

- [x] 1.1 Remove `discard_on_blur` field from struct
- [x] 1.2 Remove `discard_on_blur()` builder method
- [x] 1.3 Simplify `EditorEvent::Blurred` handler to pure event emission

## 2. Update SSH denylist subscriber

- [x] 2.1 Remove `.discard_on_blur(true)` from builder chain
- [x] 2.2 Add buffer clearing in `handle_denylisted_ssh_editor_event::Blurred`

## 3. Verify

- [x] 3.1 `cargo check` passes
```

## openspec/changes/refine-blurred-event/specs/blurred-event/spec.md

- Source: openspec/changes/refine-blurred-event/specs/blurred-event/spec.md
- Lines: 1-13
- SHA256: a9baa229f05e446f443bb582fc1036dbe152478196c7e88b2daa0adc20a0bebe

```md
# Blurred Event

## Requirements

- `SubmittableTextInput` emits `SubmittableTextInputEvent::Blurred` when the inner `EditorView` fires `EditorEvent::Blurred`
- The emission must happen on every blur, regardless of input state
- The component must NOT perform any side effects (e.g., clearing buffer) — consumers decide
- Consumers subscribe via `ctx.subscribe_to_view()` and match `SubmittableTextInputEvent::Blurred` in their handler

## Consumer Responsibilities

- A consumer that wants to discard input on blur calls `handle.update(ctx, |input, ctx| { input.editor().update(ctx, |editor, ctx| { editor.clear_buffer(ctx); }); });`
- A consumer that wants to ignore blur matches `SubmittableTextInputEvent::Blurred => {}`
```

