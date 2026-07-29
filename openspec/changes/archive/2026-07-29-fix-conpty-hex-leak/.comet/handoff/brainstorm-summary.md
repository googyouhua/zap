# Brainstorm Summary

- Change: fix-conpty-hex-leak
- Date: 2026-07-28

## Confirmed Technical Approach

Inline OSC payload protocol: `\e]9277;A;<hex>\a\e]9277;B\a`. Replace old three-stage format where hex sat outside the OSC boundary after `\a`.

## Key Trade-offs and Risks

- mkdir file lock > flock: atomic per-directory, no fd leak risk
- Fish no lock needed: begin...end subshell ensures single write
- \e\\ preexec cleanup closes dangling OSCs after kill -9
- Risk: mkdir lock leak if crash between mkdir and rmdir → preexec rmdir cleans up
- Old protocol path in Rust kept for backward compat

## Testing Strategy

cargo check for Rust compilation; shell scripts tested in CI.

## Spec Patches

None.
