# Comet Design Handoff

- Change: vim-set-number
- Phase: design
- Mode: compact
- Context hash: e78abc37dffc1b4b948251f0b0950080cc79b52acb3649a318aa0ed8ac103789

Generated-by: comet-handoff.sh

OpenSpec remains the canonical capability spec. This handoff is a deterministic, source-traceable context pack, not an agent-authored summary.

## openspec/changes/vim-set-number/proposal.md

- Source: openspec/changes/vim-set-number/proposal.md
- Lines: 1-16
- SHA256: 9f64b7dd9921e82538af3b5cac24ee1ef0f8712d5a72c476667eab781c0a41b4

```md
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
```

## openspec/changes/vim-set-number/design.md

- Source: openspec/changes/vim-set-number/design.md
- Lines: 1-16
- SHA256: 315152c67a54b86fb32b479735b3acb4b99b9179cd8cd7b2dabf621e27e9b596

```md
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
```

## openspec/changes/vim-set-number/tasks.md

- Source: openspec/changes/vim-set-number/tasks.md
- Lines: 1-4
- SHA256: eafb68c41ba6165f0b47902764cd8dc492d0e3f7d31fd794a1fa915bd5fdd28c

```md
# tasks.md

- [x] 1. 在 `EditorView` 中将 `vim_force_insert_mode` 改为 `pub`（`app/src/editor/view/mod.rs:5426`）
- [x] 2. 在终端输入的 `handle_editor_event` 中将 `EditorEvent::ExCommand` 改为插入 `:` 并切换到插入模式（`app/src/terminal/input.rs:9418-9426`）
```

