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
- 示例数据在 `viewer/examples/`。

## 操作
- 播放/暂停：`Space`
- 上一步/下一步：`←` / `→`
- 画布平移：按住拖拽
- 缩放：鼠标滚轮（以指针位置为缩放中心）

## 数据格式
`session.v1` 的建议格式见 `plans/trace-visualizer.md`。

