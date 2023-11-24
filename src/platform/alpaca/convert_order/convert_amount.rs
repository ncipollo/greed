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
            Amount::Quantity(quantity) => Self::Quantity { quantity },
            Amount::Notional(notional) => Self::Notional { notional },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::assert;
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
}
