# Verification Report: onekey-credential-system

- **Change**: `onekey-credential-system`(36 tasks / 5 capability specs / 73 files)
- **Branch**: `feature/20260731/onekey-credential-system`
- **Base-ref**: `2951bfb7` → **HEAD**: `7d32e651d67e5a96896f19e5d6dbe85a5c8a6535`(26 commits)
- **验证人/时间**: verify 阶段 full verification — 2026-08-01
- **方法**: 只读代码核验 + `cargo check -p warp_onekey -p persistence -p warp_ssh_manager` 与 `cargo check -p warp`(均绿,仅 1 条与本 change 无关的既有 `unused_variables` warning)。未跑全量测试(earlyoom 限制)。

---

## Summary Scorecard

| 维度 | 状态 |
|------|------|
| Completeness | 36/36 tasks `[x]`(已逐条读取核对);5 个 capability spec 共 25 条 Requirement 全部有实现证据;proposal 目标达成 |
| Correctness | 25/25 Requirement 有代码支撑,其中 4 条存在实现与 spec 意图偏离(见 WARNING);重点场景(存储/查找、面板搜索/发送、su_root Password-only、SSH/SFTP find_by_id、跨视图 notifier)均已覆盖 |
| Coherence | 实现整体对齐 Design Doc 与参考分支(`crates/onekey` 全量与参考分支 diff 为空);openspec `design.md` 存在 2 处未同步(迁移方式、快捷键),属文档级过期,不影响代码 |

**结论**:0 CRITICAL、5 WARNING、6 SUGGESTION。**Ready with improvements**(见最终评估)。

---

## Completeness(完整性)

### tasks.md 核对

36 个 checkbox 全部为 `[x]`,无未完成任务。其中 Task 9.3「手动验证」标注"留待人工",由 verify 阶段代码核验部分代偿。任务勾选与实际实现一致(逐项抽样核实,见下方各 spec 证据)。

### proposal 目标

- 新建 `crates/onekey` 统一凭据 crate ✓
- 统一数据模型(Password/Key + key_path)✓
- 终端搜索面板 + hotkey ✓
- 发送引擎(两种 SendMode + Zeroizing)✓
- PTY auto-send + `prompt_trigger_rules` ✓
- 设置页 OneKeyPage(CRUD + 关键词)✓
- SSH 面板数据源切换 + 变更自动刷新 ✓
- 删除旧 SSH OneKey 系统(BREAKING)✓(残留仅注释,见 Coherence)
- Feature flag `onekey_input` 默认启用 ✓

---

## Correctness — Requirement → 实现证据映射

### Spec 1: credential-store(3 条 Requirement)

| Requirement | 实现证据 | 状态 |
|---|---|---|
| 存储凭据元数据到 SQLite | `crates/onekey/src/repository.rs:104-136`(`create`:uuid 主键 + label/username/notes/kind/key_path + created_at/updated_at);migration `crates/persistence/migrations/2026-07-31-000000_onekey_credentials_and_drop_ssh_onekey/up.sql:1-18`;`db.rs:61-87` 防御性建表 | ✓ |
| 在 OS Keychain 存储 secret | `crates/onekey/src/secret_store.rs:9-32`(service `zap.onekey`);`repository.rs:44-64`(`resolve_password`/`store_password`:keychain 优先、fallback `encrypted_password` 列) | ✓(注:account 实际为 `{id}:password`,spec 文本写 `<uuid>:secret`,见 SUGGESTION S3) |
| 支持带 key_path 的密钥型凭据 | `types.rs:3-24`(`OneKeyKind::Password/Key`);`repository.rs:123-136`(`kind`+`key_path` 落库);设置页/SSH overlay kind 选择器 | ✓ |

**场景覆盖**:创建密码型(仓库测试 `repository_tests.rs:61-75`)、创建密钥型(`repository_tests.rs:124-151`)、列出全部按 label 升序(`repository.rs:66-85` + 测试 `repository_tests.rs:104-121`)、更新 kind/key_path(`repository_tests.rs:154-173`)、删除(`repository_tests.rs:92-101`)、keychain 读写删除(`secret_store.rs`)。全部覆盖。

### Spec 2: credential-panel(4 条 Requirement)

