# Comet Design Handoff

- Change: quick-credential-input
- Phase: design
- Mode: compact
- Context hash: bb3bca61b91de1d1bf7e0449bcad8a9e1a0884ec71ca7d7cc15ff8106d9c49d9

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/quick-credential-input/proposal.md

- Source: openspec/changes/quick-credential-input/proposal.md
- Lines: 1-35
- SHA256: cadb5b697e9531c4722afc1cd6940213e3c6d8d2afccdc167c915ba72625fa8f

```md
## Why

用户在终端中频繁需要输入用户名和密码（ssh 登录、web 服务 login、sudo、数据库客户端等场景），每次都手动输入既效率低又容易出错。现有 OneKey 系统仅支持 SSH Manager 托管的连接，无法覆盖普通终端中手动执行的 `ssh user@host`、`mysql -u`、`docker login` 等命令。需要一个通用的快捷凭证输入功能，让用户通过快捷键快速选择并自动发送凭证到 shell。

## What Changes

- **通用凭证存储**：扩展 OneKey 的凭证模型，支持非 SSH 场景的通用用户名/密码对
- **凭证搜索面板**：新增类似 Command Palette 的搜索面板，通过快捷键触发，模糊搜索凭证
- **两种发送模式**：用户在选择凭证后可选择「仅发送密码」或「先发送用户名再发送密码」
- **密码自动检测增强**：扩展 PTY 输出检测，密码提示弹出时同时展示通用凭证
- **凭证管理 UI**：在设置中新增页面，支持添加、编辑、删除通用凭证
- **快捷键**：注册新的可编辑快捷键绑定，在 Terminal 上下文中触发凭证面板

## Capabilities

### New Capabilities

- `credential-store`: 通用凭证存储。SQLite 存储元数据（label、username、发送模式、备注），OS Keychain 存储实际密码。支持 CRUD 操作。
- `credential-panel`: 凭证搜索选择面板。类似 Command Palette 的搜索栏，模糊匹配凭证 label 和 username，选择后弹出发送模式选项。
- `credential-send`: 凭证发送引擎。将选中的凭证按照指定模式（password-only / username-then-password）发送到当前终端的 PTY。
- `credential-management`: 设置中的凭证管理 UI。在设置界面中新增页面，用户可查看、添加、编辑、删除通用凭证。

### Modified Capabilities

- `onekey-prompt`: 扩展现有 PTY 密码自动检测机制，在检测到密码提示时同时加载通用凭证（credential-store 中的凭证），而不仅限于 SSH OneKey 凭证。

## Impact

- `app/src/ssh_manager/onekey.rs` — 扩展凭证加载逻辑，新增通用凭证加载
- `app/src/terminal/view.rs` — 集成凭证面板、注册快捷键、扩展 PTY 检测
- `app/src/settings_view/` — 新增凭证管理页面
- `app/src/search/` — 新增 `quick_credential/` 搜索面板模块
- `crates/persistence/src/model.rs` — 新增通用凭证数据模型
- `migrations/` — 新增 SQLite 迁移创建通用凭证表
- `crates/warp_ssh_manager/src/` — 扩展 repository 支持通用凭证 CRUD
```

## openspec/changes/quick-credential-input/design.md

- Source: openspec/changes/quick-credential-input/design.md
- Lines: 1-165
- SHA256: 6be529975197db19ffd44c8f8c6c61a8d7749da0abbbb24b829e5cb1b50e2c19

[TRUNCATED]

