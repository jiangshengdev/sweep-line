//! 几何谓词（predicate）：方向判定与点在线段上判定。
//!
//! 约定：
//! - 坐标来自预处理后的整数网格（`Coord = i64`，见 `geom::fixed`），计算结果是精确的整数；
//! - 线段按闭区间处理（包含端点）。

use crate::geom::fixed::{Coord, PointI64};

/// 计算二维叉积 `(b-a) × (c-a)`（也称“有向面积”的 2 倍）。
///
/// 返回值的符号约定：
/// - `> 0`：点 `c` 在向量 `a -> b` 的左侧（`a,b,c` 逆时针）
/// - `< 0`：点 `c` 在向量 `a -> b` 的右侧（`a,b,c` 顺时针）
/// - `= 0`：三点共线
///
/// 说明：返回 `i128` 以降低中间量溢出风险；该模块假设输入坐标来自量化后的有限范围。
pub fn orient(a: PointI64, b: PointI64, c: PointI64) -> i128 {
    let abx = (b.x as i128) - (a.x as i128);
    let aby = (b.y as i128) - (a.y as i128);
    let acx = (c.x as i128) - (a.x as i128);
    let acy = (c.y as i128) - (a.y as i128);
    abx * acy - aby * acx
}

/// 判断点 `p` 是否在线段 `ab` 上（闭区间，包含端点）。
///
/// 等价条件：
/// - `p` 与 `a,b` 共线（`orient(a,b,p) == 0`）
/// - `p` 位于 `a,b` 的轴对齐包围盒内（含边界）
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
