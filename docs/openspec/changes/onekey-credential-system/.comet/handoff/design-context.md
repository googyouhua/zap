# Comet Design Handoff

- Change: onekey-credential-system
- Phase: design
- Mode: compact
- Context hash: 2598b63e7d60d183993faa63498bc653c0e816c6ec51bc08bc01cc05ba535ebd

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## docs/openspec/changes/onekey-credential-system/proposal.md

- Source: docs/openspec/changes/onekey-credential-system/proposal.md
- Lines: 1-44
- SHA256: 2ed2d6e42050202d4121d2d5dcf06ca6c7ef2a3182cf18b97cb5999623346d28

```md
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

```

## docs/openspec/changes/onekey-credential-system/design.md

- Source: docs/openspec/changes/onekey-credential-system/design.md
- Lines: 1-111
- SHA256: 98febc81d4ea68c3cacc06302f3251d4870bd01b91456e4059ca5c9663144699

[TRUNCATED]

```md
## Context

现状(main 分支):SSH 面板内的旧 OneKey 系统(`SshOneKeyCredential` + `ssh_onekey_credentials` 表 + `KeychainSecretStore` + `app/src/ssh_manager/onekey.rs`)通过 `spawn_onekey_prompt_listener` 监控 PTY 输出,密码提示时弹出 `show_onekey_prompt_menu`。该系统与 SSH Manager 深度绑定,无法覆盖普通终端里手动执行的命令。另已有成熟的搜索面板模式(`app/src/search/external_secrets/` 的 `SearchBar<T>` + `SearchMixer<T>`)。动机见 proposal.md - Why。

参考分支 `feature/20260719/quick-credential-input` 已落地统一的 OneKey 系统并验证通过。本设计对齐其最终实现,结合 main 现状做适配。

## Goals / Non-Goals

**Goals:**
- 建立统一 `crates/onekey`(`warp_onekey`)凭据层:SQLite + OS Keychain + fallback,支持 Password/Key 两种类型
- SSH 面板的 OneKey UI 交互不变,底层数据源从旧 SSH DB 切到 `warp_onekey`
- SSH 连接时 OneKey 认证由 `warp_onekey::find_by_id()` 直接解析,不经过旧 `resolve_server_auth` / `KeychainSecretStore`
- 终端搜索面板 + 两种发送模式 + PTY 密码提示 auto-send
- 设置页 OneKeyPage 提供凭据与触发关键词管理
- 凭据变更通过 notifier 自动刷新所有已打开视图
- 删除旧 `ssh_onekey_credentials` 表与相关类型/API

**Non-Goals:**
- 不迁移旧 SSH OneKey 数据(升级后旧凭据作废,用户重新配置)
- 不改 SSH 面板 overlay 的 UI 布局或交互
- 不与外部密码管理器(1Password/LastPass)集成,不做云同步、密码生成器、网页表单填充

## Decisions

### Decision 1: 统一 OneKey 数据模型与存储
新建 `crates/onekey`,沿用参考分支最终模型:

```rust
pub enum OneKeyKind { Password, Key }

pub struct OneKeyCredential {
    pub id: String,
    pub label: String,
    pub username: String,
    pub notes: String,
    pub kind: OneKeyKind,
    pub key_path: Option<String>,
    pub secret: Zeroizing<String>,
}
```

DB 表 `onekey_credentials`:`kind`(TEXT, default 'password')+ `key_path`(TEXT, nullable)两列,`encrypted_password` 列保留作 keychain fallback。`prompt_trigger_rules` 表存触发关键词。Keychain service 名 `zap.onekey`,account key `<uuid>:secret`。

表结构通过 `CREATE TABLE IF NOT EXISTS` + `ensure_columns`(pragma 检查后 ALTER)在 `db.rs` 中维护,不新增 Diesel migration 目录 —— 与 `warp_ssh_manager::db` 的既有模式一致,避免为独立 crate 引入 app 层 migration 编排。

**替代方案**:复用 `warp_ssh_manager` 的 `ssh_onekey_credentials` 表加列。否决理由:旧表与 SSH 节点模型耦合(有 key_path/kind 等字段),且本 change 要删除该表;独立 crate 生命周期清晰、依赖无环(`warp_ssh_manager` 不依赖 `warp_onekey`)。

### Decision 2: 终端搜索面板复用 SearchBar + SearchMixer 模式
新建 `app/src/search/onekey/`,实现 `OneKeyPanel`,参考 `ExternalSecretsMenu`:

```
OneKeyPanel
  ├── SearchBar<OneKeyAction> — 搜索栏
  ├── SearchMixer<OneKeyAction> — 数据源(模糊匹配 label/username)
  ├── SearchBarState<OneKeyAction> — 列表状态
  └── OneKeyItem — 搜索结果项
