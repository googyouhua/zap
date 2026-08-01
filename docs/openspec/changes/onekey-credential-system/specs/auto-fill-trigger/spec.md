## Purpose

可配置的关键词到发送模式的映射规则,供终端密码提示检测器对终端输出做分类,在只有一条凭据时自动发送,否则回落 OneKey 菜单。

## ADDED Requirements

### Requirement: 首次运行时填入默认触发关键词
当功能首次启用且尚无触发规则时,系统 SHALL 填入默认规则:PasswordOnly 关键词 = {password, passphrase},UsernameThenPassword 关键词 = {login, username, user, name, email, account}。

> 注:默认关键词在首次打开 OneKey 设置页时 seed(仅当规则表为空时);auto-send 检测路径(`list_rules`)不独立 seed,因此新装用户需先打开一次设置页后 auto-send 才会生效。

#### Scenario: 首次初始化
- **WHEN** 用户首次打开 OneKey 设置页且发现规则表为空
- **THEN** 插入默认关键词集合并返回

#### Scenario: 后续加载保留用户修改
- **WHEN** 用户已自定义触发规则,系统在下次启动时加载
- **THEN** 返回用户的规则,而非默认集合

### Requirement: 新增触发关键词
系统 SHALL 允许新增带关联 SendMode 的关键词。

#### Scenario: 新增仅密码关键词
- **WHEN** 用户新增关键词 "secret" 且模式为 PasswordOnly
- **THEN** 关键词 "secret" 以 mode "password_only" 插入触发规则表

#### Scenario: 新增用户名+密码关键词
- **WHEN** 用户新增关键词 "account" 且模式为 UsernameThenPassword
- **THEN** 关键词 "account" 以 mode "username_then_password" 插入触发规则表

#### Scenario: 拒绝重复关键词
- **WHEN** 用户尝试新增已存在的关键词
- **THEN** 系统拒绝该重复项(no-op 或报错)

### Requirement: 删除触发关键词
系统 SHALL 允许从触发规则中移除关键词。

#### Scenario: 删除关键词
- **WHEN** 用户点击关键词 "passphrase" 的删除按钮
- **THEN** 关键词 "passphrase" 从触发规则表移除

### Requirement: 将触发关键词重置为默认
系统 SHALL 提供清空当前全部规则并重新插入默认关键词集合的入口。

#### Scenario: 重置为默认
- **WHEN** 用户点击 "Reset" 按钮
- **THEN** 当前全部触发规则被删除,默认关键词集合被插入

### Requirement: 对 PTY 输出按触发规则分类
系统 SHALL 将终端 PTY 输出与全部已配置触发关键词比对,判断是否存在密码型或用户名型提示。

#### Scenario: 检测到密码提示
- **WHEN** 终端输出包含 "password:" 且关键词 "password" 的规则模式为 PasswordOnly
- **THEN** 检测返回 PromptType::Password

#### Scenario: 检测到用户名提示
- **WHEN** 终端输出包含 "login:" 且关键词 "login" 的规则模式为 UsernameThenPassword
- **THEN** 检测返回 PromptType::Username

#### Scenario: 无关键词匹配
- **WHEN** 终端输出不匹配任何触发关键词
- **THEN** 检测返回 None

### Requirement: 在 PTY 输出流中持续分类提示
提示检测器 SHALL 持续监控 PTY 输出流,当终端输出匹配关键词模式时触发分类,复用既有 OneKey 密码提示检测的滑动窗口方式。

#### Scenario: 持续监控
- **WHEN** PTY 输出流动且出现匹配触发模式的片段
- **THEN** 检测器触发并进入 auto-send 逻辑

### Requirement: 恰好一条凭据时自动发送
当提示被分类时,系统 SHALL 加载全部 OneKey 凭据。若恰好存在一条凭据,系统 SHALL 按匹配关键词规则对应的 SendMode 自动发送;若为 0 条或多条,系统 SHALL 回落展示 OneKey 菜单。

#### Scenario: 单条凭据自动发送密码
- **WHEN** 检测到密码型提示且恰好存在一条 OneKey 凭据
- **THEN** 该凭据的密码被发送到 PTY(仅密码)

#### Scenario: 单条凭据自动发送用户名+密码
- **WHEN** 检测到用户名型提示且恰好存在一条 OneKey 凭据
- **THEN** 先发送该凭据用户名,~150ms 后发送密码

#### Scenario: 多条凭据回落 OneKey 菜单
- **WHEN** 检测到提示且存在多条 OneKey 凭据
- **THEN** 展示 OneKey 菜单,列出全部 OneKey 凭据与 SSH 凭据

#### Scenario: 无凭据不动作
- **WHEN** 检测到提示且无 OneKey 凭据
- **THEN** 不执行自动发送;SSH 凭据仍出现在 OneKey 菜单中
