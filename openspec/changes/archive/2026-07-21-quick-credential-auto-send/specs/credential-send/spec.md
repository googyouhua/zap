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
