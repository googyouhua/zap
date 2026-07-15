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
