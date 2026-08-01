# Proposal: fix-onekey-ssh-password-prompt

## Why

终端里执行 `ssh snic@127.0.0.1` 出现密码提示时,OneKey 菜单/自动发送没有弹出——即使 OneKey 凭据表里已保存了对应凭据(如 `snic`)。

## Root Cause

`show_onekey_prompt_menu` 的候选数据源 `load_prompt_menu_candidates`(`app/src/terminal/view.rs:15580`)在遍历 SSH 服务器时调用 `KeychainSecretStore.get(&node.id, secret_kind)?`(15613 行)。当系统没有可用的 keychain backend 时(如 headless Linux / root 无 secret-service),`get` 返回 `SshSecretStoreError::NoBackend`,`?` 把这个错误直接传播出整个函数,导致**整个候选列表加载失败**(包括本应从 `warp_onekey::find_all()` 拿到的有效 OneKey 凭据)。`show_onekey_prompt_menu` 的回调拿到 `Err` 后仅 `log::warn` 并返回,菜单永不显示。

复现(本机 `/root` 环境):`KeychainSecretStore.get(...)` → `Err("no keychain backend available on this platform")`,同时 `warp_onekey::find_all()` → 正常返回 3 条凭据(含 `snic`)。

这是 Task 5(commit `260098a5`)引入的回归:旧 `load_saved_ssh_credentials` 对 keychain 错误是容错的,新函数用 `?` 硬失败。

## What Changes

- 修改 `app/src/terminal/view.rs` 的 `load_prompt_menu_candidates`:SSH 服务器凭据循环里的 `store.get(...)` 不再用 `?` 传播错误,改为容错处理——`NoBackend` / keychain 错误时跳过该服务器节点(记 `log::debug`),不影响已从 `warp_onekey::find_all()` 收集的 OneKey 凭据。
- 行为不变:有 keychain 时 SSH 服务器凭据照常并入;无 keychain 时只跳过 SSH 服务器凭据,OneKey 凭据菜单照常弹出。

## Capabilities

- **New Capabilities**: 无
- **Modified Capabilities**: 无(spec 描述的行为不变,仅修复实现缺陷)

skip_specs: true(纯 bug 修复,行为语义不变)

## Impact

- 仅 `app/src/terminal/view.rs` 一处(`load_prompt_menu_candidates` 内 SSH 服务器凭据循环)。
- 无公共 API / 数据结构 / schema 变更。
