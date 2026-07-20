## Context

Quick Credentials currently store a `send_mode` per credential (PasswordOnly / UsernameThenPassword), set at creation time. The user must manually invoke the credential panel via Ctrl+Shift+U. The terminal already has OneKey prompt detection that shows a context menu on password prompts, but it only sends the password directly (no username+password option, no send-mode selection).

The goal is to: (1) remove `send_mode` from storage and let the user choose at send time via the panel, (2) add keyword-triggered auto-send that classifies prompts and picks the right send mode, (3) make trigger keywords configurable in the settings UI.

## Goals / Non-Goals

**Goals:**
- Remove `send_mode` column from `quick_credentials` DB table, model, and settings form
- Add `prompt_trigger_rules` table to the same SQLite DB, with keyword → SendMode mapping
- Provide default keywords: PasswordOnly={password, passphrase}, UsernameThenPassword={login, username, user, name, email, account}
- Add settings UI to view, add, delete keywords, and reset to defaults
- Modify `spawn_onekey_prompt_listener` to classify prompts by keyword type
- When exactly 1 credential exists, auto-send with the mode matching the prompt type
- When 0 or multiple credentials, fall back to existing OneKey menu
- Update panel `ItemSelected` event to carry `mode: SendMode` as a separate field
- Preserve existing OneKey behavior for SSH credentials

**Non-Goals:**
- No changes to SSH credential management or OneKey menu for SSH
- No AI integration for credential selection
- No per-credential auto-send opt-in/opt-out
- No credential auto-creation from detected prompts

## Decisions

### D1: Remove send_mode entirely, don't keep it as a runtime default

- **Chosen:** Delete `send_mode` from `QuickCredentialRow`, `QuickCredential`, repo CRUD, DB column, settings form
- **Rationale:** The field is always overridden at send time by the panel's two-button choice or by auto-send's prompt classification. Persisting it is dead data.
- **Alternative:** Keep it as a "default" and allow override at send time. Rejected: adds unnecessary complexity; user explicitly wants to choose at send time only.

### D2: Store trigger rules in the same SQLite DB

- **Chosen:** New `prompt_trigger_rules` table (id, keyword, send_mode) in the Quick Credential DB, managed via `db.rs` `ensure_columns()`
- **Rationale:** Quick Credential settings already live in this DB; no new dependency needed. CRUD is simple and co-located with the credential repository.
- **Alternative:** Store in a JSON config file. Rejected: inconsistent with the rest of the feature's storage pattern.
- **Alternative:** Use the warp settings system. Rejected: too heavy for a simple key-value list; the rules are tightly coupled to credential CRUD.

### D3: New `prompt_detection.rs` module

- **Chosen:** New file `app/src/terminal/prompt_detection.rs` housing `PromptType` enum, `classify_prompt()` function
- **Rationale:** Keeps detection logic separate from the large `view.rs` terminal file. Easy to unit test. Reused by the modified prompt listener.

### D4: Auto-send only when exactly 1 credential exists

- **Chosen:** Load Quick Credentials on prompt detection; if count == 1, auto-send silently; otherwise fall back to OneKey menu
- **Rationale:** If multiple credentials exist, the correct one is ambiguous — better to let the user pick. If none exist, nothing to send.
- **Alternative:** Use fuzzy matching against prompt context (e.g., "password for my-server:" matches credential label "my-server"). Rejected: more complex, higher risk of false positives, out of scope for this change.

### D5: Panel ItemSelected carries mode separately

- **Chosen:** `ItemSelected { credential: QuickCredential, mode: SendMode }` instead of setting `credential.send_mode`
- **Rationale:** `QuickCredential` no longer has a `send_mode` field. The mode is only meaningful at send time, so it belongs in the event.

## Risks / Trade-offs

- **Auto-send false positive:** If the terminal output happens to match a trigger keyword in non-prompt context, a credential could be sent accidentally. Mitigation: the PW/username prompt regex patterns are well-established (already used by OneKey); keywords are user-configurable so they can remove false-triggering words.
- **Multi-credential fallback shows OneKey menu without SendMode choice:** The OneKey menu only sends password directly. Mitigation: this is existing behavior; the user can always open the panel via Ctrl+Shift+U for SendMode selection. A future improvement could integrate SendMode into the OneKey menu.
- **Old DBs still have send_mode column:** `ensure_columns()` only adds missing columns, doesn't remove existing ones. Mitigation: the column is simply ignored by the updated code. Can be cleaned up in a future migration if needed.
