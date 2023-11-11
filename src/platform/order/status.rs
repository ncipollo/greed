use std::fmt::{Display, Formatter};
use crate::lowercase_enum_display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    New,
    /// The order has changed.
    Replaced,
    /// The order has been partially filled.
    PartiallyFilled,
    /// The order has been filled, and no further updates will occur for
    /// the order.
    Filled,
    /// The order is done executing for the day, and will not receive
    /// further updates until the next trading day.
    DoneForDay,
    /// The order has been canceled.
    Canceled,
    /// The order has expired, and no further updates will occur for the
    /// order.
    Expired,
    /// The order has been received by the financial platform, but hasn't yet been routed
    /// to the execution venue.
    Accepted,
    /// The order has been received by financial platform, and routed to the
    /// exchanges, but has not yet been accepted for execution.
    PendingNew,
    /// The order has been received by exchanges, and is evaluated for
    /// pricing.
    AcceptedForBidding,
    /// The order is waiting to be canceled.
    PendingCancel,
    /// The order is awaiting replacement.
    PendingReplace,
    /// The order has been stopped, and a trade is guaranteed for the
    /// order, usually at a stated price or better, but has not yet
    /// occurred
    Stopped,
    /// The order has been rejected, and no further updates will occur for
    /// the order.
    Rejected,
    /// The order has been suspended, and is not eligible for trading.
    Suspended,
    /// The order has been completed for the day (either filled or done
    /// for day), but remaining settlement calculations are still pending.
    Calculated,
    /// The order is still being held. This may be the case for legs of
    /// bracket-style orders that are not active yet because the primary
    /// order has not filled yet.
    Held,
    Unknown,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown
    }
}

lowercase_enum_display!(Status);

#[cfg(test)]
mod test {
    use crate::platform::order::status::Status;

    #[test]
    fn display() {
        let status_str = Status::New.to_string();
        assert_eq!(status_str, "new")
    }

    #[test]
    fn default() {
        let status: Status = Default::default();
        assert_eq!(status, Status::Unknown)
    }
}