```

面板事件 `OneKeyPanelEvent::{ ItemSelected { credential, mode }, Close }`,发送模式在选择凭据后即时选择,不存储在凭据上。

**替代方案**:自研轻量弹层。否决:放弃成熟的键盘导航/模糊搜索/定位渲染复用,风险更高。

### Decision 3: 发送引擎与 auto-send
`send_credential` 工具函数:清空当前行(`clear_line_editor_and_write_to_pty`)→ 写 `secret\n`;`UsernameThenPassword` 模式先写 `username\n`,~150ms 后写 `password\n`(`ctx.spawn_after`)。secret 全程 `Zeroizing<String>`。

密码提示检测复用 `spawn_onekey_prompt_listener` 滑动窗口,扩展为:加载触发规则 → 分类(PasswordOnly / UsernameThenPassword)→ 恰好一条凭据时自动发送,否则回落 `show_onekey_prompt_menu`(同时列出 OneKey 与 SSH 凭据)。OneKey 面板集成用 Cargo feature `onekey_input`(默认启用)门控;既有 `FeatureFlag::OneKeyPrompt` 继续控制提示监听,避免行为回归。

### Decision 4: SSH 面板数据源切换
`server_view.rs` 中 OneKey overlay 的渲染代码完全不动,只替换 CRUD API:
- `SshRepository::list_onekey_credentials()` → `warp_onekey::find_all()`
- `SshRepository::create_onekey_credential()` → `warp_onekey::create()`
- `SshRepository::delete_onekey_credential()` → `warp_onekey::delete()`
- `SshRepository::update_onekey_credential()` → `warp_onekey::update()`

SSH 连接流程(`open_ssh_terminal`、SFTP `build_auth_method`):`auth_type == OneKey` 时直接 `warp_onekey::find_by_id(credential_id)`,绕开旧 `resolve_server_auth` 与 keychain key 格式不兼容问题。

### Decision 5: 变更自动刷新
沿用参考分支方案:`crates/onekey` 的 repository 内 `AtomicU64 CREDENTIALS_VERSION` 计数器,`create/update/delete` 成功后 `bump_credentials_version()`;`app/src/ssh_manager/onekey_notifier.rs` 定义 `OneKeyCredentialsChangedNotifier`(`SingletonEntity`,遵循 `SshTreeChangedNotifier` 模式);写入端在成功后 emit 通知,订阅端(`SshServerView`、`OneKeyPageView`)监听后 reload 凭据列表。

**替代方案**:每次渲染时轮询版本号。否决:不够及时,且侵入渲染路径。

```

Full source: docs/openspec/changes/onekey-credential-system/design.md

## docs/openspec/changes/onekey-credential-system/tasks.md

- Source: docs/openspec/changes/onekey-credential-system/tasks.md
- Lines: 1-62
- SHA256: 084edd3152ba3307be1dab1ccb1d52938a35dc6bd9cd3d01156b99bda281cea6

