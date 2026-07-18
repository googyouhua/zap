# design.md

## Solution
将 `urlocator` 上游源码 (v0.1.4) 引入工作区作为 vendored crate，在 `is_illegal_at_end()` 中增加 CJK 和全角句尾标点字符的匹配。

## Changed Files

| File | Change |
|------|--------|
| `Cargo.toml` (root) | workspace dep 从 `"0.1.4"` 改为 `{ path = "crates/urlocator" }`；workspace members 加 `"crates/urlocator"` |
| `crates/urlocator/Cargo.toml` (new) | 同包名 `urlocator`，version 0.1.4，`#![no_std]`，无依赖 |
| `crates/urlocator/src/lib.rs` (new) | 上游源码 + `is_illegal_at_end()` 修改 |
| `crates/urlocator/src/scheme.rs` (new) | 上游源码，不变 |
| `crates/urlocator/src/tests.rs` (new) | 上游源码，不变 |

`app/Cargo.toml`、`crates/editor/Cargo.toml` 无需改动（已有 `urlocator.workspace = true`）。

## `is_illegal_at_end()` 修改

```rust
fn is_illegal_at_end(c: char) -> bool {
    match c {
        '.' | ',' | ':' | ';' | '?' | '!' | '(' | '[' | '\''
        | '\u{3001}' | '\u{3002}'          // 、。
        | '\u{3009}' | '\u{300B}'          // 〉》
        | '\u{300D}' | '\u{300F}'          // 」』
        | '\u{3011}' | '\u{3015}'          // 】〕
        | '\u{FF01}' | '\u{FF0E}'          // ！．
        | '\u{FF09}' | '\u{FF3D}'          // ）］
        | '\u{FF1A}' | '\u{FF1B}'          // ：；
        | '\u{FF1F}'                       // ？
        | '\u{FF5D}'                       // ｝
        => true,
        _ => false,
    }
}
```

## Verification
```bash
cargo check
cargo nextest run -p urlocator
```
