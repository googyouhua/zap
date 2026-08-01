# Tasks: fix-onekey-ssh-password-prompt

- [x] 1.1 修改 `app/src/terminal/view.rs` 的 `load_prompt_menu_candidates`:把 `store.get(&node.id, secret_kind)?` 改为 `.ok().flatten()` 容错,keychain 错误时跳过该服务器节点
- [x] 1.2 复现验证:`cargo check -p warp` 通过;本机(root,无 keychain backend)确认 `find_all()` 3 条凭据 + `KeychainSecretStore.get` 返回 Err 的场景下,候选列表不再整体失败
- [x] 1.3 若存在单测,补充/更新 `load_prompt_menu_candidates` 对 keychain 错误容错的断言;提交 `fix: OneKey 菜单在无 keychain 环境下不弹出的回归修复`
