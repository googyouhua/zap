---
change: conpty-osc-chunking
design-doc: docs/superpowers/specs/2026-07-24-conpty-osc-chunking-design.md
base-ref: d91595598636dc007003b48f631792ea2e72dd26
---

# ConPTY OSC Chunking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend in-band generator output protocol with `C` (chunk continue) marker to split large hex-encoded payloads into ConPTY-safe chunks.

**Architecture:** Add `WARP_IN_BAND_GENERATOR_CHUNK_BYTE = b"C"` to OSC 9277 dispatch. Extend `IsReceivingInBandCommandOutput` with `accumulated_hex` for cross-chunk accumulation. Shell `_warp_execute_command` splits hex >3KB into chunks.

**Tech Stack:** Rust (ansi/mod.rs, handler.rs, terminal_model.rs), Bash/Zsh/Fish/PowerShell (bootstrap scripts).

---

## 1. Rust — Add C marker constant + OSC routing

**Files:**
- Modify: `app/src/terminal/model/ansi/mod.rs:62`
- Modify: `app/src/terminal/model/ansi/mod.rs:1077-1089`

- [ ] **1.1 Add `WARP_IN_BAND_GENERATOR_CHUNK_BYTE` constant**

```rust
// After line 62
const WARP_IN_BAND_GENERATOR_CHUNK_BYTE: &[u8] = b"C";
```

- [ ] **1.2 Add `C` routing in `osc_dispatch`**

In the `WARP_IN_BAND_GENERATOR_OSC_MARKER` match block, add third arm between `START_BYTE` and `END_BYTE`:

```rust
Some(&WARP_IN_BAND_GENERATOR_CHUNK_BYTE) => {
    self.handler.end_in_band_command_output_chunk();
}
```

## 2. Rust — Add handler trait method

**Files:**
- Modify: `app/src/terminal/model/ansi/handler.rs:316`

- [ ] **2.1 Add `end_in_band_command_output_chunk` trait method**

After `end_in_band_command_output`:

```rust
/// Callback to handle an "in-band command output chunk" OSC.
///
/// Similar to `end_in_band_command_output` but keeps the
/// accumulator alive for more chunks.
fn end_in_band_command_output_chunk(&mut self) {}
```

## 3. Rust — Extend state machine with accumulated_hex

**Files:**
- Modify: `app/src/terminal/model/terminal_model.rs:319-326`, `3053-3102`

- [ ] **3.1 Add `accumulated_hex` field to `IsReceivingInBandCommandOutput::Yes`**

```rust
enum IsReceivingInBandCommandOutput {
    Yes {
        output: InBandCommandOutputReceiver,
        /// Hex data accumulated across chunks for the same generator command.
        accumulated_hex: String,
    },
    No,
}
```

- [ ] **3.2 Update `start_in_band_command_output` to preserve accumulated_hex on re-entry**

```rust
fn start_in_band_command_output(&mut self) {
    let starting_cursor_point = self.block_list().active_block().grid_handler().cursor_point();
    match &mut self.is_receiving_in_band_command_output {
        IsReceivingInBandCommandOutput::Yes { output, accumulated_hex } => {
            // Already receiving chunks — reset output but keep accumulated hex
            *output = InBandCommandOutputReceiver::new(starting_cursor_point, self.block_list().size());
        }
        IsReceivingInBandCommandOutput::No => {
            self.is_receiving_in_band_command_output = IsReceivingInBandCommandOutput::Yes {
                output: InBandCommandOutputReceiver::new(starting_cursor_point, self.block_list().size()),
                accumulated_hex: String::new(),
            };
        }
    }
}
```

- [ ] **3.3 Implement `end_in_band_command_output_chunk`**

New handler method on `TerminalModel`:

```rust
fn end_in_band_command_output_chunk(&mut self) {
    match &mut self.is_receiving_in_band_command_output {
        IsReceivingInBandCommandOutput::Yes { output, accumulated_hex } => {
            // Strip "{content_length};" prefix, keep hex part
            let raw = output.as_str();
            if let Some(hex_part) = raw.split_once(';').map(|(_, hex)| hex.trim()) {
                accumulated_hex.push_str(hex_part);
            }
            // Reset output for next chunk
            let starting_cursor_point = self
                .block_list()
                .active_block()
                .grid_handler()
                .cursor_point();
            *output = InBandCommandOutputReceiver::new(starting_cursor_point, self.block_list().size());
        }
        IsReceivingInBandCommandOutput::No => {
            log::warn!("Received 'end_in_band_command_output_chunk' while not expecting in-band command output.");
        }
    }
}
```

- [ ] **3.4 Update `end_in_band_command_output` to build combined payload from accumulated_hex**

