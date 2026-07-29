# Comet Design Handoff

- Change: quick-credential-auto-send
- Phase: design
- Mode: compact
- Context hash: a75ccc78b3a00c5c64ccb07dbcec429909405dddc6787223c06884cc8e78f0bd

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/quick-credential-auto-send/proposal.md

- Source: openspec/changes/quick-credential-auto-send/proposal.md
- Lines: 1-32
- SHA256: 030bee147728db2736646779c920afce50e68eca70a2bf978f3d0e26d07c7e02

```md
## Why

Currently, Quick Credentials must be manually triggered via Ctrl+Shift+U, and `send_mode` (Password Only vs Username + Password) is set at credential-creation time rather than at send time. This is inconvenient for users who want credentials to auto-fill when password or username prompts appear in the terminal, and who want to choose the send mode reactively instead of presetting it.

## What Changes

- Remove `send_mode` field from the Quick Credential model, DB schema, and settings form
- Add auto-fill trigger keyword rules (configurable per keyword → PasswordOnly or UsernameThenPassword)
- Add tooling to default/reset keyword rules
- Modify terminal prompt detection to classify prompts by keyword type and auto-send credentials when exactly one credential exists
- Add keyword rules UI to the Quick Credentials settings page (chip list, add, delete, reset)
- Update panel `ItemSelected` event to carry `mode: SendMode` separately
- Ensure the existing OneKey menu fallback behavior for SSH credentials is preserved

## Capabilities

### New Capabilities
- `auto-fill-trigger`: Configurable keyword-to-SendMode mapping rules that the prompt detector uses to classify terminal output and auto-send credentials

### Modified Capabilities
- `credential-management`: Remove `send_mode` from the settings form
- `credential-panel`: `ItemSelected` event carries `mode: SendMode` instead of reading it from the credential
- `credential-send`: Prompt detection now supports both password-type and username-type prompts; auto-send when exactly one credential matches
- `credential-store`: Remove `send_mode` column and add `prompt_trigger_rules` table

## Impact

- `crates/quick_credential/`: schema/model/repository changes (remove send_mode, add trigger_rules table/CRUD)
- `app/src/settings_view/quick_credentials_page.rs`: remove SendMode dropdown, add keyword rules section
- `app/src/terminal/`: new `prompt_detection.rs`, modify `spawn_onekey_prompt_listener` for auto-send
- `app/src/search/quick_credential/view.rs`: `ItemSelected` event gains `mode: SendMode` field
- No API changes outside Quick Credential feature
```

## openspec/changes/quick-credential-auto-send/design.md

- Source: openspec/changes/quick-credential-auto-send/design.md
- Lines: 1-61
- SHA256: baca2556d94820eb8d3406b2109d094a3882508883f6038fa0dfcd24887cd7e7

```md
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
```

## openspec/changes/quick-credential-auto-send/tasks.md

- Source: openspec/changes/quick-credential-auto-send/tasks.md
- Lines: 1-65
- SHA256: 7e3f077b83d1d793500be816af2886ccf1f3a6199dd92eaa1076b5be64cecf3b

