# SSH Denylist Edit Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** SSH denylist support `;` split batch add and click-to-edit

**Architecture:** Modify `WarpifyPageView` state + event handler for edit mode; add `EditDenylistedSshHost` action to `WarpifyPageAction`; modify `render_alternating_color_list_item` to support clickable text

**Tech Stack:** Rust, WarpUI

---
base-ref: 9be29aa3

## 1. 编辑状态与 Action

### Task 1.1: 添加 pending_edit_ssh_host_index 字段

**File:** `app/src/settings_view/warpify_page.rs`

- [x] Add `pending_edit_ssh_host_index: Option<usize>` to `WarpifyPageView` struct (near line 93)
- [x] Initialize to `None` in the constructor (near line 159)

### Task 1.2: 添加 EditDenylistedSshHost action

**File:** `app/src/settings_view/warpify_page.rs`

- [x] Add `EditDenylistedSshHost(usize)` to `WarpifyPageAction` enum (after `RemoveDenylistedSshHost`)

### Task 1.3: handle_action 处理 EditDenylistedSshHost

**File:** `app/src/settings_view/warpify_page.rs` (near line 517)

- [x] Add match arm: read `ssh_hosts_denylist[index]` → fill editor via `system_reset_buffer_text` → set `pending_edit_ssh_host_index`

## 2. ; 分割批量添加 + 编辑替换

### Task 2.1: handle_denylisted_ssh_editor_event

**File:** `app/src/settings_view/warpify_page.rs` (line 277)

- [x] On submit: `let edit_index = self.pending_edit_ssh_host_index.take();`
- [x] Loop `new_command.split(';')` → trim → skip empty
- [x] If `edit_index` is Some: replace at index, break (no split)
- [x] If `edit_index` is None: call `denylist_ssh_host` for each fragment
- [x] On Escape: `self.pending_edit_ssh_host_index = None;` then emit FocusModal

## 3. 列表渲染支持点击编辑

### Task 3.1: render_alternating_color_list_item 支持可点击文本

**File:** `app/src/settings_view/settings_page.rs` (line 1090)

- [x] Add optional parameter `edit_action: Option<SettingsPageAction>`
- [x] If `Some(action)`: wrap hostname text with `on_click` dispatching that action

### Task 3.2: SSH denylist 传入 edit_action

**File:** `app/src/settings_view/warpify_page.rs` (line 809)

- [x] When building SSH denylist list, pass `EditDenylistedSshHost(i)` as the edit action

## 4. 验证

### Task 4.1: cargo check

- [x] Run `cargo check` and fix any errors

### Task 4.2: Commit

- [x] Stage + commit all changes
