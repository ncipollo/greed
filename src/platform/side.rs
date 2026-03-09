#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Side {
    /// A long position of an asset.
    #[default]
    Long,
    /// A short position of an asset.
    Short,
}

#[cfg(test)]
mod test {
    use crate::platform::side::Side;

    #[test]
    fn default() {
        let default: Side = Default::default();
        assert_eq!(default, Side::Long)
    }
}
