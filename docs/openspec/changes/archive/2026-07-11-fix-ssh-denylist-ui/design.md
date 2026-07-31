## 方案

`warpify_page.rs:803`：
```rust
// 修复前
if enable_ssh_warpification && should_prompt_ssh_tmux_wrapper {

// 修复后
if enable_ssh_warpification {
```
