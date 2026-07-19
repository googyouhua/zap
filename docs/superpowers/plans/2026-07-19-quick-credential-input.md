---
change: quick-credential-input
design-doc: docs/superpowers/specs/2026-07-19-quick-credential-input-design.md
base-ref: 85d8bec7504a2506327065477c6c4d9985c5d8fa
---

# Quick Credential Input — Implementation Plan

## 1. Overview

在 Warp 现有 SSH OneKey 凭证系统的基础上，新增一套与 SSH 无关的通用用户名/密码凭证（Quick Credential）子系统。用户可通过快捷键 `cmd+shift+k` 触发搜索面板，从已保存的凭证列表中选择，并决定仅发送密码还是先用户名再密码。同时扩展 PTY 自动检测流程，在密码提示弹出菜单中同时展示通用凭证。设置页新增单独的 Quick Credentials 管理页面。

## 2. File Changes

### 2.1 新建文件

| # | 文件路径 | 说明 |
|---|----------|------|
| 1 | `crates/persistence/migrations/2026-07-19-000000_add_quick_credentials/up.sql` | 新建 `quick_credentials` 表 |
| 2 | `crates/persistence/migrations/2026-07-19-000000_add_quick_credentials/down.sql` | 回滚：`DROP TABLE quick_credentials` |
| 3 | `crates/quick_credential/src/lib.rs` | 新 crate `warp_quick_credential`: `QuickCredential` 结构体、`QuickCredentialRepository`、Keychain 存储适配 |
| 4 | `crates/quick_credential/Cargo.toml` | 新 crate 清单（依赖 `diesel`, `keyring`, `zeroize`, `uuid`, `chrono`, `warp-persistence`） |
| 5 | `app/src/terminal/quick_credential_sender.rs` | `send_quick_credential()` 函数：发送凭证到 PTY |
| 6 | `app/src/search/quick_credential/mod.rs` | 模块声明 |
| 7 | `app/src/search/quick_credential/quick_credential_data_source.rs` | `QuickCredentialDataSource`（`SyncDataSource` trait） |
| 8 | `app/src/search/quick_credential/quick_credential_search_item.rs` | `QuickCredentialSearchItem`（`SearchItem` trait） |
| 9 | `app/src/search/quick_credential/quick_credential_mixer.rs` | `QuickCredentialMixer` + `QuickCredentialItemAction` |
| 10 | `app/src/search/quick_credential/view.rs` | `QuickCredentialPanel` 视图（search bar + credential list + send mode selector） |
| 11 | `app/src/settings_view/quick_credentials_page.rs` | Quick Credentials 设置页面（list / add / edit / delete） |

### 2.2 修改文件

| # | 文件路径 | 修改内容 |
|---|----------|----------|
| 12 | `crates/warp_features/src/lib.rs` | 新增 `FeatureFlag::QuickCredentialInput`，加入 `PREVIEW_FLAGS` |
| 13 | `crates/persistence/src/model.rs` | 新增 `QuickCredentialRow` 结构体（`Identifiable`, `Queryable`, `Insertable`） |
| 14 | `crates/persistence/src/lib.rs` | 重新导出 `warp_quick_credential` 或确保 schema 包含新表 |
| 15 | `app/src/search/mod.rs` | 添加 `pub mod quick_credential;` 条件编译（`#[cfg(feature = "quick_credential_input")]`） |
| 16 | `app/src/terminal/view.rs` | 集成 `QuickCredentialPanel`: 创建实例、订阅事件、渲染定位、快捷键注册、扩展 OneKey prompt 菜单 |
| 17 | `app/src/settings_view/mod.rs` | 注册 Quick Credentials 设置页面入口 |
| 18 | `app/src/app_services/keymaps.rs`（或等效的 keybinding 注册处） | 注册 `terminal:toggle_quick_credential_panel` 快捷键绑定 |
| 19 | `app/src/lib.rs` | 注册 FeatureFlag + 条件编译快捷面板创建 |
| 20 | `app/Cargo.toml` | 添加 `quick_credential_input` feature flag（可选，初期仅通过运行时 FeatureFlag 控制） |

## 3. Implementation Steps

### Step 1: 数据库迁移与数据模型

**依赖**: 无

- **1a**: 创建迁移目录 `crates/persistence/migrations/2026-07-19-000000_add_quick_credentials/`
- **1b**: 编写 `up.sql`:
  ```sql
  CREATE TABLE quick_credentials (
      id         TEXT PRIMARY KEY NOT NULL,
      label      TEXT NOT NULL,
      username   TEXT NOT NULL DEFAULT '',
      send_mode  TEXT NOT NULL DEFAULT 'password_only'
                  CHECK (send_mode IN ('password_only', 'username_then_password')),
      notes      TEXT NOT NULL DEFAULT '',
      created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  );
  ```
- **1c**: 编写 `down.sql`:
  ```sql
  DROP TABLE quick_credentials;
  ```
