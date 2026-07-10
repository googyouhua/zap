## Why

目前只有 `zap_release.yml` 一个 CI 工作流，功能是发布正式 Release（编译 + 打包 + 发布到 GitHub Release）。缺少一个手动触发的、轻量的编译 CI，用于快速验证不同 channel/feature 组合的编译结果。

## What Changes

- 新建 `.github/workflows/ci-run.yml`
- `workflow_dispatch` 手动触发，支持参数选择 channel 和平台
- 复用 `.github/actions/prepare_environment`
- 三平台编译（Linux/macOS/Windows），上传二进制为 artifact
- 不发布 Release

## Capabilities

### New Capabilities
- `ci-run`: 手动触发的多平台编译 CI，支持 oss/dev/preview/stable 四种 channel 选择

## Impact

- 仅 `.github/workflows/ci-run.yml` 一个新文件
- 不修改现有 CI 或 Rust 代码
