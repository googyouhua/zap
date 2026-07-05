## ADDED Requirements

### Requirement: Auto fallback when tmux is not installed
When an SSH session is warpified and the remote host does not have tmux installed, the system SHALL automatically fall back to shell integration (precmd/preexec) instead of blocking with an error.

#### Scenario: Tmux not installed on remote host
- **WHEN** user SSHs to a host that does not have `tmux` installed
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type
- **THEN** the SSH session SHALL have block mode and command history via shell integration

### Requirement: Auto fallback when tmux version is unsupported
When an SSH session is warpified and the remote host has a tmux version below the minimum required version, the system SHALL automatically fall back to shell integration.

#### Scenario: Unsupported tmux version
- **WHEN** user SSHs to a host with tmux < 3.3
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL NOT block with "unsupported version" error
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type

### Requirement: Auto fallback when tmux installation fails
When an SSH session is warpified and the automatic tmux installation fails, the system SHALL automatically fall back to shell integration.

#### Scenario: Tmux install fails
- **WHEN** user SSHs to a host where tmux auto-install fails
- **AND** `use_ssh_tmux_wrapper` is enabled
- **THEN** the system SHALL call `trigger_subshell_bootstrap()` with the detected shell type

### Requirement: Nested SSH sessions are not auto-fallback targets
The auto-fallback mechanism SHALL only apply to the first SSH hop from the local Warp instance. Nested SSH sessions (ssh from a remote host to another host) SHALL NOT be auto-warpified.

#### Scenario: Nested SSH not auto-tracked
- **WHEN** user SSHs from local Warp to Host A
- **AND** then SSHs from Host A to Host B
- **THEN** Host B SHALL NOT receive auto shell integration injection
- **THEN** Host B SHALL only have block-mode tracking from Host A's shell hooks

### Requirement: No behavior change when tmux wrapper is disabled
When `use_ssh_tmux_wrapper` is disabled, the system SHALL preserve the original SSH session behavior without any auto-fallback.

#### Scenario: Tmux wrapper disabled
- **WHEN** user has `use_ssh_tmux_wrapper` set to false
- **THEN** the auto-fallback mechanism SHALL NOT activate
- **THEN** nested SSH tracking behavior SHALL match the original code (no auto-tracking)
