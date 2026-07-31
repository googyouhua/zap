---
change: onekey-credential-system
design-doc: docs/superpowers/specs/2026-07-31-onekey-credential-system-design.md
base-ref: 2951bfb744561eb19e328d3d8df47895e613a3b2
---

# 统一 OneKey 凭据系统(onekey-credential-system)实现计划

> **给 agentic worker:** 实现本计划必须加载 superpowers:subagent-driven-development(推荐)或 superpowers:executing-plans,按任务逐条执行。步骤用 `- [ ]` 勾选跟踪。
>
> **参考实现:** 本 change 对齐参考分支 `feature/20260719/quick-credential-input`(下称"参考分支")的最终状态。计划中每个任务都标注了对应的参考文件路径,`REF:` 前缀即指 `.worktrees/feature/20260719/quick-credential-input/` 下的文件。差异点(干净的迁移、auto-send 接入生产、su_root 保留过滤)已在 design doc Decision 中说明。

**Goal:** 在 main 上重新实现统一的 OneKey 凭据系统 —— 新建 `crates/onekey` 存储层与终端 OneKey 搜索面板,支持向普通终端/SSH/SFTP 发送凭据与 PTY 密码提示 auto-send,并将 SSH 面板与设置页的凭据数据源切到统一层,最后删除旧 SSH OneKey 系统。

**Architecture:** 存储下沉到独立 crate `warp_onekey`(SQLite 元数据 + OS Keychain secret + 版本号计数器);UI 层复用 SearchBar/SearchMixer 模式实现 `OneKeyPanel`(热键 `cmd_or_ctrl_shift("u")`);发送引擎 `onekey_sender` 与提示分类 `prompt_detection` 独立成模块;SSH 面板/设置页只切数据源不改 UI;跨视图刷新靠 `OneKeyCredentialsChangedNotifier`(SingletonEntity)+ `CREDENTIALS_VERSION`。

**Tech Stack:** Rust(Cargo workspace,`resolver = "2"`)、diesel 2.3 + SQLite、keyring 3.6(apple/windows/linux native)、zeroize 1.8、uuid 1.1、WarpUI(warpui_core/warpui)、fuzzy_match、FeatureFlag(`crates/warp_features`)。

---

## 0. 开工准备与约定

- 本 change 在 `main` 上直接开发(当前 HEAD = `2951bfb7`)。建议在 `.worktrees/onekey-credential-system/` worktree 中开发,完成后再合回。
- 验证命令(§5.1 AGENTS.md):提交/推 PR 前只需 `cargo check` 通过;全量测试用 `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`。
- 注释、commit message 一律简体中文。
- 单元测试放 `${文件名}_tests.rs`,原文件末尾挂 `#[cfg(test)] #[path = "..._tests.rs"] mod tests;`。
- 新增 FeatureFlag 前先读 `crates/warp_features/src/lib.rs` 现有 `OneKeyPrompt` 的写法与 `app/src/lib.rs:2332` 的 `#[cfg(feature = ...)]` 用法,保持两处对齐。

### 任务顺序与依赖

9 组任务有先后依赖:1(存储层)→ 2(接线)→ 3/4(面板+发送)→ 5(auto-send)→ 6/7(SSH 面板/设置页)→ 8(删除旧系统)→ 9(验证)。**8 必须在 3~7 全部就绪后才能做**,否则编译失败。1~7 之间可并行推进(写入域不重叠的部分,遵循 AGENTS.md §5.8)。

### 关键决策速记(与参考分支的差异)

1. **快捷键**用 `cmd_or_ctrl_shift("u")`(`app/src/terminal/view/init.rs`),避开 main 上 ClearBuffer 的 `ctrl+shift+k`。
2. **干净迁移**:新建一个 Diesel 迁移同时建 `onekey_credentials` + `prompt_trigger_rules` 并 drop `ssh_onekey_credentials`;**不复制**参考分支的 `2026-07-19-000000_add_quick_credentials` 死迁移。
3. **auto-send 完整接入生产流程**:参考分支的 `classify_prompt` 仅是死代码(只被测试引用),本 change 在 `spawn_onekey_prompt_listener` 中真正调用。
4. **su_root 保留 Password-only 过滤语义**(design Doc Decision 5),数据源为 `warp_onekey::find_all()` 过滤 `kind == Password`,不并入全部凭据。
5. **面板发送模式即时选择**,不持久化到凭据上。

---

## 1. 文件结构总览

### 新建文件

| 文件 | 职责 |
|------|------|
| `crates/onekey/Cargo.toml` | 新 crate(package `warp_onekey`),依赖见下 |
| `crates/onekey/src/lib.rs` | crate 根,re-export 全部 API + `#[cfg(test)] set_test_conn` |
| `crates/onekey/src/types.rs` | `OneKeyKind` / `OneKeyCredential` / `SendMode` / `PromptTriggerRule` + 默认关键词常量 |
| `crates/onekey/src/db.rs` | `set_database_path` / `with_conn` / `set_test_conn` + 运行时 `CREATE TABLE IF NOT EXISTS` + `ensure_columns` |
| `crates/onekey/src/secret_store.rs` | `OneKeySecretStore`(keyring,service `zap.onekey`,account `<uuid>:password`) |
| `crates/onekey/src/repository.rs` | `find_all` / `find_by_id` / `create` / `update` / `delete` / `list_rules` / `add_rule` / `remove_rule` / `reset_rules_for_mode` / `reset_rules_to_defaults` + `CREDENTIALS_VERSION` |
| `crates/onekey/src/types_tests.rs`、`repository_tests.rs` | 单元测试 |
| `crates/persistence/migrations/<新日期>_onekey_credentials_and_drop_ssh_onekey/up.sql`、`down.sql` | 干净迁移 |
| `app/src/search/onekey/mod.rs` | 面板模块入口 |
| `app/src/search/onekey/searcher.rs` | `OneKeySearchMixer` + `OneKeySearchItemAction` |
| `app/src/search/onekey/search_item.rs` | `OneKeySearchItem`(SearchItem impl) |
| `app/src/search/onekey/data_source.rs` + `data_source_tests.rs` | `OneKeyDataSource`(模糊匹配 label/username) |
| `app/src/search/onekey/view.rs` | `OneKeyPanel`(SearchBar + 列表 + 发送模式选择) |
| `app/src/terminal/onekey_sender.rs` + `onekey_sender_tests.rs` | 发送引擎 `send_onekey_credential` |
| `app/src/terminal/prompt_detection.rs` | `classify_prompt`(接线生产) |
| `app/src/ssh_manager/onekey_notifier.rs` | `OneKeyCredentialsChangedNotifier`(SingletonEntity) |
| `app/src/settings_view/onekey_page.rs` | 设置页凭据 CRUD + Trigger Keywords |

### 修改文件

| 文件 | 变更 |
|------|------|
| `Cargo.toml` | 无需改(workspace members 是 `crates/*`,新 crate 自动纳入) |
| `crates/warp_features/src/lib.rs` | 增 `FeatureFlag::OneKeyInput` + 加入 `DOGFOOD_FLAGS` |
| `crates/persistence/src/model.rs`、`schema.rs` | 增 `OneKeyCredentialRow` / `PromptTriggerRuleRow` 与两张表;删 `ssh_onekey_credentials` 表/joinable |
| `app/Cargo.toml` | 增 `onekey_input` feature(默认启用)+ `warp_onekey.workspace = true` 依赖 |
| `app/src/lib.rs` | `warp_onekey::set_database_path(...)` + 注册 notifier singleton |
| `app/src/search/mod.rs` | `pub mod onekey;` |
| `app/src/terminal/view/action.rs` | 增 `ToggleOneKeyPanel`(cfg onekey_input) |
| `app/src/terminal/view/init.rs` | 注册 `cmd_or_ctrl_shift("u")` → `ToggleOneKeyPanel` |
| `app/src/terminal/view.rs` | 面板实例/事件/渲染、`spawn_onekey_prompt_listener` 扩展 auto-send、`show_onekey_prompt_menu` 数据源切换、su_root 数据源切换、`clear_line_editor_and_write_to_pty` 改 `pub(crate)` |
| `app/src/ssh_manager/mod.rs` | `pub mod onekey_notifier;` + re-export |
| `app/src/ssh_manager/server_view.rs` | OneKey overlay CRUD 切 `warp_onekey::*` + notifier 订阅 + 打开时 reload + 刷新按钮 |
| `app/src/workspace/view.rs` | OneKey 认证走 `warp_onekey::find_by_id()` |
| `app/src/sftp_manager/sftp_ops.rs` | `resolve_sftp_auth` OneKey 分支走 `warp_onekey::find_by_id()` |
| `app/src/settings_view/mod.rs` | `SettingsSection::OneKey` + 页面 handle 注册 + nav 项 |
| `app/i18n/{en,zh-CN,ja}/warp.ftl` | 2 个新 key |
| `crates/warp_ssh_manager/src/{types.rs,repository.rs,secrets.rs,sync_provider.rs,lib.rs}` | 删旧 OneKey 系统 |

