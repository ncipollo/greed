use crate::platform::order::status::Status;

impl From<apca::api::v2::order::Status> for Status {
    fn from(value: apca::api::v2::order::Status) -> Self {
        match value {
            apca::api::v2::order::Status::New => Status::New,
            apca::api::v2::order::Status::Replaced => Status::Replaced,
            apca::api::v2::order::Status::PartiallyFilled => Status::PartiallyFilled,
            apca::api::v2::order::Status::Filled => Status::Filled,
            apca::api::v2::order::Status::DoneForDay => Status::DoneForDay,
            apca::api::v2::order::Status::Canceled => Status::Canceled,
            apca::api::v2::order::Status::Expired => Status::Expired,
            apca::api::v2::order::Status::Accepted => Status::Accepted,
            apca::api::v2::order::Status::PendingNew => Status::PendingNew,
            apca::api::v2::order::Status::AcceptedForBidding => Status::AcceptedForBidding,
            apca::api::v2::order::Status::PendingCancel => Status::PendingCancel,
            apca::api::v2::order::Status::PendingReplace => Status::PendingReplace,
            apca::api::v2::order::Status::Stopped => Status::Stopped,
            apca::api::v2::order::Status::Rejected => Status::Rejected,
            apca::api::v2::order::Status::Suspended => Status::Suspended,
            apca::api::v2::order::Status::Calculated => Status::Calculated,
            apca::api::v2::order::Status::Held => Status::Held,
            _ => Status::Unknown,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::order::status::Status;

    #[test]
    fn into() {
        assert_conversion(apca::api::v2::order::Status::New, Status::New);
        assert_conversion(apca::api::v2::order::Status::Replaced, Status::Replaced);
        assert_conversion(
            apca::api::v2::order::Status::PartiallyFilled,
            Status::PartiallyFilled,
        );
        assert_conversion(apca::api::v2::order::Status::Filled, Status::Filled);
        assert_conversion(apca::api::v2::order::Status::DoneForDay, Status::DoneForDay);
        assert_conversion(apca::api::v2::order::Status::Canceled, Status::Canceled);
        assert_conversion(apca::api::v2::order::Status::Expired, Status::Expired);
        assert_conversion(apca::api::v2::order::Status::Accepted, Status::Accepted);
        assert_conversion(apca::api::v2::order::Status::PendingNew, Status::PendingNew);
        assert_conversion(
            apca::api::v2::order::Status::AcceptedForBidding,
            Status::AcceptedForBidding,
        );
        assert_conversion(
            apca::api::v2::order::Status::PendingCancel,
            Status::PendingCancel,
        );
        assert_conversion(
            apca::api::v2::order::Status::PendingReplace,
            Status::PendingReplace,
        );
        assert_conversion(apca::api::v2::order::Status::Stopped, Status::Stopped);
        assert_conversion(apca::api::v2::order::Status::Rejected, Status::Rejected);
        assert_conversion(apca::api::v2::order::Status::Suspended, Status::Suspended);
        assert_conversion(apca::api::v2::order::Status::Calculated, Status::Calculated);
        assert_conversion(apca::api::v2::order::Status::Held, Status::Held);
        assert_conversion(apca::api::v2::order::Status::Unknown, Status::Unknown);
    }

    fn assert_conversion(alpaca_status: apca::api::v2::order::Status, expected: Status) {
        let status: Status = alpaca_status.into();
        assert_eq!(status, expected)
    }
}
