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