```md
## 1. 存储层 crates/onekey

- [ ] 1.1 新建 `crates/onekey`(package `warp_onekey`)crate,配置 `Cargo.toml`(diesel/keyring/zeroize/uuid/persistence/warp_ssh_manager),加入 workspace members
- [ ] 1.2 实现 `db.rs`:`CREATE TABLE IF NOT EXISTS onekey_credentials` + `ensure_columns` 兼容 ALTER + `prompt_trigger_rules` 表;`set_database_path` / `with_conn` / 测试连接注入
- [ ] 1.3 实现 `types.rs`:`OneKeyKind` / `OneKeyCredential` / `SendMode` / `PromptTriggerRule` + 默认关键词常量
- [ ] 1.4 实现 `secret_store.rs`:`OneKeySecretStore`(keyring service `zap.onekey`,account `<uuid>:secret`,失败 fallback `encrypted_password` 列)
- [ ] 1.5 实现 `repository.rs`:`find_all` / `find_by_id` / `create` / `update` / `delete` / `list_rules` / `add_rule` / `remove_rule` / `reset_rules_for_mode` / `reset_rules_to_defaults`
- [ ] 1.6 实现 `repository.rs` 的 `AtomicU64 CREDENTIALS_VERSION`,`create/update/delete` 成功后 `bump_credentials_version()`,导出 `credentials_version()`
- [ ] 1.7 编写 `types_tests.rs` 与 `repository_tests.rs` 单元测试

## 2. Feature flag 与接线

- [ ] 2.1 在 `app/Cargo.toml` 新增 `onekey_input` feature(默认启用),并确认与 `onekey_prompt` 并存
- [ ] 2.2 `app/src/lib.rs` 中初始化 `warp_onekey::set_database_path(persistence::database_file_path())`
- [ ] 2.3 确认 `FeatureFlag::OneKeyPrompt` 继续门控提示监听,不回归

## 3. 终端搜索面板

- [ ] 3.1 新建 `app/src/search/onekey/`,实现 `OneKeyPanel`(SearchBar + SearchMixer + 列表状态),复用 ExternalSecretsMenu 模式
- [ ] 3.2 实现 `OneKeyItem` / 数据源(模糊匹配 label/username)/ `OneKeyPanelEvent::{ ItemSelected { credential, mode }, Close }`
- [ ] 3.3 实现选中凭据后的发送模式选择 UI("仅发送密码" / "先用户名再密码")
- [ ] 3.4 注册快捷键 `ToggleOneKeyPanel`(`ctrl+shift+k` / macOS `cmd+shift+k`),确认无按键冲突

## 4. 发送引擎

- [ ] 4.1 实现 `send_credential()`:清行 → 写 `secret\n`;`UsernameThenPassword` 先写 `username\n`,~150ms 后写 `password\n`
- [ ] 4.2 全程使用 `Zeroizing<String>` 持有 secret
- [ ] 4.3 在 `TerminalView` 中创建面板实例、订阅事件并路由到发送引擎,`render()` 中定位渲染

## 5. PTY auto-send 扩展

- [ ] 5.1 扩展 `spawn_onekey_prompt_listener` 滑动窗口检测:加载 `list_rules()` 分类 PromptType
- [ ] 5.2 恰好一条凭据时按 SendMode 自动发送;0 或多条时回落 OneKey 菜单
- [ ] 5.3 替换 `show_onekey_prompt_menu` 数据源:同时展示 `warp_onekey::find_all()` 与 SSH 凭据

## 6. SSH 面板数据源切换

- [ ] 6.1 `server_view.rs`:OneKey overlay 增删改 API 从 `SshRepository::*_onekey_credential` 切到 `warp_onekey::*`,UI 渲染不变
- [ ] 6.2 `workspace/view.rs` 与 `sftp_manager/sftp_ops.rs`:`auth_type == OneKey` 时直接 `warp_onekey::find_by_id()` 解析认证
- [ ] 6.3 `SshServerView` 订阅 `OneKeyCredentialsChangedNotifier`,`on_save/on_delete_managed_onekey_credential` 成功后 emit 通知

## 7. 设置页 OneKeyPage

- [ ] 7.1 新建 `app/src/settings_view/onekey_page.rs` 及入口:凭据列表(label/username/kind)
- [ ] 7.2 实现凭据表单(新增/编辑):label、username、password、kind 选择器、key_path、notes;校验 label/password 必填
- [ ] 7.3 实现删除确认对话框
- [ ] 7.4 实现 Trigger Keywords 区块:两组关键词的增删 + 重置为默认
- [ ] 7.5 `OneKeyPageView` 订阅通知刷新列表,写入成功后 emit 通知
- [ ] 7.6 新建 `app/src/ssh_manager/onekey_notifier.rs`(`OneKeyCredentialsChangedNotifier`,SingletonEntity),在 `mod.rs` 导出、`app/src/lib.rs` 注册 singleton

## 8. 删除旧 SSH OneKey 系统

- [ ] 8.1 删除 `crates/warp_ssh_manager`:`SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、5 个 `*_onekey_credential` repository 方法、`resolve_server_auth` 的 OneKey 分支、`SyncOneKeyCredential`
- [ ] 8.2 删除 `crates/persistence` 中 `ssh_onekey_credentials` model/schema 引用,新增 drop 表 migration(up/down)
- [ ] 8.3 删除 `app/src/ssh_manager/onekey.rs`(`load_saved_ssh_credentials`)与终端 `show_onekey_prompt_menu` 的旧数据源
- [ ] 8.4 全局 grep `quick_credential` / `SshOneKeyCredential` 确认无残留引用

