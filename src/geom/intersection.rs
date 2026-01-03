use core::fmt;

use crate::geom::fixed::PointI64;
use crate::geom::point::PointRat;
use crate::geom::predicates::{on_segment, orient};
use crate::geom::segment::{Segment, SegmentId};
use crate::rational::Rational;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointIntersectionKind {
    /// 两条线段在各自的内部相交（交点不在任何端点上）。
    Proper,
    /// 交点落在至少一条线段的端点上（包含端点-端点、端点-内部）。
    EndpointTouch,
}

impl fmt::Display for PointIntersectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PointIntersectionKind::Proper => write!(f, "Proper"),
            PointIntersectionKind::EndpointTouch => write!(f, "EndpointTouch"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointIntersectionRecord {
    pub point: PointRat,
    pub kind: PointIntersectionKind,
    pub a: SegmentId,
    pub b: SegmentId,
}

/// 按“点”聚合后的交点记录：同一几何点只输出一次。
///
/// 说明：
/// - `endpoint_segments`：在该点以端点参与的线段集合（去重、升序）。
/// - `interior_segments`：在该点以内部点参与的线段集合（去重、升序）。
/// - 该结构用于避免同点多线段相交时输出 `O(k^2)` 的 pair 列表（见 `plans/src-code-review-findings.md` 的 #3 方案 B）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointIntersectionGroupRecord {
    pub point: PointRat,
    pub endpoint_segments: Vec<SegmentId>,
    pub interior_segments: Vec<SegmentId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentIntersection {
    /// 唯一的点交（第一阶段的主要输出）。
    Point {
        point: PointRat,
        kind: PointIntersectionKind,
    },
    /// 共线且存在长度>0 的重叠段（第二阶段输出“最大重叠段集合”时处理）。
    CollinearOverlap,
}

/// 计算两条（非零长度）线段的交集：
/// - 空集：返回 `None`
/// - 点交：返回 `Some(SegmentIntersection::Point{...})`
/// - 共线重叠段：返回 `Some(SegmentIntersection::CollinearOverlap)`
///
/// 说明：
/// - 完全重合/部分重叠属于“无穷多个交点”，第一阶段不展开枚举，用 `CollinearOverlap` 占位；
/// - 端点接触也算交点，并归类为 `EndpointTouch`，便于前端用不同颜色区分。
pub fn intersect_segments(a: &Segment, b: &Segment) -> Option<SegmentIntersection> {
    let p1 = a.a;
    let p2 = a.b;
    let q1 = b.a;
    let q2 = b.b;

    let o1 = orient(p1, p2, q1);
    let o2 = orient(p1, p2, q2);
    let o3 = orient(q1, q2, p1);
    let o4 = orient(q1, q2, p2);

    // 共线：优先处理，避免把重叠段误当作“端点命中”。
    if o1 == 0 && o2 == 0 && o3 == 0 && o4 == 0 {
        return collinear_intersection(p1, p2, q1, q2).map(|p| match p {
            CollinearResult::Point(pt) => point_intersection(pt, a, b),
            CollinearResult::Overlap => SegmentIntersection::CollinearOverlap,
        });
    }

    // 端点落在线段上（非共线的退化点交）。
    if o1 == 0 && on_segment(p1, p2, q1) {
        return Some(point_intersection(q1, a, b));
    }
    if o2 == 0 && on_segment(p1, p2, q2) {
        return Some(point_intersection(q2, a, b));
    }
    if o3 == 0 && on_segment(q1, q2, p1) {
        return Some(point_intersection(p1, a, b));
    }
    if o4 == 0 && on_segment(q1, q2, p2) {
        return Some(point_intersection(p2, a, b));
    }

    // 一般情况：严格相交（内部-内部）。
    if has_opposite_sign(o1, o2) && has_opposite_sign(o3, o4) {
        let point = line_intersection_point(p1, p2, q1, q2);
        let kind = classify_point(point, a, b);
        return Some(SegmentIntersection::Point { point, kind });
    }

    None
}

fn point_intersection(point: PointI64, a: &Segment, b: &Segment) -> SegmentIntersection {
    let point = PointRat::from_i64(point);
    let kind = classify_point(point, a, b);
    SegmentIntersection::Point { point, kind }
}

fn classify_point(point: PointRat, a: &Segment, b: &Segment) -> PointIntersectionKind {
    let a0 = PointRat::from_i64(a.a);
    let a1 = PointRat::from_i64(a.b);
    let b0 = PointRat::from_i64(b.a);
    let b1 = PointRat::from_i64(b.b);
    if point == a0 || point == a1 || point == b0 || point == b1 {
        PointIntersectionKind::EndpointTouch
    } else {
        PointIntersectionKind::Proper
    }
}

fn has_opposite_sign(a: i128, b: i128) -> bool {
    (a > 0 && b < 0) || (a < 0 && b > 0)
}

fn cross(ax: i128, ay: i128, bx: i128, by: i128) -> i128 {
    ax * by - ay * bx
}

fn line_intersection_point(p1: PointI64, p2: PointI64, q1: PointI64, q2: PointI64) -> PointRat {
    let x1 = p1.x as i128;
    let y1 = p1.y as i128;
    let rx = (p2.x as i128) - x1;
    let ry = (p2.y as i128) - y1;

    let sx = (q2.x as i128) - (q1.x as i128);
    let sy = (q2.y as i128) - (q1.y as i128);

    let qpx = (q1.x as i128) - x1;
    let qpy = (q1.y as i128) - y1;

    let denom = cross(rx, ry, sx, sy);
    debug_assert!(denom != 0, "非平行线段的交点计算不应出现 denom=0");

    let t_num = cross(qpx, qpy, sx, sy);
    let x_num = x1 * denom + rx * t_num;
    let y_num = y1 * denom + ry * t_num;

    PointRat {
        x: Rational::new(x_num, denom),
        y: Rational::new(y_num, denom),
    }
}

enum CollinearResult {
    Point(PointI64),
    Overlap,
}

fn collinear_intersection(
    p1: PointI64,
    p2: PointI64,
    q1: PointI64,
    q2: PointI64,
) -> Option<CollinearResult> {
    let vertical = p1.x == p2.x;
    debug_assert_eq!(
        vertical,
        q1.x == q2.x,
        "共线线段要么都垂直，要么都非垂直"
    );

    if vertical {
        let p_min = p1.y.min(p2.y);
        let p_max = p1.y.max(p2.y);
        let q_min = q1.y.min(q2.y);
        let q_max = q1.y.max(q2.y);

        let lo = p_min.max(q_min);
        let hi = p_max.min(q_max);
        if lo > hi {
            return None;
        }
        if lo == hi {
            let point = find_endpoint_with_y([p1, p2, q1, q2], lo)?;
            debug_assert!(on_segment(p1, p2, point) && on_segment(q1, q2, point));
            return Some(CollinearResult::Point(point));
        }
        return Some(CollinearResult::Overlap);
    }

    let p_min = p1.x.min(p2.x);
    let p_max = p1.x.max(p2.x);
    let q_min = q1.x.min(q2.x);
    let q_max = q1.x.max(q2.x);

    let lo = p_min.max(q_min);
    let hi = p_max.min(q_max);
    if lo > hi {
        return None;
    }
    if lo == hi {
        let point = find_endpoint_with_x([p1, p2, q1, q2], lo)?;
        debug_assert!(on_segment(p1, p2, point) && on_segment(q1, q2, point));
        return Some(CollinearResult::Point(point));
    }
    Some(CollinearResult::Overlap)
}

fn find_endpoint_with_x(points: [PointI64; 4], x: i64) -> Option<PointI64> {
    points.into_iter().find(|p| p.x == x)
}

fn find_endpoint_with_y(points: [PointI64; 4], y: i64) -> Option<PointI64> {
    points.into_iter().find(|p| p.y == y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(ax: i64, ay: i64, bx: i64, by: i64, source_index: usize) -> Segment {
        Segment {
            a: PointI64 { x: ax, y: ay },
            b: PointI64 { x: bx, y: by },
            source_index,
        }
    }

    #[test]
    fn detects_proper_intersection() {
        let a = seg(0, 0, 10, 10, 0);
        let b = seg(0, 10, 10, 0, 1);

        let hit = intersect_segments(&a, &b).unwrap();
        assert_eq!(
            hit,
            SegmentIntersection::Point {
                point: PointRat {
                    x: Rational::from_int(5),
                    y: Rational::from_int(5),
                },
                kind: PointIntersectionKind::Proper,
            }
        );
    }

    #[test]
    fn detects_endpoint_touch_shared_endpoint() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(10, 0, 10, 10, 1);

        let hit = intersect_segments(&a, &b).unwrap();
        assert_eq!(
            hit,
            SegmentIntersection::Point {
                point: PointRat {
                    x: Rational::from_int(10),
                    y: Rational::from_int(0),
                },
                kind: PointIntersectionKind::EndpointTouch,
            }
        );
    }

    #[test]
    fn detects_endpoint_touch_endpoint_on_interior() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(5, 0, 5, 10, 1);

        let hit = intersect_segments(&a, &b).unwrap();
        assert_eq!(
            hit,
            SegmentIntersection::Point {
                point: PointRat {
                    x: Rational::from_int(5),
                    y: Rational::from_int(0),
                },
                kind: PointIntersectionKind::EndpointTouch,
            }
        );
    }

    #[test]
    fn computes_rational_intersection_point() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(5, -5, 6, 5, 1);

        let hit = intersect_segments(&a, &b).unwrap();
        assert_eq!(
            hit,
            SegmentIntersection::Point {
                point: PointRat {
                    x: Rational::new(11, 2),
                    y: Rational::from_int(0),
                },
                kind: PointIntersectionKind::Proper,
            }
        );
    }

    #[test]
    fn reports_collinear_overlap() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(5, 0, 15, 0, 1);
        assert_eq!(
            intersect_segments(&a, &b).unwrap(),
            SegmentIntersection::CollinearOverlap
        );
    }

    #[test]
    fn reports_collinear_touch_as_point() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(10, 0, 20, 0, 1);
        assert_eq!(
            intersect_segments(&a, &b).unwrap(),
            SegmentIntersection::Point {
                point: PointRat {
                    x: Rational::from_int(10),
                    y: Rational::from_int(0),
                },
                kind: PointIntersectionKind::EndpointTouch,
            }
        );
    }

    #[test]
    fn reports_vertical_collinear_overlap() {
        let a = seg(0, 0, 0, 10, 0);
        let b = seg(0, 5, 0, 15, 1);
        assert_eq!(
            intersect_segments(&a, &b).unwrap(),
            SegmentIntersection::CollinearOverlap
        );
    }

    #[test]
    fn returns_none_for_parallel_disjoint() {
        let a = seg(0, 0, 10, 0, 0);
        let b = seg(0, 1, 10, 1, 1);
        assert!(intersect_segments(&a, &b).is_none());
    }
}
