# ConPTY Hex Leak Fix

## Why

On Windows ConPTY, when the old protocol `\e]9277;A\a<len>;<hex>\e]9277;B\a` is used, ConPTY consumes the first `\a` as the OSC terminator, leaving the hex payload (`<len>;<hex>`) outside the OSC boundary. This causes the raw hex to leak into the terminal as visible garbled text, breaking generator output display.

## What Changes

- **New OSC protocol**: Replace the old three-stage format (`A\a → <len>;<hex> PTY chars → B\a`) with a single inline format where the hex payload lives entirely inside the OSC parameters: `\e]9277;A;<hex>\a\e]9277;B\a`
- **Rust handler**: Add `start_in_band_command_output_with_payload` trait method; `osc_dispatch` extracts hex from `params[2]`; `accumulated_hex` field on `IsReceivingInBandCommandOutput` stores inline hex; `end_in_band_command_output` decodes hex directly when `accumulated_hex` is set
- **Shell scripts (bash/zsh/fish)**: Replace `OSC_START_GENERATOR_OUTPUT`/`OSC_END_GENERATOR_OUTPUT` constants with `OSC_IB_START`/`OSC_IB_END`; remove the `<byte_count>;` prefix from the printf format
- **File lock (bash/zsh)**: Add `mkdir`-based mutex around the OSC write for `PIPE_BUF` atomicity on ConPTY
- **Preexec cleanup (bash/zsh)**: After `kill -9` of orphaned generators, emit `\e\\\e]9277;B\a` to close any dangling OSC session and `rmdir` stale lock directories

## Requirements

### Requirement: Inline hex in OSC parameter
**SHALL** keep hex payload entirely within the OSC boundary by embedding it as `params[2]` instead of after the `\a` terminator.

#### Scenario: ConPTY does not leak hex to terminal
When the new protocol is used on Windows ConPTY, the hex payload stays inside the OSC boundary and is consumed by the Rust handler.

### Requirement: Backward compatibility with old protocol
**SHALL** retain the old `start_in_band_command_output` code path on the Rust side for existing sessions.

#### Scenario: Sessions using old format continue to decode
If a generator session was started before the shell scripts were updated, its output continues to decode correctly.

### Requirement: mkdir-based file lock for bash/zsh
**SHALL** use `command -p mkdir` as an atomic mutex around OSC writes to prevent ConPTY interleaving of concurrent background generators.

#### Scenario: Concurrent generators on ConPTY do not corrupt each other
When two background generators finish simultaneously, their OSC writes do not interleave.

## Capabilities

### Modified Capabilities

- `generator-output`: protocol format changed from `A\a<len>;<hex>B\a` to `A;<hex>\aB\a`

## Impact

- 6 files: 3 Rust (`handler.rs`, `mod.rs`, `terminal_model.rs`), 3 shell (`bash_body.sh`, `zsh_body.sh`, `fish.sh`)
- Protocol change: old format no longer emitted by shell scripts; Rust side retains backward-compat path (old `start_in_band_command_output` + `<len>;<hex>` parsing) for existing sessions
- No database, API, or config changes
