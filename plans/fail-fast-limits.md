# Phase 1：输出规模/执行步数熔断（fail-fast 上限）

目标：为 Phase 1 的对外输出（交点列表、trace、session JSON）增加可配置的上限检查，避免退化用例导致内存/时间/输出不可控；超限直接报错退出（不做截断）。

## Checklist

- [x] 定义统一的 `Limits`/`LimitExceeded`（含默认值）
- [x] 在 `src/sweep/bo.rs` 对 `intersections`/`trace steps`/`active` 做增量检查并 early-exit
- [x] 在 `src/run.rs` 暴露 `Phase1Options`（含 `trace_enabled`）并接入 `Limits`
- [x] 为 `session.v1` 生成增加 `max_session_bytes` 检查（返回错误而非截断）
- [x] 增加最小单测：验证超限会返回错误（含实际值/上限/建议）
- [x] 更新 `plans/src-code-review-findings.md`：将问题 #2 标记为“已实现”
