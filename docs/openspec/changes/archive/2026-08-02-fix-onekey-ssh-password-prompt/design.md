# Design: fix-onekey-ssh-password-prompt

## 问题

`load_prompt_menu_candidates` 在 SSH 服务器循环里用 `store.get(&node.id, secret_kind)?` 传播 keychain 错误,导致无 keychain 环境下整个候选列表加载失败,OneKey 菜单不弹出。

## 修复方案

把 `store.get(...)` 的 `?` 传播改为容错:

```rust
                // keychain 不可用(NoBackend / keyring 错误)时跳过该服务器凭据,
                // 不影响已从 warp_onekey::find_all() 收集的 OneKey 凭据。
                let Some(secret) = store.get(&node.id, secret_kind).ok().flatten() else {
                    continue;
                };
```

- `Ok(Some(secret))` → 正常并入候选。
- `Ok(None)`(无该凭据)→ 跳过。
- `Err(_)`(NoBackend / keyring 错误)→ **跳过该服务器节点**,`find_all()` 的 OneKey 凭据仍保留。
- 其余逻辑(host/username 拼 target、subtitle、kind 映射)不变。

## 为什么这样修

- 最小改动:只把错误语义从「硬失败」改成「跳过该节点」。
- 语义正确:OneKey 统一凭据是本功能的主体,SSH 服务器凭据是增强项;SSH keychain 缺失不应拖垮主体。
- 与旧 `load_saved_ssh_credentials` 的容错语义一致(旧实现对 keychain 错误容忍)。
- 无 keychain 时用户仍能看到 OneKey 凭据并弹出菜单,问题解决。