| Requirement | 实现证据 | 状态 |
|---|---|---|
| 快捷键唤起凭据搜索面板 | `app/src/terminal/view/init.rs:579-586`(`cmd_or_ctrl_shift("u")` → `ToggleOneKeyPanel`);`view.rs:23981-23990`(toggle + focus);`view.rs:25093-25098`(Align 居中渲染);已核实无 ctrl/cmd+shift+u 既有绑定冲突 | ✓(注:spec 场景示例仍写 `ctrl+shift+k`,见 SUGGESTION S2) |
| 模糊搜索凭据 | `app/src/search/onekey/data_source.rs:24-57`(label/username 大小写不敏感 `fuzzy_match`);测试 `data_source_tests.rs` 全场景 | ✓ |
| 键盘导航与选择凭据 | `SearchBar`/`SearchBarState` 复用(`view.rs:92-125`),Up/Down/Enter 由 SearchBar 导航,Enter → `ResultAccepted` → `handle_result_selected`(`view.rs:176-200`) | ✓ |
| 选择凭据后展示发送模式选项 | `view.rs:397-433`(`render_send_mode_selection`,两按钮);`view.rs:48-53`(`PanelMode::SendModeSelection`);事件 `OneKeyPanelEvent::ItemSelected { credential, mode }`(`view.rs:78-85`) | ✓ |

**场景覆盖**:快捷键打开(代码路径完整)、Escape 关闭(`SearchBarEvent::Close` → `view.rs:174`)、点击外部关闭(`Dismiss::new` + `on_dismiss` → `view.rs:484-507`)、按 label/username 搜索(测试覆盖)、无匹配("No results found." i18n,见 SUGGESTION)、导航选择(SearchBar 复用)。「无匹配」文案与 spec 的 "No matching credentials" 有差异但意图一致。

### Spec 3: credential-send(5 条 Requirement)

| Requirement | 实现证据 | 状态 |
|---|---|---|
| 通过面板仅发送密码 | `app/src/terminal/onekey_sender.rs:14-19`(PasswordOnly 写 `password\n`) | ⚠ 见 W1:PasswordOnly 分支未清空当前行,与 spec 场景"终端当前行被清空"及 design doc Decision 4 描述不符(参考分支同实现) |
| 通过面板先用户名再密码 | `onekey_sender.rs:20-32`(`clear_line_editor_and_write_to_pty` 写 `username\n` → `Timer::after(150ms)` 写 `password\n`) | ✓ |
| 检测到密码提示时自动发送 | `view.rs:7460-7497`(`on_password_prompt_detected`:list_rules → classify_prompt → 恰好 1 条 Password 凭据 → send) | ✓ |
| 检测到用户名提示时自动发送 | `classify_prompt`(`prompt_detection.rs:3-11`)能分类 Username;但生产路径预过滤只放行 password/passphrase 提示 | ⚠ 见 W4:username 提示无法通过滑动窗口预过滤,自动发送用户名+密码生产路径不可达 |
| 敏感数据使用 Zeroizing | `types.rs:32`(`password: Zeroizing<String>`);`onekey_sender.rs:25`(Timer 闭包内 clone 后发送);`view.rs:15870-15877`(`fill_onekey_secret` 亦用 Zeroizing) | ✓ |

**场景覆盖**:仅密码/用户名+密码的 150ms 间隔(代码明确)、自动发送密码(路径完整)、自动发送用户名+密码(W4 不可达)、发送后清零(Zeroizing 全链路)。`onekey_sender_tests.rs` 仅覆盖字符串格式/模式分派,未测真实 PTY 写入(受 ViewContext 依赖限制,可接受)。

### Spec 4: credential-management(6 条 Requirement)

| Requirement | 实现证据 | 状态 |
|---|---|---|
| 设置页列出全部已保存凭据 | `app/src/settings_view/onekey_page.rs:480-600`(列表行:label + username + Edit/Delete) | ⚠ 见 S4:列表行未显示 kind 标识(spec 要求类型标识) |
| 新增凭据 | `onekey_page.rs:767-788`(AddForm) + `805-854`(SaveForm → `warp_onekey::create`) | ⚠ 见 W2:无空 label/password 校验 |
| 编辑既有凭据 | `onekey_page.rs:789-800`(EditForm) + `827-845`(update) | ✓(改密码会重写 keychain:`repository.rs:157`) |
| 删除凭据 | `onekey_page.rs:855-900`(AlertDialogWithCallbacks 确认 → `warp_onekey::delete`) | ✓(含取消确认:`ModalButton::for_app("Cancel")`) |
| 管理触发关键词 | `onekey_page.rs:227-480`(两组关键词 + 增删/重置);`930-961`(add/remove/reset action) | ⚠ 见 S6:Reset 为分组级 |
| 凭据变更后自动刷新视图 | `onekey_notifier.rs:1-21`(SingletonEntity);`onekey_page.rs:153-161`、`server_view.rs:299-303` 订阅 reload;写入端 emit(`onekey_page.rs:848-850`、`server_view.rs:1063-1065,1094-1096`);`repository.rs:19-27` 版本号 | ✓ |