### 删除文件

| 文件 |
|------|
| `app/src/ssh_manager/onekey.rs` |
| `crates/persistence` 中 `ssh_onekey_credentials` 相关 model/schema |

---

## 2. 任务拆解

### Task 1:存储层 crates/onekey(对应 tasks.md 分组 1)

**Files:**
- Create: `crates/onekey/Cargo.toml`
- Create: `crates/onekey/src/{lib.rs,db.rs,types.rs,secret_store.rs,repository.rs,types_tests.rs,repository_tests.rs}`
- Modify: `crates/persistence/src/{model.rs,schema.rs}`(为 repository 提供 Row/table)
- Create: `crates/persistence/migrations/<TS>_onekey_credentials_and_drop_ssh_onekey/{up.sql,down.sql}`
- Test: `crates/onekey/src/{types_tests.rs,repository_tests.rs}`

REF: `.worktrees/feature/20260719/quick-credential-input/crates/onekey/` 全部文件。

- [ ] **Step 1.1:新建 crate 骨架并加入 workspace**

创建 `crates/onekey/Cargo.toml`(与参考分支一致;`zeroize` 不走 workspace 而直接用 `"1.8"`,与 `crates/warp_ssh_manager` 一致):

```toml
[package]
name = "warp_onekey"
version = "0.1.0"
edition = "2024"
authors.workspace = true
publish.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
chrono.workspace = true
diesel = { workspace = true, features = ["sqlite"] }
keyring.workspace = true
log.workspace = true
persistence.workspace = true
uuid.workspace = true
warp_ssh_manager.workspace = true
zeroize = "1.8"

[dev-dependencies]
tempfile.workspace = true
```

workspace 已用 `members = ["crates/*", "app"]`(见根 `Cargo.toml`),新目录自动成为成员,无需改根 `Cargo.toml`。

- [ ] **Step 1.2:实现 db.rs**

创建 `crates/onekey/src/db.rs`,内容与参考分支 `crates/onekey/src/db.rs` 逐字一致:
- `set_database_path(path: PathBuf)`:`DB_PATH: OnceLock<PathBuf>` 一次性写入
- `open()`:`SqliteConnection::establish` + `PRAGMA foreign_keys/busy_timeout/journal_mode=WAL` + `CREATE TABLE IF NOT EXISTS onekey_credentials`(含 `kind`/`key_path`/`encrypted_password` 列)+ `ensure_columns`
- `ensure_columns(conn)`:用 `SELECT name FROM pragma_table_info('onekey_credentials')` 检查,缺失则 `ALTER TABLE ... ADD COLUMN`(`encrypted_password`/`kind`/`key_path`),最后 `CREATE TABLE IF NOT EXISTS prompt_trigger_rules`
- `with_conn<R>(f)`:测试走 `thread_local TEST_CONN`(trait 测试注入),生产走 `CONN: OnceLock<Mutex<SqliteConnection>>`

实现 `lib.rs`(先占位,Step 1.6 完成后补全 re-export):

```rust
mod db;
pub mod repository;
mod secret_store;
mod types;

pub use db::{set_database_path, with_conn};
#[cfg(test)]
pub use db::set_test_conn;
```

- [ ] **Step 1.3:实现 types.rs + types_tests.rs**

创建 `crates/onekey/src/types.rs`,与参考分支逐字一致:`OneKeyKind::{Password, Key}`(+`as_db_str`/`parse`)、`OneKeyCredential`(`password: Zeroizing<String>`,`key_path: Option<String>`)、`SendMode::{PasswordOnly, UsernameThenPassword}`(+`as_str`)、`PromptTriggerRule`、常量 `DEFAULT_PASSWORD_ONLY_KEYWORDS`/`DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS`。文件末尾挂 `#[cfg(test)] #[path = "types_tests.rs"] mod tests;`。

创建 `crates/onekey/src/types_tests.rs`,与参考分支 `crates/onekey/src/types_tests.rs` 逐字一致(覆盖 `as_db_str`/`parse`/大小写敏感/字段默认值/key_path)。

- [ ] **Step 1.4:实现 secret_store.rs**

创建 `crates/onekey/src/secret_store.rs`,与参考分支逐字一致:
- `const SERVICE: &str = "zap.onekey";`
- `set(id, secret)` / `get(id) -> Result<Option<Zeroizing<String>>>` / `delete(id)`,account 为 `format!("{id}:password")`;`get` 对 `keyring::Error::NoEntry` 返回 `Ok(None)`。

- [ ] **Step 1.5:持久层 model/schema(为 repository 服务)**

参考分支改动位置:
- `crates/persistence/src/model.rs` 在 `Sync Meta` 之前插入 `OneKeyCredentialRow`(8 字段:`id,label,username,notes,encrypted_password,created_at,updated_at,kind,key_path`;`#[diesel(table_name = onekey_credentials)]`)与 `PromptTriggerRuleRow`(`id,keyword,send_mode`)—— 逐字拷贝参考分支 model.rs:1516-1538。
- `crates/persistence/src/schema.rs` 在 `ssh_servers` 前插入 `onekey_credentials` 与 `prompt_trigger_rules` 两张 `diesel::table!`(逐字拷贝参考分支 schema.rs:370-390)。

注意:本步骤不删除 `ssh_onekey_credentials`(那属于 Task 8)。此时两套表共存,Diesel 编译才不被破坏。

- [ ] **Step 1.6:新建干净迁移(drop 旧表 + 建新表)**

创建 `crates/persistence/migrations/2026-07-31-000000_onekey_credentials_and_drop_ssh_onekey/up.sql`(design doc Decision 2 的 SQL,一次完成"建新表 + 重建 ssh_servers 去外键 + drop 旧表";**不复制**参考分支的 `2026-07-19-000000_add_quick_credentials` 死迁移):

```sql
CREATE TABLE onekey_credentials (
  id                 TEXT PRIMARY KEY NOT NULL,
  label              TEXT NOT NULL,
  username           TEXT NOT NULL DEFAULT '',
  notes              TEXT NOT NULL DEFAULT '',
  encrypted_password TEXT NOT NULL DEFAULT '',
  kind               TEXT NOT NULL DEFAULT 'password',
  key_path           TEXT,
  created_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE prompt_trigger_rules (
  id        TEXT PRIMARY KEY NOT NULL,
  keyword   TEXT NOT NULL UNIQUE,
  send_mode TEXT NOT NULL DEFAULT 'password_only'
            CHECK (send_mode IN ('password_only', 'username_then_password'))
);

-- 重建 ssh_servers 去掉对 ssh_onekey_credentials 的外键约束
CREATE TABLE ssh_servers_new (
  node_id           TEXT PRIMARY KEY NOT NULL REFERENCES ssh_nodes(id) ON DELETE CASCADE,
  host              TEXT NOT NULL,
  port              INTEGER NOT NULL DEFAULT 22,
  username          TEXT NOT NULL DEFAULT '',
  auth_type         TEXT NOT NULL CHECK(auth_type IN ('password','key','onekey')) DEFAULT 'password',
  key_path          TEXT,
  startup_command   TEXT DEFAULT NULL,
  notes             TEXT DEFAULT NULL,
  last_connected_at TIMESTAMP,
  credential_id     TEXT
);
INSERT INTO ssh_servers_new SELECT * FROM ssh_servers;
DROP TABLE ssh_servers;
ALTER TABLE ssh_servers_new RENAME TO ssh_servers;
DROP TABLE IF EXISTS ssh_onekey_credentials;
```

创建 `down.sql`,还原旧表(`ssh_onekey_credentials` 完整 schema = 参考分支 `2026-07-30-000000_drop_ssh_onekey_credentials/down.sql` 前两段 + 重建带外键的 `ssh_servers`):

