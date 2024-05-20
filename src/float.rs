pub trait PrecisionFloor {
    fn floor_with(&self, precision: i32) -> Self;
}

impl PrecisionFloor for f64 {
    fn floor_with(&self, precision: i32) -> Self {
        let factor = 10.0_f64.powi(precision);
        (self * factor).floor() / factor
    }
}

pub trait FloatAmountRounding {
    fn round_for_notional(&self) -> Self;
    fn round_for_quantity(&self) -> Self;
}

impl FloatAmountRounding for f64 {
    fn round_for_notional(&self) -> Self {
        self.floor_with(2)
    }

    fn round_for_quantity(&self) -> Self {
        self.floor_with(7)
    }
}

pub trait PercentOps {
    fn percent_of(&self, percent: f64) -> Self;
    fn percent_above(&self, target: f64) -> Self;

    fn percent_below(&self, target: f64) -> Self;
}

impl PercentOps for f64 {
    fn percent_of(&self, percent: f64) -> Self {
        let percent_fraction = percent / 100.0;
        self * percent_fraction
    }

    fn percent_above(&self, target: f64) -> Self {
        let difference = self - target;
        let percent = difference / target;
        percent * 100.0
    }

    fn percent_below(&self, target: f64) -> Self {
        let difference = target.clone() - self;
        let percent = difference / target;
        percent * 100.0
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn floor_with_precision() {
        let num = 10.129;
        let floored = num.floor_with(2);
        let expected = 10.12;
        assert_relative_eq!(expected, floored, max_relative = 0.001)
    }

    #[test]
    fn percent_of_percent_is_fraction() {
        let num = 10.0;
        let percent = num.percent_of(10.5);
        let expected = 1.05;
        assert_relative_eq!(expected, percent, max_relative = 0.001)
    }

    #[test]
    fn percent_of_num_is_fraction() {
        let num = 10.50;
        let percent = num.percent_of(50.0);
        let expected = 5.25;
        assert_relative_eq!(expected, percent, max_relative = 0.001)
    }

    #[test]
    fn percent_of_whole_number() {
        let num = 10.0;
        let percent = num.percent_of(50.0);
        let expected = 5.0;
        assert_relative_eq!(expected, percent, max_relative = 0.001)
    }

    #[test]
    fn percent_above() {
        let num = 15.0;
        let target = 10.0;
        let percent = num.percent_above(target);
        let expected = 50.0;
        assert_relative_eq!(expected, percent, max_relative = 0.001)
    }

    #[test]
    fn percent_below() {
        let num = 5.0;
        let target = 10.0;
        let percent = num.percent_below(target);
        let expected = 50.0;
        assert_relative_eq!(expected, percent, max_relative = 0.001)
    }
}
