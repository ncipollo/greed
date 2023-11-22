#[cfg(test)]
use std::fmt::Debug;

#[cfg(test)]
pub fn conversion<T: Into<U> + PartialEq + Debug, U: PartialEq + Debug>(from: T, expected: U)
{
    let order_class: U = from.into();
    assert_eq!(order_class, expected)
}