## 9. 验证

- [ ] 9.1 `cargo check` 通过
- [ ] 9.2 运行 `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` 全量测试
- [ ] 9.3 手动验证:面板唤起/搜索/发送、SSH 面板 OneKey CRUD、设置页 CRUD、auto-send、跨视图自动刷新

```

## docs/openspec/changes/onekey-credential-system/specs/auto-fill-trigger/spec.md

- Source: docs/openspec/changes/onekey-credential-system/specs/auto-fill-trigger/spec.md
- Lines: 1-86
- SHA256: 38b6bdf0c50af7a6fad7472c9eb96a792a5bc0ace6cfd947518429a9c0549bc1

[TRUNCATED]

```md
## Purpose

可配置的关键词到发送模式的映射规则,供终端密码提示检测器对终端输出做分类,在只有一条凭据时自动发送,否则回落 OneKey 菜单。

## ADDED Requirements

### Requirement: 首次运行时填入默认触发关键词
当功能首次启用且尚无触发规则时,系统 SHALL 填入默认规则:PasswordOnly 关键词 = {password, passphrase},UsernameThenPassword 关键词 = {login, username, user, name, email, account}。

#### Scenario: 首次初始化
- **WHEN** 系统加载触发规则且发现表为空
- **THEN** 插入默认关键词集合并返回

#### Scenario: 后续加载保留用户修改
- **WHEN** 用户已自定义触发规则,系统在下次启动时加载
- **THEN** 返回用户的规则,而非默认集合

### Requirement: 新增触发关键词
系统 SHALL 允许新增带关联 SendMode 的关键词。

#### Scenario: 新增仅密码关键词
- **WHEN** 用户新增关键词 "secret" 且模式为 PasswordOnly
- **THEN** 关键词 "secret" 以 mode "password_only" 插入触发规则表

#### Scenario: 新增用户名+密码关键词
- **WHEN** 用户新增关键词 "account" 且模式为 UsernameThenPassword
- **THEN** 关键词 "account" 以 mode "username_then_password" 插入触发规则表

#### Scenario: 拒绝重复关键词
- **WHEN** 用户尝试新增已存在的关键词
- **THEN** 系统拒绝该重复项(no-op 或报错)

### Requirement: 删除触发关键词
系统 SHALL 允许从触发规则中移除关键词。

#### Scenario: 删除关键词
- **WHEN** 用户点击关键词 "passphrase" 的删除按钮
- **THEN** 关键词 "passphrase" 从触发规则表移除

