# proposal.md

## 问题描述

在 Vim 模式下按下 `:` 键后输入 `set num` 或 `set number`，期望显示行号，但当前没有任何效果。

## 根因分析

- `:` 键触发 `VimEventType::ExCommand`，但代码编辑器 (`app/src/code/editor/view/vim_handler.rs:507`) 的 `ex_command()` 是空实现
- 没有 vim 命令行模式（`: 后输入文本的界面）
- 没有 `:set` 选项解析器
- 没有 vim 选项存储状态

## 修复目标

实现基本的 `:set number` / `:set nonumber` 功能，使 Vim 模式下能切换行号显示。
