## ADDED Requirements

### Requirement: 分号分隔批量添加
系统在 denylist host 输入框提交时，应按 `;` 分割输入内容，逐条 trim 后添加，空片段跳过。

#### Scenario: 输入多个分号分隔的 host
- **WHEN** 用户输入 `host1;host2;host3` 并按下 Enter
- **THEN** 三个 host 都添加到 denylist

#### Scenario: 输入带空格的片段
- **WHEN** 用户输入 ` host1 ; host2 ; host3 ` 并按下 Enter
- **THEN** trim 后三个 host 都正确添加

#### Scenario: 输入包含空片段
- **WHEN** 用户输入 `host1;;host3` 并按下 Enter
- **THEN** 空片段被跳过，host1 和 host3 被添加
