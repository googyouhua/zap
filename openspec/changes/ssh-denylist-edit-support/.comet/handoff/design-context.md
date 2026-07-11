# Comet Design Handoff

- Change: ssh-denylist-edit-support
- Phase: design
- Mode: compact
- Context hash: 272f8afc9fbb4212864bdf7c13f39f2a621c1eebd52cec0b4545121eae5d507f

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/ssh-denylist-edit-support/proposal.md

- Source: openspec/changes/ssh-denylist-edit-support/proposal.md
- Lines: 1-28
- SHA256: aebfc8c2305606332d142e888070f46e10c2952135970bb93eb9aa33f567f75d

```md
## Why

SSH denylist 目前只能逐条添加 host，不支持批量操作。用户手动添加多个 host（如 `1@yun.com;2@yun.com;3@yun.com`）时需要反复输入提交，效率低。另外已添加的 host 无法编辑，只能删除后重新添加。

## What Changes

- `handle_denylisted_ssh_editor_event`：支持 `;` 分割批量添加 host
- `WarpifyPageView`：新增 `pending_edit_ssh_host_index` 字段，记录编辑中的条目索引
- `WarpifyPageAction`：新增 `EditDenylistedSshHost(usize)` action
- `handle_action`：处理 EditDenylistedSshHost，回填输入框内容
- 列表渲染：hostname 文本添加 `on_click` 回调，点击后触发编辑
- `render_alternating_color_list_item`：接受可选 `edit_action`，文本可点击

## Capabilities

### New Capabilities

- `ssh-denylist-batch-add`: 支持分号分隔批量添加 host
- `ssh-denylist-edit`: 支持点击已有 hostname 回填编辑框修改

### Modified Capabilities

无

## Impact

- `app/src/settings_view/warpify_page.rs`：新增编辑状态字段、action、事件处理、渲染逻辑
- `app/src/settings_view/settings_page.rs`：修改列表项渲染函数支持可点击文本
```

## openspec/changes/ssh-denylist-edit-support/design.md

- Source: openspec/changes/ssh-denylist-edit-support/design.md
- Lines: 1-37
- SHA256: 56425ef99b7b38922c7b4d8fd389795f40ea2dfe3003b9c276f9465181a5b7bf

```md
## Context

SSH denylist 列表在 `WarpifyPageView` 中渲染，使用 `build_input_list` → `render_alternating_color_list` → `render_alternating_color_list_item` 链路。每个条目仅有关闭按钮（删除），无可点击编辑入口。

输入框 `add_denylisted_ssh_editor` 已存在，提交时直接调用 `denylist_ssh_host` 追加。

## Goals / Non-Goals

**Goals:**
- 输入框支持 `;` 分割批量添加 host
- 点击列表中的 hostname → 回填输入框 → 修改 → Enter 替换原条目

**Non-Goals:**
- 不改输入框交互方式
- 不加额外编辑按钮
- 不改 denylist 底层匹配逻辑

## Decisions

**1. 编辑状态追踪**
使用 `pending_edit_ssh_host_index: Option<usize>` 字段标记当前是否处于编辑模式。`None` = 添加模式，`Some(idx)` = 编辑第 idx 条。

**2. 批量添加 vs 编辑的输入分流**
`handle_denylisted_ssh_editor_event` 中判断：
- 有 pending index → 替换该条目，忽略 `;` 分割（只处理第一个片段后 break）
- 无 pending index → 按 `;` 分割，逐条添加

**3. 点击触发编辑**
列表项渲染时 hostname 文本添加 `on_click` 回调，dispatch `EditDenylistedSshHost(i)`。handle_action 中读取当前值、回填编辑器、记录 pending index。

**4. render_alternating_color_list_item 修改**
添加可选参数 `edit_action: Option<SettingsPageAction>`。当 Some 时，文本包装 `on_click`。默认设置 `None` 保持现有行为不变。

## Risks / Trade-offs

- 编辑模式下 `;` 被忽略（只替换一条），避免误批量覆盖
- 编辑时按 Esc 清空 pending index，回到添加模式
```

## openspec/changes/ssh-denylist-edit-support/tasks.md

- Source: openspec/changes/ssh-denylist-edit-support/tasks.md
- Lines: 1-19
- SHA256: 0319c8e32414871896abd8f5efa2a492a372f1f681e07bac9009e4c4b65dc0f9

```md
## 1. 编辑状态与 Action

- [ ] 1.1 warpify_page.rs: 添加 `pending_edit_ssh_host_index: Option<usize>` 字段
- [ ] 1.2 warpify_page.rs: 添加 `EditDenylistedSshHost(usize)` 到 `WarpifyPageAction`
- [ ] 1.3 warpify_page.rs: `handle_action` 处理 `EditDenylistedSshHost` — 回填编辑框 + 设 index

## 2. ; 分割批量添加 + 编辑替换

- [ ] 2.1 warpify_page.rs: `handle_denylisted_ssh_editor_event` 支持 `;` 分割 + 编辑替换分支

## 3. 列表渲染支持点击编辑

- [ ] 3.1 settings_page.rs: `render_alternating_color_list_item` 接受可选 `edit_action`，文本加 `on_click`
- [ ] 3.2 warpify_page.rs: 渲染 SSH denylist 时传入 `EditDenylistedSshHost` edit_action

## 4. 验证

- [ ] 4.1 `cargo check` 通过
- [ ] 4.2 提交 commit
```

## openspec/changes/ssh-denylist-edit-support/specs/ssh-denylist-batch-add/spec.md

- Source: openspec/changes/ssh-denylist-edit-support/specs/ssh-denylist-batch-add/spec.md
- Lines: 1-16
- SHA256: 1d5835e44512ef2f4a32ea4d68172fd6646b9beb1cd62b156f2e2997330e4fcb

```md
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
```

## openspec/changes/ssh-denylist-edit-support/specs/ssh-denylist-edit/spec.md

- Source: openspec/changes/ssh-denylist-edit-support/specs/ssh-denylist-edit/spec.md
- Lines: 1-20
- SHA256: 4bc93a1652ba6941920b903cdf7f5360bf39cd29af6b915a3ca4665669f3fcd9

```md
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
```

