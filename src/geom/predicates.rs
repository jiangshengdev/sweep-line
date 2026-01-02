use crate::geom::fixed::{Coord, PointI64};

pub fn orient(a: PointI64, b: PointI64, c: PointI64) -> i128 {
    let abx = (b.x as i128) - (a.x as i128);
    let aby = (b.y as i128) - (a.y as i128);
    let acx = (c.x as i128) - (a.x as i128);
    let acy = (c.y as i128) - (a.y as i128);
    abx * acy - aby * acx
}

pub fn on_segment(a: PointI64, b: PointI64, p: PointI64) -> bool {
    if orient(a, b, p) != 0 {
        return false;
    }
    in_bbox(a.x, b.x, p.x) && in_bbox(a.y, b.y, p.y)
}

fn in_bbox(a: Coord, b: Coord, v: Coord) -> bool {
    let min = a.min(b);
    let max = a.max(b);
    v >= min && v <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orient_sign() {
        let a = PointI64 { x: 0, y: 0 };
        let b = PointI64 { x: 10, y: 0 };
        let c = PointI64 { x: 0, y: 10 };
        let d = PointI64 { x: 0, y: -10 };

        assert!(orient(a, b, c) > 0);
        assert!(orient(a, b, d) < 0);
        assert_eq!(orient(a, b, PointI64 { x: 20, y: 0 }), 0);
    }

    #[test]
    fn on_segment_inclusive() {
        let a = PointI64 { x: 0, y: 0 };
        let b = PointI64 { x: 10, y: 0 };

        assert!(on_segment(a, b, PointI64 { x: 0, y: 0 }));
        assert!(on_segment(a, b, PointI64 { x: 5, y: 0 }));
        assert!(on_segment(a, b, PointI64 { x: 10, y: 0 }));

        assert!(!on_segment(a, b, PointI64 { x: 11, y: 0 }));
        assert!(!on_segment(a, b, PointI64 { x: 5, y: 1 }));
    }
}

