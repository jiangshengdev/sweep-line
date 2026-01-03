# Phase 1：交点按点聚合输出（方案 B）

目标：把 Phase 1 的点交输出从 “point + (a,b) pair 列表” 改为“按点聚合：point -> {segments...}”，从根源上避免同点多线段相交导致的 `O(k^2)` 输出爆量（见 `plans/src-code-review-findings.md` 的 #3）。

## Checklist

- [x] 增加 `PointIntersectionGroupRecord`：用 `endpoint_segments` / `interior_segments` 表达“该点涉及的线段集合”
- [x] `src/sweep/bo.rs`：事件点处累计线段集合，移除 `record_endpoint_pairs` 的 pair 枚举
- [x] `src/trace.rs`：升级为 `trace.v2`（交点字段改为按点聚合结构）
- [x] `src/session.rs`：升级为 `session.v2`，并提供 `session_v2_to_json_string(_limited)`
- [x] `viewer/`：兼容解析 `session.v1/trace.v1` 与 `session.v2/trace.v2`，UI 表格改为展示 `segments/kind/point`
- [x] 更新单测与文档，并在 `plans/src-code-review-findings.md` 将问题 #3 标记为“已实现”
