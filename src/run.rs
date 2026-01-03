use crate::geom::intersection::PointIntersectionGroupRecord;
use crate::limits::{LimitExceeded, Limits};
use crate::preprocess::{InputSegmentF64, PreprocessOutput, preprocess_segments};
use crate::session::{session_v2_to_json_string, session_v2_to_json_string_limited};
use crate::sweep::bo::{
    BoError, enumerate_point_intersections_with_limits, enumerate_point_intersections_with_trace_and_limits,
};
use crate::trace::Trace;

#[derive(Clone, Debug)]
pub struct Phase1Options {
    /// 是否生成 `trace.v2.steps`（对大规模用例可关闭以降低输出与内存占用）。
    pub trace_enabled: bool,
    /// 输出规模/执行步数上限（任一触发即 fail-fast）。
    pub limits: Limits,
}

impl Default for Phase1Options {
    fn default() -> Self {
        Self {
            trace_enabled: true,
            limits: Limits::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Phase1Output {
    pub preprocess: PreprocessOutput,
    pub intersections: Vec<PointIntersectionGroupRecord>,
    pub trace: Trace,
}

/// 第一阶段一站式入口：预处理 + 点交枚举 + trace（含告警）。
pub fn run_phase1(input: &[InputSegmentF64]) -> Result<Phase1Output, BoError> {
    run_phase1_with_options(input, &Phase1Options::default())
}

pub fn run_phase1_with_options(
    input: &[InputSegmentF64],
    options: &Phase1Options,
) -> Result<Phase1Output, BoError> {
    let preprocess = preprocess_segments(input);
    let (intersections, mut trace) = if options.trace_enabled {
        enumerate_point_intersections_with_trace_and_limits(&preprocess.segments, options.limits)?
    } else {
        let intersections = enumerate_point_intersections_with_limits(&preprocess.segments, options.limits)?;
        (intersections, Trace::default())
    };

    trace.warnings = preprocess
        .warnings
        .iter()
        .map(|w| w.to_string())
        .collect();

    Ok(Phase1Output {
        preprocess,
        intersections,
        trace,
    })
}

impl Phase1Output {
    /// 将 phase1 结果打包为 `session.v2` JSON（可直接喂给 `viewer/` 回放器）。
    pub fn to_session_json_string(&self) -> String {
        session_v2_to_json_string(&self.preprocess.segments, &self.trace)
    }

    /// 将 phase1 结果打包为 `session.v2` JSON，并检查 `limits.max_session_bytes`（超限则报错）。
    pub fn to_session_json_string_limited(&self, limits: Limits) -> Result<String, LimitExceeded> {
        session_v2_to_json_string_limited(&self.preprocess.segments, &self.trace, limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_preprocess_warnings_in_trace_json() {
        let input = [InputSegmentF64 {
            ax: 0.0,
            ay: 0.0,
            bx: 2.0,
            by: 0.0,
        }];
        let out = run_phase1(&input).unwrap();
        assert_eq!(out.preprocess.segments.len(), 0);
        assert_eq!(out.preprocess.warnings.len(), 1);
        assert_eq!(out.trace.warnings.len(), 1);

        let json = out.trace.to_json_string();
        assert!(json.contains("\"warnings\":[\"第 0 条输入："));
    }
}