**场景覆盖**:查看列表、新增(缺 W2 场景未实现)、编辑 label/密码、删除确认/取消、关键词增删、Reset(S6)、增删改后跨视图自动刷新(notifier 全链路,SSH overlay 与设置页互刷)。「删除后列表即时更新」由 `refresh_list` + notifier 覆盖。

### Spec 5: auto-fill-trigger(7 条 Requirement)

| Requirement | 实现证据 | 状态 |
|---|---|---|
| 首次运行时填入默认触发关键词 | `onekey_page.rs:1006-1014`(`load_or_init_rules`:空则 `reset_rules_to_defaults`);默认关键词 `types.rs:59-61` | ⚠ 见 W3:仅设置页加载时 seed,`list_rules()` 与 auto-send 检测路径不 seed |
| 新增触发关键词 | `repository.rs:183-201`(`add_rule`);DB `UNIQUE` 约束(keyword);`onekey_page.rs:937-947` | ✓(重复关键词由 UNIQUE 约束拒绝 → `report_if_error` 记日志) |
| 删除触发关键词 | `repository.rs:203-209`(`remove_rule`);`onekey_page.rs:952-956` | ✓ |
| 将触发关键词重置为默认 | `repository.rs:236-261`(`reset_rules_to_defaults` 全量) + `211-234`(`reset_rules_for_mode` 分组);UI 走分组 Reset | ✓(语义见 S6) |
| 对 PTY 输出按触发规则分类 | `prompt_detection.rs:3-11`(`classify_prompt`:大小写不敏感 contains);单测 5 个场景 | ✓ |
| 在 PTY 输出流中持续分类提示 | `view.rs:7422-7454`(滑动窗口 stream)+ `7439` 预过滤 + `7470-7477` classify | ⚠ 见 W4:预过滤限制 |
| 恰好一条凭据时自动发送 | `view.rs:7470-7496`(单条 Password 凭据自动发送;0/多条回落 `show_onekey_prompt_menu`) | ✓ |

**场景覆盖**:首次初始化(W3)、保留用户修改(`list_rules` 返回既有规则)、新增两类关键词、拒绝重复(UNIQUE + log)、删除、重置(S6)、检测密码/用户名提示(classify 单测)、无关键词匹配(classify None)、单条自动发送、多条回落菜单(`load_prompt_menu_candidates` 同时列 OneKey 与 SSH 凭据:`view.rs:15580-15642`)、无凭据不动作。用户名提示自动发送受 W4 影响。

---

## Coherence(一致性)

### 与 Design Doc(`docs/superpowers/specs/2026-07-31-onekey-credential-system-design.md`)对比

| Design Doc 决策 | 实现 | 一致 |
|---|---|---|
| D1 统一数据模型(Password/Key + key_path + Zeroizing) | `types.rs` 完全一致 | ✓ |
| D2 干净迁移(建表 + drop 旧表 + ssh_servers 外键重建) | migration up.sql 完全一致;不复制 legacy quick_credentials 迁移 | ✓ |
| D3 面板复用 SearchBar/SearchMixer + 快捷键 `cmd_or_ctrl_shift("u")` | `app/src/search/onekey/`;init.rs:585 | ✓ |
| D4 发送引擎 + auto-send 完整接入生产(classify_prompt 从死代码改生产) | `onekey_sender.rs` + `on_password_prompt_detected`(参考分支 classify_prompt 确实仅测试引用,此处已接入生产,符合改进意图) | ✓ |
| D5 su_root 菜单 Password-only 过滤 | `view.rs:15957-15974`(find_all → filter kind==Password) | ✓ |
| D6 SSH 面板数据源切换 + `find_by_id` 认证 | `server_view.rs` 全换 `warp_onekey::*`;`workspace/view.rs:5449-5468`、`sftp_manager/sftp_ops.rs:104-110` | ✓ |
| D7 变更自动刷新(版本号 + notifier) | `repository.rs:19-27` + `onekey_notifier.rs` | ✓(版本号无消费点,见 S5) |

