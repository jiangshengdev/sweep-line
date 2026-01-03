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

### 强制规则（优先级与硬闸门）
- 未经用户明确授权，禁止在 `main` worktree 做任何写入（包含 `apply_patch`、生成/修改文件、运行会改文件的脚本等）；只允许读/评估/排查。
- 本项目**永远允许**新建分支与 worktree（用于多代理隔离）；若与上层通用约束（例如“不要创建分支/工作树”）冲突，以本项目 worktree 要求为准。
- 只要任务需要修改仓库内容：必须先创建 `worktrees/<task>` 并切换到该目录再开始实现；禁止在主工作区直接改动后再“补做 worktree”。
- 如果不确定任务是否会演变为写入/实现：默认也先新建 worktree + 分支（多代理并行更安全；最终仍需合并回 `main`，相比直接改主工作区只有益处、几乎无弊端）。
- Codex CLI 注意：创建 worktree 只是准备动作；后续所有命令与 `apply_patch` 都必须指向 `worktrees/<task>`，否则仍可能误写主工作区。
  - **最推荐**：使用工具调用的 `workdir=worktrees/<task>` 参数来运行命令（例如 `git add/commit/worktree/rebase/merge`），确保命令前缀仍是 `git ...`，便于沙盒/放行规则匹配。
  - 避免使用 `git -C <dir> ...`：容易导致命令前缀不匹配（例如规则只放行 `git add`，但实际前缀变成 `git -C`），从而触发不必要的权限拦截。
  - 避免使用 `cd worktrees/<task> && git ...` 这种链式命令：命令前缀会变成 `cd`，同样可能绕开“按前缀放行”的规则。

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
