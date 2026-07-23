# Comet Design Handoff

- Change: conpty-osc-chunking
- Phase: design
- Mode: compact
- Context hash: 75c6c1fc40dd57f30d9f45f6fafe1934280547efa075f4db2bf7084237eb3805

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/conpty-osc-chunking/proposal.md

- Source: openspec/changes/conpty-osc-chunking/proposal.md
- Lines: 1-32
- SHA256: 6341379de74c94fed967c2dc9669b85247f6ab033ff4a83dde194363024e7d1f

```md
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
```

## openspec/changes/conpty-osc-chunking/design.md

- Source: openspec/changes/conpty-osc-chunking/design.md
- Lines: 1-68
- SHA256: 5d2a233ca58c00e5968d30ed62fb2f927deef47c1e9145ce04e4a636a491f688

```md
## Context

两跳 SSH（Windows ConPTY → 跳板机 → 目标机）场景下，`load_external_commands()` 通过 in-band 生成器在目标机上执行 `compgen -c`。该命令输出 ~2137 个命令名，经 hex 编码后约 64KB，通过 OSC 9277 单次回传。ConPTY 内部缓冲区约 4KB，超出后数据被截断/分片，hex 字符泄漏到终端网格。

现有粗暴跳过 `load_external_commands` 的 fix（检查 `is_legacy_ssh_session()`）在两跳 SSH 下永远为假，完全无效。

## Goals / Non-Goals

**Goals:**
- 所有 InBand executor 场景下的 `compgen -c` 输出分片回传，单片 ≤3KB hex（< ConPTY 4KB 缓冲区）
- 回退现有无效的 `is_legacy_ssh_session()` return fix
- 覆盖 bash / zsh / fish / pwsh 四种 shell

**Non-Goals:**
- 不改动两跳 SSH 的连接检测方式
- 不改动 `is_legacy_ssh_session` 判断逻辑
- 不改动 `shell_command_to_get_executables` 的内容

## Decisions

### 1. 协议扩展：新增 OSC 标记 `C`

当前协议：`\e]9277;A\a{byte_count};{hex_data}\e]9277;B\a`

扩展为：
```
A  = start (不变)
B  = end, trigger decode (不变)
C  = chunk continue, keep accumulating (新增)
```

分片传输时，非最终片用 `C`，最终片用 `B`：
```
\e]9277;A\a{3000};{hex_1..3000}\e]9277;C\a
\e]9277;A\a{3000};{hex_3001..6000}\e]9277;C\a
...
\e]9277;A\a{1234};{hex_final}\e]9277;B\a
```

### 2. Rust 状态机：跨片累积 hex

`IsReceivingInBandCommandOutput` 新增 `accumulated_hex` 字段：

```rust
enum IsReceivingInBandCommandOutput {
    Yes {
        output: InBandCommandOutputReceiver,
        accumulated_hex: String,  // 跨片 hex 累积
    },
    No,
}
```

**`C` 到达时**：从 `output.as_str()` 提取 `{hex_chunk}`（strip `{chunk_len};` 前缀），追加到 `accumulated_hex`，重置 `output`

**`B` 到达时**：同上提取最后一片 hex，构建 `{total_len};{accumulated_hex}` → 传给 `validate_and_decode_in_band_command_output_to_bytes` → 正常处理

**`A` 到达时（已处于接收状态）**：保留 `accumulated_hex`，只重置 `output`。首次 `A` 创建新的 `accumulated_hex=""`。

### 3. `B` 之前收到 `A` 算 reset

若在 `C` 或 `B` 之前收到一个新的 `A`（非分片场景或协议错误），当前行为是重置接收器。这个不变——`A` 始终创建新上下文。

## Risks / Trade-offs

- **Reset grid OSC 发送时机**：`warp_maybe_send_reset_grid_osc` 只在 `_warp_execute_command` 最终分片完成后调用一次。中间分片不发送 reset grid。如果某中间分片导致 ConPTY 光标注移，后续分片可能累积偏移。影响限于 Windows ConPTY 且经测试需调优。
- **分片开销**：2137 个命令 ~64KB hex，每片 3KB → ~22 片。每片是独立的 OSC 序列，Rust 端需解析 22 次 OSC。但都是内存操作，不影响 PTY 写入次数——整个 `_warp_execute_command` 仍然是一次写入。
- **兼容性**：旧版 Rust 客户端不认识 `C` 标记 → fallthrough 到 `_` 分支，打 warning 日志。不导致崩溃或数据损坏。只在更新 shell 脚本后出现——建议一起发布。
```

## openspec/changes/conpty-osc-chunking/tasks.md

- Source: openspec/changes/conpty-osc-chunking/tasks.md
- Lines: 1-24
- SHA256: b5ee4f929b8d600b926cd4675a4ff3d9fa6d6ae871965331aa50137b7fbf1ee4

```md
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
```

## openspec/changes/conpty-osc-chunking/specs/in-band-output-chunking/spec.md

- Source: openspec/changes/conpty-osc-chunking/specs/in-band-output-chunking/spec.md
- Lines: 1-35
- SHA256: 9709869a848bc91dc623d53f1577d3afbe8f29f9c40f578856710f99558c0e39

```md
## ADDED Requirements

### Requirement: Large generator output SHALL be split into chunks

When in-band generator command output exceeds ConPTY-safe size, the shell SHALL split the hex-encoded payload into multiple OSC sequences. The Rust receiver SHALL accumulate the chunks and reconstruct the full payload before decoding.

#### Scenario: Small output (≤3KB hex) sent as single OSC
- **WHEN** `_warp_execute_command` produces hex output ≤ 3000 bytes
- **THEN** the shell SHALL send a single OSC sequence ending with `\e]9277;B\a`
- **THEN** the Rust receiver SHALL decode the payload immediately

#### Scenario: Large output (>3KB hex) split into chunks
- **WHEN** `_warp_execute_command` produces hex output > 3000 bytes
- **THEN** the shell SHALL split the hex into chunks of ≤ 3000 bytes each
- **THEN** non-final chunks SHALL end with `\e]9277;C\a`
- **THEN** the final chunk SHALL end with `\e]9277;B\a`
- **THEN** the Rust receiver SHALL accumulate hex data from `C` chunks
- **THEN** on receiving `B`, the Rust receiver SHALL concatenate all accumulated hex and decode as a single payload

### Requirement: Interleaved OSC sequences SHALL NOT corrupt chunk accumulation

The chunk accumulation SHALL be scoped to a single generator command. Receiving a new `\e]9277;A\a` before the current sequence terminates SHALL reset the accumulator.

#### Scenario: New start resets accumulator
- **WHEN** `\e]9277;A\a` is received while `IsReceivingInBandCommandOutput::Yes` with `accumulated_hex` is already set
- **THEN** the `accumulated_hex` SHALL be discarded
- **THEN** a new `InBandCommandOutputReceiver` SHALL be created with empty `accumulated_hex`

### Requirement: Backward compatibility with `C` marker

Old Rust clients that do not handle `\e]9277;C\a` SHALL NOT crash or corrupt data. The unrecognized marker SHALL fall through to a warning log.

#### Scenario: Old client receives unknown marker
- **WHEN** a `\e]9277;C\a` OSC is received by code that does not handle the `C` byte
- **THEN** the OSC dispatch SHALL log a warning and ignore the marker
```

