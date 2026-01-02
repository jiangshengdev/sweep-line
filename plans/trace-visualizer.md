# 方案：trace 回放可视化程序（web 离线版）

目标：基于 Rust 侧输出的 `trace.v1`（以及量化后的线段集合），实现一个可在本地浏览器直接打开的回放器，用于逐步观察扫描线事件批处理、活动集合顺序变化、交点发现过程，并能对照 `events/notes` 做问题定位。

## 输入/输出
- 推荐输入：`session.json`（`session.v1`，包含 segments + trace）。
- 兼容输入（备选）：`segments.json` + `trace.json` 两文件加载（先实现 `session.json`，后续再做两文件模式）。
- 输出：静态网页（无需后端），用户通过“选择文件/拖拽文件”加载数据。

## 数据契约（建议：session.v1）
说明：`trace.v1` 当前不包含线段几何与 fixed 比例；为了让可视化能自洽，建议用一个“包裹格式”把必要信息补齐。

```jsonc
{
  "schema": "session.v1",
  "fixed": { "scale": "1000000000" },
  "segments": [
    { "id": 0, "source_index": 12, "a": { "x": 123, "y": -456 }, "b": { "x": 789, "y": 101 } }
  ],
  "trace": { "schema": "trace.v1", "warnings": [], "steps": [] }
}
```

- `fixed.scale` 用字符串承载（与 `trace.v1` 的 `Rational.num/den` 一致），避免 JS 数字精度问题。
- 坐标统一使用“量化后”的整数单位；渲染时用 `x / scale`、`y / scale` 映射回 `[-1, 1]` 视口（或再做 viewport 变换）。
- 字段顺序（建议/用于稳定输出与回归对比）：
  - `session.v1`：`schema`、`fixed`、`segments`、`trace`。
  - `fixed`：`scale`。
  - `segment`：`id`、`source_index`、`a`、`b`（点为 `x`、`y`）。
  - `trace.v1`：沿用 Rust 侧稳定序列化字段顺序（`src/trace.rs`）。
- 最小校验规则（v0）：
  - `schema == "session.v1"`，否则提示“不是 session.v1 文件”。
  - `fixed.scale` 可解析为正整数；缺失/非法时提示“fixed.scale 无效”。
  - `segments` 为数组；每项包含 `id/a/b`；`a/b.x/y` 为整数。
  - `trace.schema == "trace.v1"`，且 `steps` 为数组；`step.kind` 仅允许 `PointBatch/VerticalFlush`。

## 范围（v0）
- 加载 `session.json`，展示 `trace.warnings`。
- Canvas 绘制：
  - 全部线段（按 `SegmentId` 固定配色）。
  - 当前 `sweep_x` 的扫描线（垂直线）。
  - `PointBatch` 的事件点（`step.point`），以及本步新增交点与累计交点。
- 高亮：
  - 当前步 `active` 中的线段（加粗/高亮透明度），可选显示“活动顺序编号”。
  - `VerticalFlush`：从 `step.events` 中解析 `Vertical(id)` 并临时高亮这些垂直线段。
- 控制：
  - 上一步/下一步、播放/暂停、速度调节、步数滑条。
  - 键盘快捷键（`Space` 播放暂停，`←/→` 步进）。
- 信息面板：
  - `step.kind / step.sweep_x / step.point`（若有）。
  - `events`、`notes` 文本列表。
  - `active` 列表与 `intersections` 表（含 `kind/a/b/point`）。

## 不包含（v0）
- 在前端直接运行算法（只做回放，不做计算）。
- phase 2（共线重叠“最大重叠段集合”）的可视化。
- 对 `events/notes` 做完整结构化语义解析（v0 仅做少量正则解析用于高亮）。

## 技术栈（确定）
- 渲染：Canvas 2D（推荐双层 canvas：静态线段层 + 动态高亮/扫描线/交点层）。
- 代码：原生 JavaScript（ESM / `<script type="module">`）。
- 页面：原生 HTML + CSS（无框架、无构建、可离线直接打开）。

## 已决定的实现取向
- 先做“纯静态前端（无构建）”：`viewer/index.html` + `viewer/app.js` + `viewer/style.css`，避免 npm/网络依赖，便于快速迭代与分享。
- `Rational` 处理：
  - 渲染：`Number(num) / Number(den)`（允许像素级误差）。
  - 展示：保留精确的 `num/den` 字符串（`den == "1"` 时可显示为整数）。
