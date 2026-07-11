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
