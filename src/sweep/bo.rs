use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::geom::intersection::{
    PointIntersectionGroupRecord, PointIntersectionKind, SegmentIntersection, intersect_segments,
};
use crate::geom::point::PointRat;
use crate::geom::segment::{SegmentId, Segments};
use crate::limits::{LimitExceeded, LimitKind, Limits};
use crate::rational::Rational;
use crate::sweep::event_queue::{Event, EventQueue};
use crate::sweep::status::{SweepStatus, SweepStatusError, TreapSweepStatus};
use crate::trace::Trace;
use crate::trace::TraceStep;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoError {
    Status(SweepStatusError),
    Limits(LimitExceeded),
}

impl From<SweepStatusError> for BoError {
    fn from(value: SweepStatusError) -> Self {
        BoError::Status(value)
    }
}

impl From<LimitExceeded> for BoError {
    fn from(value: LimitExceeded) -> Self {
        BoError::Limits(value)
    }
}

impl fmt::Display for BoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoError::Status(e) => write!(f, "{}", e),
            BoError::Limits(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Default)]
struct PointIntersectionGroupBuilder {
    endpoint: BTreeSet<SegmentId>,
    interior: BTreeSet<SegmentId>,
}

impl PointIntersectionGroupBuilder {
    fn add_segment(&mut self, segments: &Segments, point: PointRat, id: SegmentId) {
        let seg = segments.get(id);
        let a = PointRat::from_i64(seg.a);
        let b = PointRat::from_i64(seg.b);
        if point == a || point == b {
            self.endpoint.insert(id);
            self.interior.remove(&id);
            return;
        }

        if !self.endpoint.contains(&id) {
            self.interior.insert(id);
        }
    }

    fn total_segments(&self) -> usize {
        self.endpoint.len().saturating_add(self.interior.len())
    }

    fn build(&self, point: PointRat) -> PointIntersectionGroupRecord {
        PointIntersectionGroupRecord {
            point,
            endpoint_segments: self.endpoint.iter().copied().collect(),
            interior_segments: self.interior.iter().copied().collect(),
        }
    }
}

/// 第一阶段：枚举点交（包含端点接触）。
///
/// 说明：
/// - 垂直线段不进入状态结构，而是在 x 批处理结束时用 `range_by_y` 做命中查询；
/// - 对共线重叠只返回占位，不输出“重叠段”（第二阶段再做）。
pub fn enumerate_point_intersections(segments: &Segments) -> Result<Vec<PointIntersectionGroupRecord>, BoError> {
    enumerate_point_intersections_with_limits(segments, Limits::default())
}

pub fn enumerate_point_intersections_with_trace(
    segments: &Segments,
) -> Result<(Vec<PointIntersectionGroupRecord>, Trace), BoError> {
    enumerate_point_intersections_with_trace_and_limits(segments, Limits::default())
}

pub fn enumerate_point_intersections_with_limits(
    segments: &Segments,
    limits: Limits,
) -> Result<Vec<PointIntersectionGroupRecord>, BoError> {
    run_bentley_ottmann(segments, None, limits)
}

pub fn enumerate_point_intersections_with_trace_and_limits(
    segments: &Segments,
    limits: Limits,
) -> Result<(Vec<PointIntersectionGroupRecord>, Trace), BoError> {
    let mut trace = Trace::default();
    let intersections = run_bentley_ottmann(segments, Some(&mut trace), limits)?;
    Ok((intersections, trace))
}

