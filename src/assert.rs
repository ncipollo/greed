#[cfg(test)]
use std::fmt::Debug;

#[cfg(test)]
pub fn conversion<T: Into<U> + PartialEq + Debug, U: PartialEq + Debug>(alpaca_class: T, expected: U)
{
    let order_class: U = alpaca_class.into();
    assert_eq!(order_class, expected)
}