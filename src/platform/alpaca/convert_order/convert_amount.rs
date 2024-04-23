use crate::num::{NumAmountRounding};
use crate::platform::order::amount::Amount;

impl From<apca::api::v2::order::Amount> for Amount {
    fn from(value: apca::api::v2::order::Amount) -> Self {
        match value {
            apca::api::v2::order::Amount::Quantity { quantity } => Self::Quantity(quantity),
            apca::api::v2::order::Amount::Notional { notional } => Self::Notional(notional),
        }
    }
}

impl From<Amount> for apca::api::v2::order::Amount {
    fn from(value: Amount) -> Self {
        match value {
            Amount::Quantity(quantity) => Self::Quantity {
                quantity: quantity.round_for_quantity()
            },
            Amount::Notional(notional) => Self::Notional {
                notional: notional.round_for_notional(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
    use crate::num::NumFromFloat;
    use crate::platform::order::amount::Amount;
    use num_decimal::Num;

    #[test]
    fn from_notational() {
        let alpaca_amount = apca::api::v2::order::Amount::Notional {
            notional: Num::from(5),
        };
        assert::conversion(alpaca_amount, Amount::Notional(Num::from(5)));
    }

    #[test]
    fn from_notational_alpaca() {
        let alpaca_amount = apca::api::v2::order::Amount::Notional {
            notional: Num::from(5),
        };
        assert::conversion(Amount::Notional(Num::from(5)), alpaca_amount);
    }

    #[test]
    fn from_notational_alpaca_rounding() {
        let amount: apca::api::v2::order::Amount = Amount::Notional(Num::from_f64(5.559)).into();
        if let apca::api::v2::order::Amount::Notional{notional} = amount {
            assert_eq!(notional.to_f64(), Some(5.55));
        } else {
            panic!("wrong type was converted")
        }
    }

    #[test]
    fn from_quantity() {
        let alpaca_amount = apca::api::v2::order::Amount::Quantity {
            quantity: Num::from(5),
        };
        assert::conversion(alpaca_amount, Amount::Quantity(Num::from(5)));
    }

    #[test]
    fn from_quantity_alpaca() {
        let alpaca_amount = apca::api::v2::order::Amount::Quantity {
            quantity: Num::from(5),
        };
        assert::conversion(Amount::Quantity(Num::from(5)), alpaca_amount);
    }

    #[test]
    fn from_quantity_alpaca_rounding() {
        let amount: apca::api::v2::order::Amount = Amount::Quantity(Num::from_f64(5.123456789)).into();
        if let apca::api::v2::order::Amount::Quantity{quantity} = amount {
            assert_eq!(quantity.to_f64(), Some(5.1234567));
        } else {
            panic!("wrong type was converted")
        }
    }
}
