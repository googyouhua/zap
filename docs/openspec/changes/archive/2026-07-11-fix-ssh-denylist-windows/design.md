## 修复方案

将 `SSHWidget::render` 中的 denylist hosts 部分从 `add_setting(..., &use_ssh_tmux_wrapper, || { ... })` 闭包内移到闭包外，直接加在 SSHWidget 的 `column` 下：

```rust
// 闭包内只保留 tmux 开关 + 描述
add_setting(&mut column, &use_ssh_tmux_wrapper, || {
    [开关] [描述]
});

// denylist 在闭包外，只受 enable_ssh_warpification 控制
if enable_ssh_warpification {
    [denylist input list]
}
```

`add_setting` 检查 `is_supported_on_current_platform()`，`UseSshTmuxWrapper` 不支持 Windows，导致整个闭包被跳过。移出后 denylist 不再受此限制。
