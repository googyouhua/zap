# Brainstorm Summary

- Change: onekey-credential-system
- Date: 2026-07-31

## Confirmed Technical Approach

1. **统一 OneKey 数据模型与存储**:新建 `crates/onekey`(package `warp_onekey`),模型 `OneKeyKind{Password,Key}` / `OneKeyCredential{id,label,username,notes,kind,key_path,secret:Zeroizing<String>}`;DB 表 `onekey_credentials`(kind/key_path/encrypted_password 列)+ `prompt_trigger_rules` 表;Keychain service `zap.onekey`,account `<uuid>:secret`,失败 fallback `encrypted_password` 列。
2. **表结构创建方式**:干净迁移——新建 `crates/persistence/migrations/` 迁移:CREATE TABLE `onekey_credentials` + `prompt_trigger_rules`,DROP 旧 `ssh_onekey_credentials`(含 ssh_servers 外键重建);**不复制**参考分支的 legacy `quick_credentials` 死迁移;db.rs 运行时 `CREATE TABLE IF NOT EXISTS` + `ensure_columns` 保留作防御性层。
3. **终端搜索面板**:新建 `app/src/search/onekey/`,`OneKeyPanel`(SearchBar + SearchMixer + SearchBarState),复用 `app/src/search/external_secrets/` 的成熟模式;事件 `OneKeyPanelEvent::{ItemSelected{credential, mode}, Close}`;发送模式在选择凭据后即时选择,不存凭据上。
4. **快捷键**:`cmd_or_ctrl_shift("u")`(macOS cmd+shift+u,Linux/Win ctrl+shift+u)→ `TerminalAction::ToggleOneKeyPanel`;确认无冲突。
5. **发送引擎**:`send_credential()` 清行 → 写 `secret\n`;`UsernameThenPassword` 先 `username\n` 再 ~150ms 后 `password\n`;secret 全程 `Zeroizing<String>`。
6. **auto-send 完整实现**:保留 main 既有 `spawn_onekey_prompt_listener` 滑动窗口 + `show_onekey_prompt_menu`(数据源切换为 `warp_onekey::find_all()` + SSH 凭据),并把参考分支的 `classify_prompt`(PromptTriggerRule 分类)真正接入生产流程:加载 `list_rules()` → 分类 → 恰好一条凭据自动发送,否则回落菜单。**不保留**参考分支的 auto-send 死代码状态。
7. **双开关**:Cargo feature `onekey_input`(默认启用)+ 运行时 `FeatureFlag::OneKeyInput`(DOGFOOD)对齐参考;既有 `FeatureFlag::OneKeyPrompt` 继续门控提示监听避免回归;app 内 feature 门控模式对齐 `app/src/lib.rs:2333` 现有 `onekey_prompt` 用法。
8. **su_root 菜单**:保留 main 的 Password-only 过滤语义,数据源换为 `warp_onekey::find_all()`;不并入所有凭据。
9. **SSH 面板数据源切换**:`server_view.rs` OneKey overlay 渲染不变,CRUD API 从 `SshRepository::*_onekey_credential` 切到 `warp_onekey::*`;SSH 连接(`workspace/view.rs`、`sftp_manager/sftp_ops.rs`)`auth_type==OneKey` 直接 `warp_onekey::find_by_id()`。
10. **变更自动刷新**:`AtomicU64 CREDENTIALS_VERSION` + `OneKeyCredentialsChangedNotifier`(SingletonEntity,遵循 `SshTreeChangedNotifier` 模式)。
11. **设置页**:`app/src/settings_view/onekey_page.rs` 凭据 CRUD + Trigger Keywords 管理(两组关键词增删 + Reset 默认)。
12. **删除旧 SSH OneKey 系统**:删 `SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、5 个 `*_onekey_credential` repository 方法、`resolve_server_auth` OneKey 分支、`app/src/ssh_manager/onekey.rs`、`SyncOneKeyCredential`。

## Key Trade-offs and Risks

- **快捷键选择**:`u` 与 main `init.rs:254` 的 ClearBuffer(`ctrl+shift+k`)冲突,故选 `u`。
- **迁移策略**:参考分支有死迁移(`quick_credentials`),且 db.rs 运行时建表与 AGENTS.md §5.5 冲突 → 采用干净迁移,运行时建表仅作防御层。
- **auto-send**:参考分支实际是死代码(仅测试引用),本次完整接入生产流程,是对参考的改进而非完全照搬。
- **旧数据**:旧 SSH OneKey 数据不迁移,升级后作废需重新配置(已写入 proposal/design non-goals)。
- **TerminalModel 锁**:`spawn_onekey_prompt_listener` 在 view.rs 中操作 TerminalModel,需遵守 AGENTS.md §5.3,不新增嵌套锁。

## Testing Strategy

- `crates/onekey`:`types_tests.rs`(kind/key_path 序列化、SendMode、默认关键词)+ `repository_tests.rs`(CRUD、rules CRUD、CREDENTIALS_VERSION bump、fallback 加密路径)。
- `app`:`server_view_tests.rs`(OneKey overlay 数据源切换)、sftp browser 测试;`app/src/terminal` 相关测试沿用 main 既有模式。
- 全量:`cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`;PR 前 `cargo check`。
- 手动验证项(tasks.md 9.3):面板唤起/搜索/发送、SSH 面板 OneKey CRUD、设置页 CRUD、auto-send、跨视图自动刷新。

## Spec Patches

- 无 Spec Patches 计划;现有 5 个 delta spec(credential-store / credential-panel / credential-send / credential-management / auto-fill-trigger)已覆盖已确认决策。若设计实现中发现缺口,在实现阶段以 Spec Patch 补充。
