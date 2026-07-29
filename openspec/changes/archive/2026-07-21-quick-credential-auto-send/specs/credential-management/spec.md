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
