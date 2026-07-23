# Brainstorm Summary

- Change: conpty-osc-chunking
- Date: 2026-07-24

## Confirmed Technical Approach

新增 OSC 9277 标记 `C`（chunk continue）实现 in-band 生成器输出分片。Shell 端 `_warp_execute_command` 将 hex 输出拆成 ≤3KB 分片，非最终片用 `C` 结束、最终片用 `B` 结束。Rust 端 `IsReceivingInBandCommandOutput` 新增 `accumulated_hex` 字段跨片累积 hex，`B` 到达时拼接总 hex 后解码。

## Key Trade-offs and Risks

- Reset grid 只在最终分片发送
- 旧客户端不认识 `C` → fallthrough warning，无破坏
- 2137 命令 / 64KB hex → ~22 片，对 PTY 无额外开销

## Testing Strategy

cargo check 编译；本地终端 load_external_commands 行为不变。

## Spec Patches

无。