```rust
fn end_in_band_command_output(&mut self, from_osc_sequence: bool) {
    match &mut self.is_receiving_in_band_command_output {
        IsReceivingInBandCommandOutput::Yes { output, accumulated_hex } => {
            // Get the last chunk's hex
            let raw = output.as_str();
            if let Some(last_hex) = raw.split_once(';').map(|(_, hex)| hex.trim()) {
                accumulated_hex.push_str(last_hex);
            }
            // Build combined payload with total content length
            let total_hex = std::mem::take(accumulated_hex);
            let content_length = total_hex.len();
            let final_payload = format!("{content_length};{total_hex}");

            match validate_and_decode_in_band_command_output_to_bytes(&final_payload) {
                Ok(decoded_bytes) => {
                    match ExecutedExecutorCommandEvent::parse_generator_payload(decoded_bytes) {
                        Ok(event) => {
                            log::info!("Parsed generator output for command {}", event.command_id);
                            self.event_proxy
                                .send_terminal_event(Event::ExecutedInBandCommand(event));
                        }
                        Err(e) => log::warn!("Failed to parse generator output: {e:#}"),
                    }
                }
                Err(e) => log::warn!("Failed to decode generator output: {e:#}"),
            }
            self.is_receiving_in_band_command_output = IsReceivingInBandCommandOutput::No;
        }
        IsReceivingInBandCommandOutput::No => {
            log::warn!("Received 'end_in_band_command_output' while not expecting to read in-band command output.");
        }
    }
    #[cfg(windows)]
    if from_osc_sequence {
        self.ignore_reset_grid_after_in_band_generator = true;
    }
}
```

## 4. Rust — Remove the return-based fix from session.rs

**Files:**
- Modify: `app/src/terminal/model/session.rs:1095-1102`

- [ ] **4.1 Remove the stale return statement**

Remove lines 1095-1102 (the `is_legacy_ssh_session()` early return comment and condition). `load_external_commands` should run normally for all sessions.

## 5. Shell — Implement chunking in _warp_execute_command

**Files:**
- Modify: `app/assets/bundled/bootstrap/bash_body.sh:169-202`
- Modify: `app/assets/bundled/bootstrap/zsh_body.sh`
- Modify: `app/assets/bundled/bootstrap/fish.sh`
- Modify: `app/assets/bundled/bootstrap/pwsh.ps1`

- [ ] **5.1 bash_body.sh — Add OSC_CHUNK_GENERATOR_OUTPUT and modify _warp_execute_command**

Add new constant:
```bash
OSC_CHUNK_GENERATOR_OUTPUT="$(printf '\e]9277;C\a')"
```

Modify `_warp_execute_command`:
```bash
_warp_execute_command() {
    local command_id=$1
    local command="${@:2}"
    local generator_output="$( {
        echo -n "$command_id;";
        eval "$command" 2>&1;
        echo -n ";$?";
    } | command -p od -An -v -tx1 | command -p tr -d ' \n')"

    local chunk_size=3000
    local total_length=${#generator_output}
    if [ $total_length -le $chunk_size ]; then
        warp_send_generator_output_osc_pre_hex_encoded "$generator_output"
    else
        local pos=0
        while [ $pos -lt $total_length ]; do
            local chunk="${generator_output:$pos:$chunk_size}"
            local byte_count=${#chunk}
            if [ $((pos + chunk_size)) -ge $total_length ]; then
                printf "%b%i;%s%b" $OSC_START_GENERATOR_OUTPUT $byte_count $chunk $OSC_END_GENERATOR_OUTPUT
            else
                printf "%b%i;%s%b" $OSC_START_GENERATOR_OUTPUT $byte_count $chunk $OSC_CHUNK_GENERATOR_OUTPUT
            fi
            pos=$((pos + chunk_size))
        done
        warp_maybe_send_reset_grid_osc
    fi
}
```

- [ ] **5.2 zsh_body.sh — Same chunking logic**

Port the same logic to zsh. Zsh supports `${#var}`, `${var:offset:length}`, and the same `printf` patterns.

- [ ] **5.3 fish.sh — Same chunking logic**

Fish uses `string length`, `string sub` instead of bash variable expansion.

- [ ] **5.4 pwsh.ps1 — Same chunking logic**

PowerShell uses `$var.Length`, `$var.Substring(pos, chunkSize)`.

## 6. Build and Verify

- [ ] **6.1 cargo check**

```bash
cargo check -p warp 2>&1 | tail -10
```

Expected: `Finished dev profile` or similar with 0 errors.

- [ ] **6.2 Commit**

After all tasks pass, commit with message `feat: chunk in-band generator output to avoid ConPTY hex leak`.
