## Plan: src 代码审核计划

本计划旨在全面审查 `src/` 目录下的代码，重点关注安全性（审查 `unwrap`）、算法正确性（扫描线与几何判定）以及对 Phase 2 需求的准备情况。

### Steps
1. **安全性审计**: 扫描 [src/geom](src/geom) 和 [src/sweep](src/sweep) 中的 `unwrap()`/`expect()`，识别并记录需要改为 `Result` 处理的潜在运行时错误（不进行修改）。
2. **算法逻辑审查**: 重点审查 [src/sweep/bo.rs](src/sweep/bo.rs) 的垂直线段处理逻辑及 [src/geom/intersection.rs](src/geom/intersection.rs) 的共线/端点判定。
3. **Phase 2 预备检查**: 确认 [src/trace.rs](src/trace.rs) 和 [src/run.rs](src/run.rs) 的数据结构是否支持后续的“点聚合”输出及熔断机制。
4. **清理与文档**: 统计代码中的 `TODO`/`FIXME`，并评估 [src/geom/predicates.rs](src/geom/predicates.rs) 等复杂逻辑的注释完整性（不进行修改）。

### Decisions
1. **Clippy Lint**: 暂不强制开启 `#![warn(clippy::unwrap_used)]`。
2. **Phase 2 接口**: 暂不预留“最大重叠段”接口定义。