```md
## 1. Data Layer: Remove send_mode

- [ ] 1.1 Update `up.sql` migration: remove `send_mode` column from `quick_credentials` CREATE TABLE
- [ ] 1.2 Update `db.rs`: update `ensure_columns()` — use new schema without `send_mode`, add `prompt_trigger_rules` table
- [ ] 1.3 Update `schema.rs`: remove `send_mode` column from `quick_credentials` table definition
- [ ] 1.4 Update `QuickCredentialRow` model: remove `send_mode` field
- [ ] 1.5 Update `QuickCredential` domain type: remove `send_mode` field (keep `SendMode` enum)
- [ ] 1.6 Update `repository.rs`: remove `send_mode` from create/update/Row conversion
- [ ] 1.7 Update `lib.rs` re-exports if needed

## 2. Data Layer: Add prompt_trigger_rules

- [ ] 2.1 Define `PromptTriggerRule` struct in `types.rs` (id, keyword, send_mode)
- [ ] 2.2 Define `DEFAULT_KEYWORDS` constant (PasswordOnly + UsernameThenPassword groups)
- [ ] 2.3 Add `prompt_trigger_rules` table to `schema.rs`
- [ ] 2.4 Add `PromptTriggerRuleRow` to `model.rs`
- [ ] 2.5 Add CRUD functions to `repository.rs` (list_rules, add_rule, remove_rule, reset_rules_to_defaults)
- [ ] 2.6 Export new functions from `lib.rs`
- [ ] 2.7 Update `db.rs` `ensure_columns()` to create `prompt_trigger_rules` table
- [ ] 2.8 Run `cargo check -p persistence -p warp_quick_credential` to verify

## 3. Settings UI: Remove send_mode dropdown

- [ ] 3.1 Remove `send_mode_input` field from `QuickCredentialsPageView` struct
- [ ] 3.2 Remove SendMode dropdown rendering from `render_form_mode()`
- [ ] 3.3 Remove send_mode from `handle_save()` logic
- [ ] 3.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 4. Settings UI: Add trigger keywords section

- [ ] 4.1 Add `trigger_rules: Vec<PromptTriggerRule>` field + search state to `QuickCredentialsPageView`
- [ ] 4.2 Add `load_rules()` / save helpers that call repository CRUD
- [ ] 4.3 Render keyword groups (PasswordOnly chips + UsernameThenPassword chips) above credential list
- [ ] 4.4 Implement "+ Add" inline input that adds keyword to repository
- [ ] 4.5 Implement "×" on chip that removes keyword from repository
- [ ] 4.6 Implement "Reset" button that calls `reset_rules_to_defaults()`
- [ ] 4.7 Run `cargo check -p warp --features quick_credential_input` to verify

## 5. Prompt Detection

- [ ] 5.1 Create `app/src/terminal/prompt_detection.rs` with `PromptType` enum and `classify_prompt()` function
- [ ] 5.2 `classify_prompt()` loads trigger rules and compares PTY output against keywords
- [ ] 5.3 Add module declaration in terminal mod.rs
- [ ] 5.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 6. Auto-send Integration

- [ ] 6.1 Modify `spawn_onekey_prompt_listener` to load Quick Credentials on prompt detection
- [ ] 6.2 If exactly 1 credential: call `send_quick_credential` with mode based on prompt type
- [ ] 6.3 If 0 or multiple: fall back to existing OneKey show logic
- [ ] 6.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 7. Panel Event Update

- [ ] 7.1 Update `QuickCredentialPanelEvent::ItemSelected` to carry `mode: SendMode` field
- [ ] 7.2 Update panel button handlers to emit mode separately (not set on credential)
- [ ] 7.3 Update `on_quick_credential_panel_event` in terminal view.rs to use event's mode
- [ ] 7.4 Run `cargo check -p warp --features quick_credential_input` to verify

## 8. Test and Build

- [ ] 8.1 Update existing unit tests for data layer changes
- [ ] 8.2 Add unit tests for `classify_prompt()`
- [ ] 8.3 Run `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`
- [ ] 8.4 Run `scripts/run` to manually verify auto-send behavior
```

## openspec/changes/quick-credential-auto-send/specs/auto-fill-trigger/spec.md

- Source: openspec/changes/quick-credential-auto-send/specs/auto-fill-trigger/spec.md
- Lines: 1-86
- SHA256: 5b548a91d0220afb75c12b3f418268fde778a359b475728f5d7ae58a855077c8

[TRUNCATED]

