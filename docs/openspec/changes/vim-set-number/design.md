# design.md

## 设计方案

在代码编辑器的 `ex_command()` 中实现一个简易的 vim 命令行模式：

1. 创建 `VimCommandLine` 状态：`:` 键按下时进入命令行模式，显示一个输入提示
2. 用户输入 `set number` / `set nonumber` 等命令后回车执行
3. 解析命令并更新 `CodeEditorViewDisplayOptions.show_line_numbers`
4. Esc 退出命令行模式

## 关键技术选型

- 复用编辑器已有的 overlay/input 机制显示命令行提示
- 在 vim handler 中维护 `Option<String>` 作为命令行输入缓冲区
- 解析 `:set` 选项：`number`/`num`/`nu` 和 `nonumber`/`nonu`/`nonum`