fn run_bentley_ottmann(
    segments: &Segments,
    mut trace: Option<&mut Trace>,
    limits: Limits,
) -> Result<Vec<PointIntersectionGroupRecord>, BoError> {
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
    let mut out: Vec<PointIntersectionGroupRecord> = Vec::new();
    let mut trace_active_entries_total: usize = 0;

    let mut push_trace_step_with_limits = |trace: &mut Trace, step: TraceStep| -> Result<(), BoError> {
        let next_steps = trace.steps.len() + 1;
        if next_steps > limits.max_trace_steps {
            return Err(BoError::Limits(LimitExceeded {
                kind: LimitKind::TraceSteps,
                limit: limits.max_trace_steps,
                actual: next_steps,
            }));
        }

        let next_active_total = trace_active_entries_total.saturating_add(step.active.len());
        if next_active_total > limits.max_trace_active_entries_total {
            return Err(BoError::Limits(LimitExceeded {
                kind: LimitKind::TraceActiveEntriesTotal,
                limit: limits.max_trace_active_entries_total,
                actual: next_active_total,
            }));
        }

        trace_active_entries_total = next_active_total;
        trace.steps.push(step);
        Ok(())
    };

    let ensure_can_add_groups = |current_len: usize, additional: usize| -> Result<(), BoError> {
        let next_len = current_len.saturating_add(additional);
        if next_len > limits.max_intersections {
            return Err(BoError::Limits(LimitExceeded {
                kind: LimitKind::Intersections,
                limit: limits.max_intersections,
                actual: next_len,
            }));
        }
        Ok(())
    };

    while let Some((point, events)) = queue.pop_next_batch() {
        if let Some(x) = pending_x {
            if point.x != x {
                if !pending_vertical.is_empty() {
                    let hits = collect_vertical_hit_groups(segments, &status, &pending_vertical)?;
                    ensure_can_add_groups(out.len(), hits.len())?;

                    if let Some(trace) = trace.as_deref_mut() {
                        let mut step = TraceStep::vertical_flush(x);
                        step.events = pending_vertical
                            .iter()
                            .map(|id| format!("Vertical({})", id.0))
                            .collect();
                        step.active = status.snapshot_order();
                        step.intersections = hits.clone();
                        for &v_id in &pending_vertical {
                            let v = segments.get(v_id);
                            let y_min = v.a.y.min(v.b.y);
                            let y_max = v.a.y.max(v.b.y);
                            step.notes.push(format!(
                                "VerticalRange({}): y=[{},{}]",
                                v_id.0, y_min, y_max
                            ));
                        }
                        push_trace_step_with_limits(trace, step)?;
                    }

                    out.extend(hits);
                }

                pending_vertical.clear();
                pending_x = None;
            }
        }
        if pending_x.is_none() {
            pending_x = Some(point.x);
        }

        status.set_sweep_x(point.x);

        let mut step = trace
            .as_deref_mut()
            .map(|_| TraceStep::point_batch(point, point.x));
        if let Some(step) = step.as_mut() {
            step.events = events.iter().map(|e| event_to_string(*e)).collect();
        }

        let mut intersection_groups: BTreeMap<PointRat, PointIntersectionGroupBuilder> = BTreeMap::new();

        // 同一事件点上“作为端点出现”的线段：它们至少在该点存在端点-端点接触。
        let mut endpoint_ids_at_point: Vec<SegmentId> = events
            .iter()
            .filter_map(|e| match *e {
                Event::SegmentStart { segment } | Event::SegmentEnd { segment } => Some(segment),
                Event::Intersection { .. } => None,
            })
            .collect();
        endpoint_ids_at_point.sort();
        endpoint_ids_at_point.dedup();

        if !endpoint_ids_at_point.is_empty() {
            let group = intersection_groups.entry(point).or_default();
            for &id in &endpoint_ids_at_point {
                group.add_segment(segments, point, id);
            }
            if endpoint_ids_at_point.len() >= 2 {
                if let Some(step) = step.as_mut() {
                    step.notes
                        .push(format!("EndpointSegments: {}", endpoint_ids_at_point.len()));
                }
            }
        }

        // 用 U/L/C(p) 的批处理语义替代“逐条事件顺序处理”，避免退化下出现“已删除线段仍被重排”。
        let mut u: Vec<SegmentId> = Vec::new();
        let mut l: Vec<SegmentId> = Vec::new();
        let mut intersection_pairs: Vec<(SegmentId, SegmentId)> = Vec::new();

        for event in &events {
            match *event {
                Event::SegmentStart { segment } => {
                    if segments.get(segment).is_vertical() {
                        pending_vertical.insert(segment);
                        if let Some(step) = step.as_mut() {
                            step.notes.push(format!("VerticalStart({})", segment.0));
                        }
                    } else {
                        u.push(segment);
                    }
                }
                Event::SegmentEnd { segment } => {
                    if segments.get(segment).is_vertical() {
                        if let Some(step) = step.as_mut() {
                            step.notes.push(format!("VerticalEnd({})", segment.0));
                        }
                    } else {
                        l.push(segment);
                    }
                }
                Event::Intersection { a, b } => {
                    intersection_pairs.push((a, b));
                    if let Some(step) = step.as_mut() {
                        step.notes.push(format!("IntersectionEvent({},{})", a.0, b.0));
                    }
                }
            }
        }

        u.sort();
        u.dedup();
        l.sort();
        l.dedup();
        intersection_pairs.sort();
        intersection_pairs.dedup();

        // 端点接触（端点-内部）：端点线段在该点可能会“碰到”某条穿过该点的活动线段。
        // 这类交点不应作为 `Intersection` 事件调度，但仍应在该点输出。
        let mut endpoint_ids: Vec<SegmentId> = u.clone();
        endpoint_ids.extend_from_slice(&l);
        endpoint_ids.sort();
        endpoint_ids.dedup();
        record_endpoint_on_interior_hits(
            segments,
            &status,
            point,
            &endpoint_ids,
            &mut intersection_groups,
            step.as_mut(),
        )?;

        // 垂直线段的批末查询发生在 x 变化时；这会遗漏“非垂直线段在该 x 处结束”的端点接触。
        // 这里在删除结束线段之前，补齐它们与 pending_vertical 的端点接触输出。
        record_vertical_endpoint_touches_for_ending_segments(
            segments,
            &pending_vertical,
            point,
            &l,
            &mut intersection_groups,
            step.as_mut(),
        )?;

        let mut c: Vec<SegmentId> = Vec::new();
        for (a, b) in &intersection_pairs {
            let a = *a;
            let b = *b;
            if let Some(SegmentIntersection::Point { point: ip, kind }) =
                intersect_segments(segments.get(a), segments.get(b))
            {
                if let Some(step) = step.as_mut() {
                    step.notes.push(format!(
                        "IntersectionAt({},{}) -> {} @ {}",
                        a.0,
                        b.0,
                        kind,
                        format_point(ip)
                    ));
                }
                let group = intersection_groups.entry(ip).or_default();
                group.add_segment(segments, ip, a);
                group.add_segment(segments, ip, b);
                if kind == PointIntersectionKind::Proper {
                    c.push(a);
                    c.push(b);
                }
            }
        }
        c.sort();
        c.dedup();

        if let Some(step) = step.as_mut() {
            step.notes.push(format!("ULC: U={} L={} C={}", u.len(), l.len(), c.len()));
            step.notes.push(format!("U: {}", format_id_list(&u, 12)));
            step.notes.push(format!("L: {}", format_id_list(&l, 12)));
            step.notes.push(format!("C: {}", format_id_list(&c, 12)));
        }

        let mut to_remove: Vec<SegmentId> = l.clone();
        to_remove.extend_from_slice(&c);
        to_remove.sort();
        to_remove.dedup();

        let mut to_insert: Vec<SegmentId> = u.clone();
        to_insert.extend_from_slice(&c);
        to_insert.sort();
        to_insert.dedup();

        for id in &to_remove {
            if let Some(step) = step.as_mut() {
                step.notes.push(format!("Remove({})", id.0));
            }
            status.remove(*id)?;
        }

        for id in &to_insert {
            if let Some(step) = step.as_mut() {
                step.notes.push(format!("Insert({})", id.0));
            }
            status.insert(segments, *id)?;
        }

        if to_insert.is_empty() {
            // 只有删除（没有插入/重排）时：检查删除后在 p.y 附近新形成的相邻对。
            let succ = status.lower_bound_by_y(segments, point.y)?;
            let pred = succ.and_then(|id| status.pred(id));
            if let (Some(a), Some(b)) = (pred, succ) {
                schedule_or_record_pair(
                    segments,
                    &mut queue,
                    &mut scheduled,
                    point,
                    a,
                    b,
                    step.as_mut(),
                );
            }
        } else {
            for id in &to_insert {
                if let Some(pred) = status.pred(*id) {
                    schedule_or_record_pair(
                        segments,
                        &mut queue,
                        &mut scheduled,
                        point,
                        pred,
                        *id,
                        step.as_mut(),
                    );
                }
                if let Some(succ) = status.succ(*id) {
                    schedule_or_record_pair(
                        segments,
                        &mut queue,
                        &mut scheduled,
                        point,
                        *id,
                        succ,
                        step.as_mut(),
                    );
                }
            }
        }

        let mut hits: Vec<PointIntersectionGroupRecord> = Vec::new();
        for (ip, group) in &intersection_groups {
            if group.total_segments() >= 2 {
                hits.push(group.build(*ip));
            }
        }
        ensure_can_add_groups(out.len(), hits.len())?;

        if let Some(trace) = trace.as_deref_mut() {
            let mut step = step.expect("trace 存在时 step 应为 Some");
            step.active = status.snapshot_order();
            step.intersections = hits.clone();
            push_trace_step_with_limits(trace, step)?;
        }
        out.extend(hits);
    }

    if let Some(x) = pending_x {
        if !pending_vertical.is_empty() {
            let hits = collect_vertical_hit_groups(segments, &status, &pending_vertical)?;
            ensure_can_add_groups(out.len(), hits.len())?;

            if let Some(trace) = trace.as_deref_mut() {
                let mut step = TraceStep::vertical_flush(x);
                step.events = pending_vertical
                    .iter()
                    .map(|id| format!("Vertical({})", id.0))
                    .collect();
                step.active = status.snapshot_order();
                step.intersections = hits.clone();
                for &v_id in &pending_vertical {
                    let v = segments.get(v_id);
                    let y_min = v.a.y.min(v.b.y);
                    let y_max = v.a.y.max(v.b.y);
                    step.notes.push(format!(
                        "VerticalRange({}): y=[{},{}]",
                        v_id.0, y_min, y_max
                    ));
                }
                push_trace_step_with_limits(trace, step)?;
            }

            out.extend(hits);
        }
    }

    Ok(out)
}