```sql
CREATE TABLE ssh_onekey_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    username TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE ssh_onekey_credentials
ADD COLUMN kind TEXT NOT NULL DEFAULT 'password' CHECK(kind IN ('password','key'));

ALTER TABLE ssh_onekey_credentials
ADD COLUMN key_path TEXT DEFAULT NULL;

-- 重建带 FK 的 ssh_servers
CREATE TABLE ssh_servers_old (
  node_id           TEXT PRIMARY KEY NOT NULL REFERENCES ssh_nodes(id) ON DELETE CASCADE,
  host              TEXT NOT NULL,
  port              INTEGER NOT NULL DEFAULT 22,
  username          TEXT NOT NULL DEFAULT '',
  auth_type         TEXT NOT NULL CHECK(auth_type IN ('password','key','onekey')) DEFAULT 'password',
  key_path          TEXT,
  startup_command   TEXT DEFAULT NULL,
  notes             TEXT DEFAULT NULL,
  last_connected_at TIMESTAMP,
  credential_id     TEXT
);
INSERT INTO ssh_servers_old SELECT * FROM ssh_servers;
DROP TABLE ssh_servers;

CREATE TABLE ssh_servers_new (
  node_id           TEXT PRIMARY KEY NOT NULL REFERENCES ssh_nodes(id) ON DELETE CASCADE,
  host              TEXT NOT NULL,
  port              INTEGER NOT NULL DEFAULT 22,
  username          TEXT NOT NULL DEFAULT '',
  auth_type         TEXT NOT NULL CHECK(auth_type IN ('password','key','onekey')) DEFAULT 'password',
  key_path          TEXT,
  startup_command   TEXT DEFAULT NULL,
  notes             TEXT DEFAULT NULL,
  last_connected_at TIMESTAMP,
  credential_id     TEXT REFERENCES ssh_onekey_credentials(id) ON DELETE SET NULL
);
INSERT INTO ssh_servers_new SELECT * FROM ssh_servers_old;
DROP TABLE ssh_servers_old;
ALTER TABLE ssh_servers_new RENAME TO ssh_servers;

DROP TABLE onekey_credentials;
DROP TABLE prompt_trigger_rules;
```

> 提示:迁移命名时间戳 `2026-07-31-000000...` 可换成更大的合法 Diesel 时间戳,只要大于现有全部迁移即可。迁移目录与既有 `2026-06-09-160000_add_ssh_onekey_key_type` 平级。

- [ ] **Step 1.7:实现 repository.rs + repository_tests.rs**

创建 `crates/onekey/src/repository.rs`,与参考分支 `crates/onekey/src/repository.rs` 逐字一致。要点:
- `static CREDENTIALS_VERSION: AtomicU64` + `credentials_version()` / `bump_credentials_version()`
- `row_to_onekey_credential` / `resolve_password`(keyring 优先,fallback `encrypted_password` 列)/ `store_password`(keyring + 写 `encrypted_password` 列)
- `find_all`(label 升序)/ `find_by_id` / `create`(uuid 生成 id,成功后 `bump_credentials_version()`)/ `update`(不存在返回 Err)/ `delete`(删 keyring + bump)
- 规则 CRUD:`list_rules` / `add_rule` / `remove_rule` / `reset_rules_for_mode` / `reset_rules_to_defaults`
- 文件末尾挂 `#[cfg(test)] #[path = "repository_tests.rs"] mod tests;`

创建 `crates/onekey/src/repository_tests.rs`,与参考分支逐字一致(覆盖空列表、create/find_by_id、update、delete、按 label 排序、Key kind + key_path 往返、混合 kind 列表;`setup_db` 用 `NamedTempFile` + `set_test_conn`)。

- [ ] **Step 1.8:补全 lib.rs re-export 并本地验证**

把 `lib.rs` 补成参考分支最终形态(Step 1.2 的占位基础上追加):

```rust
pub use repository::{
    add_rule, credentials_version, create, delete, find_all, find_by_id, list_rules,
    remove_rule, reset_rules_for_mode, reset_rules_to_defaults, update,
};
pub use secret_store::OneKeySecretStore;
pub use types::{
    OneKeyCredential, OneKeyKind, PromptTriggerRule, SendMode,
    DEFAULT_PASSWORD_ONLY_KEYWORDS, DEFAULT_USERNAME_AND_PASSWORD_KEYWORDS,
};
```

- [ ] **Step 1.9:跑单测**

Run: `cargo test -p warp_onekey`
Expected: 全部 PASS(含 `types_tests.rs` / `repository_tests.rs`)。

- [ ] **Step 1.10:Commit**

```bash
git add crates/onekey crates/persistence/src/model.rs crates/persistence/src/schema.rs crates/persistence/migrations/2026-07-31-000000_onekey_credentials_and_drop_ssh_onekey
git commit -m "feat: 新建 warp_onekey 统一凭据存储层与干净迁移"
```

> 注意:此时 `schema.rs` 仍含 `ssh_onekey_credentials`,`onekey` 表也会被 diesel print-schema 标记——本步骤提交的是手写 schema 增量,Task 8 删表后再统一重跑 print-schema 即可。

---

### Task 2:Feature flag 与接线(对应 tasks.md 分组 2)

**Files:**
- Modify: `crates/warp_features/src/lib.rs`
- Modify: `app/Cargo.toml`
- Modify: `app/src/lib.rs`
- Modify: `app/src/ssh_manager/mod.rs`(提前把 notifier 暴露出来供 Task 6/7 用)

REF: `.worktrees/feature/20260719/quick-credential-input/crates/warp_features/src/lib.rs`(`OneKeyInput` 变体 + `DOGFOOD_FLAGS`)、`app/Cargo.toml`(feature 声明)、`app/src/lib.rs:1082-1083,1512`。

- [ ] **Step 2.1:新增 FeatureFlag::OneKeyInput**

在 `crates/warp_features/src/lib.rs` 的 `OneKeyPrompt` 变体(约 673 行)后追加(注释用简体中文,沿用 `OneKeyPrompt` 的中文注释风格):

```rust
/// 通用(非 SSH)凭据快速填充面板。允许用户在终端中输入密码/用户名
/// 提示时搜索已保存的凭据并注入到 PTY。
OneKeyInput,
```

并在 `DOGFOOD_FLAGS` 常量末尾追加 `FeatureFlag::OneKeyInput,`(参考分支 755 行)。

- [ ] **Step 2.2:app/Cargo.toml 新增 onekey_input feature**

- 在 `[dependencies]` 区新增 `warp_onekey.workspace = true`(参考分支 207 行)。
- 在 `default = [...]` 列表(现有约 576 行 `"onekey_prompt",` 之后)追加 `"onekey_input",`。
- 在 feature 声明区(约 603 行 `onekey_prompt = []` 之后)追加:

```toml
onekey_input = []
```

确认与 `onekey_prompt` 并存(两个 feature 相互独立、互不依赖)。

- [ ] **Step 2.3:app/src/lib.rs 初始化数据库路径**

在 `app/src/lib.rs:1082`(`warp_ssh_manager::set_database_path(persistence::database_file_path());`)之后追加一行:

```rust
warp_onekey::set_database_path(persistence::database_file_path());
```

- [ ] **Step 2.4:注册 notifier singleton(先建文件)**

参考分支 `app/src/ssh_manager/onekey_notifier.rs` 逐字创建 `app/src/ssh_manager/onekey_notifier.rs`:

```rust
use warpui::{Entity, SingletonEntity};

#[derive(Default)]
pub struct OneKeyCredentialsChangedNotifier {}

impl OneKeyCredentialsChangedNotifier {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug)]
pub enum OneKeyCredentialsChangedEvent {
    CredentialsChanged,
}

impl Entity for OneKeyCredentialsChangedNotifier {
    type Event = OneKeyCredentialsChangedEvent;
}

impl SingletonEntity for OneKeyCredentialsChangedNotifier {}
```

在 `app/src/ssh_manager/mod.rs` 中追加模块声明与 re-export(参考分支 mod.rs:8,22):

```rust
pub mod onekey_notifier;
// ...
pub use onekey_notifier::{OneKeyCredentialsChangedEvent, OneKeyCredentialsChangedNotifier};
```

在 `app/src/lib.rs` 的 `ctx.add_singleton_model(|_| crate::ssh_manager::SshTreeChangedNotifier::new());`(约 1509 行)之后追加:

```rust
ctx.add_singleton_model(|_| crate::ssh_manager::OneKeyCredentialsChangedNotifier::new());
```

- [ ] **Step 2.5:确认 OneKeyPrompt 门控不回归**

核对(只读,不改):`app/src/terminal/view.rs:3769` 的 `FeatureFlag::OneKeyPrompt.is_enabled()` 仍包住 `spawn_onekey_prompt_listener` 调用;`app/src/lib.rs:2332` 的 `#[cfg(feature = "onekey_prompt")]` 仍在。本 change 只在 Task 5 扩展其内部逻辑,不改门控本身。

- [ ] **Step 2.6:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过。
Commit:

```bash
git add crates/warp_features/src/lib.rs app/Cargo.toml app/src/lib.rs app/src/ssh_manager/mod.rs app/src/ssh_manager/onekey_notifier.rs
git commit -m "feat: 注册 OneKeyInput feature flag、数据库路径与变更通知器"
```

---

### Task 3:终端搜索面板(对应 tasks.md 分组 3)

**Files:**
- Create: `app/src/search/onekey/{mod.rs,searcher.rs,search_item.rs,data_source.rs,data_source_tests.rs,view.rs}`
- Modify: `app/src/search/mod.rs`

REF: `.worktrees/feature/20260719/quick-credential-input/app/src/search/onekey/` 全部文件。

- [ ] **Step 3.1:面板模块骨架**

