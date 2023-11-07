#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    /// A long position of an asset.
    Long,
    /// A short position of an asset.
    Short,
}

impl Default for Side {
    fn default() -> Self {
        Self::Long
    }
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
