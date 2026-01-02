# 计划：扫描线状态结构（活动集合）接口设计

设计一套“扫描线状态结构”（活动线段集合）的**稳定、可替换**接口：支持插入/删除、前驱后继查询、交点事件导致的顺序交换、以及（垂直线段高频所需的）按 y 区间范围查询；同时明确比较规则依赖 `sweep_x` 的前/后侧语义，保证多次运行一致。

## 已决定的确定性约定
- 状态结构维护的顺序语义固定为：**总是表示当前事件 `x` 的右侧 `x+ε` 的垂直顺序**（不提供 `x-ε` 视角切换）。
  - 含义：处理完同一事件点 `(x,y)` 的批处理后，状态结构必须处于“右侧顺序”的稳定状态。
  - 好处：事件点处多线同点相交/端点重合时，比较关系不会落入“相等/不稳定”，更容易保证可复现与不漏报。
- 垂直线段不插入状态结构：在 `x = x0` 事件处走专门的 **`range_by_y` 命中查询**路径。
- `range_by_y` 的返回先只包含 `SegmentId`（不返回命中时的 y 值）；trace/可视化若需要 y，可由上层按同一套谓词再计算（后续也可扩展接口返回 y）。
- 任意排序 tie-break 必须以稳定字段收敛（例如 `SegmentId`），禁止依赖哈希迭代顺序或随机数。
- 状态结构不持有几何数据：树内只存 `SegmentId`，几何信息通过外部只读段表（例如 `&[Segment]` / `&Segments`）访问。
- `sweep_x` 与内部比较优先使用有理数：`sweep_x` 用 `i128` 的 `num/den` 表示；线段比较基于 `y(sweep_x)` 的有理数比较（必要时将整数边界视作分母为 1 的有理数）。

## 范围
- 包含：
  - 定义 `SweepStatus`（或类似 trait/struct）接口与不变式（ordering contract）。
  - 明确 `sweep_x` 对排序的影响与 tie-break 规则（确定性，固定 `x+ε` 语义）。
  - 覆盖垂直线段常用的 `y ∈ [ymin,ymax]` 范围查询所需的 API 形状。
  - 规划可测试点（不依赖完整 BO 主流程也能测）。
- 不包含：
  - 完整 Bentley–Ottmann 主流程与事件队列实现细节。
  - 共线重叠（phase 2）的状态结构扩展。
  - 第三方库引入与数值策略升级（暂只假设固定点 `1e9` + `i128` 计算）。

## 待办
[x] 列出算法侧最小需求清单（已落到代码接口中）：`set_sweep_x/sweep_x`、`insert/remove`、`pred/succ`、`lower_bound_by_y/range_by_y`、`reorder_segments`（可覆盖 swap）、`snapshot_order`、`validate_invariants`。

[x] 定义上下文类型与语义：当前用 `Rational` 表示 `sweep_x`；状态结构始终表示 `x+ε` 的顺序（见上文约定）。

[x] 规定严格全序比较合同：`cmp(seg_a, seg_b, ctx)` 必须确定性、无随机/哈希依赖；处理“同点相交时相等”的规则（用 `x+ε` 语义 + `SegmentId` 兜底）。

[x] 实现比较器关键构件：非垂直线段的 `y_at_x(sweep_x: Rational) -> Rational` 与 `cmp_segments_at_x_plus_epsilon(...)`（先比 y，y 相等再比斜率 `dy/dx`，仍相等用 `SegmentId` 兜底）。

[x] 定义 `SweepStatus` 接口并给出一个确定性基准实现（例如基于 `Vec<SegmentId>` 的参考实现），用于在不引入 Treap 复杂度的情况下先验证 `insert/remove`、`pred/succ`、`range_by_y` 的语义与稳定性。

[x] 设计范围查询 API：提供 `lower_bound_by_y(y_min)`；`range_by_y(ymin,ymax)` 默认用 `lower_bound_by_y + succ` 迭代并在 `y>ymax` 时早停（先保证稳定性，后续可做 cursor 优化）。

[x] 设计“句柄/定位”策略：节点以 `SegmentId` 直接定位（`Vec` 下标）；删除不依赖比较器；并明确垂直线段禁止插入状态结构。

[x] 实现确定性 Treap 参考实现：新增 `TreapSweepStatus`，优先级由 `SegmentId` 固定混洗函数生成（无 RNG），实现 `insert/remove/pred/succ/snapshot_order/validate_invariants`；`range_by_y` 通过 `lower_bound_by_y + succ` 迭代，优先保证稳定性与语义正确。

[x] 规划测试与验证：已为 `VecSweepStatus` 与 `TreapSweepStatus` 覆盖插入/删除、pred/succ、range 查询稳定顺序等单测，并提供 `validate_invariants()`。

[x] 规划 trace 支撑：提供 `snapshot_order()` 返回稳定顺序的 `Vec<SegmentId>` 以便生成 `trace.json`。

## 后续扩展（可选）
- 若 trace/可视化强依赖命中时的 y：将 `range_by_y` 扩展为返回 `(SegmentId, y_at_x)`，其中 `y_at_x` 用固定点/有理数表示并保持稳定约分。
