use core::cmp::Ordering;
use core::fmt;

use crate::geom::segment::{Segment, SegmentId, Segments};
use crate::rational::Rational;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentOrderError {
    ArithmeticOverflow { operation: &'static str },
}

impl fmt::Display for SegmentOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentOrderError::ArithmeticOverflow { operation } => {
                write!(f, "算术溢出：{}", operation)
            }
        }
    }
}

pub fn y_at_x(segment: &Segment, sweep_x: Rational) -> Result<Rational, SegmentOrderError> {
    debug_assert!(
        !segment.is_vertical(),
        "垂直线段不应进入状态结构的 y_at_x 计算"
    );

    let x1 = segment.a.x as i128;
    let y1 = segment.a.y as i128;
    let dx = (segment.b.x as i128) - x1;
    let dy = (segment.b.y as i128) - (segment.a.y as i128);

    debug_assert!(dx != 0, "非垂直线段的 dx 不应为 0");
    debug_assert!(dx > 0, "线段端点应已规范化为 a.x < b.x（非垂直）");

    let p = sweep_x.num();
    let q = sweep_x.den();
    debug_assert!(q > 0, "Rational 约定分母恒为正");

    // y(x) = (y1*q*dx + dy*(p - x1*q)) / (q*dx)
    let y1_q = y1
        .checked_mul(q)
        .ok_or(SegmentOrderError::ArithmeticOverflow { operation: "y1*q" })?;
    let y1_q_dx = y1_q
        .checked_mul(dx)
        .ok_or(SegmentOrderError::ArithmeticOverflow {
            operation: "y1*q*dx",
        })?;

    let x1_q = x1
        .checked_mul(q)
        .ok_or(SegmentOrderError::ArithmeticOverflow { operation: "x1*q" })?;
    let p_minus_x1q = p
        .checked_sub(x1_q)
        .ok_or(SegmentOrderError::ArithmeticOverflow {
            operation: "p - x1*q",
        })?;
    let dy_term = dy
        .checked_mul(p_minus_x1q)
        .ok_or(SegmentOrderError::ArithmeticOverflow {
            operation: "dy*(p - x1*q)",
        })?;

    let numerator = y1_q_dx
        .checked_add(dy_term)
        .ok_or(SegmentOrderError::ArithmeticOverflow {
            operation: "y1*q*dx + dy*(p - x1*q)",
        })?;

    let denominator = q
        .checked_mul(dx)
        .ok_or(SegmentOrderError::ArithmeticOverflow { operation: "q*dx" })?;

    Ok(Rational::new(numerator, denominator))
}

pub fn slope(segment: &Segment) -> Rational {
    debug_assert!(
        !segment.is_vertical(),
        "垂直线段不应进入状态结构的 slope 计算"
    );

    let dx = (segment.b.x as i128) - (segment.a.x as i128);
    let dy = (segment.b.y as i128) - (segment.a.y as i128);
    debug_assert!(dx > 0, "线段端点应已规范化为 a.x < b.x（非垂直）");
    Rational::new(dy, dx)
}

/// 比较两条（非垂直）线段在 `sweep_x` 事件点右侧 `x+ε` 处的垂直顺序。
///
/// 约定：
/// - 先比较 `y_at_x`；
/// - 若相等，使用斜率（`dy/dx`）决定 `x+ε` 的上下顺序；
/// - 若仍相等（共线/重叠等情况），用 `SegmentId` 兜底确保全序与稳定性。
pub fn cmp_segments_at_x_plus_epsilon(
    segments: &Segments,
    a_id: SegmentId,
    b_id: SegmentId,
    sweep_x: Rational,
) -> Result<Ordering, SegmentOrderError> {
    if a_id == b_id {
        return Ok(Ordering::Equal);
    }

    let a = segments.get(a_id);
    let b = segments.get(b_id);

    debug_assert!(
        !a.is_vertical() && !b.is_vertical(),
        "垂直线段不应进入状态结构比较器"
    );

    let y_a = y_at_x(a, sweep_x)?;
    let y_b = y_at_x(b, sweep_x)?;
    match y_a.cmp(&y_b) {
        Ordering::Equal => {}
        ord => return Ok(ord),
    }

    let slope_a = slope(a);
    let slope_b = slope(b);
    match slope_a.cmp(&slope_b) {
        Ordering::Equal => {}
        ord => return Ok(ord),
    }

    Ok(a_id.cmp(&b_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;
    use crate::geom::segment::Segments;

    #[test]
    fn y_at_x_handles_rational_x() {
        let mut segments = Segments::new();
        let id = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 0,
        });

        let x = Rational::new(1, 2);
        let y = y_at_x(segments.get(id), x).unwrap();
        assert_eq!(y, Rational::new(1, 2));
    }

    #[test]
    fn y_at_x_returns_error_on_overflow() {
        let segment = Segment {
            a: PointI64 { x: 0, y: i64::MAX },
            b: PointI64 { x: 1, y: i64::MAX },
            source_index: 0,
        };
        let x = Rational::new(1, 10_i128.pow(20));
        assert_eq!(
            y_at_x(&segment, x).unwrap_err(),
            SegmentOrderError::ArithmeticOverflow { operation: "y1*q" }
        );
    }

    #[test]
    fn tie_breaks_by_slope_at_intersection() {
        let mut segments = Segments::new();
        let up = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 0,
        });
        let down = segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 1,
        });

        let x = Rational::from_int(5);
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, down, up, x).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, up, down, x).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn tie_breaks_by_segment_id_when_collinear() {
        let mut segments = Segments::new();
        let a = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let b = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 20, y: 0 },
            source_index: 1,
        });

        let x = Rational::from_int(0);
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, a, b, x).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, b, a, x).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn tie_breaks_by_slope_at_shared_start_point() {
        let mut segments = Segments::new();
        let flat = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let up = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 1,
        });

        let x = Rational::from_int(0);
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, flat, up, x).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            cmp_segments_at_x_plus_epsilon(&segments, up, flat, x).unwrap(),
            Ordering::Greater
        );
    }
}
