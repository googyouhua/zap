## 1. Data Layer: Remove send_mode

- [x] 1.1 Update `up.sql` migration: remove `send_mode` column from `quick_credentials` CREATE TABLE
- [x] 1.2 Update `db.rs`: update `ensure_columns()` — use new schema without `send_mode`, add `prompt_trigger_rules` table
- [x] 1.3 Update `schema.rs`: remove `send_mode` column from `quick_credentials` table definition
- [x] 1.4 Update `QuickCredentialRow` model: remove `send_mode` field
- [x] 1.5 Update `QuickCredential` domain type: remove `send_mode` field (keep `SendMode` enum)
- [x] 1.6 Update `repository.rs`: remove `send_mode` from create/update/Row conversion
- [x] 1.7 Update `lib.rs` re-exports if needed

## 2. Data Layer: Add prompt_trigger_rules

- [x] 2.1 Define `PromptTriggerRule` struct in `types.rs` (id, keyword, send_mode)
- [x] 2.2 Define `DEFAULT_KEYWORDS` constant (PasswordOnly + UsernameThenPassword groups)
- [x] 2.3 Add `prompt_trigger_rules` table to `schema.rs`
- [x] 2.4 Add `PromptTriggerRuleRow` to `model.rs`
- [x] 2.5 Add CRUD functions to `repository.rs` (list_rules, add_rule, remove_rule, reset_rules_to_defaults)
- [x] 2.6 Export new functions from `lib.rs`
- [x] 2.7 Update `db.rs` `ensure_columns()` to create `prompt_trigger_rules` table
- [x] 2.8 Run `cargo check -p persistence -p warp_quick_credential` to verify

## 3. Settings UI: Remove send_mode dropdown

- [x] 3.1 Remove `send_mode_input` field from `QuickCredentialsPageView` struct
- [x] 3.2 Remove SendMode dropdown rendering from `render_form_mode()`
- [x] 3.3 Remove send_mode from `handle_save()` logic
- [x] 3.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 4. Settings UI: Add trigger keywords section

- [x] 4.1 Add `trigger_rules: Vec<PromptTriggerRule>` field + search state to `QuickCredentialsPageView`
- [x] 4.2 Add `load_rules()` / save helpers that call repository CRUD
- [x] 4.3 Render keyword groups (PasswordOnly chips + UsernameThenPassword chips) above credential list
- [x] 4.4 Implement "+ Add" inline input that adds keyword to repository
- [x] 4.5 Implement "×" on chip that removes keyword from repository
- [x] 4.6 Implement "Reset" button that calls `reset_rules_to_defaults()`
- [x] 4.7 Run `cargo check -p warp --features quick_credential_input` to verify

## 5. Prompt Detection

- [x] 5.1 Create `app/src/terminal/prompt_detection.rs` with `PromptType` enum and `classify_prompt()` function
- [x] 5.2 `classify_prompt()` loads trigger rules and compares PTY output against keywords
- [x] 5.3 Add module declaration in terminal mod.rs
- [x] 5.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 6. Auto-send Integration

- [x] 6.1 Modify `spawn_onekey_prompt_listener` to load Quick Credentials on prompt detection
- [x] 6.2 If exactly 1 credential: call `send_quick_credential` with mode based on prompt type
- [x] 6.3 If 0 or multiple: fall back to existing OneKey show logic
- [x] 6.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 7. Panel Event Update

- [x] 7.1 Update `QuickCredentialPanelEvent::ItemSelected` to carry `mode: SendMode` field
- [x] 7.2 Update panel button handlers to emit mode separately (not set on credential)
- [x] 7.3 Update `on_quick_credential_panel_event` in terminal view.rs to use event's mode
- [x] 7.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 8. Test and Build

- [x] 8.1 Update existing unit tests for data layer changes
- [x] 8.2 Add unit tests for `classify_prompt()`
- [x] 8.3 Run relevant tests (warp_quick_credential, prompt_detection, quick_credential_data_source, persistence all pass)
- [x] 8.4 Run `scripts/run` to manually verify auto-send behavior