### Requirement: 将触发关键词重置为默认
系统 SHALL 提供清空当前全部规则并重新插入默认关键词集合的入口。

#### Scenario: 重置为默认
- **WHEN** 用户点击 "Reset" 按钮
- **THEN** 当前全部触发规则被删除,默认关键词集合被插入

### Requirement: 对 PTY 输出按触发规则分类
系统 SHALL 将终端 PTY 输出与全部已配置触发关键词比对,判断是否存在密码型或用户名型提示。

#### Scenario: 检测到密码提示
- **WHEN** 终端输出包含 "password:" 且关键词 "password" 的规则模式为 PasswordOnly
- **THEN** 检测返回 PromptType::Password

#### Scenario: 检测到用户名提示
- **WHEN** 终端输出包含 "login:" 且关键词 "login" 的规则模式为 UsernameThenPassword
- **THEN** 检测返回 PromptType::Username

#### Scenario: 无关键词匹配
- **WHEN** 终端输出不匹配任何触发关键词
- **THEN** 检测返回 None

### Requirement: 在 PTY 输出流中持续分类提示
提示检测器 SHALL 持续监控 PTY 输出流,当终端输出匹配关键词模式时触发分类,复用既有 OneKey 密码提示检测的滑动窗口方式。

#### Scenario: 持续监控
- **WHEN** PTY 输出流动且出现匹配触发模式的片段
- **THEN** 检测器触发并进入 auto-send 逻辑

### Requirement: 恰好一条凭据时自动发送
当提示被分类时,系统 SHALL 加载全部 OneKey 凭据。若恰好存在一条凭据,系统 SHALL 按匹配关键词规则对应的 SendMode 自动发送;若为 0 条或多条,系统 SHALL 回落展示 OneKey 菜单。

#### Scenario: 单条凭据自动发送密码
- **WHEN** 检测到密码型提示且恰好存在一条 OneKey 凭据
- **THEN** 该凭据的密码被发送到 PTY(仅密码)

#### Scenario: 单条凭据自动发送用户名+密码
- **WHEN** 检测到用户名型提示且恰好存在一条 OneKey 凭据
- **THEN** 先发送该凭据用户名,~150ms 后发送密码

#### Scenario: 多条凭据回落 OneKey 菜单

```

Full source: docs/openspec/changes/onekey-credential-system/specs/auto-fill-trigger/spec.md

## docs/openspec/changes/onekey-credential-system/specs/credential-management/spec.md

- Source: docs/openspec/changes/onekey-credential-system/specs/credential-management/spec.md
- Lines: 1-79
- SHA256: e26965a9b64e1d19a4cb9f5875bf35b994dbe147d47964e44f2283771a1acd2f

```md
## Purpose

OneKey 凭据的管理界面:设置页 `onekey_page.rs` 提供凭据的增删改查与触发关键词管理,SSH 面板内的 OneKey Manager overlay 复用统一凭据数据源,并在凭据变更后自动刷新所有已打开视图。

## ADDED Requirements

### Requirement: 设置页列出全部已保存凭据
设置页 SHALL 以列表展示全部 OneKey 凭据,每条显示 label、username 预览与类型(kind)标识。

#### Scenario: 查看凭据列表
- **WHEN** 用户导航到 OneKey 设置页
- **THEN** 全部已保存凭据以 label、username、类型展示在列表中

### Requirement: 新增凭据
设置页 SHALL 提供新增凭据表单,字段包含 label、username、password、kind 选择器、key_path、notes。

#### Scenario: 成功新增凭据
- **WHEN** 用户填完表单并点击保存
- **THEN** 凭据被持久化(SQLite + OS Keychain)并出现在列表中

#### Scenario: 缺少 label
- **WHEN** 用户未填 label 就尝试保存
- **THEN** 显示错误 "Label is required",凭据不保存

#### Scenario: 缺少密码
- **WHEN** 用户未填 password 就尝试保存
- **THEN** 显示错误 "Password is required",凭据不保存