```md
# auto-fill-trigger Specification

## Purpose
Provide configurable keyword-to-SendMode mapping rules that the terminal prompt detector uses to classify terminal output and auto-send credentials.

## Requirements
### Requirement: Default trigger keywords on first run
When the feature is first enabled and no trigger rules exist, the system SHALL populate default rules: PasswordOnly keywords = {password, passphrase}, UsernameThenPassword keywords = {login, username, user, name, email, account}.

#### Scenario: First-time initialization
- **WHEN** the system loads trigger rules and finds the table empty
- **THEN** it inserts the default keyword set and returns it

#### Scenario: Subsequent loads preserve user changes
- **WHEN** the user has customized trigger rules and the system loads them on next app launch
- **THEN** the user's custom rules are returned, not the defaults

### Requirement: Add a trigger keyword
The system SHALL allow adding a keyword with an associated SendMode.

#### Scenario: Add password-only keyword
- **WHEN** user adds keyword "secret" with mode PasswordOnly
- **THEN** the keyword "secret" is inserted into the trigger rules table with mode "password_only"

#### Scenario: Add username+password keyword
- **WHEN** user adds keyword "account" with mode UsernameThenPassword
- **THEN** the keyword "account" is inserted into the trigger rules table with mode "username_then_password"

#### Scenario: Duplicate keyword rejected
- **WHEN** user attempts to add a keyword that already exists
- **THEN** the system rejects the duplicate (no-op or shows error)

### Requirement: Delete a trigger keyword
The system SHALL allow removing a keyword from the trigger rules.

#### Scenario: Delete keyword
- **WHEN** user clicks the delete button on keyword "passphrase"
- **THEN** the keyword "passphrase" is removed from the trigger rules table

### Requirement: Reset trigger keywords to defaults
The system SHALL provide a way to clear all current rules and re-insert the default keyword set.

#### Scenario: Reset to defaults
- **WHEN** user clicks "Reset" button
- **THEN** all current trigger rules are deleted and the default keyword set is inserted

### Requirement: Classify PTY output against trigger rules
The system SHALL compare terminal PTY output against all configured trigger keywords to determine if a password-type or username-type prompt is present.

#### Scenario: Password prompt detected
- **WHEN** terminal output contains "password:" and the trigger rule for keyword "password" has mode PasswordOnly
- **THEN** the detection returns PromptType::Password

#### Scenario: Username prompt detected
- **WHEN** terminal output contains "login:" and the trigger rule for keyword "login" has mode UsernameThenPassword
- **THEN** the detection returns PromptType::Username

#### Scenario: No keyword matched
- **WHEN** terminal output does not match any trigger keyword
- **THEN** the detection returns None

### Requirement: Classify prompt in PTY output stream
The prompt detector SHALL monitor the PTY output stream and trigger classification when terminal output matches keyword patterns, using the same sliding-window approach as the existing OneKey password prompt detection.

#### Scenario: Continuous monitoring
- **WHEN** PTY output is flowing and contains a keyword-triggering pattern
- **THEN** the detector fires and triggers the auto-send logic

### Requirement: Auto-send when exactly one Quick Credential exists
When a prompt is classified, the system SHALL load all Quick Credentials. If exactly one credential exists, the system SHALL auto-send it using the SendMode from the matched keyword rule. If zero or multiple credentials exist, the system SHALL fall back to showing the OneKey menu.

#### Scenario: Single credential auto-sends password
- **WHEN** a password-type prompt is detected and exactly one Quick Credential exists
- **THEN** the credential's password is sent to the PTY (password only)

#### Scenario: Single credential auto-sends username+password
- **WHEN** a username-type prompt is detected and exactly one Quick Credential exists
- **THEN** the credential's username is sent, followed by ~150ms delay, then the password

#### Scenario: Multiple credentials fall back to OneKey menu
```

Full source: openspec/changes/quick-credential-auto-send/specs/auto-fill-trigger/spec.md

## openspec/changes/quick-credential-auto-send/specs/credential-management/spec.md

- Source: openspec/changes/quick-credential-auto-send/specs/credential-management/spec.md
- Lines: 1-69
- SHA256: a902aad8be2fc2f30800206f687dd0b57eb88a0a902454b4edb3e7e3da83b78e

```md
# credential-management Specification

## Purpose
Settings page UI for CRUD management of Quick Credentials.

## Requirements

### Requirement: List all saved credentials
The settings page SHALL display a list of all saved quick credentials, showing label and username preview for each entry.

#### Scenario: View credential list
- **WHEN** user navigates to the Quick Credentials settings page
- **THEN** all saved credentials are displayed in a list with label and username

### Requirement: Add new credential
The settings page SHALL provide a form to add a new credential with fields: label, username, password, and notes. (The send_mode selector is removed; send mode is chosen at send time.)

#### Scenario: Add credential successfully
- **WHEN** user fills in all form fields and clicks Save
- **THEN** the credential is persisted (SQLite + OS Keychain) and appears in the list

#### Scenario: Add credential with missing label
- **WHEN** user attempts to save without a label
- **THEN** an error message "Label is required" is shown and the credential is not saved

#### Scenario: Add credential with missing password
- **WHEN** user attempts to save without a password
- **THEN** an error message "Password is required" is shown and the credential is not saved

### Requirement: Edit existing credential
The settings page SHALL allow editing all fields of an existing credential.

#### Scenario: Edit credential label
- **WHEN** user edits the label of an existing credential and saves
- **THEN** the credential's label is updated in SQLite

#### Scenario: Edit credential password
- **WHEN** user edits the password of an existing credential and saves
- **THEN** the new password is stored in OS Keychain

### Requirement: Delete credential
The settings page SHALL allow deleting a credential with a confirmation dialog.

#### Scenario: Delete credential
- **WHEN** user clicks Delete on a credential and confirms
- **THEN** the credential is removed from SQLite and OS Keychain

#### Scenario: Cancel delete
- **WHEN** user clicks Delete on a credential but cancels the confirmation
- **THEN** the credential is not deleted

### Requirement: Manage auto-fill trigger keywords
The settings page SHALL display a section for managing trigger keywords, organized into two groups: PasswordOnly keywords and UsernameThenPassword keywords.

#### Scenario: View trigger keywords
- **WHEN** user views the Quick Credentials settings page
- **THEN** a "Trigger Keywords" section is shown above the credential list, with PasswordOnly and UsernameThenPassword keyword groups

#### Scenario: Add a trigger keyword
- **WHEN** user clicks "+ Add" and enters a keyword
- **THEN** the keyword is added to the appropriate group and persisted

#### Scenario: Delete a trigger keyword
- **WHEN** user clicks the × button on a keyword chip
- **THEN** the keyword is removed and persisted

#### Scenario: Reset trigger keywords to defaults
- **WHEN** user clicks the "Reset" button in the Trigger Keywords section
- **THEN** all current keywords are replaced with the default set
```