- `app/src/search/mod.rs` 在 `pub mod external_secrets;`(10 行)附近追加 `pub mod onekey;`。
- 创建 `app/src/search/onekey/mod.rs`,逐字拷贝参考分支:

```rust
mod data_source;
mod search_item;
mod searcher;
mod view;

pub use view::OneKeyPanel;
pub use view::OneKeyPanelEvent;
```

- 创建 `app/src/search/onekey/searcher.rs`(逐字拷贝):

```rust
use crate::search::mixer::SearchMixer;

pub type OneKeySearchMixer = SearchMixer<OneKeySearchItemAction>;

#[derive(Clone, Debug)]
pub enum OneKeySearchItemAction {
    SelectCredential(warp_onekey::OneKeyCredential),
}
```

- [ ] **Step 3.2:数据源 + 搜索项**

- 创建 `app/src/search/onekey/search_item.rs`,逐字拷贝参考分支:`OneKeySearchItem` 实现 `SearchItem`(icon 用 `Icon::Key`,label + username 两行,`accessibility_label` 为 `Credential: <label> (<username>)`)。
- 创建 `app/src/search/onekey/data_source.rs`,逐字拷贝参考分支:`OneKeyDataSource::new()` 调 `warp_onekey::find_all()`,`run_query` 用 `fuzzy_match::match_indices_case_insensitive` 过滤 label/username,空 query 返回全部。
- 创建 `app/src/search/onekey/data_source_tests.rs`,逐字拷贝参考分支(6 个过滤单测,走 `filter_credentials` 纯函数)。

- [ ] **Step 3.3:面板主视图(view.rs)**

创建 `app/src/search/onekey/view.rs`,逐字拷贝参考分支(529 行)。它包含:
- `PanelMode::{Searching, SendModeSelection { credential }}`
- `OneKeyPanelAction::{ResultClicked, SendPasswordOnly, SendUsernameThenPassword, Close}`
- `OneKeyPanelEvent::{ItemSelected { credential, mode }, Close, Open}`
- `OneKeyPanel::new`(SearchBarState + `OneKeySearchMixer` + `QueryResultRenderer` 样式)+ `setup`(重置 mixer、加载 `OneKeyDataSource`)+ `close`
- 发送模式选择 UI(`render_send_mode_selection`,两行可点击行)
- `render` 按 `PanelMode` 分支;`handle_action` 把两类发送动作转为 `ItemSelected { mode }`

> 样式来自 `crate::search::external_secrets::view::styles`,main 已有该模块(`app/src/search/external_secrets/view.rs:368` `pub mod styles`),无需改动。

- [ ] **Step 3.4:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过(面板尚未被任何视图引用,可能触发 `dead_code` 警告——若出现,属预期,Task 4 接线后消除)。

Commit:

```bash
git add app/src/search/mod.rs app/src/search/onekey
git commit -m "feat: 实现终端 OneKey 搜索面板(SearchBar + SearchMixer)"
```

> 快捷键注册不在本任务(归 Task 4 的 `init.rs` 步骤),因为面板要在 TerminalView 中接线后才能被热键驱动。

---

### Task 4:发送引擎与面板接线(对应 tasks.md 分组 4)

**Files:**
- Create: `app/src/terminal/onekey_sender.rs` + `onekey_sender_tests.rs`
- Modify: `app/src/terminal/view/action.rs`
- Modify: `app/src/terminal/view/init.rs`
- Modify: `app/src/terminal/view.rs`(字段、创建面板、订阅事件、action 处理、render 定位)
- Modify: `app/i18n/{en,zh-CN,ja}/warp.ftl`

REF: `.worktrees/feature/20260719/quick-credential-input/app/src/terminal/onekey_sender.rs`、`onekey_sender_tests.rs`、`view/action.rs`(ToggleOneKeyPanel)、`view/init.rs:579-586`、`view.rs`(字段 2327-2330、创建 3756-3762、事件 15483-15505、action 23547-23556、render 24659-24664)。

- [ ] **Step 4.1:发送引擎**

创建 `app/src/terminal/onekey_sender.rs`,逐字拷贝参考分支:

```rust
use crate::terminal::view::TerminalView;
use std::time::Duration;
use warp_onekey::{OneKeyCredential, SendMode};
use warpui::ViewContext;
use warpui::r#async::Timer;

pub fn send_onekey_credential(
    terminal_view: &mut TerminalView,
    credential: &OneKeyCredential,
    mode: SendMode,
    ctx: &mut ViewContext<TerminalView>,
) {
    match mode {
        SendMode::PasswordOnly => {
            terminal_view.write_to_pty(
                format!("{}\n", *credential.password).into_bytes(),
                ctx,
            );
        }
        SendMode::UsernameThenPassword => {
            terminal_view.clear_line_editor_and_write_to_pty(
                format!("{}\n", credential.username).into_bytes(),
                ctx,
            );
            let password = credential.password.clone();
            ctx.spawn(
                Timer::after(Duration::from_millis(150)),
                move |me, _, ctx| {
                    me.write_to_pty(format!("{}\n", *password).into_bytes(), ctx);
                },
            );
        }
    }
}

#[cfg(test)]
#[path = "onekey_sender_tests.rs"]
mod tests;
```

> 实现说明(design doc Decision 4):PasswordOnly 直接写 `secret\n`(不清行);UsernameThenPassword 用 `clear_line_editor_and_write_to_pty` 写 `username\n`,~150ms 后写 `password\n`。以参考分支最终态为准。

创建 `app/src/terminal/onekey_sender_tests.rs`,逐字拷贝参考分支(4 个 SendMode 冒烟测试)。

- [ ] **Step 4.2:把 clear_line_editor_and_write_to_pty 改为 pub(crate)**

`app/src/terminal/view.rs:7472` 的 `fn clear_line_editor_and_write_to_pty` 改为 `pub(crate) fn clear_line_editor_and_write_to_pty`(参考分支 7435 行即为 `pub(crate)`)。`write_to_pty` 已是 `pub(crate)`(7376 行),无需改。

- [ ] **Step 4.3:新增 TerminalAction::ToggleOneKeyPanel**

在 `app/src/terminal/view/action.rs` 的 `SuRootFillOneKeyPassword { index }`(166-168 行)后追加(参考分支 169-170 行):

```rust
    #[cfg(feature = "onekey_input")]
    ToggleOneKeyPanel,
```

同步在 `impl fmt::Display` 中追加(参考分支 648 行,`SuRootFillOneKeyPassword` 分支附近):

```rust
            #[cfg(feature = "onekey_input")]
            ToggleOneKeyPanel => write!(f, "ToggleOneKeyPanel"),
```

- [ ] **Step 4.4:注册快捷键 cmd_or_ctrl_shift("u")**

在 `app/src/terminal/view/init.rs` 的可编辑绑定数组末尾(参考分支 579-586 行),`#[cfg(feature = "onekey_input")]` 追加:

```rust
        #[cfg(feature = "onekey_input")]
        EditableBinding::new(
            "terminal:toggle_onekey_panel",
            crate::t!("keybinding-desc-terminal-toggle-onekey-panel"),
            TerminalAction::ToggleOneKeyPanel,
        )
        .with_key_binding(cmd_or_ctrl_shift("u"))
        .with_context_predicate(id!("Terminal") & !id!("IMEOpen")),
```

`cmd_or_ctrl_shift` 已由 `app/src/terminal/view/init.rs:21` 的 `use crate::util::bindings::{cmd_or_ctrl_shift, is_binding_pty_compliant};` 导入,无需新增。

**核对固定绑定冲突**(design doc 风险 4):`cmd_or_ctrl_shift("u")` 目前未被任何固定/可编辑绑定占用(main 上 ClearBuffer 是 `ctrl+shift+k`,互不冲突);若 `cmd_or_ctrl_shift("u")` 在别处(如 `app/src/terminal/input.rs`)已有用途,则换一个未被占用的按键并同步更新本计划。

- [ ] **Step 4.5:新增 i18n key**

在 `app/i18n/en/warp.ftl`、`app/i18n/zh-CN/warp.ftl` 追加(参考分支同 key):

```ftl
keybinding-desc-terminal-toggle-onekey-panel = Toggle OneKey Credential Panel
```
```ftl
keybinding-desc-terminal-toggle-onekey-panel = 切换 OneKey 凭据面板
```

> `ja/warp.ftl` 中参考分支未加此 key(缺失时 `t!` 回落返回 key 本身),如要补齐可加 `keybinding-desc-terminal-toggle-onekey-panel = OneKey クレデンシャルパネルを切り替え`。保持与参考分支一致即可。

- [ ] **Step 4.6:TerminalView 字段 + 创建面板**

在 `app/src/terminal/view.rs`:
1. 顶部 import(参考分支 59-62 行),放在现有 `use crate::search::slash_command_menu::...` 之后:

