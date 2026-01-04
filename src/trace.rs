use core::fmt;

use crate::geom::intersection::PointIntersectionGroupRecord;
use crate::geom::point::PointRat;
use crate::geom::segment::SegmentId;
use crate::rational::Rational;

#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub warnings: Vec<String>,
    pub steps: Vec<TraceStep>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceStepKind {
    /// 处理一个事件点（事件队列中按 (x,y) 排序的批处理）。
    PointBatch,
    /// 在某个 x 批处理结束后，对该 x 上的垂直线段做区间命中查询。
    VerticalFlush,
}

impl fmt::Display for TraceStepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceStepKind::PointBatch => write!(f, "PointBatch"),
            TraceStepKind::VerticalFlush => write!(f, "VerticalFlush"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TraceStep {
    pub kind: TraceStepKind,
    pub sweep_x: Rational,
    pub point: Option<PointRat>,
    pub events: Vec<String>,
    pub active: Vec<SegmentId>,
    pub intersections: Vec<PointIntersectionGroupRecord>,
    pub notes: Vec<String>,
}

impl TraceStep {
    pub fn point_batch(point: PointRat, sweep_x: Rational) -> Self {
        Self {
            kind: TraceStepKind::PointBatch,
            sweep_x,
            point: Some(point),
            events: Vec::new(),
            active: Vec::new(),
            intersections: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn vertical_flush(sweep_x: Rational) -> Self {
        Self {
            kind: TraceStepKind::VerticalFlush,
            sweep_x,
            point: None,
            events: Vec::new(),
            active: Vec::new(),
            intersections: Vec::new(),
            notes: Vec::new(),
        }
    }
}

impl Trace {
    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        write_trace_json(self, &mut out);
        out
    }
}

fn write_trace_json(trace: &Trace, out: &mut String) {
    out.push('{');
    write_kv_str(out, "schema", "trace.v2");
    out.push(',');
    write_kv_string_array(out, "warnings", &trace.warnings);
    out.push(',');
    out.push('"');
    out.push_str("steps");
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, step) in trace.steps.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        write_step_json(step, out);
    }
    out.push(']');
    out.push('}');
}

fn write_step_json(step: &TraceStep, out: &mut String) {
    out.push('{');
    write_kv_str(out, "kind", &step.kind.to_string());
    out.push(',');
    write_kv_rational(out, "sweep_x", step.sweep_x);
    out.push(',');
    out.push('"');
    out.push_str("point");
    out.push('"');
    out.push(':');
    match step.point {
        Some(p) => write_point(out, p),
        None => out.push_str("null"),
    }
    out.push(',');
    write_kv_string_array(out, "events", &step.events);
    out.push(',');
    write_kv_segment_id_array(out, "active", &step.active);
    out.push(',');
    write_kv_intersections(out, "intersections", &step.intersections);
    out.push(',');
    write_kv_string_array(out, "notes", &step.notes);
    out.push('}');
}

fn write_kv_intersections(out: &mut String, key: &str, value: &[PointIntersectionGroupRecord]) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, item) in value.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        write_intersection(out, item);
    }
    out.push(']');
}

fn write_intersection(out: &mut String, it: &PointIntersectionGroupRecord) {
    out.push('{');
    out.push('"');
    out.push_str("point");
    out.push('"');
    out.push(':');
    write_point(out, it.point);
    out.push(',');
    write_kv_segment_id_array(out, "endpoint_segments", &it.endpoint_segments);
    out.push(',');
    write_kv_segment_id_array(out, "interior_segments", &it.interior_segments);
    out.push('}');
}

fn write_kv_string_array(out: &mut String, key: &str, value: &[String]) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, item) in value.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        write_json_string(out, item);
    }
    out.push(']');
}

fn write_kv_segment_id_array(out: &mut String, key: &str, value: &[SegmentId]) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push('[');
    for (i, item) in value.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push_str(&item.0.to_string());
    }
    out.push(']');
}

fn write_kv_rational(out: &mut String, key: &str, value: Rational) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    write_rational(out, value);
}

fn write_kv_str(out: &mut String, key: &str, value: &str) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    write_json_string(out, value);
}

fn write_point(out: &mut String, p: PointRat) {
    out.push('{');
    out.push('"');
    out.push_str("x");
    out.push('"');
    out.push(':');
    write_rational(out, p.x);
    out.push(',');
    out.push('"');
    out.push_str("y");
    out.push('"');
    out.push(':');
    write_rational(out, p.y);
    out.push('}');
}

fn write_rational(out: &mut String, r: Rational) {
    out.push('{');
    write_kv_str(out, "num", &r.num().to_string());
    out.push(',');
    write_kv_str(out, "den", &r.den().to_string());
    out.push('}');
}

fn write_json_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str("\\u");
                let code = c as u32;
                out.push(hex_nibble((code >> 12) & 0xF));
                out.push(hex_nibble((code >> 8) & 0xF));
                out.push(hex_nibble((code >> 4) & 0xF));
                out.push(hex_nibble(code & 0xF));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn hex_nibble(v: u32) -> char {
    debug_assert!(v < 16);
    match v {
        0..=9 => (b'0' + v as u8) as char,
        _ => (b'a' + (v as u8 - 10)) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::fixed::PointI64;

    #[test]
    fn writes_stable_json_with_fixed_field_order() {
        let mut trace = Trace::default();
        let mut step = TraceStep::point_batch(
            PointRat::from_i64(PointI64 { x: 5, y: -2 }),
            Rational::from_int(5),
        );
        step.events.push("SegmentStart(1)".to_string());
        step.active = vec![SegmentId(1), SegmentId(3)];
        step.intersections.push(PointIntersectionGroupRecord {
            point: PointRat::from_i64(PointI64 { x: 5, y: -2 }),
            endpoint_segments: vec![SegmentId(1)],
            interior_segments: vec![SegmentId(3)],
        });
        step.notes.push("包含引号: \" 和换行\n".to_string());
        trace.steps.push(step);

        let json = trace.to_json_string();
        assert_eq!(
            json,
            "{\"schema\":\"trace.v2\",\"warnings\":[],\"steps\":[{\"kind\":\"PointBatch\",\"sweep_x\":{\"num\":\"5\",\"den\":\"1\"},\"point\":{\"x\":{\"num\":\"5\",\"den\":\"1\"},\"y\":{\"num\":\"-2\",\"den\":\"1\"}},\"events\":[\"SegmentStart(1)\"],\"active\":[1,3],\"intersections\":[{\"point\":{\"x\":{\"num\":\"5\",\"den\":\"1\"},\"y\":{\"num\":\"-2\",\"den\":\"1\"}},\"endpoint_segments\":[1],\"interior_segments\":[3]}],\"notes\":[\"包含引号: \\\" 和换行\\n\"]}]}"
        );
    }
}