### Requirement: 编辑既有凭据
设置页 SHALL 允许编辑既有凭据的全部字段。

#### Scenario: 编辑凭据 label
- **WHEN** 用户编辑既有凭据的 label 并保存
- **THEN** 该凭据的 label 在 SQLite 中更新

#### Scenario: 编辑凭据密码
- **WHEN** 用户编辑既有凭据的 password 并保存
- **THEN** 新密码存储到 OS Keychain

### Requirement: 删除凭据
设置页 SHALL 允许删除凭据,删除前需确认对话框。

#### Scenario: 删除凭据
- **WHEN** 用户点击某条凭据的删除并确认
- **THEN** 该凭据从 SQLite 与 OS Keychain 移除

#### Scenario: 取消删除
- **WHEN** 用户点击删除但取消确认
- **THEN** 凭据不被删除

### Requirement: 管理触发关键词
设置页 SHALL 展示触发关键词管理区块,分为 PasswordOnly 与 UsernameThenPassword 两组关键词,支持新增、删除、重置为默认。

#### Scenario: 查看触发关键词
- **WHEN** 用户查看 OneKey 设置页
- **THEN** 凭据列表上方展示 "Trigger Keywords" 区块,含 PasswordOnly 与 UsernameThenPassword 两组

#### Scenario: 新增触发关键词
- **WHEN** 用户点击 "+ Add" 并输入关键词
- **THEN** 关键词加入对应分组并持久化

#### Scenario: 删除触发关键词
- **WHEN** 用户点击某关键词 chip 的 × 按钮
- **THEN** 关键词被移除并持久化

#### Scenario: 重置触发关键词
- **WHEN** 用户点击 Trigger Keywords 区块的 "Reset" 按钮
- **THEN** 当前全部关键词被默认集合替换

### Requirement: 凭据变更后自动刷新视图
系统 SHALL 在任一视图新增/修改/删除 OneKey 凭据后,通知所有已打开依赖凭据列表的视图自动重新加载,无需用户手动刷新。

#### Scenario: 增删改后其它视图自动刷新
- **WHEN** 用户在设置页新增一条凭据,而 SSH 面板或 OneKey 面板已打开
- **THEN** 已打开的视图自动重新加载凭据列表并展示新凭据

#### Scenario: 删除后列表即时更新
- **WHEN** 用户在 SSH 面板 OneKey Manager 中删除一条凭据
- **THEN** 该 overlay 的凭据列表即时刷新,已删除项消失

```

## docs/openspec/changes/onekey-credential-system/specs/credential-panel/spec.md

- Source: docs/openspec/changes/onekey-credential-system/specs/credential-panel/spec.md
- Lines: 1-53
- SHA256: b37177a22ea629e156f67876d1126935ffa403b386c75e2115a7be56dd365634

```md
## Purpose

终端内的 OneKey 凭据搜索面板:用户通过快捷键在任意终端中唤起面板,模糊搜索已保存凭据,用键盘选择凭据并选择发送模式,覆盖 SSH 面板之外的普通命令密码输入场景。

## ADDED Requirements

### Requirement: 快捷键唤起凭据搜索面板
系统 SHALL 在终端中按下配置的快捷键时显示凭据搜索面板。面板 SHALL 包含搜索栏与可滚动的凭据列表。

#### Scenario: 通过快捷键打开面板
- **WHEN** 用户在终端中按下 `ctrl+shift+k`(或已配置快捷键)
- **THEN** 搜索面板出现在终端中央,聚焦于搜索栏

#### Scenario: 通过 Escape 关闭面板
- **WHEN** 面板打开且用户按下 Escape
- **THEN** 面板关闭

#### Scenario: 点击面板外部关闭
- **WHEN** 面板打开且用户点击面板外部
- **THEN** 面板关闭

### Requirement: 模糊搜索凭据
系统 SHALL 在用户输入时按 label 与 username 字段做大小写不敏感的模糊匹配,过滤凭据列表。