```rust
#[cfg(feature = "onekey_input")]
use crate::search::onekey::{OneKeyPanel, OneKeyPanelEvent};
#[cfg(feature = "onekey_input")]
use warp_onekey;
```

2. `pub struct TerminalView` 中新增字段(参考分支 2327-2330 行):

```rust
    #[cfg(feature = "onekey_input")]
    onekey_panel: Option<ViewHandle<OneKeyPanel>>,
    #[cfg(feature = "onekey_input")]
    onekey_panel_open: bool,
```

3. 在 `new()` 中创建面板并订阅事件(参考分支 3756-3762 行,放在 `FeatureFlag::OneKeyPrompt.is_enabled()` 块附近):

```rust
        #[cfg(feature = "onekey_input")]
        let onekey_panel_handle = if FeatureFlag::OneKeyInput.is_enabled() {
            let panel = crate::search::onekey::OneKeyPanel::new(ctx);
            ctx.subscribe_to_view(&panel, Self::on_onekey_panel_event);
            Some(panel)
        } else {
            None
        };
```

4. 在 `new()` 的字段初始化中赋值(参考分支 3861-3864 行):

```rust
            #[cfg(feature = "onekey_input")]
            onekey_panel: onekey_panel_handle,
            #[cfg(feature = "onekey_input")]
            onekey_panel_open: false,
```

- [ ] **Step 4.7:面板事件路由到发送引擎**

在 `app/src/terminal/view.rs` 中新增 `on_onekey_panel_event`(参考分支 15483-15505 行):

```rust
    #[cfg(feature = "onekey_input")]
    fn on_onekey_panel_event(
        &mut self,
        _panel: ViewHandle<OneKeyPanel>,
        event: &OneKeyPanelEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            OneKeyPanelEvent::ItemSelected { credential, mode } => {
                crate::terminal::onekey_sender::send_onekey_credential(
                    self,
                    credential,
                    *mode,
                    ctx,
                );
                self.onekey_panel_open = false;
            }
            OneKeyPanelEvent::Close => {
                self.onekey_panel_open = false;
            }
            OneKeyPanelEvent::Open => {}
        }
    }
```

- [ ] **Step 4.8:action 处理 + render 定位渲染**

1. `handle_action` 的 `match action` 中追加(参考分支 23547-23556 行):

```rust
            #[cfg(feature = "onekey_input")]
            ToggleOneKeyPanel => {
                self.onekey_panel_open = !self.onekey_panel_open;
                if self.onekey_panel_open {
                    if let Some(panel) = &self.onekey_panel {
                        ctx.focus(panel);
                    }
                }
                ctx.notify();
            }
```

2. `action_shortcut`(返回 `Element` 的那个 match)中给 `ToggleOneKeyPanel` 返回 `Empty`(参考分支 23405-23406 行,该分支数组风格一致):

```rust
            #[cfg(feature = "onekey_input")]
            ToggleOneKeyPanel => Empty,
```

3. `render` 中在 overlay stack 追加面板(参考分支 24659-24664 行):

```rust
        #[cfg(feature = "onekey_input")]
        if self.onekey_panel_open {
            if let Some(panel) = &self.onekey_panel {
                stack.add_child(Align::new(ChildView::new(panel).finish()).finish());
            }
        }
```

- [ ] **Step 4.9:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过,无 dead_code 警告(面板已接线)。

Commit:

```bash
git add app/src/terminal/onekey_sender.rs app/src/terminal/onekey_sender_tests.rs app/src/terminal/view.rs app/src/terminal/view/action.rs app/src/terminal/view/init.rs app/i18n
git commit -m "feat: 接入 OneKey 面板与发送引擎(快捷键 cmd/ctrl+shift+u)"
```

---

### Task 5:PTY auto-send 扩展(对应 tasks.md 分组 5)

**Files:**
- Create: `app/src/terminal/prompt_detection.rs`
- Modify: `app/src/terminal/view.rs`(`spawn_onekey_prompt_listener` 扩展、新 handler、`show_onekey_prompt_menu` 数据源切换、su_root 数据源切换)

REF: `.worktrees/feature/20260719/quick-credential-input/app/src/terminal/prompt_detection.rs`(classify_prompt + 测试)、`view.rs`(su_root 走 `warp_onekey::find_all`,见 15533-15561)。

> 本任务的 `classify_prompt` 在参考分支是死代码(只被测试引用),本 change 把它完整接入 `spawn_onekey_prompt_listener`(design doc Decision 4,对参考的改进)。

- [ ] **Step 5.1:实现 classify_prompt + 测试**

创建 `app/src/terminal/prompt_detection.rs`,逐字拷贝参考分支(`classify_prompt(text, rules) -> Option<SendMode>`,小写化后按规则顺序子串匹配,首个命中优先)+ 内联 `mod tests`(7 个用例)。

- [ ] **Step 5.2:扩展 spawn_onekey_prompt_listener**

修改 `app/src/terminal/view.rs` 的 `spawn_onekey_prompt_listener`(7395-7426 行)。现状:stream 检测到密码提示时 `yield ()`,回调固定走 `show_onekey_prompt_menu`。改为:
1. stream 在 `bytes_look_like_password_prompt(&buf)` 命中时,先把滑动窗口内容转文本并 `yield` 出去(`buf.clear()` 逻辑不变),让回调能拿到文本做分类。
2. 回调改为新的 `on_password_prompt_detected(prompt_text: String, ctx)`。

替换后的 listener 骨架(保留滑动窗口/硬上限逻辑与 `ONEKEY_PROMPT_*` 常量不变):

```rust
        let prompt_stream = stream! {
            let mut active = rx.activate_cloned();
            let mut buf: Vec<u8> = Vec::with_capacity(ONEKEY_PROMPT_SLIDING_WINDOW_BYTES);
            while let Ok(chunk) = active.recv().await {
                buf.extend_from_slice(&chunk);
                if buf.len() > ONEKEY_PROMPT_BUFFER_HARD_LIMIT {
                    let drop_n = buf.len() - ONEKEY_PROMPT_SLIDING_WINDOW_BYTES;
                    buf.drain(..drop_n);
                }
                if bytes_look_like_password_prompt(&buf) {
                    let text = String::from_utf8_lossy(&buf).into_owned();
                    buf.clear();
                    yield text;
                }
            }
        };

        let _ = ctx.spawn_stream_local(
            prompt_stream,
            |view, text, ctx| {
                view.on_password_prompt_detected(text, ctx);
            },
            |_, _| {},
        );
```

> 锁纪律(AGENTS.md §5.3):回调在主线程跑,`on_password_prompt_detected` 内不得调用 `self.model.lock()`,凭据加载全部走 `tokio::task::spawn_blocking`,不新增嵌套锁。

- [ ] **Step 5.3:新增 on_password_prompt_detected(auto-send 逻辑)**

在 `app/src/terminal/view.rs` 中新增(放在 `spawn_onekey_prompt_listener` 之后):

```rust
    /// PTY 滑动窗口检测到密码提示后的统一入口:
    /// 1. 按 prompt_trigger_rules 分类 SendMode;
    /// 2. 恰好一条 Password 凭据 → 按 SendMode 自动发送;
    /// 3. 否则回落 show_onekey_prompt_menu。
    fn on_password_prompt_detected(&mut self, prompt_text: String, ctx: &mut ViewContext<Self>) {
        if self.ssh_secret_auto_injection_in_flight
            || self
                .onekey_last_prompt_at
                .is_some_and(|instant| instant.elapsed() < ONEKEY_PROMPT_THROTTLE)
        {
            return;
        }
        self.onekey_last_prompt_at = Some(Instant::now());

        let future = async move {
            let rules = warp_onekey::list_rules()?;
            let mode = crate::terminal::prompt_detection::classify_prompt(&prompt_text, &rules);
            let credentials: Vec<_> = warp_onekey::find_all()?
                .into_iter()
                .filter(|c| c.kind == warp_onekey::OneKeyKind::Password)
                .collect();
            anyhow::Ok((mode, credentials))
        };
        ctx.spawn(future, move |view, result, ctx| {
            let Ok((mode, credentials)) = result else {
                log::warn!("onekey: failed to load credentials for auto-send");
                return;
            };
            if let (Some(mode), Some(credential)) = (mode, credentials.into_iter().next()) {
                crate::terminal::onekey_sender::send_onekey_credential(view, &credential, mode, ctx);
            } else {
                view.show_onekey_prompt_menu(ctx);
            }
        });
    }
```

> 说明:auto-send 只对 `kind == Password` 的凭据生效(与 su_root 的 Password-only 语义一致,design doc 技术风险 2 的保守触发);`find_all()` 的凭据数恰好为 1 才自动发送,0 或多条一律回落菜单。

- [ ] **Step 5.4:show_onekey_prompt_menu 数据源切换**

