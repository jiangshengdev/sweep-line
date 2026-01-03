## Plan: src 代码审核计划

本计划旨在全面审查 `src/` 目录下的代码，重点关注安全性（消除 `unwrap`）、算法正确性（扫描线与几何判定）以及对 Phase 2 需求的准备情况。

### Steps
1. **安全性审计**: 扫描 [src/geom](src/geom) 和 [src/sweep](src/sweep) 中的 `unwrap()`/`expect()`，将潜在运行时错误改为 `Result` 处理。
2. **算法逻辑审查**: 重点审查 [src/sweep/bo.rs](src/sweep/bo.rs) 的垂直线段处理逻辑及 [src/geom/intersection.rs](src/geom/intersection.rs) 的共线/端点判定。
3. **Phase 2 预备检查**: 确认 [src/trace.rs](src/trace.rs) 和 [src/run.rs](src/run.rs) 的数据结构是否支持后续的“点聚合”输出及熔断机制。
4. **清理与文档**: 分类处理代码中的 `TODO`/`FIXME`，并为 [src/geom/predicates.rs](src/geom/predicates.rs) 等复杂逻辑补充数学注释。

### Further Considerations
1. 是否需要为 `lib.rs` 强制开启 `#![warn(clippy::unwrap_used)]` 以防止未来回归？
2. 针对 Phase 2 的“最大重叠段”逻辑，是否现在就需要预留接口定义？