#### Scenario: 按 label 搜索
- **WHEN** 用户在搜索栏输入 "prod"
- **THEN** label 包含 "prod"(如 "prod-db"、"production-server")的凭据出现,并按相关性排序

#### Scenario: 按 username 搜索
- **WHEN** 用户输入 "admin"
- **THEN** username 为 "admin" 的凭据出现

#### Scenario: 无匹配结果
- **WHEN** 用户输入无法匹配任何凭据的文本
- **THEN** 列表显示 "No matching credentials"

### Requirement: 键盘导航与选择凭据
系统 SHALL 支持通过 Up/Down 方向键在凭据列表中导航,通过 Enter 选择。

#### Scenario: 导航并选择
- **WHEN** 用户按下 Down 方向键,再对选中的凭据按 Enter
- **THEN** 该凭据被选中,面板进入发送模式选择

### Requirement: 选择凭据后展示发送模式选项
系统 SHALL 在用户选中凭据后展示两个选项:"仅发送密码"与"先用户名再密码"。所选发送模式 SHALL 随 `ItemSelected` 事件一起发出,不存储在凭据本身。

#### Scenario: 展示发送模式选项
- **WHEN** 用户从面板选中一条凭据
- **THEN** 展示"仅发送密码"与"先用户名再密码"两个按钮

#### Scenario: 随事件发出模式
- **WHEN** 用户点击"仅发送密码"
- **THEN** 面板发出 `ItemSelected { credential, mode: PasswordOnly }`

```

## docs/openspec/changes/onekey-credential-system/specs/credential-send/spec.md

- Source: docs/openspec/changes/onekey-credential-system/specs/credential-send/spec.md
- Lines: 1-40
- SHA256: bb4877679b66f3fb8061a9ca13cda05bf954c1f60acaf0f4e66ee0ec6500b0ba

```md
## Purpose

把 OneKey 凭据发送到终端 PTY 的发送引擎:支持从面板手动选择发送模式,也支持由 PTY 密码提示触发自动发送,并在发送前清空当前输入行。

## ADDED Requirements

### Requirement: 通过面板仅发送密码
系统 SHALL 在用户于面板选择"仅发送密码"时,只把密码与换行写入 PTY。

#### Scenario: 发送密码到 PTY
- **WHEN** 用户选择凭据 "my-server" 并选择"仅发送密码"
- **THEN** 终端当前行被清空,然后 `password\n` 写入 PTY

### Requirement: 通过面板先用户名再密码
系统 SHALL 在用户选择"先用户名再密码"时,先把用户名与换行写入 PTY,等待 ~150ms,再把密码与换行写入 PTY。

#### Scenario: 发送用户名再密码到 PTY
- **WHEN** 用户选择凭据 "my-app"(username="admin")并选择"先用户名再密码"
- **THEN** 终端当前行被清空,`admin\n` 写入,~150ms 后 `password\n` 写入

### Requirement: 检测到密码提示时自动发送
当提示检测器将终端输出分类为密码型提示且恰好存在一条 OneKey 凭据时,系统 SHALL 只自动发送该凭据的密码。

#### Scenario: 自动发送密码
- **WHEN** 终端输出匹配 Password 关键词且恰好存在一条 OneKey 凭据
- **THEN** 该凭据的密码与换行写入 PTY

### Requirement: 检测到用户名提示时自动发送
当提示检测器将终端输出分类为用户名型提示且恰好存在一条 OneKey 凭据时,系统 SHALL 先发送用户名,等待 ~150ms,再发送密码。

#### Scenario: 自动发送用户名再密码
- **WHEN** 终端输出匹配 Username 关键词且恰好存在一条 OneKey 凭据
- **THEN** 先写入用户名,~150ms 后写入密码

### Requirement: 敏感数据使用 Zeroizing
系统 SHALL 将所有密码用 `Zeroizing<String>` 包裹,确保内存中的密码在引用全部释放后被清零。

