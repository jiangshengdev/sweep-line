# 扫描线回放器（trace.v1）

这是一个离线回放工具：读取 `session.v1`（线段 + `trace.v1`），用 Canvas 逐步展示扫描线事件批处理、活动集合顺序与交点输出。

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

## 数据格式
`session.v1` 的建议格式见 `plans/trace-visualizer.md`。
