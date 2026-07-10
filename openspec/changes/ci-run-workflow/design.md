## Context

`zap_release.yml` 是发布流水线（tag 触发，打包 DMG/AppImage/deb/rpm，发布 GitHub Release）。需要一个手动触发、只编译不打折的 CI。

## Goals / Non-Goals

**Goals:**
- 手动触发编译（workflow_dispatch）
- 支持选择 channel（oss/dev/preview/stable）
- 支持选择平台（linux/macos/windows/all）
- 复用 `prepare_environment` action
- 上传二进制为 workflow artifact

**Non-Goals:**
- 不创建 GitHub Release
- 不打包 DMG/AppImage
- 不修改现有 `zap_release.yml`
- 不修改 Rust 代码

## Decisions

### channel → 编译参数

| channel | cargo features | profile | 说明 |
|---------|---------------|---------|------|
| oss | `gui` | dev | 同 script/run |
| dev | `release_bundle,crash_reporting` | release-lto | 同正式发布 |
| preview | `release_bundle,crash_reporting` | release-lto | 同正式发布 |
| stable | `release_bundle,crash_reporting` | release-lto | 同正式发布 |

### 流程结构

对齐 `zap_release.yml`：
1. `prepare_environment` 安装依赖
2. 各平台 job 编译
3. 上传 artifact

## Risks / Trade-offs

- macOS 构建需 Xcode + Metal toolchain，耗时较长（~1h）
