---
comet_change: fix-conpty-hex-leak
role: technical-design
canonical_spec: openspec
archived-with: 2026-07-29-fix-conpty-hex-leak
status: archived
---

# ConPTY Hex Leak Fix — Design Doc

## Problem

Windows ConPTY's VT parser treats `\a` (ST, string terminator) as the end of any OSC sequence, regardless of what follows. Under the old generator output protocol:

```
\e]9277;A\a<len>;<hex>\e]9277;B\a
```

ConPTY consumes the first `\a` (after `A`) as the OSC terminator, leaving `<len>;<hex>` as bare printable text on the terminal. The hex payload that should have been consumed by Warp's `InBandCommandOutputReceiver` is instead rendered as visible garbled characters.

## Solution

Move the hex payload inside the OSC boundary so ConPTY never sees bare hex:

```
\e]9277;A;<hex>\a\e]9277;B\a
```

Now the `\a` that terminates the OSC also terminates the payload — no data leaks outside.

## Rust Changes

### handler.rs — new trait method

```rust
fn start_in_band_command_output_with_payload(&mut self, _payload: &str) {}
```

### mod.rs — osc_dispatch

For marker `9277;A`:
- If `params.len() >= 3`: extract `params[2]` as string → call `start_in_band_command_output_with_payload`
- Else (no payload): call old `start_in_band_command_output()` (backward compat)

Marker `9277;B` unchanged — calls `end_in_band_command_output(true)`.

### terminal_model.rs — core logic

**Enum change** — `IsReceivingInBandCommandOutput::Yes` gains `accumulated_hex: String`:

```rust
enum IsReceivingInBandCommandOutput {
    Yes { output: InBandCommandOutputReceiver, accumulated_hex: String },
    No,
}
```

- **input()** / **goto()** / **carriage_return()**: when `accumulated_hex` is non-empty, skip PTY char capture (payload already in params)
- **start_in_band_command_output_with_payload()**: creates `Yes { output, accumulated_hex: payload.to_string() }`
- **end_in_band_command_output()**: if `accumulated_hex` is non-empty, `hex::decode()` directly; otherwise fall back to old `validate_and_decode_in_band_command_output_to_bytes()` path

## Shell Changes

### Constants (bash/zsh/fish)

```
OSC_IB_START="$(printf '\e]9277;A;')"
OSC_IB_END="$(printf '\a\e]9277;B\a')"
```

### printf format (all three shells)

```
Old: printf "%b%i;%s%b" $OSC_START_GENERATOR_OUTPUT $byte_count $hex $OSC_END_GENERATOR_OUTPUT
New: printf '%s%s%s' "$OSC_IB_START" "$hex" "$OSC_IB_END"
```

### File lock (bash/zsh only — fish uses begin...end subshell)

```bash
command -p mkdir "$lock_dir" 2>/dev/null || return 1
printf '%s%s%s' "$OSC_IB_START" "$hex" "$OSC_IB_END"
command -p rmdir "$lock_dir"
```

`mkdir` is atomic per directory on all platforms. The lock ensures two concurrent generators don't interleave their writes past `PIPE_BUF` (~4KB on Windows).

### Preexec cleanup (bash/zsh only)

After `kill -9` of orphaned generators, emit:

```bash
printf '\e\\\e]9277;B\a'
command -p rmdir /tmp/warp-generator-lock-* 2>/dev/null
```

`\e\\` is the OSC string terminator (ST). It closes any dangling OSC session that was interrupted mid-write. The `\e]9277;B\a` then signals a clean end to the Rust handler. `rmdir` cleans up orphaned lock directories.

## Files Changed

| File | Change |
|------|--------|
| `app/src/terminal/model/ansi/handler.rs` | Add trait method |
| `app/src/terminal/model/ansi/mod.rs` | Extract params[2] in osc_dispatch |
| `app/src/terminal/model/terminal_model.rs` | accumulated_hex field + dual-path decode |
| `app/assets/bundled/bootstrap/bash_body.sh` | New constants, file lock, preexec cleanup |
| `app/assets/bundled/bootstrap/zsh_body.sh` | Same as bash |
| `app/assets/bundled/bootstrap/fish.sh` | New constants, remove byte_count prefix |