- **1d**: 在 `crates/persistence/src/model.rs` 新增 `QuickCredentialRow`：
  ```rust
  #[derive(Identifiable, Queryable, Insertable)]
  #[diesel(table_name = quick_credentials)]
  pub struct QuickCredentialRow {
      pub id: String,
      pub label: String,
      pub username: String,
      pub send_mode: String,
      pub notes: String,
      pub created_at: String,
      pub updated_at: String,
  }
  ```
- **1e**: 执行 `diesel print-schema` 更新 `app/src/persistence/schema.rs`（或确保 schema 文件包含新表）

### Step 2: 新增 `crates/quick_credential` crate

**依赖**: Step 1

- **2a**: 创建 `crates/quick_credential/` 目录结构和 `Cargo.toml`
  - 依赖：`diesel` (SQLite), `keyring` (OS keychain), `zeroize`, `uuid`, `chrono`, `serde`, `warp_persistence`, `warp_ssh_manager`（复用 `SshSecretStore` trait 或新建自己的 store trait）
- **2b**: 实现数据核心类型：
  ```rust
  pub struct QuickCredential {
      pub id: String,
      pub label: String,
      pub username: String,
      pub send_mode: SendMode,
      pub notes: String,
      pub password: Zeroizing<String>,
  }

  pub enum SendMode {
      PasswordOnly,
      UsernameThenPassword,
  }
  ```
- **2c**: 实现 `QuickCredentialRepository`（CRUD 操作通过 `warp_persistence::with_conn` 执行 SQLite 元数据操作）
  - `list(conn) -> Result<Vec<QuickCredential>>`
  - `get(conn, id) -> Result<Option<QuickCredential>>`
  - `create(conn, credential) -> Result<QuickCredential>`
  - `update(conn, credential) -> Result<QuickCredential>`
  - `delete(conn, id) -> Result<()>`
- **2d**: 实现 Keychain 密码存取。复用 `keyring` crate，service: `zap.quick-credential`, account: `<uuid>:password`
  - `get_password(id) -> Result<Option<Zeroizing<String>>>`
  - `set_password(id, password) -> Result<()>`
  - `delete_password(id) -> Result<()>`
- **2e**: 实现 `load_saved_quick_credentials()` 公开函数：遍历所有凭证行，从 Keychain 解密密码，组装为 `Vec<QuickCredential>`

### Step 3: 搜索面板 — 数据源与搜索项

**依赖**: Step 2

- **3a**: 在 `app/src/search/` 下创建 `quick_credential/` 目录
- **3b**: `quick_credential_search_item.rs` — 实现 `QuickCredentialSearchItem`：
  ```rust
  pub struct QuickCredentialSearchItem {
      pub credential: QuickCredential,
      pub fuzzy_match: FuzzyMatchResult,
  }

  impl SearchItem for QuickCredentialSearchItem {
      type Action = QuickCredentialItemAction;
      // 实现 display_name, action, into_XXX_query_result 等
  }
  ```
- **3c**: `quick_credential_data_source.rs` — `QuickCredentialDataSource` 实现 `SyncDataSource`，加载所有凭证并对 label + username 做模糊匹配
- **3d**: `quick_credential_mixer.rs` — 定义 `QuickCredentialItemAction` (enum) 和 `QuickCredentialMixer`

### Step 4: 搜索面板 — 视图

**依赖**: Step 3

- **4a**: `view.rs` — 实现 `QuickCredentialPanel` 视图
  - 结构体字段：`search_bar`, `search_bar_state`, `mixer`, `scroll_state`, `list_state`, `selected_credential: Option<QuickCredential>`
  - 面板事件 `QuickCredentialPanelEvent`: `ItemSelected { credential }`, `Close`
  - TypedActionView：处理 `ResultClicked` → 设置 `selected_credential`，切换到发送模式选择 UI
  - Render：当 `selected_credential` 为 `None` 时显示搜索栏 + 列表；为 `Some` 时显示两个发送模式按钮
  - 发送模式按钮：`"仅发送密码 (Enter)"` 和 `"先用户名再密码"`
- **4b**: `mod.rs` — 模块声明，公开 `QuickCredentialPanel` 和 `QuickCredentialPanelEvent`

### Step 5: 发送引擎

**依赖**: Step 2

- **5a**: `app/src/terminal/quick_credential_sender.rs`
  ```rust
  pub fn send_quick_credential(
      terminal_view: &mut TerminalView,
      credential: &QuickCredential,
      mode: SendMode,
      ctx: &mut ViewContext<TerminalView>,
  ) {
      match mode {
          SendMode::PasswordOnly => {
              terminal_view.clear_line_editor_and_write_to_pty(
                  format!("{}\n", credential.password).into_bytes(),
                  ctx,
              );
          }
          SendMode::UsernameThenPassword => {
              terminal_view.clear_line_editor_and_write_to_pty(
                  format!("{}\n", credential.username).into_bytes(),
                  ctx,
              );
              let password = credential.password.clone();
              ctx.spawn_after(Duration::from_millis(150), move |terminal_view, ctx| {
                  terminal_view.write_to_pty(
                      format!("{}\n", password).into_bytes(),
                      ctx,
                  );
              });
          }
      }
  }
  ```
