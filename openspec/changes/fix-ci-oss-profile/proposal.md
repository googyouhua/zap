## Why

CI 的 oss channel 用 `--profile dev --features "gui"` 导致两个问题：
1. rust-embed 在 debug 模式不嵌入字体，二进制独立运行时报 panic
2. Windows 上 build.rs 计算目标目录错误，conpty.dll 复制失败

## What Changes

ci-run.yml 中 oss channel 的 profile 从 `dev` 改为 `release-lto`，features 从 `gui` 改为 `release_bundle,standalone`

## Impact

仅 `.github/workflows/ci-run.yml`，6 行改动
