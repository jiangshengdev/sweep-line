use crate::geom::fixed::PointI64;
use crate::rational::Rational;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointRat {
    pub x: Rational,
    pub y: Rational,
}

impl PointRat {
    pub fn from_i64(point: PointI64) -> Self {
        Self {
            x: Rational::from_int(point.x as i128),
            y: Rational::from_int(point.y as i128),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_lexicographically() {
        let a = PointRat {
            x: Rational::new(0, 1),
            y: Rational::new(0, 1),
        };
        let b = PointRat {
            x: Rational::new(0, 1),
            y: Rational::new(1, 2),
        };
        let c = PointRat {
            x: Rational::new(1, 2),
            y: Rational::new(-10, 1),
        };
        assert!(a < b);
        assert!(b < c);
    }
}
