# src 代码审查：已确认的问题清单

本文件整理了对 `src/`（主要是 `src/sweep/*` 与 `src/geom/*`）进行快速审查后，确认“真实存在”的问题点（指代码中确实存在对应实现/缺口），便于后续修复或纳入 Phase 2 计划。

审查基准：commit `edd7651900d4f08f169d7a13cc8340d225d74c31`。

## 1) 潜在 panic：`y_at_x` 的 i128 溢出路径

- 位置：`../src/sweep/segment_order.rs` 的 `y_at_x`。
- 现象：多处 `checked_mul/checked_add/checked_sub(...).expect("i128 ... 溢出")`（一旦触发会直接 `panic`）。
- 影响：
  - 扫描线运行中断（可靠性风险）；若输入可控/外部可提供，属于潜在 DoS 风险。
  - 即使当前 `preprocess` 将输入坐标限制在 `[-1, 1]`（量化到 `±1e9`），该函数作为通用比较器并未从类型/接口层面保证“永不溢出”；未来若扩大坐标域或引入不同输入源，风险会上升。
- 建议方向（非实现）：
  - 将比较器相关计算改为返回 `Result` 并在上层传播/兜底；
  - 或采用不构造巨大中间量的比较方式（例如基于不等式的交叉相乘 + 继续分数比较），从算法上规避溢出。

## 2) 缺少输出规模/执行步数熔断（fail-fast 上限）

- 位置：`../src/run.rs` 的 `run_phase1` 以及 `../src/sweep/bo.rs` 的 trace/输出累计逻辑。
- 现象：当前 phase1 直接构建并返回：
  - `Vec<PointIntersectionRecord>`（点交 pair 列表）
  - `Trace`（含 `steps`/`active`/`intersections` 等）
  但没有 `max_*` 参数或硬上限检查。
- 影响：
  - 输入线段数较大或构造退化用例时，可能出现内存/时间不可控；
  - 尤其 `../src/sweep/bo.rs` 的 `record_endpoint_pairs` 在同点端点聚集时会产生 `O(k^2)` 记录，容易造成输出爆炸。
- 相关约定：`phase2-precheck.md` 已提出 `max_session_bytes`/`max_trace_steps` 等 fail-fast 策略，但代码尚未落地。

## 3) 交点输出未按点聚合（同点多线段会爆量）

- 位置：`../src/geom/intersection.rs` 的 `PointIntersectionRecord` 定义与 `../src/sweep/bo.rs` 的输出方式。
- 现象：phase1 的交点输出是“点 + (a,b)”的 pair 记录；同一点涉及 `k` 条线段时，会产生 `k*(k-1)/2` 条记录。
- 影响：
  - 对外输出/trace 容易爆量；
  - Phase 2 计划里明确希望“按点聚合输出”（见 `phase2-precheck.md`），当前数据结构需要后处理或升级 schema 才能满足。

## 4) 点交分类粒度不足（Phase 2 需求缺口）

- 位置：`../src/geom/intersection.rs` 的 `PointIntersectionKind`。
- 现象：当前仅有 `Proper` / `EndpointTouch`，无法区分 `EndpointEndpoint` 与 `EndpointInterior`。
- 影响：Phase 2 若需要对外稳定区分三类（用于前端上色/统计），需要扩展 enum 或在输出层增加子字段。

## 5) 可维护性：几何谓词缺少语义注释

- 位置：`../src/geom/predicates.rs`。
- 现象：`orient`/`on_segment` 缺少 doc 注释解释符号意义与闭区间约定（当前仅有少量测试覆盖了符号与端点包含性）。
- 影响：后续在 Phase 2 做 1D 投影、端点语义、退化处理时，理解成本更高、也更容易误用。