```md
## Context

Warp 已有 SSH OneKey 凭证系统（`app/src/ssh_manager/onekey.rs`），使用 SQLite 存储元数据 + OS Keychain（`keyring` crate）存储密码，并通过 `spawn_onekey_prompt_listener` 监控 PTY 输出实现密码提示自动检测。但该系统与 SSH Manager 深度绑定，无法覆盖普通终端中手动执行的命令（`ssh user@host`、`mysql -u`、`docker login` 等）。

同时代码库中有成熟的搜索面板模式（`app/src/search/external_secrets/` 中的 `ExternalSecretsMenu`），使用 `SearchBar<T>` + `SearchMixer<T>` 实现类似 Command Palette 的交互。

## Goals / Non-Goals

**Goals:**
- 通用凭证存储：独立于 SSH 的通用用户名/密码对存储，支持 CRUD
- 快捷键触发搜索面板，选择和发送凭证
- 两种发送模式：仅密码 / 先用户名再密码
- 扩展 PTY 自动检测，同时展示通用凭证
- 设置中新增凭证管理 UI

**Non-Goals:**
- 不与外部密码管理器（1Password/LastPass）集成
- 不涉及云同步
- 不修改 SSH Manager 现有流程（兼容共存）
- 不实现密码生成器
- 不自动填充网页表单

## Decisions

### D1: 新建通用凭证表，而非扩展现有 SSH 表

**选择**: 新建 `quick_credentials` SQLite 表。

**理由**: 现有 `ssh_onekey_credentials` 表与 SSH 节点模型耦合（有 `key_path`、`kind=Password|Key` 等 SSH 特定字段）。通用凭证模型更简单（label, username, send_mode, notes），且生命周期管理独立。两类凭证在查询时可 union 展示。

**Schema:**
```sql
CREATE TABLE quick_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    username TEXT NOT NULL DEFAULT '',
    send_mode TEXT NOT NULL DEFAULT 'password_only',  -- 'password_only' | 'username_then_password'
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### D2: OS Keychain 存储密码（复用 OneKey 的 `keyring` 机制）

**选择**: 使用 `keyring` crate，service name 为 `zap.quick-credential`，account key 为 `<uuid>:password`。

**理由**: 复用已有的 `KeychainSecretStore` 模式，无需引入新依赖。跨平台兼容（macOS Keychain / Linux Secret Service / Windows Credential Manager）。

### D3: 搜索面板模式（参考 ExternalSecretsMenu）

**选择**: 新建 `app/src/search/quick_credential/` 模块，实现 `QuickCredentialPanel` 视图。

**架构:**
```
QuickCredentialPanel
  ├── SearchBar<QuickCredentialAction>  — 搜索栏
  ├── SearchMixer<QuickCredentialAction> — 数据源
  ├── SearchBarState<QuickCredentialAction> — 列表状态
  └── QuickCredentialItem — 搜索结果项
```

面板事件:
```rust
enum QuickCredentialPanelEvent {
    ItemSelected { credential: QuickCredential },
    Close,
}
```

**理由**: `ExternalSecretsMenu` 已有完整的 SearchBar + SearchMixer 集成模式，直接参考可减少实现风险。支持模糊搜索、键盘导航、选中执行。

### D4: 选择凭证后弹出发送模式选择

**选择**: 选中凭证后，在面板下方或当前位置弹出两个选项的菜单。

**选项:**
1. "仅发送密码 (Enter)" — 适合 SSH、sudo 等场景
2. "先用户名再密码" — 适合 login、web 服务等场景

```

Full source: openspec/changes/quick-credential-input/design.md

## openspec/changes/quick-credential-input/tasks.md

- Source: openspec/changes/quick-credential-input/tasks.md
- Lines: 1-47
- SHA256: 77f777d27f53644ddcd5b1ec64c0c724f7861a23270ffbbf634a41d18cc2c47b

```md
## 1. 数据库迁移与数据模型

- [ ] 1.1 创建 SQLite 迁移文件：新建 `quick_credentials` 表（id, label, username, send_mode, notes, created_at, updated_at）
- [ ] 1.2 在 `crates/persistence/src/model.rs` 中添加 `QuickCredentialRow` 结构体
- [ ] 1.3 执行 `diesel print-schema` 更新 `app/src/persistence/schema.rs`

## 2. 存储层

- [ ] 2.1 在 `crates/warp_ssh_manager` 中新增 `QuickCredentialRepository`（list, get, create, update, delete）
- [ ] 2.2 实现 OS Keychain 存储适配（service: `zap.quick-credential`）
- [ ] 2.3 实现 `load_saved_quick_credentials()` 函数（加载所有通用凭证，从 SQLite + Keychain 解密）

## 3. 搜索面板

- [ ] 3.1 新建 `app/src/search/quick_credential/` 模块
- [ ] 3.2 定义 `QuickCredentialItem` struct 和 `QuickCredentialSearchItemAction` 枚举
- [ ] 3.3 实现 `QuickCredentialDataSource`（实现 `SyncDataSource`）
- [ ] 3.4 实现 `QuickCredentialPanel` 视图（SearchBar + SearchMixer）
- [ ] 3.5 实现面板事件：`ItemSelected`, `Close`

## 4. 发送引擎

