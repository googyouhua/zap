## Why

当前 main 分支只有一套与 SSH Manager 深度绑定的旧 OneKey 系统(`SshOneKeyCredential` + `ssh_onekey_credentials` 表 + `load_saved_ssh_credentials`),只能覆盖 SSH 面板内的服务器凭据,无法处理普通终端中手动执行命令(`ssh user@host`、`mysql -u`、`docker login`)的密码输入。同时这套系统与 Quick Credential 等其它凭据方案功能重叠、数据隔离。参考分支 `feature/20260719/quick-credential-input` 已验证并落地了一套统一的 OneKey 凭据系统,本 change 在 main 上重新实现并完整对齐该分支的最终状态。

## What Changes

- **新建 `crates/onekey`(`warp_onekey`)统一凭据 crate**:SQLite(`onekey_credentials` 表)存储元数据 + OS Keychain(`keyring`,service `zap.onekey`)存储 secret,keychain 不可用时 fallback 到 `encrypted_password` 列
- **统一数据模型**:`OneKeyCredential` 支持 `kind`(Password/Key)与 `key_path`,覆盖旧 OneKey 的密码与密钥/口令两类
- **终端搜索面板**:`app/src/search/onekey/` 实现 `OneKeyPanel`(SearchBar + SearchMixer),hotkey(`ctrl+shift+k`)触发,模糊搜索 + 键盘导航 + 选中后发送模式选择
- **发送引擎**:`SendMode::{PasswordOnly, UsernameThenPassword}`,清行后写 PTY,`UsernameThenPassword` 模式间隔 ~150ms,secret 全程 `Zeroizing<String>`
- **PTY 自动检测 + auto-send**:扩展密码提示监听,`prompt_trigger_rules` 表存可配置关键词(默认 PasswordOnly={password, passphrase}, UsernameThenPassword={login, username, user, name, email, account});恰好一条凭据时自动发送,否则回落 OneKey 菜单
- **设置页**:`app/src/settings_view/onekey_page.rs` 提供凭据 CRUD + 触发关键词管理
- **SSH 面板数据源切换**:`server_view.rs` OneKey overlay 的增删改 UI 不变,底层从 `SshRepository::*_onekey_credential` 切换到 `warp_onekey::*`;SSH 连接时 OneKey 认证由 `warp_onekey::find_by_id()` 直接解析
- **变更自动刷新**:`OneKeyCredentialsChangedNotifier`(SingletonEntity)+ `AtomicU64` 版本号,凭据增删改后所有已打开视图自动 reload
- **删除旧 SSH OneKey 系统** **BREAKING**:删 `SshOneKeyCredential`、`ssh_onekey_credentials` 表(migration drop)、`SecretKind::OneKeyPassword`、`SshRepository::*_onekey_credential` CRUD、`app/src/ssh_manager/onekey.rs`、终端 `show_onekey_prompt_menu`
- **Feature flag**:`quick_credential_input` → `onekey_input`,默认启用

> 注:本 change 覆盖多个可独立交付的模块(存储 crate、SSH 整合、终端面板、auto-send、设置页),参考分支本身用 5 个 change 完成。经确认,本次**保持单一 change**,原因:统一 OneKey 各模块共享同一数据模型与存储层,合并实现可避免重复设计/迁移中间态,且所有模块依赖 `crates/onekey` 这一共同基础。

## Capabilities

### New Capabilities
- `credential-store`: 统一 OneKey 持久化层 —— `onekey_credentials` 表 + OS Keychain + fallback,支持 Password/Key 两种类型与 `key_path`
- `credential-panel`: 终端搜索面板 —— hotkey 触发、模糊搜索、键盘导航、发送模式选择
- `credential-send`: 发送引擎 —— 仅密码 / 先用户名再密码写入 PTY,`Zeroizing` 保护 secret
- `credential-management`: 凭据管理 UI —— 设置页 `onekey_page.rs` + SSH 面板 OneKey Manager overlay(CRUD + 变更自动刷新)
- `auto-fill-trigger`: PTY 密码提示触发规则 —— 可配置关键词、默认关键词、auto-send 与回落逻辑

### Modified Capabilities
<!-- main 上无既有 credential spec,全部为新增能力。 -->

## Impact

- 新 crate:`crates/onekey`(package `warp_onekey`),依赖 `diesel`/`keyring`/`zeroize`/`warp_ssh_manager`/`persistence`
- `crates/warp_ssh_manager/`:删 `SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、5 个 onekey repository 方法、`resolve_server_auth` 的 OneKey 分支
- `crates/persistence/`:删 `ssh_onekey_credentials` model/schema,加 drop table migration
- `app/src/ssh_manager/onekey.rs`:删除;`app/src/ssh_manager/server_view.rs`:OneKey overlay 数据源切换 + 通知订阅
- `app/src/terminal/view.rs`:删 `show_onekey_prompt_menu`,保留并增强 auto-send,集成 `OneKeyPanel`
- `app/src/workspace/view.rs`、`app/src/sftp_manager/sftp_ops.rs`:OneKey 认证直接走 `warp_onekey::find_by_id()`
- `app/src/settings_view/`:新增 `onekey_page.rs` 及入口
- `app/src/search/onekey/`:新增面板模块
- `app/src/lib.rs`、`app/src/ssh_manager/mod.rs`:注册 notifier singleton 与数据库路径初始化
- Feature flag:`crates/warp_core/src/features.rs` 新增 `onekey_input`
- 旧 SSH OneKey 数据不迁移,升级后旧凭据作废,需重新配置
