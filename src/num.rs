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

pub trait NumPercent {
    fn percent_above(&self, target: Num) -> Self;

    fn percent_below(&self, target: Num) -> Self;

}

impl NumPercent for Num {
    fn percent_above(&self, target: Num) -> Self {
        let difference = self - target.clone();
        let percent = difference / target.clone();
        percent * 100
    }

    fn percent_below(&self, target: Num) -> Self {
        let difference = target.clone() - self;
        let percent = difference / target.clone();
        percent * 100
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

    #[test]
    fn percent_above() {
        let num = Num::from_f64(15.0);
        let target = Num::from_f64(10.0);
        let percent = num.percent_above(target);
        let expected = Num::from_f64(50.0);
        assert_eq!(expected, percent)
    }

    #[test]
    fn percent_below() {
        let num = Num::from_f64(5.0);
        let target = Num::from_f64(10.0);
        let percent = num.percent_below(target);
        let expected = Num::from_f64(50.0);
        assert_eq!(expected, percent)
    }
}
