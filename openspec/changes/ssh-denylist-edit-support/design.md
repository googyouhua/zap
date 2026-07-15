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