fn collect_vertical_hit_groups(
    segments: &Segments,
    status: &impl SweepStatus,
    vertical: &BTreeSet<SegmentId>,
) -> Result<Vec<PointIntersectionGroupRecord>, BoError> {
    if vertical.is_empty() || status.is_empty() {
        return Ok(Vec::new());
    }

    let mut groups: BTreeMap<PointRat, PointIntersectionGroupBuilder> = BTreeMap::new();
    for &v_id in vertical {
        let v = segments.get(v_id);
        debug_assert!(v.is_vertical(), "collect_vertical_hit_groups 仅应处理垂直线段");

        let v_a = PointRat::from_i64(v.a);
        let v_b = PointRat::from_i64(v.b);

        let y_min = Rational::from_int(v.a.y.min(v.b.y) as i128);
        let y_max = Rational::from_int(v.a.y.max(v.b.y) as i128);

        let candidates = status.range_by_y(segments, y_min, y_max)?;
        for s_id in candidates {
            let Some(SegmentIntersection::Point { point, .. }) =
                intersect_segments(v, segments.get(s_id))
            else {
                continue;
            };

            let s = segments.get(s_id);
            if (point == v_a || point == v_b)
                && (point == PointRat::from_i64(s.a) || point == PointRat::from_i64(s.b))
            {
                // 端点-端点接触在事件点已输出，避免在 VerticalFlush 重复输出。
                continue;
            }

            let group = groups.entry(point).or_default();
            group.add_segment(segments, point, v_id);
            group.add_segment(segments, point, s_id);
        }
    }

    let mut hits: Vec<PointIntersectionGroupRecord> = Vec::new();
    for (point, group) in groups {
        if group.total_segments() >= 2 {
            hits.push(group.build(point));
        }
    }

    Ok(hits)
}

