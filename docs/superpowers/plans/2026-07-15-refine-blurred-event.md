---
change: refine-blurred-event
design-doc: docs/superpowers/specs/2026-07-15-refine-blurred-event-design.md
base-ref: 8a8ae0f1964769638a5edf643c54fe47b826df2d
---

# Implementation Plan: Blurred Event Refinement

## Tasks (All Complete)

1. Remove `discard_on_blur` field + builder from `SubmittableTextInput`
2. Simplify `EditorEvent::Blurred` handler to pure event emission
3. Remove `.discard_on_blur(true)` from SSH denylist builder chain
4. Add `clear_buffer` in `handle_denylisted_ssh_editor_event::Blurred`
5. `cargo check` — pass
