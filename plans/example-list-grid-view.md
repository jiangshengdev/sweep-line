# 计划：示例列表支持「列表/网格」切换

目标：为 `viewer/` 左侧「示例列表」增加网格视图（可切换、可持久化），减少纵向滚动，提升选择效率。

## 验收标准

- 示例列表提供视图切换：`列表 / 网格`（默认列表）。
- 切换仅影响示例列表区域，不影响画布与右侧信息面板布局。
- 网格视图为多列卡片布局：在 280px 左侧面板内至少可形成 2 列（随宽度自适应）。
- 分组标题（例如“Rust 生成 / 内置示例”）在网格模式下仍清晰可见，并能占满整行（跨列）。
- 当前已加载的示例在两种视图下都有一致的「激活态」高亮。
- 视图选择写入 `localStorage`，刷新页面后保持。
- 不引入构建依赖，仍保持纯静态离线运行（`cd viewer && python3 -m http.server 8000`）。

## 待办

- [x] 设计 UI：在左侧面板 `刷新` 旁增加 `列表/网格` 切换控件（按钮组或下拉）
- [x] 增加设置项：`appState.settings.sessionListViewMode`（`list|grid`）与对应 `storageKeys`
- [x] 实现持久化：启动时从 `localStorage` 读取，切换时写回并立即生效
- [x] CSS 网格布局：为网格模式增加 `grid-template-columns` 与卡片紧凑样式（标题截断、meta 更紧凑）
- [x] 处理分组标题：网格模式下 `.session-list__group` 跨列显示（`grid-column: 1 / -1`）
- [x] 保持可用性：hover/active/focus-visible 在两种视图下可辨识；键盘 Tab 可正常选择
- [x] 基础校验：`node --check viewer/app.js` + 手工验证切换/持久化/加载示例