fn record_endpoint_on_interior_hits(
    segments: &Segments,
    status: &impl SweepStatus,
    point: PointRat,
    endpoint_ids: &[SegmentId],
    intersection_groups: &mut BTreeMap<PointRat, PointIntersectionGroupBuilder>,
    mut trace_step: Option<&mut TraceStep>,
) -> Result<(), BoError> {
    if endpoint_ids.is_empty() || status.is_empty() {
        return Ok(());
    }

    let endpoint_set: BTreeSet<SegmentId> = endpoint_ids.iter().copied().collect();

    // 找出所有在 x=point.x 处 y 恰好等于 point.y 的活动线段：它们穿过事件点，且未必以该点为端点。
    let candidates = status.range_by_y(segments, point.y, point.y)?;
    if candidates.is_empty() {
        return Ok(());
    }

    let mut added = 0_usize;
    for &e_id in endpoint_ids {
        for &s_id in &candidates {
            if s_id == e_id {
                continue;
            }
            if endpoint_set.contains(&s_id) {
                // 两端点都在该点的情况已由端点集合覆盖，避免重复探测。
                continue;
            }

            let Some(SegmentIntersection::Point { point: ip, kind }) =
                intersect_segments(segments.get(e_id), segments.get(s_id))
            else {
                continue;
            };
            if ip != point || kind != PointIntersectionKind::EndpointTouch {
                continue;
            }

            let group = intersection_groups.entry(ip).or_default();
            group.add_segment(segments, ip, e_id);
            group.add_segment(segments, ip, s_id);
            added += 1;
        }
    }

    if added != 0 {
        if let Some(step) = trace_step.as_mut() {
            step.notes.push(format!("EndpointOnInterior: {}", added));
        }
    }
    Ok(())
}

