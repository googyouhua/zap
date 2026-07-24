## Why

在两跳 SSH（Windows ConPTY → 跳板机 → 目标机）场景下，`load_external_commands()` 执行的 `compgen -c` 输出数千个命令名（~64KB hex），通过 in-band 生成器协议经 ConPTY 发送时超出其 4KB 缓冲区，导致 hex 载荷被分片后泄漏到终端显示为乱码。当前粗暴地跳过整个 `load_external_commands` 会丢失远端命令补全功能。

## What Changes

- 新增 OSC 标记 `C`（chunk continue），扩展 in-band 生成器输出协议支持分片
- Shell 端 `_warp_execute_command`：输出 >3KB hex 时拆成多个 OSC 序列发送，非最终片用 `C` 标记，最终片用 `B` 标记
- Rust 端状态机：`IsReceivingInBandCommandOutput::Yes` 新增 `accumulated_hex` 字段，`C` 标记到达时累加 hex 数据不清除接收器，`B` 标记到达时拼接总 hex 后解码处理
- 回退 `session.rs` 中 `if self.is_legacy_ssh_session() { return; }` 的粗暴修复
- Shell 侧 4 种实现：bash、zsh、fish、PowerShell

## Capabilities

### New Capabilities
- `in-band-output-chunking`: in-band 生成器输出协议分片传输，解决 ConPTY 缓冲区溢出导致的 hex 泄漏

### Modified Capabilities

无。本 change 不涉及产品级能力需求变更，纯属协议实现层改进。

## Impact

- `app/src/terminal/model/ansi/mod.rs` — 新增 `WARP_IN_BAND_GENERATOR_CHUNK_BYTE` 常量及路由
- `app/src/terminal/model/ansi/handler.rs` — 新增 `end_in_band_command_output_chunk()` trait 方法
- `app/src/terminal/model/terminal_model.rs` — 状态机改造：`accumulated_hex` 跨片累积、分片结束处理
- `app/src/terminal/model/session.rs` — 回退 `load_external_commands` 的 SSH 跳过逻辑
- `app/assets/bundled/bootstrap/bash_body.sh` — `_warp_execute_command` 分片输出
- `app/assets/bundled/bootstrap/zsh_body.sh` — 同上
- `app/assets/bundled/bootstrap/fish.sh` — 同上
- `app/assets/bundled/bootstrap/pwsh.ps1` — 同上
- `crates/warp_terminal/src/shell/mod.rs` — 无改动（`shell_command_to_get_executables` 保持不变）
