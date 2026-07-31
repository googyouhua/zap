# Comet Design Handoff

- Change: fix-conpty-hex-leak
- Phase: design
- Mode: compact
- Context hash: cf7a17f601aa54bf32790127a0a4f3c3737fe2c8e0cc05f0b45d3c78be1a4d2c

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/fix-conpty-hex-leak/proposal.md

- Source: openspec/changes/fix-conpty-hex-leak/proposal.md
- Lines: 1-25
- SHA256: 5ae76269b3af16f093544245f6257b1955295616ebc02551a5d44471a14bd56b

```md
## Why

On Windows ConPTY, when the old protocol `\e]9277;A\a<len>;<hex>\e]9277;B\a` is used, ConPTY consumes the first `\a` as the OSC terminator, leaving the hex payload (`<len>;<hex>`) outside the OSC boundary. This causes the raw hex to leak into the terminal as visible garbled text, breaking generator output display.

## What Changes

- **New OSC protocol**: Replace the old three-stage format (`A\a → <len>;<hex> PTY chars → B\a`) with a single inline format where the hex payload lives entirely inside the OSC parameters: `\e]9277;A;<hex>\a\e]9277;B\a`
- **Rust handler**: Add `start_in_band_command_output_with_payload` trait method; `osc_dispatch` extracts hex from `params[2]`; `accumulated_hex` field on `IsReceivingInBandCommandOutput` stores inline hex; `end_in_band_command_output` decodes hex directly when `accumulated_hex` is set
- **Shell scripts (bash/zsh/fish)**: Replace `OSC_START_GENERATOR_OUTPUT`/`OSC_END_GENERATOR_OUTPUT` constants with `OSC_IB_START`/`OSC_IB_END`; remove the `<byte_count>;` prefix from the printf format
- **File lock (bash/zsh)**: Add `mkdir`-based mutex around the OSC write for `PIPE_BUF` atomicity on ConPTY
- **Preexec cleanup (bash/zsh)**: After `kill -9` of orphaned generators, emit `\e\\\e]9277;B\a` to close any dangling OSC session and `rmdir` stale lock directories

## Capabilities

### New Capabilities

### Modified Capabilities

- `generator-output`: protocol format changed from `A\a<len>;<hex>B\a` to `A;<hex>\aB\a`

## Impact

- 6 files: 3 Rust (`handler.rs`, `mod.rs`, `terminal_model.rs`), 3 shell (`bash_body.sh`, `zsh_body.sh`, `fish.sh`)
- Protocol change: old format no longer emitted by shell scripts; Rust side retains backward-compat path (old `start_in_band_command_output` + `<len>;<hex>` parsing) for existing sessions
- No database, API, or config changes
```

## openspec/changes/fix-conpty-hex-leak/design.md

- Source: openspec/changes/fix-conpty-hex-leak/design.md
- Lines: 1-32
- SHA256: 83c9ddcd4aef17a8308b9fb03898c872baa0443672385e8645da2d686d456e3d