fn record_vertical_endpoint_touches_for_ending_segments(
    segments: &Segments,
    pending_vertical: &BTreeSet<SegmentId>,
    point: PointRat,
    ending_ids: &[SegmentId],
    intersection_groups: &mut BTreeMap<PointRat, PointIntersectionGroupBuilder>,
    mut trace_step: Option<&mut TraceStep>,
) -> Result<(), BoError> {
    if pending_vertical.is_empty() || ending_ids.is_empty() {
        return Ok(());
    }

    let mut added = 0_usize;
    for &s_id in ending_ids {
        let s = segments.get(s_id);
        debug_assert!(!s.is_vertical(), "ending_ids 不应包含垂直线段");

        for &v_id in pending_vertical {
            let v = segments.get(v_id);
            debug_assert!(v.is_vertical(), "pending_vertical 仅应包含垂直线段");

            // 垂直线段的端点接触（端点-端点）已在事件点按端点集合输出，避免重复。
            if point == PointRat::from_i64(v.a) || point == PointRat::from_i64(v.b) {
                continue;
            }

            let Some(SegmentIntersection::Point { point: ip, kind }) = intersect_segments(v, s) else {
                continue;
            };
            if ip != point || kind != PointIntersectionKind::EndpointTouch {
                continue;
            }

            let group = intersection_groups.entry(ip).or_default();
            group.add_segment(segments, ip, v_id);
            group.add_segment(segments, ip, s_id);
            added += 1;
        }
    }

    if added != 0 {
        if let Some(step) = trace_step.as_mut() {
            step.notes.push(format!("VerticalEndpointTouch(end): {}", added));
        }
    }
    Ok(())
}

