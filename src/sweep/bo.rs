use std::collections::BTreeSet;

use crate::geom::intersection::{
    PointIntersectionKind, PointIntersectionRecord, SegmentIntersection, intersect_segments,
};
use crate::geom::point::PointRat;
use crate::geom::segment::{SegmentId, Segments};
use crate::rational::Rational;
use crate::sweep::event_queue::{Event, EventQueue};
use crate::sweep::status::{SweepStatus, SweepStatusError, TreapSweepStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoError {
    Status(SweepStatusError),
}

impl From<SweepStatusError> for BoError {
    fn from(value: SweepStatusError) -> Self {
        BoError::Status(value)
    }
}

/// 第一阶段：枚举点交（包含端点接触）。
///
/// 说明：
/// - 当前版本只覆盖非垂直线段；垂直线段路径会在下一步补齐；
/// - 对共线重叠只返回占位，不输出“重叠段”（第二阶段再做）。
pub fn enumerate_point_intersections(segments: &Segments) -> Result<Vec<PointIntersectionRecord>, BoError> {
    let mut queue = EventQueue::new();
    for id in 0..segments.len() {
        let id = SegmentId(id);
        let seg = segments.get(id);
        queue.push(PointRat::from_i64(seg.a), Event::SegmentStart { segment: id });
        queue.push(PointRat::from_i64(seg.b), Event::SegmentEnd { segment: id });
    }

    let mut status = TreapSweepStatus::new(Rational::from_int(0));
    let mut scheduled: BTreeSet<(PointRat, SegmentId, SegmentId)> = BTreeSet::new();
    let mut pending_vertical: BTreeSet<SegmentId> = BTreeSet::new();
    let mut pending_x: Option<Rational> = None;
    let mut out: Vec<PointIntersectionRecord> = Vec::new();

    while let Some((point, events)) = queue.pop_next_batch() {
        if let Some(x) = pending_x {
            if point.x != x {
                flush_vertical_hits(segments, &status, &pending_vertical, &mut out);
                pending_vertical.clear();
                pending_x = None;
            }
        }
        if pending_x.is_none() {
            pending_x = Some(point.x);
        }

        status.set_sweep_x(point.x);

        // 基础覆盖：同一事件点上“作为端点出现”的线段两两之间一定相交于该点。
        // 这能补齐例如 “一条线段在此结束、另一条线段在此开始” 的端点接触情形。
        record_endpoint_pairs(point, &events, &mut out);

        for event in events {
            match event {
                Event::SegmentEnd { segment } => {
                    if segments.get(segment).is_vertical() {
                        continue;
                    }

                    let pred = status.pred(segment);
                    let succ = status.succ(segment);
                    status.remove(segment)?;

                    if let (Some(a), Some(b)) = (pred, succ) {
                        schedule_or_record_pair(
                            segments,
                            &mut queue,
                            &mut scheduled,
                            point,
                            a,
                            b,
                            &mut out,
                        );
                    }
                }
                Event::SegmentStart { segment } => {
                    if segments.get(segment).is_vertical() {
                        pending_vertical.insert(segment);
                        continue;
                    }

                    status.insert(segments, segment)?;

                    if let Some(pred) = status.pred(segment) {
                        schedule_or_record_pair(
                            segments,
                            &mut queue,
                            &mut scheduled,
                            point,
                            pred,
                            segment,
                            &mut out,
                        );
                    }
                    if let Some(succ) = status.succ(segment) {
                        schedule_or_record_pair(
                            segments,
                            &mut queue,
                            &mut scheduled,
                            point,
                            segment,
                            succ,
                            &mut out,
                        );
                    }
                }
                Event::Intersection { a, b } => {
                    if let Some(SegmentIntersection::Point { point: ip, kind }) =
                        intersect_segments(segments.get(a), segments.get(b))
                    {
                        out.push(PointIntersectionRecord {
                            point: ip,
                            kind,
                            a: a.min(b),
                            b: a.max(b),
                        });
                    }

                    // 通过 remove+insert 让 (a,b) 在 `x+ε` 的顺序恢复为比较器决定的稳定全序。
                    status.reorder_segments(segments, &[a, b])?;

                    for id in [a, b] {
                        if let Some(pred) = status.pred(id) {
                            schedule_or_record_pair(
                                segments,
                                &mut queue,
                                &mut scheduled,
                                point,
                                pred,
                                id,
                                &mut out,
                            );
                        }
                        if let Some(succ) = status.succ(id) {
                            schedule_or_record_pair(
                                segments,
                                &mut queue,
                                &mut scheduled,
                                point,
                                id,
                                succ,
                                &mut out,
                            );
                        }
                    }
                }
            }
        }
    }

    flush_vertical_hits(segments, &status, &pending_vertical, &mut out);
    Ok(out)
}

fn flush_vertical_hits(
    segments: &Segments,
    status: &impl SweepStatus,
    vertical: &BTreeSet<SegmentId>,
    out: &mut Vec<PointIntersectionRecord>,
) {
    if vertical.is_empty() || status.is_empty() {
        return;
    }

    for &v_id in vertical {
        let v = segments.get(v_id);
        debug_assert!(v.is_vertical(), "flush_vertical_hits 仅应处理垂直线段");

        let y_min = Rational::from_int(v.a.y.min(v.b.y) as i128);
        let y_max = Rational::from_int(v.a.y.max(v.b.y) as i128);

        let candidates = status.range_by_y(segments, y_min, y_max);
        for s_id in candidates {
            let Some(SegmentIntersection::Point { point, kind }) =
                intersect_segments(v, segments.get(s_id))
            else {
                continue;
            };

            let (a, b) = if v_id <= s_id { (v_id, s_id) } else { (s_id, v_id) };
            out.push(PointIntersectionRecord { point, kind, a, b });
        }
    }
}

fn record_endpoint_pairs(point: PointRat, events: &[Event], out: &mut Vec<PointIntersectionRecord>) {
    let mut ids: Vec<SegmentId> = events
        .iter()
        .filter_map(|e| match *e {
            Event::SegmentStart { segment } | Event::SegmentEnd { segment } => Some(segment),
            Event::Intersection { .. } => None,
        })
        .collect();
    ids.sort();
    ids.dedup();

    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            out.push(PointIntersectionRecord {
                point,
                kind: PointIntersectionKind::EndpointTouch,
                a: ids[i],
                b: ids[j],
            });
        }
    }
}

