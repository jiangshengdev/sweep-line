# 计划：viewer reset.css 引入后的样式回归修复

背景：引入 `viewer/reset.css` 后，出现示例列表导致整页高度异常、滚动后无法回到主界面、列表编号/长数字溢出等问题，需要收敛为“页面不滚动、面板内滚动”的应用式布局，并统一列表的跨浏览器表现。

## 目标

- 整页不再因为示例列表内容变长而拉高（滚动应发生在左/右侧面板内部）。
- `events` 等列表的编号不再溢出左侧，长数字/长 token 不再把布局撑坏。
- 不引入第三方依赖；尽量保持现有 UI 风格，仅做必要的兼容性收敛。

## 行动项

[x] 1. 复现并定位“整页变高”的根因（`html/body` 高度、overflow、grid item 的 `min-height` 行为）。
[x] 2. 修复布局约束：确保 `html/body` 有确定高度且禁止页面滚动；grid 子项可收缩并在面板内滚动。
[x] 3. 修复列表溢出：`events/notes/warnings/active` 的编号与长数字换行策略跨浏览器一致。
[x] 4. 复查 `session-list` 分组（`session-list__folder`）在 list/grid 两种视图下的布局与溢出行为，必要时补充防御性样式。
[ ] 5. 手工冒烟：加载包含大量示例的 `generated/index.json`，验证“示例列表滚动 / 主界面可见 / events 不溢出”。

## 定位结果（简要）

- 主要问题：`reset.css` 中只设置了 `body { min-height: 100% }`，但没有给 `body` 一个确定高度，导致 `body` 仍会随内容增长；示例列表项一多就把整页高度拉高，滚动发生在页面本身而不是面板内部。
- 修复策略：让 `html/body` 具备确定高度并禁止页面滚动；同时让 grid 子项可收缩（`min-height: 0`），把滚动限制在 `.list-panel/.side-panel` 内。