fn schedule_or_record_pair(
    segments: &Segments,
    queue: &mut EventQueue,
    scheduled: &mut BTreeSet<(PointRat, SegmentId, SegmentId)>,
    current_point: PointRat,
    a: SegmentId,
    b: SegmentId,
    mut trace_step: Option<&mut TraceStep>,
) {
    if a == b {
        return;
    }
    let (a, b) = if a <= b { (a, b) } else { (b, a) };

    let Some(hit) = intersect_segments(segments.get(a), segments.get(b)) else {
        if let Some(step) = trace_step.as_mut() {
            step.notes.push(format!("Check({},{}) -> none", a.0, b.0));
        }
        return;
    };

    match hit {
        SegmentIntersection::CollinearOverlap => {
            // 第一阶段暂不输出“重叠段”。后续会引入“最大重叠段集合”输出。
            if let Some(step) = trace_step.as_mut() {
                step.notes
                    .push(format!("Check({},{}) -> CollinearOverlap(phase2)", a.0, b.0));
            }
        }
        SegmentIntersection::Point { point, kind } => {
            if point == current_point {
                // 端点接触输出由事件点批处理统一负责（端点集合 + 必要的端点-内部/垂直补齐）。
                // 这里仅用于“不要调度过去/当前点”的防重复保护。
                return;
            }
            if point < current_point {
                if let Some(step) = trace_step.as_mut() {
                    step.notes.push(format!(
                        "Check({},{}) -> past @ {} (ignored)",
                        a.0,
                        b.0,
                        format_point(point)
                    ));
                }
                return;
            }

            // `Intersection` 事件仅代表“需要在 x+ε 发生重排”的交点；端点接触不应触发重排，
            // 且更适合在事件点批处理里通过 U/L（以及必要的端点-内部检测）统一输出。
            if kind == PointIntersectionKind::EndpointTouch {
                if let Some(step) = trace_step.as_mut() {
                    step.notes.push(format!(
                        "SkipScheduleEndpointTouch({},{}) @ {}",
                        a.0,
                        b.0,
                        format_point(point)
                    ));
                }
                return;
            }

            let key = (point, a, b);
            if scheduled.insert(key) {
                queue.push(point, Event::intersection(a, b));
                if let Some(step) = trace_step.as_mut() {
                    step.notes.push(format!(
                        "ScheduleIntersection({},{}) @ {}",
                        a.0,
                        b.0,
                        format_point(point)
                    ));
                }
            } else if let Some(step) = trace_step.as_mut() {
                step.notes.push(format!(
                    "ScheduleIntersection({},{}) @ {} (dedup)",
                    a.0,
                    b.0,
                    format_point(point)
                ));
            }
        }
    }
}

fn event_to_string(event: Event) -> String {
    match event {
        Event::SegmentStart { segment } => format!("SegmentStart({})", segment.0),
        Event::SegmentEnd { segment } => format!("SegmentEnd({})", segment.0),
        Event::Intersection { a, b } => format!("Intersection({},{})", a.0, b.0),
    }
}

fn format_point(p: PointRat) -> String {
    format!("({}, {})", p.x, p.y)
}