### 与 openspec `design.md` 的矛盾(文档级)

- **C1(迁移方式)**:openspec `design.md:44` 写明"表结构通过 CREATE TABLE IF NOT EXISTS + ensure_columns 在 db.rs 中维护,**不新增 Diesel migration 目录**";实际新增了 migration(`crates/persistence/migrations/2026-07-31-000000_...`),与 Design Doc D2 一致。openspec design.md 过期,未随 Task 8.2 更新(实现同时保留 db.rs 防御层,代码与两份文档各自兼容,但 design.md 描述与事实不符)。
- **C2(快捷键)**:openspec `proposal.md:9`、`design.md` Open Questions 与 `credential-panel/spec.md:11` 仍写 `ctrl+shift+k`;实际为 `cmd_or_ctrl_shift("u")`(init.rs:585)。Design Doc D3 与 tasks.md 3.4 已更新,其余文档未同步。

### 旧系统删除残留

`SshOneKeyCredential`、`OneKeyCredentialKind`、`SecretKind::OneKeyPassword`、`SyncOneKeyCredential`、`load_saved_ssh_credentials`、`app/src/ssh_manager/onekey.rs` 均已删除。全局 grep 仅剩注释提及旧表名(`crates/persistence/src/model.rs:1457`、`crates/warp_ssh_manager/src/repository.rs:461-466` 迁移回放注释),无类型/API 引用残留。`ssh_onekey_credentials` 表由 migration drop。

### 代码模式一致性

- `crates/onekey` 与参考分支全量 diff 为空(存储层 1:1 对齐)。
- 错误处理:统一 `anyhow::Result` + `log::warn/error`;UI 侧 `report_if_error!`。
- 文件命名/目录:`app/src/search/onekey/`、`app/src/terminal/prompt_detection.rs`、`onekey_notifier.rs`、`onekey_page.rs` 均符合既有模式。
- Feature flag:`warp_features` 中 `OneKeyInput`(DOGFOOD)与既有 `OneKeyPrompt` 并存;app 内 `#[cfg(feature = "onekey_input")]` + 运行时 `FeatureFlag::OneKeyInput.is_enabled()` 双开关,与 Design Doc D4 一致。

---

## Issues by Priority

### CRITICAL

无。

### WARNING

1. **W1 — credential-send「仅发送密码」未清空当前行**
   `app/src/terminal/onekey_sender.rs:14-19`:PasswordOnly 分支直接 `write_to_pty`,未调用 `clear_line_editor_and_write_to_pty`;spec 场景「终端当前行被清空,然后 password\n 写入 PTY」与 Design Doc D4 描述「清空当前行 → 写 secret\n」均不符。参考分支实现完全一致(仅 UsernameThenPassword 清行),行为在常见场景(提示时行内无输入)可用。
   **建议**:在 PasswordOnly 分支同样先清行以严格满足 spec,或修订 spec/design 文本为「PasswordOnly 直接写,UsernameThenPassword 清行」。至少二选一消除文档与代码的偏差。

2. **W2 — 设置页「缺少 label/密码」校验未实现**
   `app/src/settings_view/onekey_page.rs:805-854`(SaveForm)对空 label、空 password 无任何校验,直接 `warp_onekey::create/update` 落库空值;spec 场景「缺少 label → 'Label is required'」「缺少密码 → 'Password is required'」未实现。SSH overlay 仅校验 label(`server_view.rs:987-994`),不校验密码。tasks.md 7.2 声称「校验 label/password 必填」亦未落地。
   **建议**:在 SaveForm(及可选 SSH overlay)增加 label 非空、密码非空校验,空时显示错误并中止保存。

3. **W3 — 默认触发关键词仅在设置页加载时 seed**
   `onekey_page.rs:1006-1014`:`load_or_init_rules` 在设置页首次打开且表空时 `reset_rules_to_defaults`;但 `repository.rs:164-181`(`list_rules`)与 auto-send 检测路径(`view.rs:7471`)不做 seed。全新安装未访问设置页时,`list_rules()` 为空 → classify 返回 None → auto-send 永不触发(需先打开设置页)。spec「首次运行时填入默认触发关键词」仅在设置页路径满足。
   **建议**:在 `list_rules()` 或 `on_password_prompt_detected` 检测到空规则时自动 seed 默认关键词;或在 spec 中明确该行为依赖首次访问设置页。

