# Phase 2 语义与输出契约：共线重叠（最大重叠段）+ 同点相交

目的：在开始 `## 第二阶段：共线重叠线段（最大重叠段）`（见 `plans/bo-sweep-line.md`）之前，把关键语义与输出契约先定下来，避免实现到一半因为“输出含义/规模/溯源”不同而返工。

## 现状（Phase 1 已实现）

- 点交枚举已实现（含 `Proper` / `EndpointTouch`），并提供 `trace.v2` 与 `session.v2` 稳定输出：
  - 入口：`src/run.rs` 的 `run_phase1`
  - BO 主流程：`src/sweep/bo.rs`
- 共线重叠当前仅占位：`intersect_segments` 返回 `CollinearOverlap`，并在 phase1 记录 notes，不输出“重叠段”：
  - `src/geom/intersection.rs`：`SegmentIntersection::CollinearOverlap`
  - `src/sweep/bo.rs`：`Check(a,b) -> CollinearOverlap(phase2)`

## 已确定的语义与输出契约（Phase 2）

### 1) “最大重叠段集合”定义（采用覆盖≥2 的极大连续区间）

- 定义：对每组共线线段做 1D 投影，记覆盖次数为 `c(t)`；输出集合 `{ t | c(t) ≥ 2 }` 的所有连通分量（按包含关系极大）。
  - 为了避免歧义：
    - `c(t) ≥ 2` 且长度>0 的连通分量输出为“重叠段”。
    - 仅在单点处满足 `c(t) ≥ 2`（端点接触导致的退化连通分量）输出为“重叠点”。
  - 不采用：只输出“全局最大覆盖”或“最长区间”作为本项目的默认语义（如未来需要，可作为额外查询能力）。
- 裁判例子（3 条共线线段）：
  - `s1=[0,8]`，`s2=[2,8]`，`s3=[6,12]`（同一直线、闭区间）。
  - 覆盖次数：`x∈[2,6)` 时 `c(x)=2`；`x∈[6,8]` 时 `c(x)=3`（全局最大）。
  - “最大重叠段集合”输出：`[2,8]`（覆盖≥2 的极大连续区间）。

### 2) 边界语义：闭区间 + “端点相同也算重叠”

- 线段按闭区间处理（包含端点）。
- 重叠不要求长度>0：仅端点相同也算重叠，但输出为“重叠点”，与“重叠段”区分。
- 现状说明：`src/geom/intersection.rs` 的 `CollinearOverlap` 仍按“长度>0 才占位”实现；端点相同在 phase1 以点交（`EndpointTouch`）输出。

### 3) 点交分类（用于对外输出/前端上色）

- 对外至少区分三类：
  - `Proper`：内部–内部
  - `EndpointEndpoint`：端点–端点
  - `EndpointInterior`：端点–内部
- 已采用方案 2（输出角色信息，让下游派生三分类）：
  - `trace.v2` 的按点聚合交点记录输出 `endpoint_segments` 与 `interior_segments`（见 `src/trace.rs`）。
  - 派生规则：
    - `Proper`：`endpoint_segments` 为空
    - `EndpointEndpoint`：`endpoint_segments.len() >= 2`
    - `EndpointInterior`：`!endpoint_segments.is_empty()` 且 `!interior_segments.is_empty()`
  - 注意：同一几何点可能同时满足 `EndpointEndpoint` 与 `EndpointInterior`（多个端点重合且还有线段穿过）；输出保持集合完整，不强制压成单一 kind，前端可按需显示为多标签或设定优先级。

### 4) 覆盖次数：仅基于预处理后的线段集合

- 预处理会去重（`src/preprocess.rs`）：重复输入不作为多重覆盖（multiset）。
- 因此预处理后不存在“完全相同（含反向）”的线段；但仍可能存在共线的部分重叠/包含（这是 Phase 2 要处理的重叠）。

### 5) Phase 2 输出：`session.v2`（为未来可视化准备数据）

- 输出作为新的顶层 schema（`session.v2` 或等价新版本），不塞进 `trace.v2` 的 `warnings/notes`。
- 可视化不在本次任务范围内，但 phase2 输出必须满足“可画 + 可追溯”：
  - 几何：同时表达“重叠段（长度>0）”与“重叠点（退化）”。
  - 覆盖：能推导覆盖次数（至少能判断覆盖≥2；如需按层数上色，建议输出覆盖常值的原子子段或等价信息）。
  - 溯源：每个重叠/交点结果都能映射到贡献它的**预处理后线段**集合（`SegmentId` 列表或等价表示）。

### 6) 原子子段替换：ID 与溯源（以及交点输出方案 B）

- Phase 2 将把共线重叠组切分为“原子不重叠子段”，并用这些子段替换输入后再跑点交扫描线（避免 overlap 扰动 BO 的事件逻辑）。
- 原子子段 ID：引入 `atomic_segment_id`（不要复用 `SegmentId`），要求确定性与稳定排序。
- 溯源：`session.v2` 需要提供 `atomic_segment_id -> [SegmentId...]`，用于把原子子段映射回预处理后线段集合。
- 交点对外输出（方案 B）：phase2 的交点记录引用 `atomic_segment_id`（而非 `SegmentId` pair）；viewer 可用溯源表高亮“涉及哪些预处理后线段”。

### 7) 同点多线段相交：按点聚合输出（不展开全 pair）

- 同一几何点 `p` 上涉及 `k` 条线段时：只输出一次该点，并记录“该点涉及的线段集合”。
- 不在输出里展开 `k*(k-1)/2` 的 pair 列表：避免同点 `O(k^2)` 爆量；可视化按“点→线段集合”高亮即可。

### 8) 输出规模与超限策略：fail-fast

- 策略：超限直接报错退出（fail-fast），不做限流/截断（避免“看似成功但数据不完整”）。
- 默认硬上限（以最终 `session.v2` JSON 为准；任一超限即报错）：
  - `max_session_bytes = 33554432`（32 MiB）
  - `max_trace_steps = 20000`
  - `max_trace_active_entries_total = 3500000`（所有 step 的 `active.len()` 之和）
  - 错误信息需包含：实际值/上限/建议（例如降低 `GRID_N`、`SPIDER_*`、random 用例规模，或关闭 trace）。
