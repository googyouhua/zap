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
