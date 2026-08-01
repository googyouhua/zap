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

表结构通过新增 Diesel migration(`crates/persistence/migrations/2026-07-31-000000_onekey_credentials_and_drop_ssh_onekey`)创建:`up.sql` 建 `onekey_credentials` + `prompt_trigger_rules` 表,drop 旧 `ssh_onekey_credentials` 表并重建 `ssh_servers`(去掉对其的外键约束)。`db.rs` 保留 `CREATE TABLE IF NOT EXISTS` + `ensure_columns`(pragma 检查后 ALTER)作防御性层,与 `warp_ssh_manager::db` 的既有模式一致。

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

### Decision 6: 删除旧 SSH OneKey
`SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、`SshRepository::*_onekey_credential` CRUD、`sync_provider` 中 `SyncOneKeyCredential`、`app/src/ssh_manager/onekey.rs`(`load_saved_ssh_credentials`)全部删除;`ssh_onekey_credentials` 表加 migration drop。旧 keychain entry 保留但不再访问。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 旧 SSH OneKey 凭据升级后不可用 | 一次性成本,用户重新配置;proposal 已声明 BREAKING |
| Linux 无桌面环境(WSL/headless)时 `keyring` 不可用 | `encrypted_password` 列 fallback |
| 150ms 延迟在不同 shell 中可能不足 | 与参考分支一致,先保持固定值;必要时做成可配置 |
| OneKey 与 SSH 凭据混合展示可能混乱 | 通过图标/类型标识区分 |
| Cargo feature `onekey_input` 与 FeatureFlag 两套开关易混 | 单一用途:`onekey_input` 门控面板集成,`OneKeyPrompt` 门控提示监听,design 中明确 |
| feature 门控漏改引用 | 全局 grep `quick_credential` / `onekey_input` 确认 |

## Migration Plan

1. 新建 `crates/onekey`,实现存储层与 repository + 单元测试
2. 新增 `onekey_input` Cargo feature(默认启用),接入 FeatureFlag 列表
3. 终端面板 + 发送引擎 + auto-send 扩展(替换 `show_onekey_prompt_menu` 数据源)
4. SSH 面板数据源切换;SSH 连接/SFTP 认证改用 `warp_onekey::find_by_id()`
5. 设置页 OneKeyPage + notifier 自动刷新
6. 删除旧 OneKey 类型/API/`onekey.rs` + drop 表 migration
7. `cargo check` + `cargo nextest` 全量验证

**回滚**:整个 change 走单一分支合并;若出现问题可整体 revert(除 drop 表 migration 外均无破坏性数据操作)。旧 keychain entry 不删除,回滚后仍可手动恢复。

## Open Questions

- 快捷键:已定为 `cmd_or_ctrl_shift("u")`(macOS cmd+shift+u,Linux/Win ctrl+shift+u)→ `ToggleOneKeyPanel`,避开与 main 中 ClearBuffer 的 `ctrl+shift+k` 冲突;已在 `app/src/terminal/view/init.rs` 注册并确认无绑定冲突(与 Design Doc D3 一致)。
- `encrypted_password` 列的加密实现:参考分支留作 keychain fallback,具体加解密方式实现时确定。
