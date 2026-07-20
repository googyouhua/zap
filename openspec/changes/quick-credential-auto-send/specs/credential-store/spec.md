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
