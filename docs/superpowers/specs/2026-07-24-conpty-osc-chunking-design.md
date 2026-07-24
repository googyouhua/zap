---
comet_change: conpty-osc-chunking
role: technical-design
canonical_spec: openspec
---

# ConPTY OSC Chunking — Design Doc

## Problem

两跳 SSH（Windows ConPTY → 跳板机 → 目标机）场景下，`load_external_commands()` 通过 in-band 生成器协议在目标机上执行 `compgen -c`，输出约 2137 个命令名，hex 编码后约 64KB。该负载经 ConPTY 回传时超出其 ~4KB 缓冲区，导致 hex 字符碎片泄漏到终端显示。

## Solution

新增 OSC 标记 `C`（chunk continue），扩展 in-band 生成器输出协议支持分片传输。单片 ≤3KB hex（安全小于 ConPTY 4KB 缓冲区）。

### 协议格式

```
正常（小输出）：
  \e]9277;A\a{byte_count};{hex}\e]9277;B\a

分片（大输出）：
  \e]9277;A\a{3000};{hex_1..3000}\e]9277;C\a
  \e]9277;A\a{3000};{hex_3001..6000}\e]9277;C\a
  ...
  \e]9277;A\a{1234};{hex_final}\e]9277;B\a
```

### Rust 状态机变更

`IsReceivingInBandCommandOutput` 新增 `accumulated_hex: String`：

| 事件 | 行为 |
|------|------|
| `A` 首次到达 | 创建新 receiver，`accumulated_hex=""` |
| `A` 再次到达（分片中） | 保留 `accumulated_hex`，重置 output receiver |
| `C` 到达 | 从 `output.as_str()` 提取 hex（strip `{len};`），追加到 `accumulated_hex`，重置 output receiver |
| `B` 到达 | 提取最后一片 hex → 构建 `{总长};{accumulated_hex+最后片}` → 解码 → 处理 |

### Shell 端变更

`_warp_execute_command` 输出 >3KB hex 时分片：

```bash
local chunk_size=3000
if [ ${#generator_output} -le $chunk_size ]; then
    warp_send_generator_output_osc_pre_hex_encoded "$generator_output"
else
    for chunk in split(generator_output, chunk_size); do
        if final_chunk; then
            printf "\e]9277;A\a${#chunk};$chunk\e]9277;B\a"
        else
            printf "\e]9277;A\a${#chunk};$chunk\e]9277;C\a"
        fi
    done
    warp_maybe_send_reset_grid_osc
fi
```

覆盖 bash / zsh / fish / PowerShell。

### 回退无效 fix

移除 `session.rs` 中 `if self.is_legacy_ssh_session() { return; }`。该 fix 在两跳 SSH 下 `is_legacy_ssh_session` 永远为 `No`，完全无效。

## File Changes

| 文件 | 改动 |
|------|------|
| `app/src/terminal/model/ansi/mod.rs` | 加 `C` 常量 + OSC 路由 |
| `app/src/terminal/model/ansi/handler.rs` | 加 `end_in_band_command_output_chunk()` trait 方法 |
| `app/src/terminal/model/terminal_model.rs` | 状态机 `accumulated_hex` + 分片处理 |
| `app/src/terminal/model/session.rs` | 回退 `is_legacy_ssh_session()` return |
| `app/assets/bundled/bootstrap/bash_body.sh` | `_warp_execute_command` 分片 |
| `app/assets/bundled/bootstrap/zsh_body.sh` | 同上 |
| `app/assets/bundled/bootstrap/fish.sh` | 同上 |
| `app/assets/bundled/bootstrap/pwsh.ps1` | 同上 |

## Risks

- **ConPTY reset grid**：中间分片不发送 reset grid，只在最终分片后发送。若某片导致光注偏移，后续可能累积偏移。
- **旧客户端兼容**：不认识 `C` 标记的代码会 fallthrough 到 `_` 分支打 warning，不影响稳定性。shell 和 Rust 应一起发布。
