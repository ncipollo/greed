use num_decimal::num_rational::BigRational;
use num_decimal::Num;

pub trait NumFromFloat {
    fn from_f64(float: f64) -> Self;
}

impl NumFromFloat for Num {
    fn from_f64(float: f64) -> Self {
        let rational: BigRational = BigRational::from_float(float).unwrap_or_default();
        let (numer, denom) = rational.into();
        Num::new(numer, denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_f64_fraction() {
        let num = Num::from_f64(0.5);
        let expected = Num::new(1, 2);
        assert_eq!(expected, num)
    }

    #[test]
    fn from_f64_whole_number() {
        let num = Num::from_f64(10.0);
        let expected = Num::from(10);
        assert_eq!(expected, num)
    }

    #[test]
    fn from_f64_zero() {
        let num = Num::from_f64(0.0);
        let expected = Num::from(0);
        assert_eq!(expected, num)
    }
}
