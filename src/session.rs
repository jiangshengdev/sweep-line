use crate::geom::fixed::{PointI64, SCALE};
use crate::geom::segment::{SegmentId, Segments};
use crate::trace::Trace;

pub const SESSION_SCHEMA: &str = "session.v1";

/// 将（量化后的）线段集合与 `trace.v1` 打包为 `session.v1` JSON（字段顺序固定，便于回归与复现）。
pub fn session_v1_to_json_string(segments: &Segments, trace: &Trace) -> String {
    let mut out = String::new();
    write_session_json(segments, trace, &mut out);
    out
}

fn write_session_json(segments: &Segments, trace: &Trace, out: &mut String) {
    out.push('{');
    write_kv_str(out, "schema", SESSION_SCHEMA);
    out.push(',');

    out.push('"');
    out.push_str("fixed");
    out.push('"');
    out.push(':');
    out.push('{');
    write_kv_str(out, "scale", &SCALE.to_string());
    out.push('}');
    out.push(',');

    out.push('"');
    out.push_str("segments");
    out.push('"');
    out.push(':');
    out.push('[');
    for id in 0..segments.len() {
        if id != 0 {
            out.push(',');
        }
        write_segment_json(out, SegmentId(id), segments.get(SegmentId(id)));
    }
    out.push(']');
    out.push(',');

    out.push('"');
    out.push_str("trace");
    out.push('"');
    out.push(':');
    out.push_str(&trace.to_json_string());

    out.push('}');
}

fn write_segment_json(out: &mut String, id: SegmentId, seg: &crate::geom::segment::Segment) {
    out.push('{');
    write_kv_usize(out, "id", id.0);
    out.push(',');
    write_kv_usize(out, "source_index", seg.source_index);
    out.push(',');
    out.push('"');
    out.push_str("a");
    out.push('"');
    out.push(':');
    write_point_i64(out, seg.a);
    out.push(',');
    out.push('"');
    out.push_str("b");
    out.push('"');
    out.push(':');
    write_point_i64(out, seg.b);
    out.push('}');
}

fn write_point_i64(out: &mut String, p: PointI64) {
    out.push('{');
    write_kv_i64(out, "x", p.x);
    out.push(',');
    write_kv_i64(out, "y", p.y);
    out.push('}');
}

fn write_kv_usize(out: &mut String, key: &str, value: usize) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push_str(&value.to_string());
}

fn write_kv_i64(out: &mut String, key: &str, value: i64) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    out.push_str(&value.to_string());
}

fn write_kv_str(out: &mut String, key: &str, value: &str) {
    out.push('"');
    out.push_str(key);
    out.push('"');
    out.push(':');
    write_json_string(out, value);
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
    use crate::geom::intersection::PointIntersectionKind;
    use crate::geom::point::PointRat;
    use crate::geom::segment::Segment;
    use crate::rational::Rational;
    use crate::trace::TraceStep;

    #[test]
    fn writes_stable_session_json_with_fixed_field_order() {
        let mut segments = Segments::new();
        let a = segments.push(Segment {
            a: PointI64 { x: -10, y: 0 },
            b: PointI64 { x: 10, y: 0 },
            source_index: 7,
        });
        let b = segments.push(Segment {
            a: PointI64 { x: 0, y: -10 },
            b: PointI64 { x: 0, y: 10 },
            source_index: 8,
        });

        let mut trace = Trace::default();
        let mut step = TraceStep::point_batch(
            PointRat::from_i64(PointI64 { x: 0, y: 0 }),
            Rational::from_int(0),
        );
        step.active = vec![a, b];
        step.intersections.push(crate::geom::intersection::PointIntersectionRecord {
            point: PointRat::from_i64(PointI64 { x: 0, y: 0 }),
            kind: PointIntersectionKind::EndpointTouch,
            a,
            b,
        });
        trace.warnings.push("示例告警".to_string());
        trace.steps.push(step);

        let json = session_v1_to_json_string(&segments, &trace);
        assert_eq!(
            json,
            concat!(
                "{\"schema\":\"session.v1\",",
                "\"fixed\":{\"scale\":\"1000000000\"},",
                "\"segments\":[",
                "{\"id\":0,\"source_index\":7,\"a\":{\"x\":-10,\"y\":0},\"b\":{\"x\":10,\"y\":0}},",
                "{\"id\":1,\"source_index\":8,\"a\":{\"x\":0,\"y\":-10},\"b\":{\"x\":0,\"y\":10}",
                "}",
                "],",
                "\"trace\":",
                "{\"schema\":\"trace.v1\",",
                "\"warnings\":[\"示例告警\"],",
                "\"steps\":[",
                "{\"kind\":\"PointBatch\",",
                "\"sweep_x\":{\"num\":\"0\",\"den\":\"1\"},",
                "\"point\":{\"x\":{\"num\":\"0\",\"den\":\"1\"},\"y\":{\"num\":\"0\",\"den\":\"1\"}},",
                "\"events\":[],",
                "\"active\":[0,1],",
                "\"intersections\":[",
                "{\"a\":0,\"b\":1,\"kind\":\"EndpointTouch\",",
                "\"point\":{\"x\":{\"num\":\"0\",\"den\":\"1\"},\"y\":{\"num\":\"0\",\"den\":\"1\"}}}",
                "],",
                "\"notes\":[]",
                "}",
                "]",
                "}",
                "}"
            )
        );
        // 防止未来不小心改了 SCALE 导致“看似稳定，实际不兼容”的情况。
        assert_eq!(SCALE, 1_000_000_000);
    }
}
