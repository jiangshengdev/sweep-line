# 扫描线回放器（trace.v1/trace.v2）

这是一个离线回放工具：读取 `session.v1/session.v2`（线段 + `trace.v1/trace.v2`），用 Canvas 逐步展示扫描线事件批处理、活动集合顺序与交点输出。

## 启动方式
推荐用本地静态服务启动（因为浏览器对 ESM 的 `file://` 加载限制不一致）：

```bash
cd viewer
python3 -m http.server 8000
```

然后打开：`http://localhost:8000/`

## 加载数据
- 点击页面左上角“加载 session.json”，或把文件拖拽到画布区域。
- 左侧“示例列表”会自动尝试加载：
  - `viewer/generated/index.json`（Rust 生成的示例索引）
  - `viewer/examples/index.json`（仓库内置示例）
- 示例列表与“选择示例”弹层会先按来源分组，再按二级文件夹分区展示（例如 `generated/curated|perf|random`；无二级目录时显示“（根目录）”）。

## 生成示例（Rust）
在仓库根目录运行：

```bash
pnpm gen:sessions
```

会生成 `viewer/generated/*.json` 与 `viewer/generated/index.json`（该目录已加入 `.gitignore`，不提交）。

## 操作
- 播放/暂停：`Space`
- 上一步/下一步：`←` / `→`
- 画布平移：按住拖拽
- 缩放：鼠标滚轮（以指针位置为缩放中心）
- 主题：工具栏选择 `跟随系统 / 浅色 / 深色`（会记住设置）
- 累计交点：工具栏开关“累计交点”（会记住设置）
- 交点大小：工具栏滑条调整“累计大小 / 当前大小”（会记住设置）
- 激活线段：默认不加粗；可用“激活加粗”开关切换（会记住设置）
- 当前执行点：用十字准星标记，避免遮挡交点

## 代码结构（ESM 多模块）
- 入口：`viewer/app.js`（装配 modules、安装事件、启动渲染与索引加载）
- 工具：`viewer/lib/`（storage/dom/format/numbers/color）
- 协议解析：`viewer/schema/`（`session.v1/v2`、`trace.v1/v2`、`session-index.v1`）
- 渲染：`viewer/render/renderer.js`（双层 canvas、viewport/camera、标脏渲染）
- UI：`viewer/ui/`（列表/弹层渲染、右侧面板刷新）
- 控制器：`viewer/controller/`（settings/playback/loaders）

## 开发提示
- 本项目使用 `// @ts-check` + JSDoc 提供编辑器提示；不需要安装任何依赖或构建步骤（VS Code 自带 TS 语言服务即可生效）。
- 由于浏览器对 `file://` 下的 ESM 加载限制不一致，建议始终通过本地静态服务运行（见“启动方式”）。

## 手工回归清单（建议）
- 加载：文件选择与拖拽加载均可用；加载失败时状态栏给出中文错误提示
- 示例：可加载 `generated/index.json` / `examples/index.json`；列表与弹层可按来源+二级目录分组；搜索可过滤
- 回放：上一步/下一步、播放/暂停、速度、步数滑条、快捷键（`Space`/`←`/`→`）可用
- 视图：拖拽平移、滚轮缩放（以指针为中心）、重置视图可用
- 设置：主题/累计交点/交点大小/激活加粗/列表视图切换可记住（localStorage 不可用时不影响核心功能）
- VerticalFlush：包含 `Vertical(id)` + `VerticalRange(...)` 的数据可正确高亮与画端点帽

## 数据格式
`session.v2` 的建议格式见 `plans/trace-visualizer.md`（同时兼容加载旧的 `session.v1`）。
