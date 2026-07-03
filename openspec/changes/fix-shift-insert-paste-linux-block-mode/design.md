## 问题

Shift+Insert 粘贴键绑定被错误地放在 `if ChannelState::channel() == Channel::Integration { }` 守卫内（`app/src/terminal/view/init.rs:238-296`），导致生产构建（Stable/Preview/Dogfood）中该绑定从未被注册，无法工作。

此 feature 的原始提交 `feat(terminal): add shift-insert paste keybinding` 将绑定放入了 Integration 测试专属区块，导致所有非测试频道均不生效。

## 修复方案

将 `FixedBinding::new("shift-insert", TerminalAction::Paste, ...)` 从 `if ChannelState::channel() == Channel::Integration` 块中移出，放到其他 `shift-*` 固定绑定旁（第 330 行后）。

**改动文件**: `app/src/terminal/view/init.rs`
- 删除第 276-280 行（Integration 块内的 shift-insert 绑定）
- 在第 330 行后插入 shift-insert 绑定

## 回归风险

低。仅是移动绑定注册位置，不影响 Paste 动作逻辑本身。
