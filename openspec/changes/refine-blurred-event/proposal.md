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