`show_onekey_prompt_menu`(15507 行)现走 `tokio::task::spawn_blocking(load_saved_ssh_credentials)`。改为新的合并数据源:

新增模块级自由函数(放在 `show_onekey_prompt_menu` 附近,替代 `load_saved_ssh_credentials` 的职责):

```rust
    /// 加载 OneKey 提示菜单候选:统一凭据 + SSH 服务器保存的凭据。
    /// (原 app/src/ssh_manager/onekey.rs::load_saved_ssh_credentials 的替代,
    /// 共享凭据部分改由 warp_onekey::find_all() 提供。)
    fn load_prompt_menu_candidates() -> anyhow::Result<Vec<OneKeyPromptCandidate>> {
        let mut candidates: Vec<OneKeyPromptCandidate> = Vec::new();

        for credential in warp_onekey::find_all()? {
            candidates.push(OneKeyPromptCandidate {
                label: credential.label,
                subtitle: if credential.username.is_empty() {
                    String::new()
                } else {
                    credential.username
                },
                secret: credential.password,
                kind: OneKeyCredentialKind::Password,
            });
        }

        // SSH 服务器自身保存的凭据(node 级 password/key auth)
        let store = warp_ssh_manager::KeychainSecretStore;
        warp_ssh_manager::with_conn(|conn| {
            use warp_ssh_manager::{AuthType, NodeKind, SecretKind, SshRepository, SshSecretStore};
            let nodes = SshRepository::list_nodes(conn)?;
            for node in nodes {
                if node.kind != NodeKind::Server {
                    continue;
                }
                let Some(server) = SshRepository::get_server(conn, &node.id)? else {
                    continue;
                };
                let (secret_kind, kind) = match server.auth_type {
                    AuthType::Password => (SecretKind::Password, OneKeyCredentialKind::Password),
                    AuthType::Key => (SecretKind::Passphrase, OneKeyCredentialKind::Passphrase),
                    AuthType::OneKey => continue,
                };
                let Some(secret) = store.get(&node.id, secret_kind)? else {
                    continue;
                };
                if secret.is_empty() {
                    continue;
                }
                let target = if server.username.is_empty() {
                    format!("{}:{}", server.host, server.port)
                } else {
                    format!("{}@{}:{}", server.username, server.host, server.port)
                };
                let subtitle = match server.auth_type {
                    AuthType::Key => {
                        let key_path = server.key_path.as_deref().unwrap_or("key");
                        format!("{key_path} for {target}")
                    }
                    _ => target,
                };
                candidates.push(OneKeyPromptCandidate {
                    label: node.name,
                    subtitle,
                    secret,
                    kind,
                });
            }
            Ok(())
        })?;

        Ok(candidates)
    }
```

然后把 `show_onekey_prompt_menu` 内 `tokio::task::spawn_blocking(load_saved_ssh_credentials)` 一行改为 `tokio::task::spawn_blocking(Self::load_prompt_menu_candidates)`;其 `ctx.spawn` 回调里对候选的 `kind` 做 `OneKeyCredentialKind::Password` 过滤与菜单渲染逻辑保持不变。

> 备选方案(供 review 决策):若确认提示菜单不再需要 SSH 服务器凭据,则 `load_prompt_menu_candidates` 可退化为只返回 `warp_onekey::find_all()` 映射的候选,删除上述 SSH 服务器循环(design doc Decision 4 写的是 `warp_onekey::find_all() + SSH 凭据`,默认按该语义实现)。此决策已在 design doc 数据流图中固化,实现时若有疑问在此处与 reviewer 确认一次即可。

- [ ] **Step 5.5:su_root 数据源切换(保留 Password-only 过滤)**

`show_su_root_confirm_menu`(15764 行)内的 `tokio::task::spawn_blocking(load_saved_ssh_credentials)`(15781 行)改为:

```rust
            tokio::task::spawn_blocking(warp_onekey::find_all)
```

其回调中(15785-15817 行),`credentials` 现在是 `Vec<warp_onekey::OneKeyCredential>`,把 `credentials.into_iter().map(|c| OneKeyPromptCandidate { label: c.label, subtitle: c.username, secret: c.password, kind: OneKeyCredentialKind::Password })` 映射到 `view.onekey_prompt_candidates`(参考分支 15548-15560 行的映射写法),随后 `su_root_onekey_candidates` 的 Password-only 过滤(15810-15817 行)保持不变。这样 su_root 菜单只展示 `kind == Password` 的统一凭据(design doc Decision 5)。

- [ ] **Step 5.6:清理 import**

