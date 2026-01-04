use core::cmp::Ordering;
use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rational {
    num: i128,
    den: i128,
}

impl Rational {
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "分母不能为 0");
        if num == 0 {
            return Self { num: 0, den: 1 };
        }

        let (mut num, mut den) = (num, den);
        if den < 0 {
            num = -num;
            den = -den;
        }

        let gcd = gcd_u128(num.unsigned_abs(), den as u128) as i128;
        Self {
            num: num / gcd,
            den: den / gcd,
        }
    }

    pub fn from_int(value: i128) -> Self {
        Self { num: value, den: 1 }
    }

    pub fn num(self) -> i128 {
        self.num
    }

    pub fn den(self) -> i128 {
        self.den
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        match (self.num.cmp(&0), other.num.cmp(&0)) {
            (Ordering::Less, Ordering::Greater) | (Ordering::Less, Ordering::Equal) => {
                return Ordering::Less;
            }
            (Ordering::Greater, Ordering::Less) | (Ordering::Equal, Ordering::Less) => {
                return Ordering::Greater;
            }
            (Ordering::Equal, Ordering::Equal) => return Ordering::Equal,
            _ => {}
        }

        let (a_num_abs, a_den) = (self.num.unsigned_abs(), self.den as u128);
        let (b_num_abs, b_den) = (other.num.unsigned_abs(), other.den as u128);
        let ord = cmp_non_negative_fraction(a_num_abs, a_den, b_num_abs, b_den);
        if self.num.is_negative() {
            ord.reverse()
        } else {
            ord
        }
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            return write!(f, "{}", self.num);
        }
        write!(f, "{}/{}", self.num, self.den)
    }
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn cmp_non_negative_fraction(
    mut a_num: u128,
    mut a_den: u128,
    mut b_num: u128,
    mut b_den: u128,
) -> Ordering {
    debug_assert!(a_den > 0 && b_den > 0);

    let mut reversed = false;
    loop {
        let a_q = a_num / a_den;
        let a_r = a_num % a_den;
        let b_q = b_num / b_den;
        let b_r = b_num % b_den;

        if a_q != b_q {
            let ord = a_q.cmp(&b_q);
            return if reversed { ord.reverse() } else { ord };
        }

        if a_r == 0 || b_r == 0 {
            let ord = a_r.cmp(&b_r);
            return if reversed { ord.reverse() } else { ord };
        }

        a_num = a_den;
        a_den = a_r;
        b_num = b_den;
        b_den = b_r;
        reversed = !reversed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_sign_and_reduces() {
        assert_eq!(Rational::new(2, 4), Rational::new(1, 2));
        assert_eq!(Rational::new(1, -2), Rational::new(-1, 2));
        assert_eq!(Rational::new(0, 5), Rational::from_int(0));
    }

    #[test]
    fn compares_without_overflow() {
        let den = 10_i128.pow(18);
        let a = Rational::new(10_i128.pow(28), den);
        let b = Rational::new(10_i128.pow(28) + 1, den);
        assert!(a < b);

        let c = Rational::new(-10_i128.pow(28), den);
        assert!(c < a);
    }
}
