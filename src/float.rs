pub trait PrecisionFloor {
    fn floor_with(&self, precision: i32) -> Self;
}

impl PrecisionFloor for f64 {
    fn floor_with(&self, precision: i32) -> Self {
        let factor = 10.0_f64.powi(precision );
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