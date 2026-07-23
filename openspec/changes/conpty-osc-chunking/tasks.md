## 1. Rust 协议层

- [ ] 1.1 `ansi/mod.rs`: 新增 `WARP_IN_BAND_GENERATOR_CHUNK_BYTE = b"C"` 常量，`osc_dispatch` 中增加 `C` 的路由
- [ ] 1.2 `handler.rs`: 新增 `end_in_band_command_output_chunk()` trait 方法
- [ ] 1.3 `terminal_model.rs`: `IsReceivingInBandCommandOutput::Yes` 增加 `accumulated_hex: String` 字段
- [ ] 1.4 `terminal_model.rs`: 实现 `end_in_band_command_output_chunk()` — 从 `output.as_str()` 提取 hex 追加到 `accumulated_hex`，重置 `output`
- [ ] 1.5 `terminal_model.rs`: 修改 `end_in_band_command_output()` — 支持拼接 `accumulated_hex` 后解码
- [ ] 1.6 `terminal_model.rs`: 修改 `start_in_band_command_output()` — 已处于接收状态时保留 `accumulated_hex`

## 2. Shell 端

- [ ] 2.1 `bash_body.sh`: 新增 `OSC_CHUNK_GENERATOR_OUTPUT` 常量，修改 `_warp_execute_command` 支持分片
- [ ] 2.2 `zsh_body.sh`: 同上
- [ ] 2.3 `fish.sh`: 同上
- [ ] 2.4 `pwsh.ps1`: 同上

## 3. 回退无效 fix

- [ ] 3.1 `session.rs`: 移除 `load_external_commands` 中的 `if self.is_legacy_ssh_session() { return; }`

## 4. 验证

- [ ] 4.1 `cargo check` 编译通过
- [ ] 4.2 本地终端 `load_external_commands` 行为不变