## openspec/changes/quick-credential-auto-send/specs/credential-panel/spec.md

- Source: openspec/changes/quick-credential-auto-send/specs/credential-panel/spec.md
- Lines: 1-54
- SHA256: e9d84e1fb47f546ece0f7668134540345a738b619b30235c0dc94657f97a5362

```md
# credential-panel Specification

## Purpose
In-terminal search panel for selecting Quick Credentials, with send mode selection before sending.

## Requirements

### Requirement: Show credential search panel on hotkey
The system SHALL display a search panel when the user presses the configured hotkey in a terminal. The panel SHALL contain a search bar and a scrollable list of credentials.

#### Scenario: Open panel via hotkey
- **WHEN** user presses `ctrl+shift+k` (or configured shortcut) in a terminal
- **THEN** a search panel appears at the center of the terminal, focused on the search bar

#### Scenario: Close panel via Escape
- **WHEN** panel is open and user presses Escape
- **THEN** the panel closes

#### Scenario: Close panel via clicking outside
- **WHEN** panel is open and user clicks outside it
- **THEN** the panel closes

### Requirement: Fuzzy search credentials
The system SHALL filter the credential list as the user types, using case-insensitive fuzzy matching on the label and username fields.

#### Scenario: Search by label
- **WHEN** user types "prod" in the search bar
- **THEN** credentials with labels containing "prod" (e.g., "prod-db", "production-server") appear, sorted by relevance

#### Scenario: Search by username
- **WHEN** user types "admin" in the search bar
- **THEN** credentials with username "admin" appear

#### Scenario: No matches
- **WHEN** user types text that matches no credentials
- **THEN** the list shows "No matching credentials"

### Requirement: Select credential with keyboard
The system SHALL support keyboard navigation in the credential list via Up/Down arrow keys, and selection via Enter.

#### Scenario: Navigate and select
- **WHEN** user presses Down arrow, then Enter on the selected credential
- **THEN** the credential is selected and the panel transitions to send mode selection

### Requirement: Show send mode options after credential selection
After the user selects a credential from the panel, the system SHALL display two buttons: "Send Password Only" and "Send Username + Password". The selected mode SHALL be emitted alongside the credential in the ItemSelected event, not stored on the credential itself.

#### Scenario: Show send mode options
- **WHEN** user selects a credential from the panel
- **THEN** two buttons are shown: "Send Password Only" and "Send Username + Password"

#### Scenario: Emit mode with ItemSelected
- **WHEN** user clicks "Send Password Only"
- **THEN** the panel emits `ItemSelected { credential, mode: PasswordOnly }`
```

## openspec/changes/quick-credential-auto-send/specs/credential-send/spec.md

- Source: openspec/changes/quick-credential-auto-send/specs/credential-send/spec.md
- Lines: 1-41
- SHA256: d48a5f3e91df15412fbea92b997b958d60885d7f2153f5e7017b2a40af7695fb

