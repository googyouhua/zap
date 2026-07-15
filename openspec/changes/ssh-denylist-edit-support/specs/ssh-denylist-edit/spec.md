## ADDED Requirements

### Requirement: 点击 hostname 编辑
系统应支持通过点击列表中的 hostname 回填输入框进行编辑。编辑模式下提交替换原条目而非追加。

#### Scenario: 点击 hostname 开始编辑
- **WHEN** 用户点击 denylist 列表中的 hostname
- **THEN** 该 hostname 填入编辑框，内容可修改

#### Scenario: 编辑后提交替换
- **WHEN** 用户编辑后按下 Enter
- **THEN** 原 hostname 被替换为新值，列表刷新

#### Scenario: 编辑时按下 Esc 取消
- **WHEN** 用户按下 Esc
- **THEN** 编辑状态清除，输入框清空

#### Scenario: 编辑模式下不支持批量
- **WHEN** 处于编辑模式时输入包含 `;` 的内容并提交
- **THEN** 只替换当前条目，不执行 `;` 分割
