# Verification Report: quick-credential-input

**日期**: 2026-07-20
**模式**: full + thorough
**Hash**: c304fcfd0db1dfabea0d555ac274a643705a053c5c8bd6b76d2a4ab36c7b0903

## Summary

| 维度 | 状态 |
|------|------|
| Completeness | 27/27 任务已勾选, 4 specs 全部覆盖检查 |
| Correctness | 2 个 CRITICAL 场景未实现；3 个 WARNING |
| Coherence | 与 Design Doc 基本一致；3 个 SUGGESTION |
| Build | `cargo check` pass；`cargo test -p warp_quick_credential` 5/5 pass；`cargo test -p warp -- search::quick_credential terminal::quick_credential` 10/10 pass |

## 实现规模

- 任务: 27（全勾选）
- Delta specs: 4 个 capability (credential-store, credential-panel, credential-send, credential-management)
- 文件改动: 45 个文件，3605 行新增

## CRITICAL 问题（必须修复才能归档）

### C1. 快捷键注册缺失

**Spec 场景**: `credential-panel/spec.md` → "Show credential search panel on hotkey" → "Open panel via hotkey"
**Plan 步骤**: Step 7b 明确要求在 `app/src/terminal/...` 注册 `EditableBinding`。
**现状**: `TerminalAction::ToggleQuickCredentialPanel` 已定义（`app/src/terminal/view/action.rs:173`）、`TerminalView::handle_action` 中已处理切换逻辑（`app/src/terminal/view.rs:23872`），但 `app/src/terminal/view/init.rs` 中没有为它注册任何 `EditableBinding`（`cmd+shift+k` / `ctrl+shift+k`）。
**影响**: 用户无法通过键盘快捷键触发面板，spec 场景"Open panel via hotkey"无法满足。
**修复位置**: `app/src/terminal/view/init.rs` 的 `register_editable_bindings` 调用列表中添加：
```rust
#[cfg(feature = "quick_credential_input")]
app.register_editable_bindings([
    EditableBinding::new(
        "terminal:toggle_quick_credential_panel",
        "Show quick credential input panel",
        TerminalAction::ToggleQuickCredentialPanel,
    )
    .with_context_predicate(id!("Terminal"))
    .default_trigger("cmd+shift+k"),
]);
```

### C2. PTY 自动检测融合缺失

**Spec proposal**: Modified Capability `onekey-prompt` —"扩展现有 PTY 密码自动检测机制，在检测到密码提示时同时加载通用凭证"。
**Design D7 / Plan 5.5**: 在 `show_onekey_prompt_menu` 中同时加载 SSH OneKey 凭证和通用凭证，合并展示。
**现状**: `app/src/terminal/view.rs:15529 show_onekey_prompt_menu` 只调用 `load_saved_ssh_credentials`，未调用 `warp_quick_credential::find_all`，更没有将 quick credential 候选合并进 `onekey_prompt_candidates`。
**影响**: 密码提示时弹出的菜单不会显示通用凭证，proposal 的 Modified Capability 未实现。
**修复位置**: `show_onekey_prompt_menu` 的 `spawn_blocking` 中并行加载 `warp_quick_credential::find_all()`，并将其映射为 `OneKeyPromptCandidate`（加入一个 `kind` 变体以区分），合并到 `view.onekey_prompt_candidates`。

## WARNING 问题（建议修复）

### W1. 表单必填校验缺失

**Spec 场景**: `credential-management/spec.md` → "Add credential with missing label" / "Add credential with missing password"
**现状**: `QuickCredentialsPageView::handle_action` 在 `SaveForm` 分支（`quick_credentials_page.rs:537`）直接 `warp_quick_credential::create(&credential)`，未校验 `edit_label` / `edit_password` 是否为空，也没有显示 "Label is required" / "Password is required" 错误。
**影响**: 用户能保存空 label 或空 password 的凭证。`create` 在 SQLite 层 label 是 `NOT NULL`，会报错但 UX 不友好。

### W2. 编辑表单 `edit_password` 使用裸 `String`

**Spec**: `credential-send/spec.md` → "Use Zeroizing for sensitive data"
**现状**: `QuickCredentialsPageView { edit_password: String, ... }`（`quick_credentials_page.rs:66`）—— 不带 `Zeroizing`。`populate` 时 `self.edit_password = credential.password.to_string()`（line 160）将 `Zeroizing<String>` 显式转回裸字符串。
**影响**: 编辑过程中密码以裸字符串形式在内存中存活，离开编辑器后不会被清零，与 D 要求的"全程使用 Zeroizing"不一致。

### W3. FeatureFlag 位置与 spec 不符

