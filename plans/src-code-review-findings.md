# src 代码审查：已确认的问题清单

本文件整理了对 `src/`（主要是 `src/sweep/*` 与 `src/geom/*`）进行快速审查后，确认“真实存在”的问题点（指代码中确实存在对应实现/缺口），便于后续修复或纳入 Phase 2 计划。

审查基准：commit `edd7651900d4f08f169d7a13cc8340d225d74c31`。

## 1) 潜在 panic：`y_at_x` 的 i128 溢出路径

- 状态：已实现（方案 A）——`y_at_x` 已改为返回 `Result` 并向上游传播 `ArithmeticOverflow`，避免 `panic` 中断（见 `../src/sweep/segment_order.rs` / `../src/sweep/status.rs`）。
- 位置：`../src/sweep/segment_order.rs` 的 `y_at_x`。
- 修复前现象：多处 `checked_mul/checked_add/checked_sub(...).expect("i128 ... 溢出")`（一旦触发会直接 `panic`）。
- 现状评估（基于现有输入约束）：若线段坐标均来自 `preprocess`（`Coord=i64` 且量化范围 `[-1e9,1e9]`），并且 `sweep_x` 仅来自端点或两线交点（分母量级约 `≤1e19`），则 `y_at_x` 的中间量粗略估算在 `1e37` 量级，低于 `i128::MAX`，因此在“按当前入口使用”的情况下很难触发溢出；但该前提目前未被类型/接口显式约束。
- 影响：
  - 扫描线运行中断（可靠性风险）；若输入可控/外部可提供，属于潜在 DoS 风险。
  - 即使当前 `preprocess` 将输入坐标限制在 `[-1, 1]`（量化到 `±1e9`），该函数作为通用比较器并未从类型/接口层面保证“永不溢出”；未来若扩大坐标域或引入不同输入源，风险会上升。
- 建议方向（非实现）：
  - 方案 A（稳健/面向未来）：将比较器相关计算改为返回 `Result`，新增“算术溢出”错误并在 `SweepStatus` → `BoError` 方向 fail-fast 传播，避免 `panic`；
  - 方案 B（算法层面）：避免构造大中间量的 `y_at_x`，改为“比较时按分数比较/连分数”的方式直接比较两段在 `x+ε` 的高低；
  - 风险控制：不要用 `f64` 近似或饱和/截断来“继续排序”，否则容易破坏状态结构所需的全序一致性；可补充边界用例测试，覆盖最大坐标与极端交点分母。

## 2) 缺少输出规模/执行步数熔断（fail-fast 上限）

- 状态：已实现（见 `plans/fail-fast-limits.md`，以及 `../src/limits.rs` / `../src/run.rs` / `../src/sweep/bo.rs` / `../src/session.rs`）。
- 位置：`../src/run.rs` 的 `run_phase1` 以及 `../src/sweep/bo.rs` 的 trace/输出累计逻辑。
- 现象：当前 phase1 直接构建并返回：
  - `Vec<PointIntersectionGroupRecord>`（按点聚合交点列表）
  - `Trace`（含 `steps`/`active`/`intersections` 等）
    但没有 `max_*` 参数或硬上限检查。
- 影响：
  - 输入线段数较大或构造退化用例时，可能出现内存/时间不可控；
  - 尤其 `../src/sweep/bo.rs` 的 `record_endpoint_pairs` 在同点端点聚集时会产生 `O(k^2)` 记录，容易造成输出爆炸。
- 相关约定：`phase2-precheck.md` 已提出 `max_session_bytes`/`max_trace_steps` 等 fail-fast 策略（已落地）。
- 实现要点（已落地）：
  - 定义统一的 `Limits`（或 `Phase1Options`）参数：至少包含 `max_trace_steps`、`max_intersections`、`max_trace_active_entries_total`、`max_session_bytes`（默认值可直接沿用 `phase2-precheck.md`）；
  - 在生成过程中做“增量计数 + 早退出”：每次追加交点/trace step/active snapshot 时检查上限并返回错误；
  - 错误信息包含：实际值/上限/建议（缩小输入规模、关闭 trace、降低随机用例参数等），避免“看似成功但输出不完整”的截断策略。

## 3) 交点输出未按点聚合（同点多线段会爆量）

