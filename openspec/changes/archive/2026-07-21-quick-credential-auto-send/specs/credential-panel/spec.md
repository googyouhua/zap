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
