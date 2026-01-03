# Phase 2 前置确认清单：共线重叠（最大重叠段）+ 同点相交语义

目的：在开始 `## 第二阶段：共线重叠线段（最大重叠段）`（见 `plans/bo-sweep-line.md`）之前，把关键语义与输出契约先定下来，避免实现到一半因为“输出含义/规模/溯源”不同而返工。

## 现状（Phase 1 已实现）
- 点交枚举已实现（含 `Proper` / `EndpointTouch`），并提供 `trace.v1` 与 `session.v1` 稳定输出：
  - 入口：`src/run.rs` 的 `run_phase1`
  - BO 主流程：`src/sweep/bo.rs`
- 共线重叠当前仅占位：`intersect_segments` 返回 `CollinearOverlap`，并在 phase1 记录 notes，不输出“重叠段”：
  - `src/geom/intersection.rs`：`SegmentIntersection::CollinearOverlap`
  - `src/sweep/bo.rs`：`Check(a,b) -> CollinearOverlap(phase2)`

## 需要在开始 Phase 2 前确认的问题（建议逐条定结论）

### 1) “最大重叠段集合”到底指什么
- [ ] 选项 A：输出“覆盖次数 ≥ 2 的所有**极大连续区间**”（按包含关系极大，区间两端都不能再延伸仍保持覆盖≥2）。
- [ ] 选项 B：输出“覆盖次数达到**全局最大值**的区间”（例如最大覆盖为 5，则只输出覆盖=5 的区间）。
- [ ] 选项 C：输出“长度最大”的重叠区间（可能多个并列）。
- [ ] 需要一个小例子写进文档作为裁判（建议包含：3 条共线线段，产生 2 个重叠区间且覆盖次数不同）。

### 2) 重叠/端点的边界语义（闭区间 vs 开区间）
- [ ] 重叠段是否要求“长度 > 0”（仅端点相同算点交，不算重叠段）？
  - 现状：`src/geom/intersection.rs` 已按“长度>0 才算 `CollinearOverlap`”实现。
- [ ] 区间端点是否是闭区间（包含端点）？这会影响“端点接触”在 phase2 的分段边界上如何解释。

### 3) 重复线段（duplicate）是否影响“覆盖次数”
- [ ] 现状：预处理会去重（`src/preprocess.rs`），重复输入只保留一个代表并给 warning。
- [ ] Phase 2 的“覆盖次数”是否需要把重复输入当作多重覆盖（multiset）？
  - 若需要：不能简单去重；至少需要保留 multiplicity 或在 phase2 计算覆盖时引入计数权重。

### 4) Phase 2 输出放在哪里（接口 / schema）
- [ ] 输出作为新的顶层字段（例如 `session.v2` / 新 schema），还是塞进 `trace.v1` 的 `warnings/notes`？
- [ ] 是否需要前端可视化 phase2 重叠段？
  - 现状：`plans/trace-visualizer.md` 明确 v0 不包含 phase2 可视化；若要做，需另起计划或扩展现有计划。

### 5) “原子不重叠子段替换”后的溯源与 ID 规则
- [ ] Phase 2 是否会把共线重叠输入替换为“原子子段”后再跑点交扫描线？
  - 计划：`plans/bo-sweep-line.md` 已写了该思路。
- [ ] 若替换：需要明确
  - 原子子段的 `SegmentId` 如何分配（确定性、稳定排序）
  - 原子子段如何映射回原始输入（建议提供 `atomic_segment_id -> [source_index...]` 或类似结构）
  - phase1/phase2 的交点输出是否仍以“原始 segment”语义对外暴露（否则调用方需要升级解析逻辑）

### 6) “同点多线段相交”时，交点输出是否需要“全 pair 组合”
- [ ] 选项 A（pair 语义）：若同一点 `p` 上有 `k` 条线段都包含 `p`，输出所有无序对 `{si,sj}`，共 `k*(k-1)/2` 条 `PointIntersectionRecord`。
- [ ] 选项 B（point 聚合语义）：每个点只输出一次，记录“该点涉及的线段集合”，由调用方（或前端）自行展开 pairs。
- [ ] 选项 C（混合语义）：保持现有 `PointIntersectionRecord` 输出，但额外提供“按点聚合”的索引表（不破坏兼容性但会扩展 schema）。

### 7) 输出规模与性能上限（必须提前定）
- [ ] pair 语义在同点处是 `O(k^2)` 输出；phase2 原子化会显著增加共点端点，可能导致爆量（trace/session 体积、前端渲染、生成器耗时）。
- [ ] 是否需要全局/单点限流策略（例如只在 trace 里记录前 N 条，或对某类退化只输出聚合信息）？

## 同点相交的“全 pair 组合”：详细说明（用于做决策）

### 定义
同一点 `p` 上有线段集合 `S(p) = {s1..sk}`，则输出集合：
`{ (min(si,sj), max(si,sj), kind(si,sj,p), p) | i<j }`。

其中 `kind` 对每一对单独判定：只要 `p` 是两条线段里任意一条的端点，则为 `EndpointTouch`；否则为 `Proper`。

### 当前实现与差距
- 现状里，“端点出现在事件点”的线段会两两输出 `EndpointTouch`（这是全 pair 的一个子集）：
  - `src/sweep/bo.rs`：`record_endpoint_pairs`
- 但对于“所有线段都在该点内部相交（无端点落在 p）”的情形：
  - 经典 BO 通常只保证枚举交点“点”，并通过相邻对调度来达到正确性；
  - **不会天然保证**在同一交点把 `k*(k-1)/2` 个 pair 都输出（尤其当 `k` 很大时）。

### 选择 pair 语义的工程含义
- 需要明确：算法的“正确性目标”变成“枚举所有相交 pair”，而不是“枚举所有相交点”。
- 对同点退化：可能需要在每个事件点显式构造 `S(p)` 并输出所有 pairs（这会改变性能特征与实现复杂度）。

### 选择 point 聚合语义的工程含义
- 更贴近“事件点批处理”的真实结构（`U/L/C(p)` + 垂直命中），且能避免 `O(k^2)` 输出爆炸；
- 但需要改变/扩展 schema（当前 `trace.v1`/viewer 以 `PointIntersectionRecord` 列表为核心展示单元）。

