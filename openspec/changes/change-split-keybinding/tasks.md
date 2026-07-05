## 1. 修改快捷键

- [x] 1.1 将 `app/src/terminal/view/init.rs` 中 `SplitBlock` 的 FixedBinding 键名从 `"ctrl-shift-\\"` 改为 `"ctrl-b"`
- [x] 1.2 `cargo check` 通过
- [x] 1.3 将绑定从 `Channel::Integration` 条件块内移到外部的 `register_fixed_bindings` 块，确保生产环境生效
