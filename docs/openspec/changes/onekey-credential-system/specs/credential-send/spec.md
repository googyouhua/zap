## Purpose

把 OneKey 凭据发送到终端 PTY 的发送引擎:支持从面板手动选择发送模式,也支持由 PTY 密码提示触发自动发送,并在发送前清空当前输入行。

## ADDED Requirements

### Requirement: 通过面板仅发送密码
系统 SHALL 在用户于面板选择"仅发送密码"时,清空终端当前行,然后把密码与换行写入 PTY。

#### Scenario: 发送密码到 PTY
- **WHEN** 用户选择凭据 "my-server" 并选择"仅发送密码"
- **THEN** 终端当前行被清空,然后 `password\n` 写入 PTY

### Requirement: 通过面板先用户名再密码
系统 SHALL 在用户选择"先用户名再密码"时,先把用户名与换行写入 PTY,等待 ~150ms,再把密码与换行写入 PTY。

#### Scenario: 发送用户名再密码到 PTY
- **WHEN** 用户选择凭据 "my-app"(username="admin")并选择"先用户名再密码"
- **THEN** 终端当前行被清空,`admin\n` 写入,~150ms 后 `password\n` 写入

### Requirement: 检测到密码提示时自动发送
当提示检测器将终端输出分类为密码型提示且恰好存在一条 OneKey 凭据时,系统 SHALL 只自动发送该凭据的密码。

#### Scenario: 自动发送密码
- **WHEN** 终端输出匹配 Password 关键词且恰好存在一条 OneKey 凭据
- **THEN** 该凭据的密码与换行写入 PTY

### Requirement: 检测到用户名提示时自动发送
当提示检测器将终端输出分类为用户名型提示且恰好存在一条 OneKey 凭据时,系统 SHALL 先发送用户名,等待 ~150ms,再发送密码。

#### Scenario: 自动发送用户名再密码
- **WHEN** 终端输出匹配 Username 关键词且恰好存在一条 OneKey 凭据
- **THEN** 先写入用户名,~150ms 后写入密码

### Requirement: 敏感数据使用 Zeroizing
系统 SHALL 将所有密码用 `Zeroizing<String>` 包裹,确保内存中的密码在引用全部释放后被清零。

#### Scenario: 发送后密码被清零
- **WHEN** 凭据的密码已写入 PTY 且所有引用被释放
- **THEN** 先前持有密码的内存被清零
