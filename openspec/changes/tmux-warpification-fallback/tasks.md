## 1. terminal_manager.rs 还原原始条件

- [x] 1.1 在 `terminal_manager.rs` 第 620-626 行恢复 `&& !use_ssh_tmux_wrapper` 守卫（原始代码已存在，无需修改）
- [x] 1.2 `cargo check` 编译通过

## 2. view.rs 三个失败事件改为触发 shell integration

- [x] 2.1 `TmuxNotInstalled` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入检测到的 shell 类型
- [x] 2.2 `UnsupportedTmuxVersion` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入检测到的 shell 类型
- [x] 2.3 `TmuxInstallFailed` 处理分支改为调用 `trigger_subshell_bootstrap()` 并传入 `get_shell_type()` 结果
- [x] 2.4 确认移除了对应分支中的 installer 弹框和 error block 逻辑
- [x] 2.5 `cargo check` 编译通过

## 3. bootstrap.rs 移除初始化脚本重定向

- [x] 3.1 `init_subshell_command` 中移除 `>/dev/null 2>&1`（原始代码不包含此重定向，无需修改）
- [x] 3.2 `cargo check` 编译通过

## 4. 验证

- [ ] 4.1 构建并测试：SSH 到没有 tmux 的主机，确认自动注入 shell integration
- [ ] 4.2 确认 `use_ssh_tmux_wrapper = false` 时嵌套 SSH 不追踪
- [ ] 4.3 确认删除的 `use` 没有引起编译警告
