# 计划：扫描线状态结构（活动集合）接口设计

设计一套“扫描线状态结构”（活动线段集合）的**稳定、可替换**接口：支持插入/删除、前驱后继查询、交点事件导致的顺序交换、以及（垂直线段高频所需的）按 y 区间范围查询；同时明确比较规则依赖 `sweep_x` 的前/后侧语义，保证多次运行一致。

## 范围
- 包含：
  - 定义 `SweepStatus`（或类似 trait/struct）接口与不变式（ordering contract）。
  - 明确 `sweep_x`/`sweep_side` 对排序的影响与 tie-break 规则（确定性）。
  - 覆盖垂直线段常用的 `y ∈ [ymin,ymax]` 范围查询所需的 API 形状。
  - 规划可测试点（不依赖完整 BO 主流程也能测）。
- 不包含：
  - 完整 Bentley–Ottmann 主流程与事件队列实现细节。
  - 共线重叠（phase 2）的状态结构扩展。
  - 第三方库引入与数值策略升级（暂只假设固定点 `1e9` + `i128` 计算）。

## 待办
[ ] 列出算法侧最小需求清单：`insert/remove`、`pred/succ`、`neighbors`、`swap/reorder`、`range_by_y`、`snapshot_for_trace`、`set_sweep_context`。

[ ] 定义上下文类型与语义：`SweepX`（量化/有理）、`SweepSide`（Left/Right 或 Before/After）、以及“状态结构始终表示 `x+ε` 的顺序”等约定。

[ ] 规定严格全序比较合同：`cmp(seg_a, seg_b, ctx)` 必须确定性、无随机/哈希依赖；处理“同点相交时相等”的规则（用 `x+ε` 语义 + `SegmentId` 兜底）。

[ ] 设计范围查询 API：提供 `lower_bound_y(y)` + `next(cursor)` 或 `range_iter(ymin,ymax)` 的抽象（保证可做 `O(log n + m)` 的垂直查询）。

[ ] 设计“句柄/定位”策略：是否需要 `SegmentId -> NodeHandle` 的映射以支持稳定删除与 `O(log n)` 邻居查询；并规定垂直线段是否禁止插入状态结构（建议禁止）。

[ ] 规划参考实现的接口对齐策略：以“确定性 Treap/Skiplist”为默认实现，优先级由 `SegmentId` 经固定混洗函数生成（无 RNG），确保可复现；未来可替换为红黑树实现而不改算法层。

[ ] 规划测试与验证：为接口写单元测试（插入/删除回归、邻居一致性、range 查询正确性、同输入多次运行 snapshot 一致性），并加入 `validate_invariants()` 供 debug/测试使用。

[ ] 规划 trace 支撑：定义 `snapshot_order()`/`debug_dump()` 返回稳定顺序的 `Vec<SegmentId>`（或包含 y-at-x 的派生值）以便生成 `trace.json`。

## 开放问题
- 状态结构的排序语义是否统一采用“总是 `x+ε`（事件点右侧）”，并在需要“左侧”时通过临时 `ctx` 切换？
- `range_by_y` 的返回需要包含“命中时的 y 值”（便于 trace/可视化），还是只返回 `SegmentId` 即可？
- 是否接受“状态结构不插入垂直线段”，由算法层在 `x=x0` 事件时单独走 range 查询路径（我默认接受）？
