# Subagent-Driven Development — 进度检查点

- Change: onekey-credential-system
- Phase: build
- review_mode: standard
- tdd_mode: tdd
- isolation: worktree (`.worktrees/onekey-credential-system`, branch `feature/20260731/onekey-credential-system`)

## 状态:build 阶段全部完成,待 guard 进入 verify

## 任务清单(9 组 36 任务全绿)

| # | 计划任务标题 | 对应 OpenSpec tasks.md | 状态 |
|---|--------------|------------------------|------|
| 1 | Task 1:存储层 crates/onekey | 分组 1 (1.1-1.7) | ✅ done(70b5d054,3d2badaf) |
| 2 | Task 2:Feature flag 与接线 | 分组 2 (2.1-2.3) | ✅ done(1d67d69d) |
| 3 | Task 3:终端搜索面板 | 分组 3 (3.1-3.4) | ✅ done(d48ea96c) |
| 4 | Task 4:发送引擎与面板接线 | 分组 4 (4.1-4.3) | ✅ done(24724011) |
| 5 | Task 5:PTY auto-send 扩展 | 分组 5 (5.1-5.3) | ✅ done(260098a5+b33dc917) |
| 6 | Task 6:SSH 面板数据源切换 | 分组 6 (6.1-6.3) | ✅ done(1628d344+1adbaa9b) |
| 7 | Task 7:设置页 OneKeyPage | 分组 7 (7.1-7.6) | ✅ done(8997b807) |
| 8 | Task 8:删除旧 SSH OneKey 系统 | 分组 8 (8.1-8.4) | ✅ done(bf3638e0) |
| 9 | Task 9:验证 | 分组 9 (9.1-9.3) | ✅ done(静态盘查通过;手动 UI 验证留待人工) |

## 最终收尾审查(standard review_mode 的 final lightweight review)

- 结论:REQUEST_CHANGES → 修复 IMPORTANT-1 后 APPROVE
- **I-1(已修复,871a11c2)**:`OneKeyPanel::setup()` 从未被调用 → 面板打开后 mixer 无数据源查不到凭据。修复:ToggleOneKeyPanel 打开分支调用 `panel.update(ctx, |panel, ctx| panel.setup(ctx))`。
- **MINOR(M-1~M-9,已接受并记录,不修)**:
  - M-1 `encrypted_password` 列实为明文列(命名误导),与参考一致;文档已注明。
  - M-2 设置页编辑回填真实 secret 到普通 String(非 Zeroizing),有掩码显示,风险低;建议后续与 SSH 面板对齐(故意清空)。
  - M-3 设置页编辑清空密码会静默清空凭据(无 `is_empty` 守卫),SSH 面板有守卫;建议后续对齐。
  - M-4 设置页无必填校验(任务 7.2 声明 label/password 必填);建议后续补。
  - M-5 默认触发关键词只在设置页首次打开时 seed(auto-send 依赖);建议后续移到启动路径/migration。
  - M-6 设置页 OneKey section 未用 FeatureFlag::OneKeyInput 门控(终端面板用);建议后续对齐。
  - M-7 `credentials_version()` 死代码(视图刷新由 notifier 驱动);与参考一致。
  - M-8 跨设备降级:SSH 同步 payload 带 credential_id,另一设备缺凭据时降级无注入直连;可接受。
  - M-9 设置页构造器同步 DB I/O(最多阻塞 busy_timeout 2s);可接受。

## 既有问题(非本 change,已确认)

- `ssh_manager::server_view::tests::selecting_onekey_dropdown_item_does_not_rebuild_dropdown_while_it_is_borrowed`:base(2951bfb7)即失败(测试未注册 `KeybindingChangedNotifier` singleton),非本 change 引入;建议在 base 侧单独修。

## 验证证据

- `cargo check -p warp -p warp_onekey -p warp_ssh_manager -p persistence` 全绿。
- 分 crate 测试:全部通过(见 Task 9 implementer 报告;仅前述 base 既有 1 例失败)。
- 手动 UI 验证(9.3)需人工:面板唤起/搜索/发送、SSH 面板 CRUD、设置页 CRUD、auto-send、跨视图刷新。

## verify 修复轮(round 1)

- verify-fail(verify_failures=1)返回 build,用户裁决 W1/W2/W4 修复、W3 接受并同步文档。
- 修复 commit `30de871a`(11 files):
  - W1 PasswordOnly 清行(onekey_sender.rs)
  - W2 设置页 label/password 必填校验 + EditForm 空密码保持原值(onekey_page.rs)
  - W4 新增 `bytes_look_like_onekey_prompt` 宽正则供 onekey 滑窗(su 路径不受影响)
  - S1/S2/W1/W3 文本同步(design.md/proposal.md/specs/tasks.md)
- `cargo check -p warp -p warp_onekey -p warp_ssh_manager -p persistence` ✅
- 待办:重新过 build guard → 再进 verify。