- [ ] 4.1 定义 `SendMode` 枚举（`PasswordOnly`, `UsernameThenPassword`）
- [ ] 4.2 在选中凭证后显示发送模式选择 UI
- [ ] 4.3 实现 `send_credential()` 逻辑：清行 → 发送 → 延迟 → 发送
- [ ] 4.4 使用 `Zeroizing<String>` 确保密码内存安全

## 5. TerminalView 集成

- [ ] 5.1 在 `TerminalView` 中创建 `QuickCredentialPanel` 实例
- [ ] 5.2 订阅面板事件并路由到发送引擎
- [ ] 5.3 在 `TerminalView::render()` 中添加面板定位渲染
- [ ] 5.4 注册快捷键 `ToggleQuickCredentialPanel`（EditableBinding）
- [ ] 5.5 扩展 `spawn_onekey_prompt_listener`：密码提示时同时加载通用凭证

## 6. 凭证管理 UI

- [ ] 6.1 在 `app/src/settings_view/` 中新增 Quick Credentials 页面入口
- [ ] 6.2 实现凭证列表视图
- [ ] 6.3 实现添加凭证表单（label, username, password, send_mode, notes）
- [ ] 6.4 实现编辑凭证功能
- [ ] 6.5 实现删除凭证（带确认对话框）

## 7. Feature Flag 与配置

- [ ] 7.1 新增 `FeatureFlag::QuickCredentialInput`，加入 Preview flags
- [ ] 7.2 用 feature flag 包裹面板创建和快捷键注册
```

## openspec/changes/quick-credential-input/specs/credential-management/spec.md

- Source: openspec/changes/quick-credential-input/specs/credential-management/spec.md
- Lines: 1-45
- SHA256: 04343723fffa99d0770c1c67be93e7d68d0e441051da004b46f7c09420ef7567

```md
## ADDED Requirements

### Requirement: List all saved credentials
The settings page SHALL display a list of all saved quick credentials, showing label, username preview, and send mode icon for each entry.

#### Scenario: View credential list
- **WHEN** user navigates to the Quick Credentials settings page
- **THEN** all saved credentials are displayed in a list with label, username, and send mode

### Requirement: Add new credential
The settings page SHALL provide a form to add a new credential with fields: label, username, password, send mode selector, and notes.

#### Scenario: Add credential successfully
- **WHEN** user fills in all form fields and clicks Save
- **THEN** the credential is persisted (SQLite + OS Keychain) and appears in the list

#### Scenario: Add credential with missing label
- **WHEN** user attempts to save without a label
- **THEN** an error message "Label is required" is shown and the credential is not saved

#### Scenario: Add credential with missing password
- **WHEN** user attempts to save without a password
- **THEN** an error message "Password is required" is shown and the credential is not saved

### Requirement: Edit existing credential
The settings page SHALL allow editing all fields of an existing credential.

#### Scenario: Edit credential label
- **WHEN** user edits the label of an existing credential and saves
- **THEN** the credential's label is updated in SQLite

#### Scenario: Edit credential password
- **WHEN** user edits the password of an existing credential and saves
- **THEN** the new password is stored in OS Keychain

### Requirement: Delete credential
The settings page SHALL allow deleting a credential with a confirmation dialog.

#### Scenario: Delete credential
- **WHEN** user clicks Delete on a credential and confirms
- **THEN** the credential is removed from SQLite and OS Keychain

#### Scenario: Cancel delete
- **WHEN** user clicks Delete on a credential but cancels the confirmation
- **THEN** the credential is not deleted
```

## openspec/changes/quick-credential-input/specs/credential-panel/spec.md

- Source: openspec/changes/quick-credential-input/specs/credential-panel/spec.md
- Lines: 1-38
- SHA256: 7d3b7a5ca7b3389f7e63644760692096cb642b6e2f2a42d98bd2253af7a79099

```md
## ADDED Requirements

### Requirement: Show credential search panel on hotkey
The system SHALL display a search panel when the user presses the configured hotkey in a terminal. The panel SHALL contain a search bar and a scrollable list of credentials.

#### Scenario: Open panel via hotkey
- **WHEN** user presses `ctrl+shift+k` (or configured shortcut) in a terminal
- **THEN** a search panel appears at the center of the terminal, focused on the search bar

#### Scenario: Close panel via Escape
- **WHEN** panel is open and user presses Escape
- **THEN** the panel closes

#### Scenario: Close panel via clicking outside
- **WHEN** panel is open and user clicks outside it
- **THEN** the panel closes

### Requirement: Fuzzy search credentials
The system SHALL filter the credential list as the user types, using case-insensitive fuzzy matching on the label and username fields.

