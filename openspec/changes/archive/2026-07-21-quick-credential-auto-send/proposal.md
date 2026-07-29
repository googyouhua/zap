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