```md
# credential-send Specification

## Purpose
Sending Quick Credentials to the terminal PTY, including auto-send triggered by prompt detection.

## Requirements

### Requirement: Send password only via panel
The system SHALL send only the password followed by newline to the PTY when "Send Password Only" is chosen in the panel.

#### Scenario: Send password to PTY
- **WHEN** user selects credential "my-server" and chooses "Send Password Only"
- **THEN** the terminal's current line is cleared, then `password\n` is written to the PTY

### Requirement: Send username then password via panel
The system SHALL send the username followed by newline, wait ~150ms, then send the password followed by newline when "Send Username + Password" is chosen in the panel.

#### Scenario: Send username then password to PTY
- **WHEN** user selects credential "my-app" (username: "admin") and chooses "Send Username + Password"
- **THEN** the terminal's current line is cleared, `admin\n` is written, then after ~150ms `password\n` is written

### Requirement: Auto-send on password prompt detection
When the prompt detector classifies a password-type prompt and exactly one Quick Credential exists, the system SHALL auto-send only the password.

#### Scenario: Auto-send password
- **WHEN** terminal output contains a Password keyword match and exactly one Quick Credential exists
- **THEN** the credential's password is written to the PTY with a newline

### Requirement: Auto-send on username prompt detection
When the prompt detector classifies a username-type prompt and exactly one Quick Credential exists, the system SHALL auto-send the username followed by ~150ms delay, then the password.

#### Scenario: Auto-send username then password
- **WHEN** terminal output contains a Username keyword match and exactly one Quick Credential exists
- **THEN** the credential's username is written, then after ~150ms the password is written

### Requirement: Use Zeroizing for sensitive data
The system SHALL wrap all passwords in `Zeroizing<String>` to ensure they are zeroed out in memory on drop.

#### Scenario: Password is zeroed after send
- **WHEN** a credential's password has been sent to the PTY and all references are dropped
- **THEN** the memory previously holding the password is zeroed
```

## openspec/changes/quick-credential-auto-send/specs/credential-store/spec.md

- Source: openspec/changes/quick-credential-auto-send/specs/credential-store/spec.md
- Lines: 1-59
- SHA256: c9eddf0054e0703e4a287f0629a8f9243bb4e32713fc44c8e7e24eb7ba2be659

```md
# credential-store Specification

## Purpose
Persistent storage for Quick Credentials (SQLite + OS Keychain) and prompt trigger rules (SQLite).

## Requirements

### Requirement: Store credential metadata in SQLite
The system SHALL persist credential metadata (label, username, notes) in a SQLite `quick_credentials` table. Each credential MUST have a unique UUID as its primary key. The `send_mode` column is removed; the table has columns: id, label, username, notes, encrypted_password, created_at, updated_at.

#### Scenario: Create a new credential
- **WHEN** user saves a new credential with label "my-server", username "admin"
- **THEN** a new row is inserted into `quick_credentials` with a generated UUID, and the created_at/updated_at timestamps are set

#### Scenario: List all credentials
- **WHEN** system loads credentials for the panel
- **THEN** all rows from `quick_credentials` are returned, ordered by label ascending

#### Scenario: Update an existing credential
- **WHEN** user edits the label of an existing credential
- **THEN** the row's label is updated and updated_at is refreshed

#### Scenario: Delete a credential
- **WHEN** user deletes a credential
- **THEN** the row is removed from `quick_credentials` and the corresponding secret is deleted from OS keychain

### Requirement: Store secrets in OS Keychain
The system SHALL store actual passwords in the OS keychain via the `keyring` crate, using service name `zap.quick-credential` and account key `<credential-uuid>:password`.

#### Scenario: Save a new secret
- **WHEN** a credential is created with password "s3cret!"
- **THEN** the keychain entry `zap.quick-credential / <uuid>:password` contains "s3cret!"

#### Scenario: Retrieve a secret
- **WHEN** the system needs to send a credential's password
- **THEN** it reads the password from the keychain entry `zap.quick-credential / <uuid>:password`

#### Scenario: Delete a secret
- **WHEN** a credential is deleted
- **THEN** the corresponding keychain entry is also deleted

### Requirement: Store prompt trigger rules in SQLite
The system SHALL store trigger keywords in a `prompt_trigger_rules` table with columns: id (UUID primary key), keyword (TEXT, unique), send_mode (TEXT, one of "password_only" or "username_then_password").

#### Scenario: Create trigger rule
- **WHEN** user adds keyword "secret" with mode PasswordOnly
- **THEN** a row (id, keyword="secret", send_mode="password_only") is inserted

#### Scenario: Delete trigger rule
- **WHEN** user removes keyword "secret"
- **THEN** the row with keyword="secret" is deleted

#### Scenario: List all trigger rules
- **WHEN** system needs to classify a prompt
- **THEN** all rows from `prompt_trigger_rules` are returned

#### Scenario: Reset trigger rules
- **WHEN** user clicks Reset
- **THEN** all rows in `prompt_trigger_rules` are deleted and the default set is inserted
```

