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
- **WHEN** a prompt is detected and multiple Quick Credentials exist
- **THEN** the OneKey menu is shown with all Quick Credentials and SSH credentials listed

#### Scenario: No credentials, no action
- **WHEN** a prompt is detected and no Quick Credentials exist
- **THEN** no auto-send occurs; SSH credentials still appear in the OneKey menu
