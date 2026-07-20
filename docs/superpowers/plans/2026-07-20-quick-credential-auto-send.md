---
change: quick-credential-auto-send
design-doc: docs/superpowers/specs/2026-07-20-quick-credential-auto-send-design.md
base-ref: 4280ff876a6e3d21c5d86aef06c37633412809c5
---

## Execution Plan

### Phase 1: Data Layer — Remove send_mode

**Files to modify**: up.sql, db.rs, schema.rs, model.rs, types.rs, repository.rs, lib.rs

1. **up.sql**: remove `send_mode` column from CREATE TABLE
2. **db.rs**: update CREATE TABLE IF NOT EXISTS (no send_mode), add `prompt_trigger_rules` table to ensure_columns
3. **schema.rs**: remove `send_mode` column from table definition
4. **model.rs** (`QuickCredentialRow`): remove `send_mode` field, add `PromptTriggerRuleRow`
5. **types.rs** (`QuickCredential`): remove `send_mode` field, keep `SendMode` enum, add `PromptTriggerRule` struct + `DEFAULT_KEYWORDS`
6. **repository.rs**: remove send_mode from create/update, add CRUD for prompt_trigger_rules
7. **lib.rs**: export new types/functions

### Phase 2: Settings UI — Remove dropdown + add keywords section

**Files to modify**: quick_credentials_page.rs

1. Remove `send_mode_input` field, dropdown rendering, save logic
2. Add trigger keywords section (chips, +Add, × delete, Reset)
3. Load/display rules from repository

### Phase 3: Prompt Detection + Auto-send

**Files to modify**: terminal/prompt_detection.rs (new), view.rs, quick_credential_sender.rs

1. `prompt_detection.rs`: `PromptType` enum, `classify_prompt()` function
2. Modify `spawn_onekey_prompt_listener` → classify + load QCs + auto-send or fallback

### Phase 4: Panel Event Update

**Files to modify**: quick_credential/view.rs, view.rs (terminal)

1. `ItemSelected` gains `mode: SendMode`
2. Button handlers set mode separately
3. Terminal event handler passes mode to sender
