## 1. 存储层 crates/onekey

- [x] 1.1 新建 `crates/onekey`(package `warp_onekey`)crate,配置 `Cargo.toml`(diesel/keyring/zeroize/uuid/persistence/warp_ssh_manager),加入 workspace members
- [x] 1.2 实现 `db.rs`:`CREATE TABLE IF NOT EXISTS onekey_credentials` + `ensure_columns` 兼容 ALTER + `prompt_trigger_rules` 表;`set_database_path` / `with_conn` / 测试连接注入
- [x] 1.3 实现 `types.rs`:`OneKeyKind` / `OneKeyCredential` / `SendMode` / `PromptTriggerRule` + 默认关键词常量
- [x] 1.4 实现 `secret_store.rs`:`OneKeySecretStore`(keyring service `zap.onekey`,account `<uuid>:secret`,失败 fallback `encrypted_password` 列)
- [x] 1.5 实现 `repository.rs`:`find_all` / `find_by_id` / `create` / `update` / `delete` / `list_rules` / `add_rule` / `remove_rule` / `reset_rules_for_mode` / `reset_rules_to_defaults`
- [x] 1.6 实现 `repository.rs` 的 `AtomicU64 CREDENTIALS_VERSION`,`create/update/delete` 成功后 `bump_credentials_version()`,导出 `credentials_version()`
- [x] 1.7 编写 `types_tests.rs` 与 `repository_tests.rs` 单元测试

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
