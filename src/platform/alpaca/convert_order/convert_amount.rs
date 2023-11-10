use crate::platform::order::amount::Amount;

impl From<apca::api::v2::order::Amount> for Amount {
    fn from(value: apca::api::v2::order::Amount) -> Self {
        match value {
            apca::api::v2::order::Amount::Quantity { quantity } => Self::Quantity(quantity),
            apca::api::v2::order::Amount::Notional { notional } => Self::Notional(notional),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::amount::Amount;
    use num_decimal::Num;

    #[test]
    fn from_notational() {
        let alpaca_amount = apca::api::v2::order::Amount::Notional {
            notional: Num::from(5),
        };
        let amount: Amount = alpaca_amount.into();
        assert_eq!(amount, Amount::Notional(Num::from(5)))
    }

    #[test]
    fn from_quantity() {
        let alpaca_amount = apca::api::v2::order::Amount::Quantity {
            quantity: Num::from(5),
        };
        let amount: Amount = alpaca_amount.into();
        assert_eq!(amount, Amount::Quantity(Num::from(5)))
    }
}
