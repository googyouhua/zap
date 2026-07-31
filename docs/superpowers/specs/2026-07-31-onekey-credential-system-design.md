---
comet_change: onekey-credential-system
role: technical-design
canonical_spec: openspec
---

# 统一 OneKey 凭据系统 — Design Doc

## Problem

main 分支只有一套与 SSH Manager 深度绑定的旧 OneKey 系统(`SshOneKeyCredential` + `ssh_onekey_credentials` 表 + `load_saved_ssh_credentials`),只能覆盖 SSH 面板内的服务器凭据,无法处理普通终端中手动执行命令(`ssh user@host`、`mysql -u`、`docker login`)的密码输入。参考分支 `feature/20260719/quick-credential-input` 已落地统一的 OneKey 凭据系统,本 change 在 main 上重新实现并完整对齐该分支的最终状态。

## 设计决策

### Decision 1: 统一 OneKey 数据模型与存储

新建 `crates/onekey`(package `warp_onekey`),沿用参考分支最终模型:

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

### Decision 2: 表结构通过干净迁移创建

新建 Diesel 迁移创建 `onekey_credentials` + `prompt_trigger_rules` 表,并 drop 旧 `ssh_onekey_credentials` 表(含 `ssh_servers` 外键重建):

```sql
-- up.sql
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

**不复制**参考分支的 legacy `quick_credentials` 死迁移。`db.rs` 运行时 `CREATE TABLE IF NOT EXISTS` + `ensure_columns`(pragma 检查后 ALTER)保留作防御性层,与 `warp_ssh_manager::db` 的既有模式一致。

**替代方案**:复用 `warp_ssh_manager` 的 `ssh_onekey_credentials` 表加列。否决理由:旧表与 SSH 节点模型耦合,且本 change 要删除该表;独立 crate 生命周期清晰、依赖无环。

### Decision 3: 终端搜索面板复用 SearchBar + SearchMixer 模式

新建 `app/src/search/onekey/`,实现 `OneKeyPanel`,参考 `ExternalSecretsMenu`:

```
OneKeyPanel
  ├── SearchBar<OneKeyAction> — 搜索栏
  ├── SearchMixer<OneKeyAction> — 数据源(模糊匹配 label/username)
  ├── SearchBarState<OneKeyAction> — 列表状态
  └── OneKeyItem — 搜索结果项
```

面板事件 `OneKeyPanelEvent::{ ItemSelected { credential, mode }, Close }`,发送模式在选择凭据后即时选择,不存储在凭据上。

**快捷键**:`cmd_or_ctrl_shift("u")`(macOS cmd+shift+u,Linux/Win ctrl+shift+u)→ `TerminalAction::ToggleOneKeyPanel`,注册于 `app/src/terminal/view/init.rs`,避开与 main 中 ClearBuffer 的 `ctrl+shift+k` 冲突。

**替代方案**:自研轻量弹层。否决:放弃成熟的键盘导航/模糊搜索/定位渲染复用,风险更高。

### Decision 4: 发送引擎与 auto-send 完整实现

`send_credential()` 工具函数:清空当前行(`clear_line_editor_and_write_to_pty`)→ 写 `secret\n`;`UsernameThenPassword` 模式先写 `username\n`,~150ms 后写 `password\n`(`ctx.spawn_after`)。secret 全程 `Zeroizing<String>`。

密码提示检测**复用并保留** main 既有的 `spawn_onekey_prompt_listener` 滑动窗口与 `show_onekey_prompt_menu`,扩展为:

1. 加载触发规则 `warp_onekey::list_rules()`
2. 用 `classify_prompt`(参考分支的 `PromptTriggerRule` 分类器)对滑动窗口文本分类出 `PromptType::{Password, Username}`
3. 恰好一条凭据时按 SendMode 自动发送;0 条或多条时回落 `show_onekey_prompt_menu`(数据源切换为 `warp_onekey::find_all()` + SSH 凭据)

参考分支的 `classify_prompt` 在最终态是死代码(仅测试引用),本次**完整接入生产流程**,是对参考的改进而非照搬。

**双开关**:Cargo feature `onekey_input`(默认启用)+ 运行时 `FeatureFlag::OneKeyInput`(DOGFOOD)。既有 `FeatureFlag::OneKeyPrompt` 继续门控提示监听,避免行为回归。app 内 feature 门控模式对齐 `app/src/lib.rs:2333` 现有 `onekey_prompt` 用法。

### Decision 5: su_root 菜单保留过滤语义

main 的 `show_su_root_confirm_menu` 已有 `su_root_onekey_candidates` 且过滤 Password-only 凭据。保留该过滤语义,数据源从 `load_saved_ssh_credentials` 切换为 `warp_onekey::find_all()`(过滤 kind==Password)。不并入全部凭据类型。

### Decision 6: SSH 面板数据源切换

`server_view.rs` 中 OneKey overlay 的渲染代码完全不动,只替换 CRUD API:

- `SshRepository::list_onekey_credentials()` → `warp_onekey::find_all()`
- `SshRepository::create_onekey_credential()` → `warp_onekey::create()`
- `SshRepository::delete_onekey_credential()` → `warp_onekey::delete()`
- `SshRepository::update_onekey_credential()` → `warp_onekey::update()`

SSH 连接流程(`workspace/view.rs`、`sftp_manager/sftp_ops.rs`):`auth_type == OneKey` 时直接 `warp_onekey::find_by_id(credential_id)`,绕开旧 `resolve_server_auth` 与 keychain key 格式不兼容问题。

### Decision 7: 变更自动刷新

沿用参考分支方案:`crates/onekey` 的 repository 内 `AtomicU64 CREDENTIALS_VERSION` 计数器,`create/update/delete` 成功后 `bump_credentials_version()`;`app/src/ssh_manager/onekey_notifier.rs` 定义 `OneKeyCredentialsChangedNotifier`(`SingletonEntity`,遵循 `SshTreeChangedNotifier` 模式);写入端在成功后 emit 通知,订阅端(`SshServerView`、`OneKeyPageView`)监听后 reload 凭据列表。

**替代方案**:每次渲染时轮询版本号。否决:不够及时,且侵入渲染路径。

## 数据流

```
OneKeyPanel(hotkey u) ──ItemSelected──▶ send_credential(clear_line → secret\n / username\n +150ms password\n)
                                           ▲
