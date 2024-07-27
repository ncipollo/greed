use num_decimal::num_rational::BigRational;
use num_decimal::Num;
use num_decimal::num_bigint::BigInt;

pub trait NumFromFloat {
    fn from_f64(float: f64) -> Self;
}

pub trait NumFloor {
    fn floor_with(&self, precision: u32) -> Self;
}

#[allow(dead_code)]
pub trait NumAmountRounding {
    fn round_for_notional(&self) -> Self;
    fn round_for_quantity(&self) -> Self;
}

impl NumFromFloat for Num {
    fn from_f64(float: f64) -> Self {
        let rational: BigRational = BigRational::from_float(float).unwrap_or_default();
        let (numer, denom) = rational.into();
        Num::new(numer, denom)
    }
}

impl NumFloor for Num {
    fn floor_with(&self, precision: u32) -> Self {
        let factor = BigInt::from(10).pow(precision);
        let value = self * factor.clone();
        value.trunc() / factor.clone()
    }
}

impl NumAmountRounding for Num {
    fn round_for_notional(&self) -> Self {
        self.floor_with(2)
    }

    fn round_for_quantity(&self) -> Self {
        self.floor_with(7)
    }
}

#[allow(dead_code)]
pub trait NumPercent {
    fn percent_of(&self, percent: f64) -> Self;
    fn percent_above(&self, target: Num) -> Self;

    fn percent_below(&self, target: Num) -> Self;
}

impl NumPercent for Num {
    fn percent_of(&self, percent: f64) -> Self {
        let percent_num = Num::from_f64(percent / 100.0);
        self * percent_num
    }

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
    fn floor_with_precision() {
        let num = Num::from_f64(10.129);
        let floored = num.floor_with(2);
        let expected = Num::from_f64(10.12);
        assert_eq!(expected.to_f64(), floored.to_f64())
    }

    #[test]
    fn percent_of_percent_is_fraction() {
        let num = Num::from_f64(10.0);
        let percent = num.percent_of(10.5);
        let expected = Num::from_f64(1.05);
        assert_eq!(expected.to_f64().unwrap(), percent.to_f64().unwrap())
    }

    #[test]
    fn percent_of_num_is_fraction() {
        let num = Num::from_f64(10.50);
        let percent = num.percent_of(50.0);
        let expected = Num::from_f64(5.25);
        assert_eq!(expected.to_f64().unwrap(), percent.to_f64().unwrap())
    }

    #[test]
    fn percent_of_whole_number() {
        let num = Num::from_f64(10.0);
        let percent = num.percent_of(50.0);
        let expected = Num::from_f64(5.0);
        assert_eq!(expected, percent)
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