`app/src/terminal/view.rs:59` 的 `use crate::ssh_manager::onekey::{load_saved_ssh_credentials, OneKeyCredentialKind};` 中:
- 删除 `load_saved_ssh_credentials`(Task 8 才会删文件,这里先删 import);
- `OneKeyCredentialKind` 改为本地定义:在 view.rs 顶部(参考分支 63-67 行)新增私有枚举:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OneKeyCredentialKind {
    Password,
    Passphrase,
}
```

- [ ] **Step 5.7:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过(此时 `app/src/ssh_manager/onekey.rs` 仍存在、`load_saved_ssh_credentials` 已无引用,可能触发 dead_code 警告——预期内,Task 8 删除)。

Commit:

```bash
git add app/src/terminal/prompt_detection.rs app/src/terminal/view.rs
git commit -m "feat: PTY 密码提示接入 classify_prompt 规则分类与 auto-send"
```

---

### Task 6:SSH 面板数据源切换(对应 tasks.md 分组 6)

**Files:**
- Modify: `app/src/ssh_manager/server_view.rs`
- Modify: `app/src/workspace/view.rs`
- Modify: `app/src/sftp_manager/sftp_ops.rs`
- Modify: `app/i18n/{en,zh-CN,ja}/warp.ftl`(刷新按钮文案)

REF: `.worktrees/feature/20260719/quick-credential-input/app/src/ssh_manager/server_view.rs`(CRUD 切换 1023/1034/1075、notifier 订阅 300-308、reload 368/550、刷新按钮 1755-1771/2024)、`workspace/view.rs:5444-5468`、`sftp_manager/sftp_ops.rs:86-104`、`crates/onekey/src/repository.rs`。

- [ ] **Step 6.1:替换 OneKey overlay 的数据类型与 CRUD**

`app/src/ssh_manager/server_view.rs` 中,OneKey overlay 的渲染代码**不动**,只替换数据与 CRUD(design doc Decision 6):

1. 结构体字段 `onekey_credentials: Vec<SshOneKeyCredential>`(137 行)→ `Vec<warp_onekey::OneKeyCredential>`;`managed_onekey_kind: OneKeyCredentialKind`(141 行)→ `warp_onekey::OneKeyKind`。字段初始化处同步改(`OneKeyCredentialKind::Password` → `warp_onekey::OneKeyKind::Password`,241 行)。
2. import(35-36 行):删 `OneKeyCredentialKind`、`SshOneKeyCredential`、`SshSecretStoreError`(若仅 OneKey 路径使用),加 `use warp_onekey::{OneKeyCredential, OneKeyKind};`(参考分支 42 行)。
3. `reload`(344 行)与 `reload_onekey_credentials`(528 行)中 `warp_ssh_manager::with_conn(|c| Ok(SshRepository::list_onekey_credentials(c)?))` → `warp_onekey::find_all().unwrap_or_default()`(参考分支 368/550 行)。`reload` 不再需要把 onekey 列表塞进 `with_conn` 结果元组——从 `with_conn` 块里移出来单独调用。
4. `on_save_managed_onekey_credential`(969 行):
   - `managed_onekey_kind` 类型改为 `OneKeyKind`,`key_path_for_db` 分支(997-1000 行)用 `OneKeyKind::Password => None / OneKeyKind::Key => Some(...)` 表达(逐字参考分支 997-1000 行)。
   - 更新分支:`SshRepository::update_onekey_credential(conn, &credential)?` → `warp_onekey::update(&credential)`,并去掉 `with_conn` 包裹(参考分支 1002-1023 行:先从 `self.onekey_credentials` 找到 `existing`,改 label/username/kind/key_path,secret 非空才覆盖 password)。
   - 创建分支:`SshRepository::create_onekey_credential(...)` → `warp_onekey::create(&credential)`(参考分支 1024-1035 行)。
   - 成功后:更新 `managed_onekey_credential_id`/`selected_onekey_credential_id`、`reload_onekey_credentials(ctx)`、emit 通知(参考分支 1052-1057 行)。
5. `on_delete_managed_onekey_credential`(1072 行):`SshRepository::delete_onekey_credential(conn, &id)?` → `warp_onekey::delete(&id)`(去掉 `with_conn`),成功后 emit 通知(参考分支 1075-1089 行)。

- [ ] **Step 6.2:SSH 连接认证解析走 find_by_id**

- `app/src/workspace/view.rs` 的 SSH 连接认证解析(约 5448-5492 行):把 `AuthType::OneKey` 分支改为直接 `warp_onekey::find_by_id(&server.credential_id)`,命中后用凭据的 username/password 覆盖 `server_for_connection`(`auth_type = AuthType::Password`,secret_lookup 用 credential_id,`secret_kind = SecretKind::Password`),未命中回落现有 fallback(逐字参考分支 5449-5468 行)。
- `app/src/sftp_manager/sftp_ops.rs` 的 `resolve_sftp_auth`(86-104 行):`auth_type == OneKey` 时 `warp_onekey::find_by_id(...)` 返回 `ResolvedSshAuth { username: credential.username, auth_type: AuthType::Password, key_path: credential.key_path, secret_lookup_id: credential_id, secret_kind: SecretKind::Password }`,错误映射为 `SftpOpsError::NoCredentials`(逐字参考分支 86-104 行)。

- [ ] **Step 6.3:订阅 notifier + 打开时 reload + 刷新按钮**

1. `new()` 中订阅 `OneKeyCredentialsChangedNotifier`,收到 `CredentialsChanged` 后 `reload_onekey_credentials(ctx)` + `ctx.notify()`(参考分支 300-308 行)。
2. `OpenOneKeyManager` action 中确保打开时 `reload_onekey_credentials(ctx)`(参考分支 commit "OneKey Manager overlay now reloads credentials on open";在现有 open 处理里补一次 reload)。
3. 新增 `SshServerAction::RefreshOneKeyCredentialList` 变体 + 字段 `onekey_manager_refresh_btn_state: MouseStateHandle`(参考分支 78/136/239 行)+ 渲染刷新按钮(参考分支 1755-1771 行,文案用 `crate::t!("workspace-left-panel-ssh-manager-onekey-refresh")`)+ action 处理(参考分支 2024 行,调 `reload_onekey_credentials` + notify)。
4. i18n 三个语言文件追加(参考分支同 key):
   - en:`workspace-left-panel-ssh-manager-onekey-refresh = Refresh`
   - zh-CN:`workspace-left-panel-ssh-manager-onekey-refresh = 刷新`
   - ja:`workspace-left-panel-ssh-manager-onekey-refresh = リフレッシュ`

> `server_view.rs` 中其它仍引用 `SshRepository`/`KeychainSecretStore` 的既有逻辑(server 表单保存等)保持不变;OneKey overlay 的渲染段(下拉、表单、列表行、删除确认)一行动都不动。

- [ ] **Step 6.4:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过(仍可能有 `SshRepository::*_onekey_credential` 相关 dead_code,Task 8 删除)。

Commit:

```bash
git add app/src/ssh_manager/server_view.rs app/src/workspace/view.rs app/src/sftp_manager/sftp_ops.rs app/i18n
git commit -m "feat: SSH 面板与连接层切换到 warp_onekey 数据源并订阅变更通知"
```

---

### Task 7:设置页 OneKeyPage(对应 tasks.md 分组 7)

**Files:**
- Create: `app/src/settings_view/onekey_page.rs`
- Modify: `app/src/settings_view/mod.rs`
- Modify: `app/src/settings_view/settings_page.rs`(若 `SettingsPageViewHandle` 需要新增 OneKey 变体,随 mod.rs 一起改)

REF: `.worktrees/feature/20260719/quick-credential-input/app/src/settings_view/onekey_page.rs`(全文)、`mod.rs`(OneKey section/页面/导航)。

- [ ] **Step 7.1:新增 SettingsSection::OneKey**

`app/src/settings_view/mod.rs`:
1. `SettingsSection` 枚举在 `CloudSync`(198 行)后追加 `OneKey,`(参考分支 200 行)。
2. `impl Display` 追加 `SettingsSection::OneKey => "OneKey".to_string(),`(参考分支 232 行)。
3. `impl FromStr`(约 290 行)追加 `"OneKey" | "快速凭证" => Ok(Self::OneKey),`(参考分支 314 行)。
4. 若存在 `SettingsPageViewHandle` 枚举与 `update_settings_page_view!` 宏,追加 `OneKey(handle)` 分支(参考分支 941 行 `SettingsPageViewHandle::OneKey(handle) => $ctx.update_view(handle, $update)` 与 1829 行 `should_render`)。

- [ ] **Step 7.2:创建 onekey_page.rs**

创建 `app/src/settings_view/onekey_page.rs`,逐字拷贝参考分支全文(1127 行)。其结构:
- `OneKeyPageAction`(14 个变体,含凭据表单 + Trigger Keywords 增删改)
- `OneKeyPageView`:凭据列表、Add/Edit 表单(label/username/password/kind 切换/notes/key_path)、删除确认 `AlertDialogWithCallbacks`、Trigger Keywords 两组(`render_keyword_group` chips + OK/Cancel + Reset)
- `new()` 订阅 `OneKeyCredentialsChangedNotifier`,`CredentialsChanged` → `refresh_list()` + notify(参考分支 153-161 行)
- `SaveForm` / 删除回调里写入成功后 emit `OneKeyCredentialsChangedEvent::CredentialsChanged`(参考分支 848-850 / 871-878 行)
- 辅助函数 `load_credentials()` / `load_rules()` / `load_or_init_rules()`(规则为空时 `reset_rules_to_defaults()`)与 `build_editor` / `build_password_editor`(`is_password: true`)等
- `impl SettingsPageMeta for OneKeyPageView`:`section()` 返回 `SettingsSection::OneKey`

- [ ] **Step 7.3:注册页面 handle 与导航项**

`app/src/settings_view/mod.rs`:
1. `use onekey_page::OneKeyPageView;` 与 `mod onekey_page;`(参考分支 32/85 行)。
2. 在创建其它页面 handle 处追加(参考分支 1058-1060 行):

```rust
        // OneKey 设置页。
        let onekey_page_handle =
            ctx.add_typed_action_view(OneKeyPageView::new);
```

3. `settings_pages` 追加 `settings_pages.push(SettingsPage::new(onekey_page_handle));`(参考分支 1112 行)。
4. `nav_items` 中(参考分支 1127 行,插在 `About` 前):

```rust
            SettingsNavItem::Page(SettingsSection::OneKey),
