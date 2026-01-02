use crate::geom::intersection::PointIntersectionRecord;
use crate::preprocess::{InputSegmentF64, PreprocessOutput, preprocess_segments};
use crate::sweep::bo::{BoError, enumerate_point_intersections_with_trace};
use crate::trace::Trace;

#[derive(Clone, Debug)]
pub struct Phase1Output {
    pub preprocess: PreprocessOutput,
    pub intersections: Vec<PointIntersectionRecord>,
    pub trace: Trace,
}

/// 第一阶段一站式入口：预处理 + 点交枚举 + trace（含告警）。
pub fn run_phase1(input: &[InputSegmentF64]) -> Result<Phase1Output, BoError> {
    let preprocess = preprocess_segments(input);
    let (intersections, mut trace) =
        enumerate_point_intersections_with_trace(&preprocess.segments)?;

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