fn format_id_list(ids: &[SegmentId], limit: usize) -> String {
    let mut out = String::new();
    out.push('[');
    let shown = ids.len().min(limit);
    for i in 0..shown {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&ids[i].0.to_string());
    }
    if ids.len() > shown {
        out.push_str(",...");
        out.push_str(&(ids.len() - shown).to_string());
        out.push_str(" more");
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;
    use crate::geom::segment::Segment;
    use crate::limits::{LimitExceeded, LimitKind, Limits};
    use crate::trace::TraceStepKind;

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
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(5),
                    y: Rational::from_int(5),
                },
                endpoint_segments: vec![],
                interior_segments: vec![a, b],
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
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(10),
                    y: Rational::from_int(0),
                },
                endpoint_segments: vec![a, b],
                interior_segments: vec![],
            }]
        );
    }

    #[test]
    fn does_not_crash_for_shared_end_endpoint_touch() {
        // 回归测试：历史上会把端点接触也调度为 Intersection，导致在同点批处理里先删除后重排触发 SegmentNotFound。
        let mut segments = Segments::new();
        let a = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
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
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(10),
                    y: Rational::from_int(0),
                },
                endpoint_segments: vec![a, b],
                interior_segments: vec![],
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
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(0),
                    y: Rational::from_int(3),
                },
                endpoint_segments: vec![],
                interior_segments: vec![vertical, other],
            }]
        );
    }

    #[test]
    fn does_not_duplicate_endpoint_touch_when_vertical_and_other_share_endpoint() {
        // 回归测试：同一个端点接触不应同时由 `record_endpoint_pairs` 与 `VerticalFlush` 重复输出。
        let mut segments = Segments::new();
        let vertical = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 0,
        });
        let other = segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 1,
        });

        let out = enumerate_point_intersections(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(0),
                    y: Rational::from_int(0),
                },
                endpoint_segments: vec![vertical, other],
                interior_segments: vec![],
            }]
        );
    }

    #[test]
    fn reports_endpoint_touch_when_segment_ends_on_vertical_interior() {
        let mut segments = Segments::new();
        let vertical = segments.push(Segment {
            a: PointI64 { x: 0, y: -10 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 0,
        });
        let ending = segments.push(Segment {
            a: PointI64 { x: -10, y: 3 },
            b: PointI64 { x: 0, y: 3 },
            source_index: 1,
        });

        let out = enumerate_point_intersections(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(0),
                    y: Rational::from_int(3),
                },
                endpoint_segments: vec![ending],
                interior_segments: vec![vertical],
            }]
        );
    }

    #[test]
    fn produces_trace_steps_for_point_batches_and_vertical_flush() {
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

        let (out, trace) = enumerate_point_intersections_with_trace(&segments).unwrap();
        assert_eq!(
            out,
            vec![PointIntersectionGroupRecord {
                point: PointRat {
                    x: Rational::from_int(0),
                    y: Rational::from_int(3),
                },
                endpoint_segments: vec![],
                interior_segments: vec![vertical, other],
            }]
        );

        let flush = trace
            .steps
            .iter()
            .find(|s| s.kind == TraceStepKind::VerticalFlush)
            .expect("应包含一次垂直线段的批末查询记录");
        assert_eq!(flush.sweep_x, Rational::from_int(0));
        assert_eq!(flush.intersections, out);
    }

    #[test]
    fn trace_json_is_byte_identical_across_runs() {
        let mut segments = Segments::new();
        segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 0,
        });
        segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 1,
        });
        segments.push(Segment {
            a: PointI64 { x: 2, y: 0 },
            b: PointI64 { x: 2, y: 10 },
            source_index: 2,
        });

        let (out1, t1) = enumerate_point_intersections_with_trace(&segments).unwrap();
        let (out2, t2) = enumerate_point_intersections_with_trace(&segments).unwrap();
        assert_eq!(out1, out2);
        assert_eq!(t1.to_json_string(), t2.to_json_string());
    }

    #[test]
    fn handles_multiple_segments_intersecting_at_same_point_stably() {
        let mut segments = Segments::new();
        segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 10 },
            source_index: 0,
        });
        segments.push(Segment {
            a: PointI64 { x: 0, y: 10 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 1,
        });
        segments.push(Segment {
            a: PointI64 { x: 0, y: 5 },
            b: PointI64 { x: 10, y: 5 },
            source_index: 2,
        });

        let (out1, t1) = enumerate_point_intersections_with_trace(&segments).unwrap();
        let (out2, t2) = enumerate_point_intersections_with_trace(&segments).unwrap();

        let p = PointRat {
            x: Rational::from_int(5),
            y: Rational::from_int(5),
        };
        assert!(out1.iter().any(|it| it.point == p));
        assert_eq!(out1, out2);
        assert_eq!(t1.to_json_string(), t2.to_json_string());
    }

    #[test]
    fn handles_many_vertical_segments_stably() {
        let n = 50_i64;
        let mut segments = Segments::new();
        let horizontal = segments.push(Segment {
            a: PointI64 { x: -1, y: 0 },
            b: PointI64 { x: n + 1, y: 0 },
            source_index: 0,
        });
        for i in 0..n {
            segments.push(Segment {
                a: PointI64 { x: i, y: -10 },
                b: PointI64 { x: i, y: 10 },
                source_index: (i + 1) as usize,
            });
        }

        let (out1, t1) = enumerate_point_intersections_with_trace(&segments).unwrap();
        let (out2, t2) = enumerate_point_intersections_with_trace(&segments).unwrap();
        assert_eq!(out1, out2);
        assert_eq!(t1.to_json_string(), t2.to_json_string());

        assert_eq!(out1.len() as i64, n);
        assert!(out1.iter().all(|it| it.endpoint_segments.is_empty()));
        assert!(out1.iter().all(|it| it.interior_segments.len() == 2));
        assert!(out1.iter().all(|it| it.interior_segments.contains(&horizontal)));

        let flush_count = t1
            .steps
            .iter()
            .filter(|s| s.kind == TraceStepKind::VerticalFlush)
            .count();
        assert_eq!(flush_count as i64, n);
    }

    #[test]
    fn fails_fast_when_trace_steps_exceed_limit() {
        let mut segments = Segments::new();
        segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });

        let limits = Limits {
            max_trace_steps: 1,
            ..Limits::default()
        };
        let err = enumerate_point_intersections_with_trace_and_limits(&segments, limits).unwrap_err();
        match err {
            BoError::Limits(LimitExceeded {
                kind: LimitKind::TraceSteps,
                limit,
                actual,
            }) => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("期望 TraceSteps 超限，但得到：{other:?}"),
        }
        assert!(err.to_string().contains("建议："));
    }

    #[test]
    fn fails_fast_when_trace_active_entries_total_exceed_limit() {
        let mut segments = Segments::new();
        segments.push(Segment {
            a: PointI64 { x: 0, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 0,
        });

        let limits = Limits {
            max_trace_active_entries_total: 0,
            ..Limits::default()
        };
        let err = enumerate_point_intersections_with_trace_and_limits(&segments, limits).unwrap_err();
        match err {
            BoError::Limits(LimitExceeded {
                kind: LimitKind::TraceActiveEntriesTotal,
                limit,
                actual,
            }) => {
                assert_eq!(limit, 0);
                assert_eq!(actual, 1);
            }
            other => panic!("期望 TraceActiveEntriesTotal 超限，但得到：{other:?}"),
        }
        assert!(err.to_string().contains("建议："));
    }

    #[test]
    fn fails_fast_when_intersection_groups_exceed_limit() {
        let mut segments = Segments::new();
        for i in 0_usize..4 {
            segments.push(Segment {
                a: PointI64 { x: 0, y: 0 },
                b: PointI64 {
                    x: 10,
                    y: i as i64,
                },
                source_index: i,
            });
        }

        // 4 条线段共享同一点端点时：按点聚合只输出 1 条记录；因此超限也按 group 数统计。
        let limits = Limits {
            max_intersections: 0,
            ..Limits::default()
        };
        let err = enumerate_point_intersections_with_limits(&segments, limits).unwrap_err();
        match err {
            BoError::Limits(LimitExceeded {
                kind: LimitKind::Intersections,
                limit,
                actual,
            }) => {
                assert_eq!(limit, 0);
                assert_eq!(actual, 1);
            }
            other => panic!("期望 Intersections 超限，但得到：{other:?}"),
        }
        assert!(err.to_string().contains("建议："));
    }
}
