# Brainstorm Summary

- Change: quick-credential-input
- Date: 2026-07-19

## Confirmed Technical Approach

Extend existing OneKey infrastructure with a parallel QuickCredential system:
- New SQLite table `quick_credentials` for metadata (independent from SSH `ssh_onekey_credentials`)
- OS Keychain (`keyring` crate) for password storage, service name `zap.quick-credential`
- New search panel `QuickCredentialPanel` (reference pattern: `ExternalSecretsMenu`)
- Send mode selection after credential choice (password-only / username-then-password)
- Send engine: `clear_line_editor_and_write_to_pty` for username, `write_to_pty` for password, 150ms fixed delay for username-then-password mode
- Always append `\n` after sending

## Key Trade-offs and Risks

- Linux keyring unavailability in headless/WSL → show warning, consider file-based fallback
- 150ms delay might not be enough for slow shells → fixed for now, extendable
- Panel may briefly obscure terminal → semi-transparent + Escape to dismiss
- Passwords handled with `Zeroizing<String>` throughout

## Testing Strategy

- Unit: repository CRUD with in-memory SQLite
- Unit: sender logic with mock TerminalView
- Unit: fuzzy search filtering
- Integration: full end-to-end panel flow

## Spec Patches

None needed (existing specs already align with design decisions).
