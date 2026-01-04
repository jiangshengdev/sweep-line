# 计划：viewer 样式重置与兼容性收敛（reset.css）

目标：为 `viewer/` 引入一个轻量的 `reset.css`，把“基础重置/浏览器默认差异收敛”与 `style.css` 的主题/组件样式分离，减少跨浏览器默认样式差异带来的 UI 波动（不引入第三方依赖、不做侵入式重置）。

## 行动项

[x] 1. 评估现有 `viewer/style.css` 中已包含的 reset（`box-sizing`、`body margin`、`html/body height`）。
[x] 2. 新增 `viewer/reset.css`：统一表单控件字体继承、默认 margin、`text-size-adjust`、`canvas` baseline 等。
[x] 3. 调整 `viewer/style.css`：移除基础 reset；补充 Safari 兼容（`-webkit-backdrop-filter`）；为 `:focus-visible` 增加 `:focus` 回退（不改变现代浏览器行为）。
[x] 4. 调整 `viewer/index.html`：在 `style.css` 之前引入 `reset.css`。
[ ] 5. 手工冒烟：Chrome / Firefox / Safari 下检查布局、按钮/输入焦点样式、主题切换与回放交互无回归（记录差异并回补）。

## 设计约束

- 不引入第三方依赖（保持纯静态离线）。
- 不移除列表序号/默认焦点环等“语义默认行为”（避免侵入式 reset）。