#### Scenario: Search by label
- **WHEN** user types "prod" in the search bar
- **THEN** credentials with labels containing "prod" (e.g., "prod-db", "production-server") appear, sorted by relevance

#### Scenario: Search by username
- **WHEN** user types "admin" in the search bar
- **THEN** credentials with username "admin" appear

#### Scenario: No matches
- **WHEN** user types text that matches no credentials
- **THEN** the list shows "No matching credentials"

### Requirement: Select credential with keyboard
The system SHALL support keyboard navigation in the credential list via Up/Down arrow keys, and selection via Enter.

#### Scenario: Navigate and select
- **WHEN** user presses Down arrow, then Enter on the selected credential
- **THEN** the credential is selected and the panel transitions to send mode selection
```

## openspec/changes/quick-credential-input/specs/credential-send/spec.md

- Source: openspec/changes/quick-credential-input/specs/credential-send/spec.md
- Lines: 1-29
- SHA256: d1e4c2bde97dd3dc2986e4a9ee0c8b98b8d3cea880d63c06efb2bea2f0d5b837

```md
## ADDED Requirements

### Requirement: Show send mode options after credential selection
After the user selects a credential from the panel, the system SHALL display two send mode options: "Send Password Only" and "Send Username + Password".

#### Scenario: Show send mode options
- **WHEN** user selects a credential from the panel
- **THEN** two buttons/options are shown: "Send Password Only" and "Send Username + Password"

### Requirement: Send password only
The system SHALL send only the password followed by newline to the shell when "Send Password Only" is selected.

#### Scenario: Send password to shell
- **WHEN** user selects credential "my-server" and chooses "Send Password Only"
- **THEN** the terminal's current line is cleared, then `password\n` is written to the PTY

### Requirement: Send username then password
The system SHALL send the username followed by newline, wait for the shell to process it, then send the password followed by newline when "Send Username + Password" is selected.

#### Scenario: Send username then password to shell
- **WHEN** user selects credential "my-app" (username: "admin") and chooses "Send Username + Password"
- **THEN** the terminal's current line is cleared, `admin\n` is written, then after ~150ms `password\n` is written

### Requirement: Use Zeroizing for sensitive data
The system SHALL wrap all passwords in `Zeroizing<String>` to ensure they are zeroed out in memory on drop.

#### Scenario: Password is zeroed after send
- **WHEN** a credential's password has been sent to the PTY and all references are dropped
- **THEN** the memory previously holding the password is zeroed
```

## openspec/changes/quick-credential-input/specs/credential-store/spec.md

- Source: openspec/changes/quick-credential-input/specs/credential-store/spec.md
- Lines: 1-35
- SHA256: 4c2a6686140810cf28d88e7350673e26756be4d0ebfd3f8a456676895ee6fbda

```md
## ADDED Requirements

### Requirement: Store credential metadata in SQLite
The system SHALL persist credential metadata (label, username, send_mode, notes) in a SQLite `quick_credentials` table. Each credential MUST have a unique UUID as its primary key.

#### Scenario: Create a new credential
- **WHEN** user saves a new credential with label "my-server", username "admin", send_mode "password_only"
- **THEN** a new row is inserted into `quick_credentials` with a generated UUID, and the created_at/updated_at timestamps are set

#### Scenario: List all credentials
- **WHEN** system loads credentials for the panel
- **THEN** all rows from `quick_credentials` are returned, ordered by label ascending

#### Scenario: Update an existing credential
- **WHEN** user edits the label of an existing credential
- **THEN** the row's label is updated and updated_at is refreshed

#### Scenario: Delete a credential
- **WHEN** user deletes a credential
- **THEN** the row is removed from `quick_credentials` and the corresponding secret is deleted from OS keychain

### Requirement: Store secrets in OS Keychain
The system SHALL store actual passwords in the OS keychain via the `keyring` crate, using service name `zap.quick-credential` and account key `<credential-uuid>:password`.

#### Scenario: Save a new secret
- **WHEN** a credential is created with password "s3cret!"
- **THEN** the keychain entry `zap.quick-credential / <uuid>:password` contains "s3cret!"

#### Scenario: Retrieve a secret
- **WHEN** the system needs to send a credential's password
- **THEN** it reads the password from the keychain entry `zap.quick-credential / <uuid>:password`

#### Scenario: Delete a secret
- **WHEN** a credential is deleted
- **THEN** the corresponding keychain entry is also deleted
```

