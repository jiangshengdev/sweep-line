use core::fmt;

pub type Coord = i64;

pub const SCALE: Coord = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuantizeError {
    NonFinite,
    OutOfRange,
}

impl fmt::Display for QuantizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantizeError::NonFinite => write!(f, "输入坐标不是有限浮点数"),
            QuantizeError::OutOfRange => write!(f, "输入坐标超出允许范围 [-1, 1]"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointI64 {
    pub x: Coord,
    pub y: Coord,
}

pub fn quantize_coord(value: f64) -> Result<Coord, QuantizeError> {
    if !value.is_finite() {
        return Err(QuantizeError::NonFinite);
    }
    if value < -1.0 || value > 1.0 {
        return Err(QuantizeError::OutOfRange);
    }

    let scaled = value * (SCALE as f64);
    Ok(scaled.round() as Coord)
}

pub fn quantize_point(x: f64, y: f64) -> Result<PointI64, QuantizeError> {
    Ok(PointI64 {
        x: quantize_coord(x)?,
        y: quantize_coord(y)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_coord_rounds() {
        assert_eq!(quantize_coord(0.0).unwrap(), 0);
        assert_eq!(quantize_coord(1.0).unwrap(), SCALE);
        assert_eq!(quantize_coord(-1.0).unwrap(), -SCALE);
        assert_eq!(quantize_coord(0.25).unwrap(), 250_000_000);
        assert_eq!(quantize_coord(-0.25).unwrap(), -250_000_000);
    }

    #[test]
    fn quantize_coord_rejects_out_of_range() {
        assert_eq!(
            quantize_coord(1.0000000001).unwrap_err(),
            QuantizeError::OutOfRange
        );
        assert_eq!(
            quantize_coord(-1.0000000001).unwrap_err(),
            QuantizeError::OutOfRange
        );
    }

    #[test]
    fn quantize_coord_rejects_non_finite() {
        assert_eq!(
            quantize_coord(f64::NAN).unwrap_err(),
            QuantizeError::NonFinite
        );
        assert_eq!(
            quantize_coord(f64::INFINITY).unwrap_err(),
            QuantizeError::NonFinite
        );
        assert_eq!(
            quantize_coord(f64::NEG_INFINITY).unwrap_err(),
            QuantizeError::NonFinite
        );
    }
}
