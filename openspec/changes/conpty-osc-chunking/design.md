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