- 状态：已实现（见 `plans/aggregate-intersections-by-point.md`，以及 `../src/geom/intersection.rs` / `../src/sweep/bo.rs` / `../src/trace.rs` / `../src/session.rs` / `../viewer/schema/session.js` / `../viewer/ui/panels.js`）。
- 位置：`../src/geom/intersection.rs` 的 `PointIntersectionGroupRecord` 定义与 `../src/sweep/bo.rs` 的输出方式。
- 现象（修复前）：phase1 的交点输出是“点 + (a,b)”的 pair 记录；同一点涉及 `k` 条线段时，会产生 `k*(k-1)/2` 条记录。
- 影响：
  - 对外输出/trace 容易爆量；
  - Phase 2 计划里明确希望“按点聚合输出”（见 `phase2-precheck.md`），当前数据结构需要后处理或升级 schema 才能满足。
- 建议方向（非实现）：
  - 方案 A（最小变更）：保留内部 pair 记录，但在输出层对 `Vec<PointIntersectionRecord>` 按 `point` 分组，生成 `point -> {segments...}` 的聚合视图（适用于 `session.v2`/viewer）；注意这无法避免 phase1 内部已经产生的 `O(k^2)` 记录量；
  - 方案 B（根本降爆量）：在扫描线事件点处直接累计“该点涉及的线段集合”，trace/输出只写一次该点（并记录参与线段集合），避免 `record_endpoint_pairs` 枚举所有 pair；若需要进一步分类，可同时记录“端点参与集合/内部参与集合”（可顺带解决 #4 的分类需求）；
  - 兜底策略：仅在小规模或调试模式输出 pair 级列表；默认对外输出使用按点聚合，配合 #2 的 fail-fast 上限。

## 4) 点交分类粒度不足（Phase 2 需求缺口）

- 状态：已实现（方案 2）——Phase 1 的 `trace.v2/session.v2` 输出 `endpoint_segments`/`interior_segments`，viewer 侧派生并展示三类（不要求扩展核心 enum）。
- 位置：
  - 核心：`../src/geom/intersection.rs` 的 `PointIntersectionKind`。
  - Viewer：`../viewer/schema/session.js` 的 `deriveIntersectionKind`（产出 `kindDetail`）。
- 现象：`PointIntersectionKind` 当前仅有 `Proper` / `EndpointTouch`，无法显式区分 `EndpointEndpoint` 与 `EndpointInterior`。
- 影响：
  - 若 Phase 2 需要对外稳定区分三类（用于前端上色/统计），需要明确“派生规则”或扩展对外字段；
  - 若未来有逻辑需要在 phase1/phase2 内部依赖该分类（而非仅用于展示），则仅靠 `PointIntersectionKind` 不够。
- 现状补充（可派生）：对按点聚合的 `PointIntersectionGroupRecord`，用两集合即可推导三类：
  - `Proper`：`endpoint_segments.is_empty()`
  - `EndpointInterior`：`!endpoint_segments.is_empty()` 且 `!interior_segments.is_empty()`
  - `EndpointEndpoint`：`endpoint_segments.len() >= 2`（通常此时 `interior_segments` 为空；若同点还有内部穿过，则该点同时包含两种端点相关关系）
- 建议方向（非实现）：
  - 若需要“单一 kind 字段”：在输出层/前端按约定做派生，并在文档里固定优先级；
  - 若需要“完整语义且不丢信息”：保持 `endpoint_segments/interior_segments` 作为 canonical 表达，前端按需派生三类或多标签；
  - 若内部算法也要区分：再扩展 `PointIntersectionKind`（或增加新字段）而不是复用 `EndpointTouch`。

## 5) 可维护性：几何谓词缺少语义注释

- 状态：已实现——已在 `../src/geom/predicates.rs` 补充模块说明与 `orient/on_segment` 的 doc 注释，明确符号约定与闭区间语义。
- 位置：`../src/geom/predicates.rs`。
- 修复前现象：`orient`/`on_segment` 缺少 doc 注释解释符号意义与闭区间约定（当前仅有少量测试覆盖了符号与端点包含性）。
- 影响：后续在 Phase 2 做 1D 投影、端点语义、退化处理时，理解成本更高、也更容易误用。
- 建议方向（非实现）：
  - 在 `orient` 上补充：返回值符号对应的几何意义（左转/右转/共线）、坐标系方向约定；
  - 在 `on_segment`/`in_bbox` 上补充：闭区间语义（端点包含）以及对共线点的要求；
  - 明确该模块假设“输入已量化为整数网格”（`SCALE=1e9`）及其对鲁棒性的意义，避免后续引入浮点坐标时误用。