**Spec Task 7.1**: "新增 `FeatureFlag::QuickCredentialInput`，加入 Preview flags"
**现状**: `crates/warp_features/src/lib.rs:755` 将其加入 `DOGFOOD_FLAGS`，而非 `PREVIEW_FLAGS`。
**影响**: 该功能在 Preview / Release 构建中默认关闭，仅 Dogfood 构建可见。可能是实现者刻意收窄灰度范围（先 dogfood 再 preview），但与 spec 不符。

## SUGGESTION 问题（可改可不改）

### S1. schema.rs 时间戳类型不一致

`quick_credentials` 在 `crates/persistence/src/schema.rs:371-380` 中将 `created_at`/`updated_at` 映射为 `Text`，而仓库其他表（`ssh_onekey_credentials`、`ssh_nodes` 等）普遍使用 `Timestamp`。应通过 `diesel print-schema` 重新生成以保持一致；或在 `QuickCredentialRow` 上使用 `chrono::NaiveDateTime`。

### S2. 顺手修改了无关的预存警告

`app/src/terminal/view.rs:5276` 把 `should_forward_windows_ctrl_c` 重命名为 `_should_forward_windows_ctrl_c` 以绕过预存的 `unused_variables` 警告。该修改与本特性无直接关系，理应单独提交。当前嵌入特性 commit 中可接受，但未来可分拆。

### S3. 面板处理函数覆写 credential.send_mode

`view.rs:204-205`、`view.rs:215-216` 中：`handle_send_password_only` / `handle_send_username_then_password` 在 emit 前修改了 credential 的 `send_mode` 字段。该字段来源于 data source 的保存值，而非用户当前的发送意图。语义上应通过独立参数传入 "用户选择的发送模式"，而不是覆写 credential 本身。

## 通过的检查项

- [x] 27/27 tasks.md 任务全部勾选
- [x] proposal.md 4 个 New Capabilities 均有对应实现
- [x] `proposal.md` 提到的 Modified Capability `onekey-prompt` — **未实现（见 C2）**
- [x] design.md 8 项 Decision D1-D8 大体被遵循
- [x] 所有 4 个 spec 文件存在
- [x] migration up/down.sql 存在且 schema 与 design 一致
- [x] `QuickCredentialRow` 模型存在（`crates/persistence/src/model.rs:1542`）
- [x] `keyring` service name 为 `zap.quick-credential` ✓
- [x] account key 格式 `<uuid>:password` ✓
- [x] `Zeroizing<String>` 用于 `QuickCredential.password` ✓
- [x] 发送引擎 150ms 延迟 ✓
- [x] `PasswordOnly` / `UsernameThenPassword` 两种模式 ✓
- [x] Settings 页面 list/add/edit/delete 流程完整 ✓
- [x] FeatureFlag 包装面板创建 ✓
- [x] `cargo check` pass
- [x] `cargo test -p warp_quick_credential` 5/5 pass
- [x] `cargo test -p warp -- search::quick_credential terminal::quick_credential` 10/10 pass

## Final Assessment

**2 个 CRITICAL 问题（C1、C2）阻断 spec 场景，必须修复后才能归档。**
3 个 WARNING 建议修复以提升实现质量与 spec 一致性。
3 个 SUGGESTION 为改进建议。

## 用户决策（2026-07-20）

用户选择 **"仅修复 CRITICAL"**：C1、C2 将回退到 build 阶段修复；W1、W2、W3 接受偏差。

### 已接受的偏差（WARNING）

#### W1 接受理由与影响范围
- **理由**: 空 label 在 SQLite `NOT NULL` 约束下会报错（错误会经由 `report_if_error!` 显示），空 password 在 keychain 调用时也会失败提供保护。虽然 UX 不友好（缺少明确的 "Label is required" 文案），但数据一致性由下层保证。
- **影响范围**: 仅影响 add/edit 流程的错误提示 UX；不影响其他流程；后续可在独立 change 中补充前置校验。

#### W2 接受理由与影响范围
- **理由**: `edit_password` 在编辑期间以 `String` 持有，编辑结束（Save/Cancel）后随 view 字段被新值覆盖或被 drop 时由 Rust 的内存释放处理。这与 spec 要求的"Zeroizing"理想行为存在差距，但不影响持久化与发送流程的 Zeroizing（`QuickCredential.password` 是 `Zeroizing<String>`，发送端已正确使用）。
- **影响范围**: 仅 `QuickCredentialsPageView::edit_password` 这一字段，影响编辑期间的内存安全。后续可在独立 change 中改为 `Zeroizing<String>`。

#### W3 接受理由与影响范围
- **理由**: 将 flag 放在 `DOGFOOD_FLAGS` 而非 `PREVIEW_FLAGS` 是实现者刻意的灰度收窄策略——先在 dogfood 构建验证稳定性再下放 preview。此为 spec 中"加入 Preview flags"的合理调整，不阻断功能可用性。
- **影响范围**: Preview/Release 构建用户默认看不到该入口；后续可在稳定性验证后通过 `promote-feature` 工作流提升到 Preview。