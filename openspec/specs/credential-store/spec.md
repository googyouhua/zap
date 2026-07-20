# credential-store Specification

## Purpose
TBD - created by archiving change quick-credential-input. Update Purpose after archive.
## Requirements
### Requirement: Store credential metadata in SQLite
The system SHALL persist credential metadata (label, username, send_mode, notes) in a SQLite `quick_credentials` table. Each credential MUST have a unique UUID as its primary key.

#### Scenario: Create a new credential
- **WHEN** user saves a new credential with label "my-server", username "admin", send_mode "password_only"
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