spawn_onekey_prompt_listener(滑动窗口) ──▶ classify_prompt(list_rules) ──▶ 1条凭据? auto-send
                                                                          └▶ 0/多条? show_onekey_prompt_menu
SSH 面板 server_view(OneKey overlay) ──▶ warp_onekey::create/update/delete ──▶ bump_credentials_version()
设置页 onekey_page ──▶ warp_onekey::* ──▶ bump_credentials_version()           └▶ emit OneKeyCredentialsChangedNotifier
warp_onekey::find_by_id(credential_id) ◀── SSH 连接(SFTP/workspace)
```

## 文件变更

| 文件 | 变更 |
|------|------|
| `crates/onekey/` | 新 crate:`lib.rs`、`db.rs`、`types.rs`、`secret_store.rs`、`repository.rs`、`types_tests.rs`、`repository_tests.rs` |
| `crates/persistence/migrations/` | 新迁移:建 `onekey_credentials` + `prompt_trigger_rules`,drop `ssh_onekey_credentials` |
| `crates/persistence/src/schema.rs` | 增 `onekey_credentials` / `prompt_trigger_rules` table!,删 `ssh_onekey_credentials` |
| `crates/warp_core/src/features.rs` | 增 `FeatureFlag::OneKeyInput`(DOGFOOD) |
| `app/Cargo.toml` | 增 `onekey_input` feature(默认启用) |
| `app/src/lib.rs` | 注册 `warp_onekey::set_database_path` + notifier singleton |
| `app/src/search/onekey/` | 新面板模块(`mod.rs`、`view.rs`) |
| `app/src/terminal/view.rs` | 删 `show_onekey_prompt_menu` 旧数据源,集成 `OneKeyPanel` + auto-send 接线,su_root 数据源切换 |
| `app/src/terminal/view/init.rs` | 注册 `cmd_or_ctrl_shift("u")` → `ToggleOneKeyPanel` |
| `app/src/terminal/prompt_detection.rs` | 新建:接入 `classify_prompt` + `list_rules` 生产流程 |
| `app/src/terminal/onekey_sender.rs` | 新建:发送引擎 |
| `app/src/ssh_manager/server_view.rs` | OneKey overlay CRUD 数据源切换 + notifier 订阅 |
| `app/src/ssh_manager/onekey_notifier.rs` | 新建 `OneKeyCredentialsChangedNotifier` |
| `app/src/ssh_manager/onekey.rs` | 删除(`load_saved_ssh_credentials`) |
| `app/src/settings_view/onekey_page.rs` | 新建:凭据 CRUD + Trigger Keywords |
| `app/src/workspace/view.rs`、`app/src/sftp_manager/sftp_ops.rs` | OneKey 认证走 `warp_onekey::find_by_id()` |
| `crates/warp_ssh_manager/` | 删 `SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、5 个 onekey repository 方法、`resolve_server_auth` OneKey 分支、`SyncOneKeyCredential` |

## 技术风险

1. **TerminalModel 锁**:`spawn_onekey_prompt_listener` 中操作 TerminalModel 需遵守 AGENTS.md §5.3,不新增嵌套锁;已锁引用沿调用栈下传。
2. **auto-send 误触发**:分类基于关键词子串匹配,存在误报可能。缓解:复用既有滑动窗口 + su_root Password-only 过滤语义,保持保守触发。
3. **旧数据作废**:旧 SSH OneKey 数据不迁移,升级后作废需重新配置(proposal/design non-goals 已写明)。
4. **快捷键冲突**:已选 `u` 避开 ClearBuffer 的 `ctrl+shift+k`;仍需在实现时核对全部固定绑定。

## 不涉及

- 不迁移旧 SSH OneKey 数据
- 不改 SSH 面板 overlay 的 UI 布局或交互
- 不与外部密码管理器集成,不做云同步、密码生成器、网页表单填充
- 不引入参考分支的 legacy `quick_credentials` 迁移