fn schedule_or_record_pair(
    segments: &Segments,
    queue: &mut EventQueue,
    scheduled: &mut BTreeSet<(PointRat, SegmentId, SegmentId)>,
    current_point: PointRat,
    a: SegmentId,
    b: SegmentId,
    out: &mut Vec<PointIntersectionRecord>,
) {
    if a == b {
        return;
    }
    let (a, b) = if a <= b { (a, b) } else { (b, a) };

    let Some(hit) = intersect_segments(segments.get(a), segments.get(b)) else {
        return;
    };

    match hit {
        SegmentIntersection::CollinearOverlap => {
            // 第一阶段暂不输出“重叠段”。后续会引入“最大重叠段集合”输出。
        }
        SegmentIntersection::Point { point, kind } => {
            if point == current_point {
                // 在第一阶段里：此分支主要用于补齐“端点接触”这类不会被调度为未来交点事件的情况。
                // 对于严格相交（Proper），正常应通过 `Intersection` 事件在该点统一输出，避免重复记录。
                if kind == PointIntersectionKind::EndpointTouch {
                    out.push(PointIntersectionRecord {
                        point,
                        kind,
                        a,
                        b,
                    });
                }
                return;
            }
            if point < current_point {
                return;
            }

            let key = (point, a, b);
            if scheduled.insert(key) {
                queue.push(point, Event::intersection(a, b));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;
    use crate::geom::segment::Segment;

    #[test]
    fn reports_single_proper_intersection() {
        let mut segments = Segments::new();
        let a = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 0,
        });
        let b = segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 1,
        });

        let out = enumerate_point_intersections(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionRecord {
                point: PointRat {
                    x: Rational::from_int(5),
                    y: Rational::from_int(5),
                },
                kind: PointIntersectionKind::Proper,
                a,
                b,
            }]
        );
    }

    #[test]
    fn reports_endpoint_touch_when_one_ends_and_other_starts_at_same_point() {
        let mut segments = Segments::new();
        let a = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });
        let b = segments.push(Segment {
            a: PointI64 { x: 10, y: 0 },
            b: PointI64 { x: 20, y: 10 },
            source_index: 1,
        });

        let out = enumerate_point_intersections(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionRecord {
                point: PointRat {
                    x: Rational::from_int(10),
                    y: Rational::from_int(0),
                },
                kind: PointIntersectionKind::EndpointTouch,
                a,
                b,
            }]
        );
    }

    #[test]
    fn supports_vertical_segments_via_range_query() {
        let mut segments = Segments::new();
        let vertical = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 0,
        });
        let other = segments.push(Segment {
            a: PointI64 { x: -10, y: 3 },
            b: PointI64 { x: 10, y: 3 },
            source_index: 1,
        });

        let out = enumerate_point_intersections(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionRecord {
                point: PointRat {
                    x: Rational::from_int(0),
                    y: Rational::from_int(3),
                },
                kind: PointIntersectionKind::Proper,
                a: vertical,
                b: other,
            }]
        );
    }
}
