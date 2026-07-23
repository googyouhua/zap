## 1. Remove discard_on_blur from SubmittableTextInput

- [x] 1.1 Remove `discard_on_blur` field from struct
- [x] 1.2 Remove `discard_on_blur()` builder method
- [x] 1.3 Simplify `EditorEvent::Blurred` handler to pure event emission

## 2. Update SSH denylist subscriber

- [x] 2.1 Remove `.discard_on_blur(true)` from builder chain
- [x] 2.2 Add buffer clearing in `handle_denylisted_ssh_editor_event::Blurred`

## 3. Verify

- [x] 3.1 `cargo check` passes
