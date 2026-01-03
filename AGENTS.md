# 仓库约定（给 Codex / 自动化助手）

## 语言规范
- 用户可见输出（CLI 输出、错误信息、日志、trace 字段说明等）优先使用中文。
- 代码注释与文档（含 `README.md`、`plans/`、`docs/` 等）使用中文撰写。
- 代码标识符（类型/函数/变量/模块名）保持英文，避免拼音；必要时用中文注释解释含义。

## 命名规范
- 仓库内文件名与目录名使用英文（推荐 `kebab-case`），避免中文文件名。

## 计划执行规范
- 如果本次工作是以 `plans/*.md` 中的清单为起点开展：每完成一项可验证的工作，就在对应计划文档中将该条目从 `[ ]` 更新为 `[x]`。
- 若工作内容超出计划：先在对应计划文档中补充新的 `[ ]` 条目，再开始实现，避免计划与实现脱节。

## Git worktree 工作流（多代理隔离）
目标：本地多个代理/并行任务时，避免互相污染工作区；主工作区保持干净，任务在独立 worktree 中完成，合并到 `main` 时使用 squash 保持历史整洁。

### 约定
- 任务开发不要直接在主工作区（`main` worktree）改代码；每个任务使用独立分支 + 独立 worktree。
- worktree 统一放在仓库内的 `worktrees/` 目录（已在 `.gitignore` 中忽略）。
- 分支命名建议：`feat/<task>` / `fix/<task>` / `chore/<task>`；`<task>` 用英文 `kebab-case`。
- 默认不删除分支（便于复盘/审查）；仅删除 worktree 目录。

### 完整流程（新建 worktree → 做任务 → squash 合并 → 删除 worktree）
1) 主工作区准备（在仓库根目录执行）
   - 确认干净：`git status --porcelain` 为空
   - 确保目录存在：`mkdir -p worktrees`

2) 新建任务 worktree（从 `main` 分出新分支）
   - `git worktree add worktrees/<task> -b <branch> main`
   - `cd worktrees/<task>`

3) 在 worktree 中执行任务（按计划逐步提交）
   - 新建/更新计划：`plans/<task>.md`
   - 每完成一个可验证步骤：把计划条目从 `[ ]` 改为 `[x]`，并提交 1 个 commit
   - 期间按项目习惯运行最小验证（例如 `node --check viewer/app.js` / `cargo test` 等）

4) 结束前同步 `main`（在任务 worktree 内）
   - `git rebase main`
   - 解决冲突后确保干净：`git status --porcelain` 为空

5) squash 合并到 `main`（回到主工作区/仓库根目录）
   - `cd <repo-root>`
   - `git checkout main`
   - `git status --porcelain` 为空
   - `git merge --squash <branch>`
   - 运行验证（同上）
   - `git commit -m "<一句话总结改动>"`

6) 删除 worktree（不删分支）
   - `git worktree remove worktrees/<task>`
   - （可选）清理记录：`git worktree prune`

### 复用分支（可选）
如果需要继续某个已有分支但 worktree 已删除：
- `git worktree add worktrees/<task> <branch>`