- **5b**: 使用 `Zeroizing<String>` 全程持有密码，发送后不保留额外副本

### Step 6: TerminalView 集成

**依赖**: Steps 4, 5

- **6a**: 在 `TerminalView` 中新增字段：
  ```rust
  #[cfg(feature = "quick_credential_input")]
  quick_credential_panel: Option<ViewHandle<QuickCredentialPanel>>,
  ```
- **6b**: 在 `TerminalView::new()` 中创建面板实例：
  ```rust
  #[cfg(feature = "quick_credential_input")]
  {
      if FeatureFlag::QuickCredentialInput.is_enabled() {
          let panel = ctx.add_typed_action_view(|ctx| QuickCredentialPanel::new(ctx));
          ctx.subscribe_to_view(&panel, Self::on_quick_credential_event);
          self.quick_credential_panel = Some(panel);
      }
  }
  ```
- **6c**: 实现事件处理函数 `on_quick_credential_event`：
  - 收到 `ItemSelected` → 调用 `send_quick_credential`
  - 收到 `Close` → 隐藏面板
- **6d**: 在 `render()` 中如果面板打开，渲染为覆盖层（参照 ExternalSecretsMenu 的 overlay 模式）
- **6e**: 处理 `TerminalAction::ToggleQuickCredentialPanel` action → 切换面板显示状态
- **6f**: 扩展 `show_onekey_prompt_menu`：在加载 SSH 凭证时同时调用 `load_saved_quick_credentials()`，合并展示候选列表，用不同图标区分类型

### Step 7: Feature Flag 与快捷键

**依赖**: Steps 1-6

- **7a**: 在 `crates/warp_features/src/lib.rs` 新增 enum variant `QuickCredentialInput`，加入 `PREVIEW_FLAGS`（或 `DOGFOOD_FLAGS` 取决开发阶段）
- **7b**: 注册快捷键绑定：
  ```rust
  EditableBinding::new(
      "terminal:toggle_quick_credential_panel",
      "Show quick credential input panel",
      TerminalAction::ToggleQuickCredentialPanel,
  )
  .with_context_predicate(id!("Terminal"))
  .default_trigger("cmd+shift+k")
  ```
- **7c**: 在 `app/src/lib.rs` 的条件编译位置（参考 `onekey_prompt` 的处理方式）注册 feature flag

### Step 8: 设置管理页面

**依赖**: Step 2

- **8a**: `app/src/settings_view/quick_credentials_page.rs`
  - 列表视图：遍历 `QuickCredentialRepository::list()`，每行显示 label、username 预览、send_mode 图标
  - "添加"按钮 → 表单视图（label, username, password 输入, send_mode 下拉, notes 文本区）
  - 编辑 → 同表单预填充
  - 删除 → 确认对话框 → 调用 `delete()`
  - 密码输入使用 masked `EditorView`（参考现有 SSH Manager 密码字段）
- **8b**: 在 `app/src/settings_view/mod.rs` 中新增入口路由，将其加入设置页面列表

### Step 9: 测试

- **9a**: `crates/quick_credential` 单测 — Repository CRUD 的 SQLite in-memory 测试
- **9b**: `quick_credential_sender_tests.rs` — 用 mock TerminalView 测试发送逻辑
- **9c**: `quick_credential_data_source_tests.rs` — 模糊搜索过滤测试
- **9d**: 集成测试：添加凭证 → 打开面板 → 搜索 → 选择 → 发送

### 依赖关系图

```
Step 1 (migration) ──→ Step 2 (crate) ──→ Step 3 (data source)
                                      │         │
                                      │         ▼
                                      │──→ Step 4 (panel view)
                                      │         │
                                      │         ▼
                                      │──→ Step 5 (sender) ──→ Step 6 (TerminalView)
                                      │                              │
                                      │                              ▼
                                      │                         Step 7 (feature flag)
                                      │
                                      └──→ Step 8 (settings UI)

Step 9 (tests) — parallel after Steps 2-8
```

## 4. 关键设计决策（实现时注意）

1. **新 crate vs 放入现有 crate**：设计文档提出放在 `crates/warp_ssh_manager` 或新建 `crates/quick_credential`。推荐新建 crate 以避免与 SSH Manager 耦合，保持关注点分离。
2. **Keychain 复用**：复用 `crates/warp_ssh_manager/src/secrets.rs` 的 `KeychainSecretStore` 模式，但新建 `QuickCredentialSecretStore` 使用独立的 service name (`zap.quick-credential`)。
3. **Feature flag 层级**：运行时 `FeatureFlag::QuickCredentialInput` + Cargo feature `quick_credential_input` 双保险。Cargo feature 控制代码是否编译，运行时 flag 控制是否启用。
4. **PTY 自动检测融合**：在 `show_onekey_prompt_menu` 中合并通用凭证，不修改 SSH 凭证的加载流程。通用凭证在前端以不同图标展示。
5. **密码安全问题**：全程使用 `Zeroizing<String>`，发送后及时清除。密码在面板聚焦期间持有，面板关闭后置空 `selected_credential` 使密码被 drop。
