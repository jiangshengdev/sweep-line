## Plan: src 代码审核计划

本计划旨在全面审查 `src/` 目录下的代码，重点关注安全性（审查 `unwrap`）、算法正确性（扫描线与几何判定）以及对 Phase 2 需求的准备情况。

### Steps
1. **安全性审计**: [x] 扫描 [src/geom](src/geom) 和 [src/sweep](src/sweep) 中的 `unwrap()`/`expect()`。
    - 发现 `src/sweep/segment_order.rs` 中存在 `i128` 乘法溢出的 `expect`，建议改为返回 `Result`。 (**已修复**: `y_at_x` 已改为返回 `Result`)
    - 其他 `unwrap` 主要在测试代码中，或由逻辑不变量保证（如 `bo.rs` 中的 trace step）。
2. **算法逻辑审查**: [x] 重点审查 [src/sweep/bo.rs](src/sweep/bo.rs) 的垂直线段处理逻辑及 [src/geom/intersection.rs](src/geom/intersection.rs) 的共线/端点判定。
    - 垂直线段通过 `pending_vertical` 批处理和 `collect_vertical_hits` 查询状态树，逻辑看似正确。
    - 相交判定优先处理共线，并区分 `Proper` 和 `EndpointTouch`。
3. **Phase 2 预备检查**: [x] 确认 [src/trace.rs](src/trace.rs) 和 [src/run.rs](src/run.rs) 的数据结构是否支持后续的“点聚合”输出及熔断机制。
    - **缺失**: `src/run.rs` 中未发现 `max_steps` 或 `max_intersections` 等熔断机制。 (**已修复**: 已引入 `Limits` 和 `Phase1Options`)
    - 数据结构: `PointIntersectionRecord` 为扁平对，点聚合需后处理。 (**已修复**: 输出已改为 `PointIntersectionGroupRecord`)
4. **清理与文档**: [x] 统计代码中的 `TODO`/`FIXME`，并评估 [src/geom/predicates.rs](src/geom/predicates.rs) 等复杂逻辑的注释完整性。
    - **未发现** `TODO` 或 `FIXME` 标记。
    - `src/geom/predicates.rs` 缺少数学含义注释（如 `orient` 返回值的几何意义）。 (**已修复**: 已补充详细文档注释)

### Decisions
1. **Clippy Lint**: 暂不强制开启 `#![warn(clippy::unwrap_used)]`。
2. **Phase 2 接口**: 暂不预留“最大重叠段”接口定义。
