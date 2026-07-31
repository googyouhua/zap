## Purpose

OneKey 凭据的管理界面:设置页 `onekey_page.rs` 提供凭据的增删改查与触发关键词管理,SSH 面板内的 OneKey Manager overlay 复用统一凭据数据源,并在凭据变更后自动刷新所有已打开视图。

## ADDED Requirements

### Requirement: 设置页列出全部已保存凭据
设置页 SHALL 以列表展示全部 OneKey 凭据,每条显示 label、username 预览与类型(kind)标识。

#### Scenario: 查看凭据列表
- **WHEN** 用户导航到 OneKey 设置页
- **THEN** 全部已保存凭据以 label、username、类型展示在列表中

### Requirement: 新增凭据
设置页 SHALL 提供新增凭据表单,字段包含 label、username、password、kind 选择器、key_path、notes。

#### Scenario: 成功新增凭据
- **WHEN** 用户填完表单并点击保存
- **THEN** 凭据被持久化(SQLite + OS Keychain)并出现在列表中

#### Scenario: 缺少 label
- **WHEN** 用户未填 label 就尝试保存
- **THEN** 显示错误 "Label is required",凭据不保存

#### Scenario: 缺少密码
- **WHEN** 用户未填 password 就尝试保存
- **THEN** 显示错误 "Password is required",凭据不保存

### Requirement: 编辑既有凭据
设置页 SHALL 允许编辑既有凭据的全部字段。

#### Scenario: 编辑凭据 label
- **WHEN** 用户编辑既有凭据的 label 并保存
- **THEN** 该凭据的 label 在 SQLite 中更新

#### Scenario: 编辑凭据密码
- **WHEN** 用户编辑既有凭据的 password 并保存
- **THEN** 新密码存储到 OS Keychain

### Requirement: 删除凭据
设置页 SHALL 允许删除凭据,删除前需确认对话框。

#### Scenario: 删除凭据
- **WHEN** 用户点击某条凭据的删除并确认
- **THEN** 该凭据从 SQLite 与 OS Keychain 移除

#### Scenario: 取消删除
- **WHEN** 用户点击删除但取消确认
- **THEN** 凭据不被删除

### Requirement: 管理触发关键词
设置页 SHALL 展示触发关键词管理区块,分为 PasswordOnly 与 UsernameThenPassword 两组关键词,支持新增、删除、重置为默认。

#### Scenario: 查看触发关键词
- **WHEN** 用户查看 OneKey 设置页
- **THEN** 凭据列表上方展示 "Trigger Keywords" 区块,含 PasswordOnly 与 UsernameThenPassword 两组

#### Scenario: 新增触发关键词
- **WHEN** 用户点击 "+ Add" 并输入关键词
- **THEN** 关键词加入对应分组并持久化

#### Scenario: 删除触发关键词
- **WHEN** 用户点击某关键词 chip 的 × 按钮
- **THEN** 关键词被移除并持久化

#### Scenario: 重置触发关键词
- **WHEN** 用户点击 Trigger Keywords 区块的 "Reset" 按钮
- **THEN** 当前全部关键词被默认集合替换

### Requirement: 凭据变更后自动刷新视图
系统 SHALL 在任一视图新增/修改/删除 OneKey 凭据后,通知所有已打开依赖凭据列表的视图自动重新加载,无需用户手动刷新。

#### Scenario: 增删改后其它视图自动刷新
- **WHEN** 用户在设置页新增一条凭据,而 SSH 面板或 OneKey 面板已打开
- **THEN** 已打开的视图自动重新加载凭据列表并展示新凭据

#### Scenario: 删除后列表即时更新
- **WHEN** 用户在 SSH 面板 OneKey Manager 中删除一条凭据
- **THEN** 该 overlay 的凭据列表即时刷新,已删除项消失
