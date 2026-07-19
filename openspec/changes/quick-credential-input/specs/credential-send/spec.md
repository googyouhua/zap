## ADDED Requirements

### Requirement: Show send mode options after credential selection
After the user selects a credential from the panel, the system SHALL display two send mode options: "Send Password Only" and "Send Username + Password".

#### Scenario: Show send mode options
- **WHEN** user selects a credential from the panel
- **THEN** two buttons/options are shown: "Send Password Only" and "Send Username + Password"

### Requirement: Send password only
The system SHALL send only the password followed by newline to the shell when "Send Password Only" is selected.

#### Scenario: Send password to shell
- **WHEN** user selects credential "my-server" and chooses "Send Password Only"
- **THEN** the terminal's current line is cleared, then `password\n` is written to the PTY

### Requirement: Send username then password
The system SHALL send the username followed by newline, wait for the shell to process it, then send the password followed by newline when "Send Username + Password" is selected.

#### Scenario: Send username then password to shell
- **WHEN** user selects credential "my-app" (username: "admin") and chooses "Send Username + Password"
- **THEN** the terminal's current line is cleared, `admin\n` is written, then after ~150ms `password\n` is written

### Requirement: Use Zeroizing for sensitive data
The system SHALL wrap all passwords in `Zeroizing<String>` to ensure they are zeroed out in memory on drop.

#### Scenario: Password is zeroed after send
- **WHEN** a credential's password has been sent to the PTY and all references are dropped
- **THEN** the memory previously holding the password is zeroed
