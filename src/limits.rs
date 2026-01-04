use core::fmt;

/// Phase 1 相关的输出规模/执行步数上限。
///
/// 约定：
/// - 任一上限触发即 fail-fast 报错退出；
/// - 不做截断/降采样，避免“看似成功但数据不完整”。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// `session.v2` JSON 的最大字节数（UTF-8）。
    pub max_session_bytes: usize,
    /// `trace.v2.steps` 的最大条目数。
    pub max_trace_steps: usize,
    /// 所有 trace step 的 `active.len()` 之和的上限。
    pub max_trace_active_entries_total: usize,
    /// Phase 1 点交输出（按点聚合）的最大条目数。
    pub max_intersections: usize,
}

impl Default for Limits {
    fn default() -> Self {
        // 默认值与 `plans/phase2-precheck.md` 的 fail-fast 约定保持一致；
        // 其中 `max_intersections` 为 phase1 的额外兜底（phase2-precheck 未单列）。
        Self {
            max_session_bytes: 33_554_432, // 32 MiB
            max_trace_steps: 20_000,
            max_trace_active_entries_total: 3_500_000,
            max_intersections: 200_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitKind {
    SessionBytes,
    TraceSteps,
    TraceActiveEntriesTotal,
    Intersections,
}

impl LimitKind {
    fn label_cn(&self) -> &'static str {
        match self {
            LimitKind::SessionBytes => "session.v2 字节数",
            LimitKind::TraceSteps => "trace 步数（steps）",
            LimitKind::TraceActiveEntriesTotal => "trace 活动集合条目总数（Σ active.len）",
            LimitKind::Intersections => "点交输出条目数（intersections）",
        }
    }

    fn suggestion_cn(&self) -> &'static str {
        match self {
            LimitKind::SessionBytes => {
                "缩小输入规模，或关闭 trace（Phase1Options.trace_enabled=false），或提高 max_session_bytes。"
            }
            LimitKind::TraceSteps => {
                "缩小输入规模，或关闭 trace（Phase1Options.trace_enabled=false），或提高 max_trace_steps。"
            }
            LimitKind::TraceActiveEntriesTotal => {
                "缩小输入规模，或关闭 trace（Phase1Options.trace_enabled=false），或提高 max_trace_active_entries_total。"
            }
            LimitKind::Intersections => {
                "缩小输入规模（减少线段数/用例参数），或提高 max_intersections。"
            }
        }
    }
}

impl fmt::Display for LimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label_cn())
    }
}

/// 触发 fail-fast 上限的错误（包含实际值/上限/建议）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitExceeded {
    pub kind: LimitKind,
    pub limit: usize,
    pub actual: usize,
}

impl LimitExceeded {
    pub fn suggestion_cn(&self) -> &'static str {
        self.kind.suggestion_cn()
    }
}

impl fmt::Display for LimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} 超限：实际={}，上限={}；建议：{}",
            self.kind,
            self.actual,
            self.limit,
            self.suggestion_cn()
        )
    }
}
