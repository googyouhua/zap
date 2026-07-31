## Purpose

统一 OneKey 凭据的持久化层:用 SQLite(`onekey_credentials` 表)存储凭据元数据,用 OS Keychain(`keyring`,service `zap.onekey`)存储 secret,keychain 不可用时 fallback 到 `encrypted_password` 列,并支持 Password 与 Key 两种凭据类型。

## ADDED Requirements

### Requirement: 存储凭据元数据到 SQLite
系统 SHALL 将凭据元数据(label、username、notes、kind、key_path)持久化到 SQLite `onekey_credentials` 表。每条凭据 MUST 以唯一 UUID 作为主键。

#### Scenario: 创建密码型凭据
- **WHEN** 用户保存一条 kind="password"、label="my-server"、username="admin"、secret="s3cret!" 的新凭据
- **THEN** `onekey_credentials` 表插入一行,kind='password'、key_path=null,并设置 created_at/updated_at

#### Scenario: 创建密钥型凭据
- **WHEN** 用户保存一条 kind="key"、label="my-key"、key_path="/home/user/.ssh/id_ed25519"、secret="passphrase123" 的新凭据
- **THEN** `onekey_credentials` 表插入一行,kind='key'、key_path='/home/user/.ssh/id_ed25519'

#### Scenario: 列出全部凭据
- **WHEN** 系统为面板加载凭据
- **THEN** 按 label 升序返回 `onekey_credentials` 表全部行

#### Scenario: 更新凭据的 kind 与 key_path
- **WHEN** 用户把某条凭据从 kind="password" 改为 "key" 并设置 key_path
- **THEN** 该行的 kind 与 key_path 字段被更新,updated_at 刷新

#### Scenario: 删除凭据
- **WHEN** 用户删除一条凭据
- **THEN** 该行从 `onekey_credentials` 表移除,对应 OS keychain 条目一并删除

### Requirement: 在 OS Keychain 存储 secret
系统 SHALL 通过 `keyring` crate 将 secret(密码或口令)存入 OS keychain,service 名为 `zap.onekey`,account key 为 `<credential-uuid>:secret`。OS keychain 不可用时,secret SHALL fallback 到 `encrypted_password` 列。

#### Scenario: 保存新 secret
- **WHEN** 一条凭据以 secret "s3cret!" 创建
- **THEN** keychain 条目 `zap.onekey / <uuid>:secret` 包含 "s3cret!"

#### Scenario: 读取 secret
- **WHEN** 系统需要发送某凭据的 secret
- **THEN** 从 keychain `zap.onekey / <uuid>:secret` 读取,keychain 失败时 fallback 到 `encrypted_password` 列

#### Scenario: 删除 secret
- **WHEN** 一条凭据被删除
- **THEN** 对应 keychain 条目同步删除

### Requirement: 支持带 key_path 的密钥型凭据
系统 SHALL 通过 `kind` 字段支持两种凭据类型:`password`(username + secret)与 `key`(username + key_path + secret/passphrase)。密码型 secret 直接用于 SSH 密码认证;密钥型提供私钥路径与可选口令,用于 SSH 公钥认证。

#### Scenario: 创建密钥型凭据
- **WHEN** 用户创建 kind="key"、key_path="/home/user/.ssh/id_ed25519" 的凭据
- **THEN** 凭据以 kind='key'、key_path 已设为该值保存

#### Scenario: 列表展示所有类型
- **WHEN** 系统列出凭据
- **THEN** 密码型与密钥型凭据都出现在列表中
