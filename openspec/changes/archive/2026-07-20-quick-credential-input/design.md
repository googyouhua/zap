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

**理由**: 用户在需求中明确表示"每次弹窗选择"发送模式。复用现有的 context menu 机制（`Menu<TerminalAction>`）或添加内联选项栏。

实现方式：在 `QuickCredentialPanel` 中维护状态 `selected_credential: Option<QuickCredential>`，选中后显示发送模式按钮。

### D5: 发送引擎

**选择**: `QuickCredentialSender` 工具函数，接收凭证和发送模式，调用 `TerminalView` 的写入方法。

```rust
fn send_credential(
    terminal_view: &mut TerminalView,
    credential: &QuickCredential,
    mode: SendMode,
    ctx: &mut ViewContext<TerminalView>,
) {
    match mode {
        SendMode::PasswordOnly => {
            terminal_view.clear_line_editor_and_write_to_pty(
                format!("{}\n", credential.password),
                ctx,
            );
        }
        SendMode::UsernameThenPassword => {
            terminal_view.clear_line_editor_and_write_to_pty(
                format!("{}\n", credential.username),
                ctx,
            );
            // 延迟 ~150ms 后发送密码，等待 shell 处理用户名
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

**理由**: `clear_line_editor_and_write_to_pty` 在发送前清空当前行（避免与已输入内容冲突）。150ms 延迟经测试足够覆盖大部分 shell 的用户名处理时间，又不会让用户感觉卡顿。

### D6: 快捷键注册

**选择**: 注册 `EditableBinding` 在 Terminal 上下文中。

```rust
app.register_editable_bindings([
    EditableBinding::new(
        "terminal:toggle_quick_credential_panel",
        "Show quick credential input panel",
        TerminalAction::ToggleQuickCredentialPanel,
    )
    .with_context_predicate(id!("Terminal")),
]);
```

默认快捷键: `cmd+shift+k` (macOS) / `ctrl+shift+k` (Linux/Windows)。

### D7: PTY 自动检测增强

**选择**: 在 `show_onekey_prompt_menu` 中同时加载 SSH OneKey 凭证和通用凭证，合并展示。

**理由**: 通用凭证也需要密码提示自动弹出功能。合并展示让用户可以在一个面板中看到所有可用凭证。通过 `SendMode` 区分 SSH（仅密码）和通用凭证。

### D8: 凭证管理 UI

**选择**: 在设置中新增 "Quick Credentials" 页面，位于 `app/src/settings_view/` 中。

条目列表 + CRUD 表单：
- 列表：label, username 预览, send_mode 图标
- 添加/编辑：label, username, password (input), send_mode (dropdown), notes
- 删除：确认对话框

**密码输入**: 使用 `EditorView` 但 masked 显示（类似现有 SSH Manager 中的密码字段处理）。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Linux 无桌面环境（WSL/headless）时 `keyring` 不可用 | fallback 到 SQLite 加密存储，或提示用户使用 SSH key 认证 |
| 150ms 延迟在不同 shell 中可能不足 | 可配置延迟时间；或检测到下一个提示再发送密码 |
| 通用凭证与 SSH OneKey 凭证混合展示可能导致混乱 | 通过图标/标签区分类型（SSH 密钥图标 vs 通用钥匙图标） |
| 密码在内存中以 `Zeroizing<String>` 存在但仍有残留风险 | 复用 `Zeroizing` 类型，发送后及时 clear |
| 多个用户同时使用同一台机器 | 依赖 OS Keychain 的多用户隔离机制 |
