use core::fmt;
use std::collections::BTreeMap;

use crate::geom::fixed::{PointI64, QuantizeError, quantize_coord};
use crate::geom::segment::{Segment, SegmentId, SegmentKey, Segments};

#[derive(Clone, Copy, Debug)]
pub struct InputSegmentF64 {
    pub ax: f64,
    pub ay: f64,
    pub bx: f64,
    pub by: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputCoord {
    Ax,
    Ay,
    Bx,
    By,
}

impl fmt::Display for InputCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputCoord::Ax => write!(f, "起点 x"),
            InputCoord::Ay => write!(f, "起点 y"),
            InputCoord::Bx => write!(f, "终点 x"),
            InputCoord::By => write!(f, "终点 y"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarningKind {
    DroppedInvalidCoordinate { coord: InputCoord, error: QuantizeError },
    DroppedZeroLength,
    DroppedDuplicate { kept_input_index: usize },
}

impl fmt::Display for WarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarningKind::DroppedInvalidCoordinate { coord, error } => {
                write!(f, "已丢弃：{} 无效（{}）", coord, error)
            }
            WarningKind::DroppedZeroLength => write!(f, "已丢弃：零长度线段"),
            WarningKind::DroppedDuplicate { kept_input_index } => {
                write!(f, "已丢弃：与第 {} 条输入重复", kept_input_index)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    pub input_index: usize,
    pub kind: WarningKind,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 条输入：{}", self.input_index, self.kind)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PreprocessOutput {
    pub segments: Segments,
    pub input_to_segment: Vec<Option<SegmentId>>,
    pub warnings: Vec<Warning>,
}

pub fn preprocess_segments(input: &[InputSegmentF64]) -> PreprocessOutput {
    let mut segments = Segments::new();
    let mut warnings = Vec::new();
    let mut input_to_segment = vec![None; input.len()];
    let mut seen: BTreeMap<SegmentKey, (SegmentId, usize)> = BTreeMap::new();

    for (input_index, seg) in input.iter().enumerate() {
        let ax = match quantize_coord(seg.ax) {
            Ok(v) => v,
            Err(error) => {
                warnings.push(Warning {
                    input_index,
                    kind: WarningKind::DroppedInvalidCoordinate {
                        coord: InputCoord::Ax,
                        error,
                    },
                });
                continue;
            }
        };
        let ay = match quantize_coord(seg.ay) {
            Ok(v) => v,
            Err(error) => {
                warnings.push(Warning {
                    input_index,
                    kind: WarningKind::DroppedInvalidCoordinate {
                        coord: InputCoord::Ay,
                        error,
                    },
                });
                continue;
            }
        };
        let bx = match quantize_coord(seg.bx) {
            Ok(v) => v,
            Err(error) => {
                warnings.push(Warning {
                    input_index,
                    kind: WarningKind::DroppedInvalidCoordinate {
                        coord: InputCoord::Bx,
                        error,
                    },
                });
                continue;
            }
        };
        let by = match quantize_coord(seg.by) {
            Ok(v) => v,
            Err(error) => {
                warnings.push(Warning {
                    input_index,
                    kind: WarningKind::DroppedInvalidCoordinate {
                        coord: InputCoord::By,
                        error,
                    },
                });
                continue;
            }
        };

        let a = PointI64 { x: ax, y: ay };
        let b = PointI64 { x: bx, y: by };
        let key = SegmentKey::new(a, b);

        if key.a == key.b {
            warnings.push(Warning {
                input_index,
                kind: WarningKind::DroppedZeroLength,
            });
            continue;
        }

        if let Some((_kept_id, kept_input_index)) = seen.get(&key) {
            warnings.push(Warning {
                input_index,
                kind: WarningKind::DroppedDuplicate {
                    kept_input_index: *kept_input_index,
                },
            });
            continue;
        }

        let id = segments.push(Segment {
            a: key.a,
            b: key.b,
            source_index: input_index,
        });
        seen.insert(key, (id, input_index));
        input_to_segment[input_index] = Some(id);
    }

    PreprocessOutput {
        segments,
        input_to_segment,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_endpoints() {
        let input = [InputSegmentF64 {
            ax: 0.5,
            ay: 0.5,
            bx: -0.5,
            by: 0.5,
        }];
        let out = preprocess_segments(&input);

        assert_eq!(out.segments.len(), 1);
        let seg = out.segments.get(SegmentId(0));
        assert!(seg.a <= seg.b);
        assert_eq!(seg.source_index, 0);
        assert_eq!(out.input_to_segment, vec![Some(SegmentId(0))]);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn drops_zero_length_and_duplicates_with_stable_warnings() {
        let input = [
            InputSegmentF64 {
                ax: 0.0,
                ay: 0.0,
                bx: 0.0,
                by: 0.0,
            },
            InputSegmentF64 {
                ax: 0.0,
                ay: 0.0,
                bx: 1.0,
                by: 0.0,
            },
            InputSegmentF64 {
                ax: 1.0,
                ay: 0.0,
                bx: 0.0,
                by: 0.0,
            },
        ];
        let out = preprocess_segments(&input);

        assert_eq!(out.segments.len(), 1);
        assert_eq!(out.input_to_segment, vec![None, Some(SegmentId(0)), None]);
        assert_eq!(
            out.warnings,
            vec![
                Warning {
                    input_index: 0,
                    kind: WarningKind::DroppedZeroLength
                },
                Warning {
                    input_index: 2,
                    kind: WarningKind::DroppedDuplicate { kept_input_index: 1 }
                }
            ]
        );
    }

    #[test]
    fn drops_out_of_range() {
        let input = [InputSegmentF64 {
            ax: 2.0,
            ay: 0.0,
            bx: 0.0,
            by: 0.0,
        }];
        let out = preprocess_segments(&input);
        assert_eq!(out.segments.len(), 0);
        assert_eq!(out.input_to_segment, vec![None]);
        assert_eq!(
            out.warnings,
            vec![Warning {
                input_index: 0,
                kind: WarningKind::DroppedInvalidCoordinate {
                    coord: InputCoord::Ax,
                    error: QuantizeError::OutOfRange
                }
            }]
        );
    }
}