4. **W4 — 用户名提示自动发送生产路径不可达**
   `view.rs:7439` 滑动窗口预过滤 `bytes_look_like_password_prompt`(`app/src/ssh_manager/password_prompt.rs:4`,正则 `(?im)(password|passphrase)[^\n]*:\s*$`)只放行含 password/passphrase 的提示行;`login:`/`username:` 等用户名提示永远无法 yield,`classify_prompt` 的 Username 分类因此在生产不可达。credential-send「检测到用户名提示时自动发送」与 auto-fill-trigger「检测到用户名提示」场景未覆盖。
   **建议**:在预过滤正则中放行用户名类关键词(或直接让 classify 在窗口上运行),使 UsernameThenPassword 自动发送可达;或修订 spec 明确仅密码提示触发自动发送。

### SUGGESTION

1. **S1 — openspec design.md 迁移描述过期**
   `docs/openspec/changes/onekey-credential-system/design.md:44` 声称「不新增 Diesel migration 目录」,与实际新增 migration 不符(实现与 Design Doc D2 一致)。建议同步 design.md 至「迁移 + db.rs 防御层」双轨描述。

2. **S2 — 快捷键文档未同步**
   `proposal.md:9`、`design.md`(Open Questions)、`credential-panel/spec.md:11` 仍写 `ctrl+shift+k`;实际为 `cmd_or_ctrl_shift("u")`(init.rs:585,已核实无绑定冲突)。建议同步三处文档。

3. **S3 — keychain account key 文本偏差**
   `credential-store/spec.md:31` 写 account 为 `<credential-uuid>:secret`,实现(`secret_store.rs:12`)与参考分支均为 `{id}:password`,功能不受影响。建议修正 spec 文本。

4. **S4 — 设置页列表行缺 kind 标识**
   `onekey_page.rs:520-567` 列表行仅显示 label + username;spec「每条显示 label、username 预览与类型(kind)标识」未含类型标识。建议在行内追加 Password/Key 徽标。

5. **S5 — `credentials_version()` 无消费点**
   `repository.rs:19-27` 维护 `CREDENTIALS_VERSION` 并导出,但全仓无读取方(notifier 是实际刷新机制)。建议标注为预留/未来轮询用,或评估删除以精简 API。

6. **S6 — 关键词 Reset 为分组级**
   `onekey_page.rs:957-961` Reset 走 `reset_rules_for_mode`(单组);spec「重置为默认」描述为单一 Reset 全量替换。功能上两组各有 Reset 可达成目标。建议补充一个「Reset All」或修订 spec 描述。

---

## 裁决记录(用户决策,2026-08-01 verify 阶段)

用户裁决:**W1、W2、W4 修复;W3 保持(接受并同步文档)**。

- **W1(PasswordOnly 不清行)** → 修复:PasswordOnly 分支也先清行(与 UsernameThenPassword 对齐)。
- **W2(设置页无必填校验)** → 修复:SaveForm 增加 label 非空、password 非空校验。
- **W4(用户名提示自动发送不可达)** → 修复:滑动窗口预过滤放行用户名类关键词,使 UsernameThenPassword 自动发送可达。
- **W3(默认关键词仅设置页 seed)** → 接受:与参考分支行为一致,不修。接受理由:首次打开设置页即 seed,行为可预期;影响范围:新装用户未开设置页前 auto-send 不触发。同步 spec 文本描述该行为。

其余 S1/S2/S3/S4/S5/S6(文档过期与风格建议)按上述裁决一并处理(S1/S2 文档同步)。

**处理方式**:返回 build 阶段,按 subagent-driven-development 补实现任务修复 W1/W2/W4,同步文档(S1/S2/W3 文本)。

## 最终评估

**0 CRITICAL;5 WARNING;6 SUGGESTION。用户裁决 W1/W2/W4 修复、W3 接受并同步文档后,进入 build 修复轮。**

- 实现质量高:存储层与权威参考分支 1:1 对齐;auto-send 相对参考的「死代码 → 生产接入」改进落地;旧系统删除干净;notifier 跨视图刷新、su_root Password-only 过滤、SSH/SFTP `find_by_id` 认证均正确实现。`cargo check` 全绿。
- W1–W4 属 spec/代码行为偏差(其中 W1/W3/W4 与权威参考一致,主要矛盾在 spec/design 文本未同步,仅 W2 是真实功能缺失)。按「降级原则」均未升为 CRITICAL。
- 归档前最低限度建议:更新 openspec `design.md` 迁移描述与快捷键两处过期文本(S1/S2);后续迭代补齐 W2 校验。
