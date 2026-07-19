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