- 文案与错误提示使用中文；代码标识符保持英文。

## 待办
[x] 明确 `session.v1` 字段顺序与最小校验规则（schema/version、字段缺失时的中文错误提示）。
[x] Rust 侧补齐 `session.v1` 输出能力：把 `Segments` + `Trace` 写成稳定 JSON（字段顺序固定），并提供 `Phase1Output -> session.v1` 的便捷入口。
[x] 新增 `viewer/` 静态页面骨架（页面布局、基础样式）。
[x] 实现 `session.json` 加载（文件选择/拖拽）与解析（含 `Rational`/Point/segments）。
[x] 实现坐标系统与 viewport：适配屏幕、缩放/平移（鼠标滚轮缩放、拖拽平移）。
[x] 实现渲染层：静态层（线段）与动态层（扫描线/点/高亮），避免每帧全量重绘。
[x] 实现回放控制：步进/播放/速度/进度条/快捷键。
[x] 实现侧边信息面板：warnings、step 元信息、events/notes、active、intersections（支持复制 JSON/文本）。
[x] 支持 `VerticalFlush` 高亮：解析 `Vertical(id)` + 显示 `VerticalRange`（从 `notes` 中提取 y 范围）。
[ ] 性能验证：大输入（大量垂直线段、steps 多）下仍能流畅拖动与播放。
  - 目标：构造“点很多、steps 多、垂直命中多”的可复现输入，验证 viewer 的加载/渲染/拖拽/播放不会明显卡顿或崩溃。
  - 统一约定：线段为“长线段”（不做短线段切分），便于聚焦在点数与 steps。
  - L 档参数（先用常量，后续统一改动）：
    - `GRID_N = 100`（正交网格与 45° 网格的每方向线段数）
    - `SPIDER_SPOKES = 64`，`SPIDER_RINGS = 40`（蜘蛛网用例）
  - 规划用例（Rust 生成，写入 `viewer/generated/`，不提交生成结果）：
    - `perf-grid-orthogonal`：`GRID_N` 条水平长线段 + `GRID_N` 条垂直长线段，形成 `GRID_N^2` 个点交（含大量 `VerticalFlush` 命中）。
    - `perf-grid-diagonal-45`：`GRID_N` 条斜率 `+1` 长线段 + `GRID_N` 条斜率 `-1` 长线段，形成大量有理数交点事件（steps 多）。
    - `perf-spider-web`：`SPIDER_SPOKES` 条辐射线 + `SPIDER_RINGS` 圈环（多段折线）构成“蜘蛛网”，包含大量多段同点端点接触（退化压力）。
  - 验收动作（人工）：加载上述 L 档用例，检查缩放/拖拽不卡顿，10fps 播放可用；必要时在 viewer 增加“降载开关”（例如隐藏 active 列表/关闭累计交点）。
  - [ ] Rust：为上述 perf 用例实现确定性生成器（常量 `GRID_N/SPIDER_*`）。
  - [ ] Rust：为 perf 用例增加稳定性回归测试（至少“同参数两次生成字节一致”）。
  - [ ] viewer：可从左侧列表加载 perf 用例并正常回放。
  - [ ] （可选）viewer：增加“降载开关”以便极大输入下仍可操作。
  - [ ] 人工验收并勾选本条（记录机器/浏览器与主观体验）。
[x] 准备 2–3 个可复现示例 `viewer/examples/*.json`（与单测/手工用例一致），用于快速验收与回归。
[x] Rust 示例生成器：输出大量可复现的 `viewer/generated/*.json`，并生成 `viewer/generated/index.json`（稳定顺序，供前端自动加载列表）。
[x] 前端左侧列表：启动时自动加载 `viewer/generated/index.json`（不存在则回退到 `viewer/examples/index.json`），点击条目自动加载 session。
[x] 工程脚本：新增 `pnpm gen:sessions`（生成示例与索引），并把 `viewer/generated/` 加入 `.gitignore`（只提交生成器，不提交生成结果）。
[x] 稳定性验证：为 `session.v1` 与示例生成器增加“字节级稳定”测试（至少覆盖 1–2 个固定样例）。

## 备选路线（不阻塞 v0）
- Rust GUI（`egui/eframe`）：数据与算法同语言同进程，交互更强，但需要引入依赖与打包。
- wasm：把 Rust 算法编译到浏览器侧，做到“输入即计算即回放”，但工程复杂度更高。
