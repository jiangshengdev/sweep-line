# 计划：viewer ESM 模块化重构

目标：在不改变核心功能的前提下，将 `viewer/app.js` 拆分为多个 ESM 模块，提升可读性、可维护性与后续扩展效率；同时整理 `viewer/index.html` / `viewer/style.css` 的结构，并在允许范围内做少量可访问性与交互细节优化。

## 范围
- In：
  - 拆分 `viewer/app.js`：state / schema 解析 / render / ui / controller 分层，落到 `viewer/**.js` 多模块结构。
  - 为关键模块补充 JSDoc 类型标注；必要时在模块顶部加 `// @ts-check`（无需运行时依赖，仅用于编辑器/语言服务提示）。
  - 允许小幅行为调整：改进弹层焦点管理/ARIA、状态提示一致性、错误信息可读性等（不改变数据契约）。
  - 更新 `viewer/README.md`：补充模块结构说明 + 手工回归清单。
- Out：
  - 修改 `session.v1/trace.v1` JSON 协议（字段名/含义/校验语义）。
  - 引入框架、打包构建、第三方依赖（保持纯静态离线）。
  - 大幅 UI 改版或新增复杂功能。

## 模块划分（确定）
- `viewer/app.js`
  - 入口/装配（wire-up）：创建 `elements/appState`，组合各模块，安装事件，启动渲染与索引加载。
- `viewer/lib/*`
  - `storage.js`：`safeStorageGetItem/safeStorageSetItem`
  - `numbers.js`：`clampNumber/roundToHalf`
  - `format.js`：`formatSizeValue`（用于 range 数值展示）
  - `dom.js`：`clearChildren/appendListItems/appendKvLines`
  - `color.js`：`stableColorForSegmentId`
- `viewer/schema/*`
  - `session.js`：`UserError` + `parseSession`（兼容 `session.v1/session.v2` 与 `trace.v1/trace.v2`）
  - `session-index.js`：`parseSessionIndex`（`session-index.v1`）
- `viewer/render/renderer.js`
  - `createRenderer({ elements, appState })`：封装 viewport/camera、world↔canvas 变换、静态/动态层绘制、palette 读取、`requestRender/resize/resetView`。
- `viewer/ui/*`
  - `panels.js`：`refreshUiForSession/refreshUiForStep/updateStepControls/updateSessionListSelection`
  - `session-list.js`：分组/按文件夹渲染（列表/网格），对外提供 `renderSessionListInto(...)`
  - `session-picker.js`：picker 显隐/搜索状态与渲染（复用 `renderSessionListInto`）
- `viewer/controller/*`
  - `settings.js`：加载/应用设置（主题/开关/滑条/列表视图），持久化到 localStorage，并触发 render 标脏。
  - `playback.js`：步进/播放/速度/快捷键（依赖 `panels` 与 `renderer` 提供的回调）。
  - `loaders.js`：统一加载管线：`loadFromFile/loadFromUrl` → `parseSession` → `prepareSessionForPlayback` → 更新 state/ui/render；索引加载 `loadIndexAndRenderList`。

## 行动项
[x] 1. 读取最新 `main` 变更记录，确认 viewer 相关改动点与潜在冲突面（避免重构时回归）。
[x] 2. 设计最终模块边界与导出 API（先定接口，再迁移实现），并回填更新本文件“模块划分”小节。
[x] 3. 新建 `viewer/lib/`：迁移 `safeStorage*`、clamp/round/format 等基础工具，统一状态变更与持久化写入路径。
[x] 4. 新建 `viewer/schema/`：迁移 `UserError`、`parseSession`、`parseSessionIndex`，保持现有中文错误提示语义不变。
[x] 5. 新建 `viewer/render/`：封装 viewport/dpr、变换函数、静态层/动态层绘制与 palette 读取；确保“标脏→requestRender”路径单一。
[x] 6. 新建 `viewer/ui/`：封装 session list/picker 的渲染与筛选；封装右侧面板（meta/warnings/events/notes/active/intersections）刷新。
[x] 7. 新建 `viewer/controller/`：封装 step/playback/快捷键；合并 `loadFromFile`/`loadFromUrl` 到单一 `loadSession(json, source)` 加载入口。
[x] 8. 调整 `viewer/index.html`：在不破坏布局的前提下，完善 dialog/status 的 ARIA 与焦点返回策略（允许小幅行为调整）。
[x] 9. 整理 `viewer/style.css`：按组件分区组织、抽公共工具类（例如 `.hidden`）、减少重复与特例选择器；确保暗/亮主题不回归。
[ ] 10. 手工回归：用 `viewer/examples/` 与 `viewer/generated/`（含 perf 用例）验证加载、拖拽、缩放、播放、列表/搜索、VerticalFlush 高亮、主题与设置持久化；记录发现并补充到本计划。
[x] 11. 收尾：`git status --porcelain` 确认干净、`git rebase main` 同步；回到主工作区 squash 合并（按仓库 worktree 流程）。

## 验证方式（最小）
- 启动：`pnpm start` 或 `cd viewer && python3 -m http.server 8000`
- 浏览器检查：
  - 页面正常渲染，控制台无报错
  - “手工回归”条目逐项通过

## 风险与对策
- 路径/导出变更导致运行时 import 错误：每完成一个模块迁移就用浏览器快速冒烟。
- 循环依赖：底层（lib/schema/render）尽量不依赖 ui/controller；入口统一装配。
- DOM 结构调整导致事件绑定失效：优先保持 `id` 稳定；若必须调整，提供兼容层与回归清单兜底。