```

- [ ] **Step 7.4:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过。

Commit:

```bash
git add app/src/settings_view/onekey_page.rs app/src/settings_view/mod.rs app/src/settings_view/settings_page.rs
git commit -m "feat: 新增设置页 OneKeyPage(凭据 CRUD + 触发关键词管理)"
```

---

### Task 8:删除旧 SSH OneKey 系统(对应 tasks.md 分组 8)

**Files:**
- Modify: `crates/warp_ssh_manager/src/{types.rs,repository.rs,secrets.rs,sync_provider.rs,lib.rs}`
- Modify: `crates/persistence/src/schema.rs`
- Delete: `app/src/ssh_manager/onekey.rs`

REF: `.worktrees/feature/20260719/quick-credential-input/crates/warp_ssh_manager/`(最终态:无 `SshOneKeyCredential`/`OneKeyCredentialKind`/`SecretKind::OneKeyPassword`/onekey 方法/`SyncOneKeyCredential`;`resolve_server_auth` 对 OneKey 返回 `Err(NotFound)`)。

- [ ] **Step 8.1:删除 warp_ssh_manager 中的旧 OneKey 系统**

1. `crates/warp_ssh_manager/src/types.rs`:
   - 删 `OneKeyCredentialKind`(62 行)与其 `as_db_str`/`parse`(67-80 行)。
   - 删 `SshOneKeyCredential`(147 行)与其 impl(157 行起)。
2. `crates/warp_ssh_manager/src/secrets.rs`:`SecretKind::OneKeyPassword`(17 行)与其 as_db_str 分支(26 行)、测试(181-193 行)。
3. `crates/warp_ssh_manager/src/repository.rs`:
   - 删 `list_onekey_credentials` / `get_onekey_credential` / `create_onekey_credential` / `update_onekey_credential` / `delete_onekey_credential`(213-282 行)。
   - `resolve_server_auth` 的 OneKey 分支改为 `AuthType::OneKey => Err(SshRepositoryError::NotFound("onekey credential".to_string()))`(参考分支 227 行)。
   - 删 `onekey_from_row`(459 行)。
   - 删对应单测 `create_list_and_update_onekey_credential` / `server_can_reference_onekey_credential` / `onekey_key_credential_resolves_to_key_auth` / `deleting_onekey_credential_clears_server_reference`(688-808 行)。
4. `crates/warp_ssh_manager/src/sync_provider.rs`:
   - 删 `SyncOneKeyCredential`(56 行)、`SshSyncData.onekey_credentials` 字段(73 行)、`onekey_secret_kind` 辅助(460 行)、export 与 import 中的 onekey 逻辑(101-110、210-213、295-297、371 行)。
   - 保留 `test_ssh_sync_data_deserializes_legacy_payload_without_onekey_fields`(参考分支 591 行,验证旧 payload 反序列化兼容)。
5. `crates/warp_ssh_manager/src/lib.rs`:删 re-export(24、28 行的 `SyncOneKeyCredential`、`SshOneKeyCredential`、`OneKeyCredentialKind`)。
6. `crates/warp_ssh_manager/src/ssh_command_tests.rs`:`test_connection_requires_password_for_onekey_auth` / `onekey_key_auth_emits_dash_i_when_key_path_is_resolved`(108/121 行)改为 OneKey 服务器直接返回"无凭据"语义的断言(参考参考分支最终态;若参考分支已删,直接删这两个用例)。
7. 若删后 `zeroize`/`keyring` 在 `crates/warp_ssh_manager/Cargo.toml` 变为未用依赖,`cargo check` 会告警——按告警清理(该 crate 其它路径仍用 keyring/zeroize,通常无需动)。

- [ ] **Step 8.2:删除 persistence 中 ssh_onekey_credentials 的 model/schema**

1. `crates/persistence/src/schema.rs`:删 `ssh_onekey_credentials` 的 `diesel::table!`(371 行)、`joinable!(ssh_servers -> ssh_onekey_credentials ...)`(541 行)、`allow_tables_to_appear_in_same_query!(ssh_nodes, ssh_onekey_credentials, ssh_servers, ...)`(561 行)中的 `ssh_onekey_credentials`。
2. `crates/persistence/src/model.rs`:删 `SshOneKeyCredentialRow` 与 `NewSshOneKeyCredential` 定义(grep 确认行号)。
3. 若仓库有 `diesel print-schema` 脚本,重跑并确认 schema 与迁移一致;否则手改 schema.rs 并保证与 Task 1 的迁移 SQL 对齐(design doc: schema.rs 由 print-schema 生成,但本仓库迁移后需手动同步)。

> 注:Task 1 的迁移已 drop 表,这里删的是编译期的 model/schema 定义,顺序不冲突。

- [ ] **Step 8.3:删除 app 侧旧文件与旧数据源**

1. 删除 `app/src/ssh_manager/onekey.rs`(含 `load_saved_ssh_credentials`)。
2. `app/src/ssh_manager/mod.rs`:删 `pub mod onekey;`(8 行)。
3. `app/src/terminal/view.rs`:Task 5 已把 `show_onekey_prompt_menu` / `show_su_root_confirm_menu` 数据源切换掉;确认全文不再引用 `load_saved_ssh_credentials`(grep)。

- [ ] **Step 8.4:全局 grep 残留**

Run: `cargo check -p warp` 前先全局扫一遍残留:

```bash
grep -rn "quick_credential\|SshOneKeyCredential\|OneKeyCredentialKind\|OneKeyPassword\|load_saved_ssh_credentials\|ssh_onekey_credentials\|SyncOneKeyCredential\|list_onekey_credentials\|create_onekey_credential\|update_onekey_credential\|delete_onekey_credential" app crates --include=*.rs
```

Expected: 无任何命中(除 `crates/persistence/migrations/` 下 Task 1 迁移注释中的历史表名——迁移 SQL 必须保留原表名做 down,属预期残留)。

- [ ] **Step 8.5:验证 + Commit**

Run: `cargo check -p warp`
Expected: 编译通过,无 dead_code 告警。

Commit:

```bash
git add -A crates/warp_ssh_manager crates/persistence app/src/ssh_manager app/src/terminal/view.rs
git commit -m "refactor: 删除旧 SSH OneKey 系统(SshOneKeyCredential/ssh_onekey_credentials/load_saved_ssh_credentials)"
```

---

### Task 9:验证(对应 tasks.md 分组 9)

**Files:**
- 无新文件(只读验证)

- [ ] **Step 9.1:cargo check**

Run: `cargo check`
Expected: 全 workspace 编译通过。

- [ ] **Step 9.2:全量测试**

Run: `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`
Expected: 全绿。重点观察 `warp_onekey` 的 `types_tests` / `repository_tests`、`search::onekey::data_source_tests`、`terminal::prompt_detection` 内联测试、`terminal::onekey_sender` 内联测试、`warp_ssh_manager` 剩余测试。

- [ ] **Step 9.3:手动验证清单**

在本地 dev 构建(启用 `onekey_input` + `onekey_prompt` + `FeatureFlag::OneKeyInput/OneKeyPrompt`)逐项验证:
1. 面板:`cmd/ctrl+shift+u` 唤起 `OneKeyPanel`,搜索(按 label/username 模糊匹配)、键盘上下导航、Enter 选中后出现发送模式二选一,两项都能把凭据写入 PTY(PasswordOnly 直接写 `secret\n`;UsernameThenPassword 先写 `username\n` 再 ~150ms 写 `password\n`)。
2. 面板关闭:Esc / 点击外部关闭。
3. 发送模式在选中凭据后即时选择,再次打开面板不残留上次模式。
4. SSH 面板 OneKey overlay:新增/编辑/删除凭据正常,UI 布局与改动前一致;打开时列表 reload;保存/删除后其它已打开的视图(设置页)自动刷新。
5. 设置页 OneKeyPage:凭据 CRUD(label/password 必填校验、kind 切换后显示 key_path 输入框)、Trigger Keywords 两组增删与 Reset。
6. auto-send:终端里执行 `ssh user@host`(目标用 OneKey 密码),PTY 出现 `password:` 提示且库中恰有 1 条 Password 凭据时自动发送;≥2 条时回落 OneKey 菜单;`login:`/`username:` 提示命中 UsernameThenPassword 关键词时按模式发送。
7. su_root:`su`/`sudo -i` 触发 root 密码确认菜单,候选仅包含 Password 类型凭据。
8. 跨视图自动刷新:设置页改凭据 → SSH 面板 dropdown 立即刷新,反之亦然。
9. SSH 连接(SFTP / workspace 面板)对 `auth_type == OneKey` 的服务器能正确用 `warp_onekey::find_by_id` 解析并连上。
10. 热键冲突:`cmd/ctrl+shift+u` 在终端内不触发其它动作。

- [ ] **Step 9.4:收尾(可选)**

若确认 feature 稳定,按 `promote-feature` skill 流程将 `OneKeyInput` 从 DOGFOOD 提升/清理 flag——**不在本 change 范围内**,仅在计划里留档。

---

## 3. 自检(Self-Review)

对照 design doc / tasks.md 逐项核对:

| spec 要求 | 对应任务 |
|---|---|
| Decision 1:统一数据模型与存储(`crates/onekey`) | Task 1 |
| Decision 2:干净迁移(建两表 + drop 旧表,不复制 quick_credentials 死迁移) | Task 1 Step 1.6 |
| Decision 3:SearchBar/SearchMixer 面板 + 热键 `cmd_or_ctrl_shift("u")` | Task 3、Task 4 Step 4.4 |
| Decision 4:发送引擎 + auto-send 完整接入生产 | Task 4、Task 5 |
| Decision 5:su_root 保留 Password-only 过滤 | Task 5 Step 5.5 |
| Decision 6:SSH 面板数据源切换(UI 不变) | Task 6 |
| Decision 7:变更自动刷新(`OneKeyCredentialsChangedNotifier` + `CREDENTIALS_VERSION`) | Task 1 Step 1.7、Task 2 Step 2.4、Task 6 Step 6.3、Task 7 |
| tasks.md 分组 1(1.1~1.7) | Task 1 |
| tasks.md 分组 2(2.1~2.3) | Task 2 |
| tasks.md 分组 3(3.1~3.4) | Task 3、Task 4 Step 4.4 |
| tasks.md 分组 4(4.1~4.3) | Task 4 |
| tasks.md 分组 5(5.1~5.3) | Task 5 |
| tasks.md 分组 6(6.1~6.3) | Task 6 |
| tasks.md 分组 7(7.1~7.6) | Task 7 |
| tasks.md 分组 8(8.1~8.4) | Task 8 |
| tasks.md 分组 9(9.1~9.3) | Task 9 |
| design doc 文件变更表中的每个文件 | 1~8 的 Files 段已全覆盖 |

**已知的与参考分支的差异**(design doc 明确):
- 不复制 `2026-07-19-000000_add_quick_credentials` 死迁移(Decision 2)。
- `classify_prompt` 不是死代码,接入生产(Decision 4)。
- su_root 数据源用 `warp_onekey::find_all()` 过滤 Password,而非参考分支的全量映射(Decision 5)。

**执行顺序红线**:Task 8 删除旧系统必须在 Task 5/6 全部完成之后;否则 `app/src/terminal/view.rs` 与 `server_view.rs` 引用 `load_saved_ssh_credentials` / `SshRepository::*_onekey_credential` 会编译失败。若并行执行,Task 8 只允许在 5/6 的 commit 之后落地。