```md
## Context

Generator output is sent from shell to Warp via OSC 9277 sequences. On Windows ConPTY, the old protocol's structure `\e]9277;A\a<len>;<hex>\e]9277;B\a` causes ConPTY to consume the first `\a` as the OSC terminator, leaving `<len>;<hex>` as bare printable text on the terminal.

The root cause is that ConPTY's VT parser treats `\a` (ST) as the end of any OSC sequence, regardless of whether more data follows. The hex payload that should have been consumed internally by the Rust `InBandCommandOutputReceiver` is instead rendered as visible garbled characters.

## Goals / Non-Goals

**Goals:**
- Eliminate bare hex leakage on Windows ConPTY by keeping all payload data inside the OSC boundary
- Maintain backward compatibility with existing sessions (old protocol on Rust side)
- Zero changes to the Rust `ExecutedExecutorCommandEvent::parse_generator_payload` — the decoded byte format (`<command_id>;<output>;<exit_code>`) is unchanged

**Non-Goals:**
- Not changing the pwsh.ps1 PowerShell script (out of scope for this branch)
- No changes to the PID-tracking or preexisting generator lifecycle logic
- No SSH_CLIENT guard changes (doesn't exist on origin/main)

## Decisions

- **Inline OSC payload** (`\e]9277;A;<hex>\a\e]9277;B\a`) over alternative approaches:
  - DCS: would require VTE parser changes across the entire codebase
  - Chunking: unnecessary complexity for a protocol where the payload is typically <64KB
- **`mkdir` file lock** for bash/zsh: ConPTY can interleave writes from concurrent background generators if they exceed `PIPE_BUF` (~4KB on Windows). `mkdir` is atomic per-directory, cheaper than `flock`
- **No file lock for fish**: fish wraps the entire generator execution in a `begin...end` subshell (`fish -c`), so there's only one concurrent write per PTY
- **`\e\\` preexec cleanup**: `\e\\` (OSC string terminator) closes any dangling OSC session if `kill -9` interrupts a generator mid-write

## Risks / Trade-offs

- [ConPTY behavior varies across Windows versions] → Inline format is strictly within OSC spec; risk is low
- [`mkdir` lock can leak if process crashes between mkdir and rmdir] → Preexec `rmdir /tmp/warp-generator-lock-*` cleans up stale locks
- [Old protocol still handled in Rust for backward compat] → Dead code path once all shells are updated; can be cleaned up in a follow-up
```

## openspec/changes/fix-conpty-hex-leak/tasks.md

- Source: openspec/changes/fix-conpty-hex-leak/tasks.md
- Lines: 1-21
- SHA256: e9e89dbd2a2ef850aef8dab8c06fc1549047e5c86c0af287e8a8eeb5bffaaafb

```md
## 1. Rust handler changes

- [x] Add `start_in_band_command_output_with_payload` trait method in `handler.rs`
- [x] Update `osc_dispatch` in `mod.rs` to extract `params[2]` for 9277;A
- [x] Add `accumulated_hex` field to `IsReceivingInBandCommandOutput` enum
- [x] Update `input()` / `goto()` / `carriage_return()` to skip PTY char capture when inline payload
- [x] Update `end_in_band_command_output()` to decode `accumulated_hex` directly
- [x] Implement `start_in_band_command_output_with_payload` in `terminal_model.rs`

## 2. Shell script upgrades

- [x] Replace constants in `bash_body.sh` (OSC_IB_START/END), remove byte_count prefix
- [x] Add `mkdir` file lock to `bash_body.sh` warp_send_generator_output_osc_pre_hex_encoded
- [x] Add `\e\\\e]9277;B\a` + `rmdir` preexec cleanup to `bash_body.sh`
- [x] Replace constants in `zsh_body.sh` (OSC_IB_START/END), remove byte_count prefix
- [x] Add `mkdir` file lock + preexec cleanup to `zsh_body.sh`
- [x] Replace constants in `fish.sh` (OSC_IB_START/END), remove byte_count prefix

## 3. Verification

- [x] `cargo check` passes (no compilation errors)
```

## openspec/changes/fix-conpty-hex-leak/specs/generator-output/spec.md

- Source: openspec/changes/fix-conpty-hex-leak/specs/generator-output/spec.md
- Lines: 1-19
- SHA256: 1e8b8e3620f36beac10d3324785888326e7b2524790f08f9edd90dcaff235bbb

```md
# Generator Output — Delta Spec

## MODIFIED Requirements

### OSC 9277 Protocol

Old format:
```
\e]9277;A\a<content_length>;<hex_encoded_payload>\e]9277;B\a
```

New format:
```
\e]9277;A;<hex_encoded_payload>\a\e]9277;B\a
```

- Payload format (post-hex-decode) unchanged: `<command_id>;<output>;<exit_code>`
- Content length prefix removed — no longer needed since the payload is delimited by the OSC boundary
- Rust side retains backward-compat path for old format
```

