## MODIFIED Requirements

### Requirement: 输入内容原样保存
系统在 denylist host 输入框提交时，应将输入内容作为一个完整条目保存，不做 `;` 分割。

#### Scenario: 输入包含分号的内容
- **WHEN** 用户输入 `host1;host2` 并按下 Enter
- **THEN** `host1;host2` 作为单个条目添加到 denylist