#### Scenario: 发送后密码被清零
- **WHEN** 凭据的密码已写入 PTY 且所有引用被释放
- **THEN** 先前持有密码的内存被清零

```

## docs/openspec/changes/onekey-credential-system/specs/credential-store/spec.md

- Source: docs/openspec/changes/onekey-credential-system/specs/credential-store/spec.md
- Lines: 1-54
- SHA256: 62b4099111e04e647fc19092f6b9b7d7b1031e4b06f44bb4708875cbb125bc31

```md
## Purpose

统一 OneKey 凭据的持久化层:用 SQLite(`onekey_credentials` 表)存储凭据元数据,用 OS Keychain(`keyring`,service `zap.onekey`)存储 secret,keychain 不可用时 fallback 到 `encrypted_password` 列,并支持 Password 与 Key 两种凭据类型。

## ADDED Requirements

### Requirement: 存储凭据元数据到 SQLite
系统 SHALL 将凭据元数据(label、username、notes、kind、key_path)持久化到 SQLite `onekey_credentials` 表。每条凭据 MUST 以唯一 UUID 作为主键。

#### Scenario: 创建密码型凭据
- **WHEN** 用户保存一条 kind="password"、label="my-server"、username="admin"、secret="s3cret!" 的新凭据
- **THEN** `onekey_credentials` 表插入一行,kind='password'、key_path=null,并设置 created_at/updated_at

#### Scenario: 创建密钥型凭据
- **WHEN** 用户保存一条 kind="key"、label="my-key"、key_path="/home/user/.ssh/id_ed25519"、secret="passphrase123" 的新凭据
- **THEN** `onekey_credentials` 表插入一行,kind='key'、key_path='/home/user/.ssh/id_ed25519'

#### Scenario: 列出全部凭据
- **WHEN** 系统为面板加载凭据
- **THEN** 按 label 升序返回 `onekey_credentials` 表全部行

#### Scenario: 更新凭据的 kind 与 key_path
- **WHEN** 用户把某条凭据从 kind="password" 改为 "key" 并设置 key_path
- **THEN** 该行的 kind 与 key_path 字段被更新,updated_at 刷新

#### Scenario: 删除凭据
- **WHEN** 用户删除一条凭据
- **THEN** 该行从 `onekey_credentials` 表移除,对应 OS keychain 条目一并删除

### Requirement: 在 OS Keychain 存储 secret
系统 SHALL 通过 `keyring` crate 将 secret(密码或口令)存入 OS keychain,service 名为 `zap.onekey`,account key 为 `<credential-uuid>:secret`。OS keychain 不可用时,secret SHALL fallback 到 `encrypted_password` 列。

#### Scenario: 保存新 secret
- **WHEN** 一条凭据以 secret "s3cret!" 创建
- **THEN** keychain 条目 `zap.onekey / <uuid>:secret` 包含 "s3cret!"

#### Scenario: 读取 secret
- **WHEN** 系统需要发送某凭据的 secret
- **THEN** 从 keychain `zap.onekey / <uuid>:secret` 读取,keychain 失败时 fallback 到 `encrypted_password` 列

#### Scenario: 删除 secret
- **WHEN** 一条凭据被删除
- **THEN** 对应 keychain 条目同步删除

### Requirement: 支持带 key_path 的密钥型凭据
系统 SHALL 通过 `kind` 字段支持两种凭据类型:`password`(username + secret)与 `key`(username + key_path + secret/passphrase)。密码型 secret 直接用于 SSH 密码认证;密钥型提供私钥路径与可选口令,用于 SSH 公钥认证。

#### Scenario: 创建密钥型凭据
- **WHEN** 用户创建 kind="key"、key_path="/home/user/.ssh/id_ed25519" 的凭据
- **THEN** 凭据以 kind='key'、key_path 已设为该值保存

#### Scenario: 列表展示所有类型
- **WHEN** 系统列出凭据
- **THEN** 密码型与密钥型凭据都出现在列表中

```
