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
