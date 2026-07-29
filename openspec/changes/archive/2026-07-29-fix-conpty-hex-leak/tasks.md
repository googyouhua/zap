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
