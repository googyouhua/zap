---
comet_change: quick-credential-input
role: technical-design
canonical_spec: openspec
archived-with: 2026-07-20-quick-credential-input
status: final
---

# Quick Credential Input — Technical Design

## Overview

Extend the existing Warp OneKey credential system to support generic username/password pairs beyond SSH. Users can save credentials (label, username, password), trigger a search panel via hotkey, select a credential, choose a send mode (password-only or username-then-password), and have it automatically typed into the shell.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    QuickCredentialPanel                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  SearchBar   │  │  SearchMixer │  │ QuickCredential   │  │
│  │  (filter)    │──│  (data src)  │──│ DataSource        │  │
│  └──────────────┘  └──────────────┘  └────────┬─────────┘  │
│                                                │            │
│  ┌─────────────────────────────────────────────┴──────────┐ │
│  │  QuickCredentialItem list (fuzzy filtered)             │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                │            │
│           on ItemSelected                      │            │
└────────────────────────────────────────────────┼────────────┘
                                                 ▼
                          ┌──────────────────────────────────┐
                          │     SendMode Selector            │
                          │  [Password Only] [User+Pass]     │
                          └──────────────┬───────────────────┘
                                         ▼
                          ┌──────────────────────────────────┐
                          │  QuickCredentialSender           │
                          │  write_to_pty(cred, mode)        │
                          └──────────────┬───────────────────┘
                                         ▼
                          ┌──────────────────────────────────┐
                          │  TerminalView                     │
                          │  clear_line_editor_and_write...   │
                          │  write_to_pty(...)                │
                          └──────────────────────────────────┘
```

## Data Model

### SQLite: `quick_credentials` table

```sql
CREATE TABLE quick_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    username TEXT NOT NULL DEFAULT '',
    send_mode TEXT NOT NULL DEFAULT 'password_only'
        CHECK (send_mode IN ('password_only', 'username_then_password')),
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### OS Keychain

- Service name: `zap.quick-credential`
- Account key pattern: `<credential-uuid>:password`
- Value: the actual password (cleartext, but OS keychain encrypts at rest)
- Use `keyring` crate (same as existing OneKey)

### Rust Types

```rust
pub struct QuickCredential {
    pub id: String,
    pub label: String,
    pub username: String,
    pub send_mode: SendMode,
    pub notes: String,
    pub password: Zeroizing<String>,  // loaded from keychain on demand
}

pub enum SendMode {
    PasswordOnly,
    UsernameThenPassword,
}
```

## Component Design

### 1. Repository Layer (`crates/warp_ssh_manager` 或新建 `crates/quick_credential`)

```
QuickCredentialRepository
  ├── list() -> Vec<QuickCredential>
  ├── get(id) -> Option<QuickCredential>
  ├── create(credential) -> QuickCredential
  ├── update(credential) -> QuickCredential
  └── delete(id)
```

- Metadata operations go through SQLite (via Diesel)
- Password operations go through OS Keychain (via `keyring`)
- `get` and `list` load passwords from keychain lazily or eagerly depending on use case

### 2. Search Panel (`app/src/search/quick_credential/`)

Reference implementation: `app/src/search/external_secrets/`

```
QuickCredentialPanel
  ├── Fields: search_bar, search_bar_state, mixer, handle
  ├── Events: QuickCredentialPanelEvent
  │     ├── ItemSelected { credential: QuickCredential }
  │     └── Close
  └── Render: centered overlay with search bar + credential list
  
QuickCredentialDataSource (SyncDataSource)
  ├── Load all credentials on setup
  └── Filter by fuzzy match on label + username
```

### 3. Send Mode Selection

After user selects a credential in the panel, the panel transitions to show two buttons:
- "Send Password Only" (Enter)
- "Send Username + Password"

This is a state within `QuickCredentialPanel` (`selected_credential: Option<QuickCredential>`), not a separate view.

### 4. Sender Logic (`app/src/terminal/quick_credential_sender.rs`)

```rust
pub fn send_quick_credential(
    terminal_view: &mut TerminalView,
    credential: &QuickCredential,
    mode: SendMode,
    ctx: &mut ViewContext<TerminalView>,
) {
    match mode {
        SendMode::PasswordOnly => {
            terminal_view.write_to_pty(
                format!("{}\n", credential.password).into_bytes(),
                ctx,
            );
        }
        SendMode::UsernameThenPassword => {
            terminal_view.clear_line_editor_and_write_to_pty(
                format!("{}\n", credential.username).into_bytes(),
                ctx,
            );
            let password = credential.password.clone();
            ctx.spawn_after(Duration::from_millis(150), move |terminal_view, ctx| {
                terminal_view.write_to_pty(
                    format!("{}\n", password).into_bytes(),
                    ctx,
                );
            });
        }
    }
}
```

### 5. TerminalView Integration

In `TerminalView`:
- Store `quick_credential_panel: ViewHandle<QuickCredentialPanel>`
- Create during `TerminalView::new()`:
  ```rust
  let panel = ctx.add_typed_action_view(|ctx| QuickCredentialPanel::new(ctx));
  ctx.subscribe_to_view(&panel, Self::on_quick_credential_event);
  ```
- Render as positioned overlay when open
- Register action `TerminalAction::ToggleQuickCredentialPanel`

### 6. PTY Auto-Detect Enhancement

Modify `show_onekey_prompt_menu` to also call `load_saved_quick_credentials()` and merge results:
- SSH OneKey credentials show with SSH key icon
- Quick credentials show with generic key icon
- Send mode for quick credentials defaults to the credential's configured mode

### 7. Settings UI (`app/src/settings_view/`)

New "Quick Credentials" page:
- List view with label, username preview, send mode icon
- "Add" button → form view (label, username, password, send_mode, notes)
- Edit → same form pre-populated
- Delete → confirmation dialog

## Key Bindings

```rust
EditableBinding::new(
    "terminal:toggle_quick_credential_panel",
    "Show quick credential input panel",
    TerminalAction::ToggleQuickCredentialPanel,
)
.with_context_predicate(id!("Terminal"))
.default_trigger("cmd+shift+k")
```

## Feature Flag

```rust
#[cfg(feature = "quick_credential_input")]
pub mod quick_credential;
```

Runtime flag: `FeatureFlag::QuickCredentialInput` in `crates/warp_core/src/features.rs`

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Linux headless/WSL: keyring unavailable | Detect at init time, show warning, allow file-based fallback |
| 150ms delay insufficient for slow shells | Make delay configurable via `notes` field or per-credential setting |
| Password echo in terminal visible | Use `clear_line_editor_and_write_to_pty` to minimize on-screen exposure |
| Memory: password remains in process heap | Use `Zeroizing<String>` throughout; explicit clear after send |
| Panel obscures shell output | Render with semi-transparent background; allow Escape to dismiss |

## Testing Strategy

- **Unit**: QuickCredentialRepository CRUD operations with SQLite in-memory test
- **Unit**: SendMode selection and sender logic with mock TerminalView
- **Unit**: Fuzzy search filtering in QuickCredentialDataSource
- **Integration**: Full flow: add credential → open panel → search → select → send
