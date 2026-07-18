# proposal.md

## Problem
终端中点击 URL 时，如果 URL 后面紧跟中文/全角标点字符（如 `。` `、` `，` `！` `？` `）》` 等），这些标点会被当作 URL 的一部分，导致打开链接失败。

## Root Cause
`urlocator` crate (v0.1.4) 的 `is_illegal_at_end()` 函数只识别 ASCII 标点字符（`.` `,` `:` `;` `?` `!` `(` `[` `'`）作为 URL 末尾禁止字符。全角版本（`。` `！` `，` `：` `；` `？` `）` `］` `｝` 等）及 CJK 标点（`、` `》` `」` `』` `】` `〕` 等）均不在其中，被当作 URL 内容的一部分。

## Fix Goal
将 CJK 句尾标点和全角标点字符加入 `is_illegal_at_end()`，确保包含这些字符的 URL 在末尾被正确 trim。
