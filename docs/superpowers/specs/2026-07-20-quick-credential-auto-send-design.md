---
comet_change: quick-credential-auto-send
role: technical-design
canonical_spec: openspec
---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    Quick Credential Flow                  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Terminal PTY output                                     │
│       │                                                  │
│       ▼                                                  │
│  prompt_detection.rs                                     │
│  classify_prompt(bytes, rules) → Option<PromptType>      │
│       │                                                  │
│       ▼                                                  │
│  spawn_prompt_listener (view.rs)                         │
│  ├─ PromptType::Password + 1 QC → auto-send password     │
│  ├─ PromptType::Username + 1 QC → auto-send usr+pwd      │
│  └─ 0/multiple QCs → OneKey menu (existing)              │
│                                                          │
│  Panel (Ctrl+Shift+U)                                    │
│  ├─ Search → select → SendMode buttons                   │
│  └─ ItemSelected { credential, mode } → send_credential  │
│                                                          │
│  Settings Page                                           │
│  ├─ CRUD credentials (no send_mode)                      │
│  └─ Trigger Keywords section (+ Add, ×, Reset)           │
└──────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### D1: Remove send_mode from storage entirely
- Delete column from SQLite, field from Row/Domain model, CRUD, form UI
- SendMode stays as enum for panel buttons and auto-send logic

### D2: prompt_trigger_rules table in same SQLite DB
- Columns: id (UUID), keyword (TEXT UNIQUE), send_mode (TEXT)
- Default: PasswordOnly={password,passphrase}, UsernameThenPassword={login,username,user,name,email,account}

### D3: classify_prompt() in new prompt_detection.rs
- Pure function: load rules, match against PTY output buffer
- Returns PromptType::Password / Username / None

### D4: Auto-send only with exactly 1 QC
- If 1 credential exists → silent send with matched mode
- If 0 or many → OneKey fallback

### D5: Panel event carries mode separately
- `ItemSelected { credential, mode: SendMode }`

## Implementation Order
1. Data layer: remove send_mode + add trigger_rules table
2. Settings UI: remove dropdown + add keywords section
3. Prompt detection module
4. Auto-send integration
5. Panel event update
