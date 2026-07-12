## 1. 编辑状态与 Action

- [x] 1.1 warpify_page.rs: 添加 `pending_edit_ssh_host_index: Option<usize>` 字段
- [x] 1.2 warpify_page.rs: 添加 `EditDenylistedSshHost(usize)` 到 `WarpifyPageAction`
- [x] 1.3 warpify_page.rs: `handle_action` 处理 `EditDenylistedSshHost` — 回填编辑框 + 设 index

## 2. ; 分割批量添加 + 编辑替换

- [x] 2.1 warpify_page.rs: `handle_denylisted_ssh_editor_event` 支持 `;` 分割 + 编辑替换分支

## 3. 列表渲染支持点击编辑

- [x] 3.1 warpify_page.rs: 新增 `render_ssh_denylist_item` 函数，使用 `Hoverable` 使 hostname 可点击 dispatch `EditDenylistedSshHost`
- [x] 3.2 warpify_page.rs: SSH denylist 改用自定义渲染 loop，不走 `build_input_list`

## 4. 验证

- [x] 4.1 `cargo check` 通过
- [x] 4.2 提交 commit
