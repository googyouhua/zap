# Verification Report: fix-onekey-ssh-password-prompt

## Summary
| 检查项 | 结果 |
|--------|------|
| tasks.md 全勾选 | ✅ 3/3 |
| 变更文件匹配 | ✅ 仅 `app/src/terminal/view.rs`(+3/-1) |
| Build 通过 | ✅ `cargo check -p warp` 通过;guard build PASS |
| 相关测试 | ✅ `cargo test -p warp --lib password_prompt` 12/12 |
| 安全问题 | ✅ 无(修复仅使 keychain 读取容错,不涉及 secret 日志/泄漏) |
| 代码 review | ⏭️ 跳过(review_mode=off,hotfix 预设;修复为单点容错,1 文件 4 行) |

## 根因与修复
- **根因**: `load_prompt_menu_candidates`(`app/src/terminal/view.rs:15580`)在 SSH 服务器循环里 `store.get(&node.id, secret_kind)?` 用 `?` 传播 `SshSecretStoreError::NoBackend`,导致无 keychain 环境下整个候选列表加载失败,OneKey 菜单不弹出(Task 5 commit 260098a5 引入的回归)。
- **修复**: 改为 `store.get(...).ok().flatten()`,keychain 错误时跳过该 SSH 服务器节点,`warp_onekey::find_all()` 收集的 OneKey 凭据不受影响。
- **复现证据**: `/root` 环境 `KeychainSecretStore.get → Err("no keychain backend available on this platform")`,同时 `find_all → 3 creds`。

## 结论
通过。修复消除了 root cause,无回归风险,可归档。
