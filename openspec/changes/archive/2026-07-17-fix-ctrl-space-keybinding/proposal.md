# Proposal: Fix Ctrl+Space Keybinding Conflict with IME

## Problem
Ctrl+Space is the standard system shortcut for switching input methods (IME) on Linux, Windows, and macOS. Warp currently binds this key to `CustomAction::NewAgentModePane` (toggle Agent Mode pane), preventing users from using it for IME switching.

## Root Cause
- `app/src/util/bindings.rs:407` hardcodes `ctrl-space` as the default keystroke for `NewAgentModePane`
- `app/src/editor/accept_autosuggestion_keybinding_view.rs` and `app/src/settings_view/features_page.rs` also use `ctrl-space` as a dynamic fallback for "Open Completions" when Tab is assigned to autosuggestions

## Fix Goal
Replace all hardcoded `ctrl-space` defaults with a non-conflicting keystroke (`ctrl-alt-space`) that doesn't interfere with OS IME switching.
