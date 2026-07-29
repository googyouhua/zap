---
change: fix-conpty-hex-leak
design-doc: docs/superpowers/specs/2026-07-28-fix-conpty-hex-leak-design.md
base-ref: 92ed9f1197665c7e843661b3ae50a55af14ce849
status: archived
archived-with: 2026-07-29-fix-conpty-hex-leak
---

# Implementation Plan: ConPTY Hex Leak Fix

## Tasks (all completed)

1. Rust: add `start_in_band_command_output_with_payload` handler trait method
2. Rust: update osc_dispatch to extract params[2]
3. Rust: add accumulated_hex field + dual-path decode
4. Shell: bash_body.sh — new constants, file lock, preexec cleanup
5. Shell: zsh_body.sh — new constants, file lock, preexec cleanup
6. Shell: fish.sh — new constants, remove byte_count prefix
7. Verify: cargo check passes
