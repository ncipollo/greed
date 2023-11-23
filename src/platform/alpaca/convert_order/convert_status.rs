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
    use crate::assert;
    use crate::platform::order::status::Status;

    #[test]
    fn into() {
        assert::conversion(apca::api::v2::order::Status::New, Status::New);
        assert::conversion(apca::api::v2::order::Status::Replaced, Status::Replaced);
        assert::conversion(
            apca::api::v2::order::Status::PartiallyFilled,
            Status::PartiallyFilled,
        );
        assert::conversion(apca::api::v2::order::Status::Filled, Status::Filled);
        assert::conversion(apca::api::v2::order::Status::DoneForDay, Status::DoneForDay);
        assert::conversion(apca::api::v2::order::Status::Canceled, Status::Canceled);
        assert::conversion(apca::api::v2::order::Status::Expired, Status::Expired);
        assert::conversion(apca::api::v2::order::Status::Accepted, Status::Accepted);
        assert::conversion(apca::api::v2::order::Status::PendingNew, Status::PendingNew);
        assert::conversion(
            apca::api::v2::order::Status::AcceptedForBidding,
            Status::AcceptedForBidding,
        );
        assert::conversion(
            apca::api::v2::order::Status::PendingCancel,
            Status::PendingCancel,
        );
        assert::conversion(
            apca::api::v2::order::Status::PendingReplace,
            Status::PendingReplace,
        );
        assert::conversion(apca::api::v2::order::Status::Stopped, Status::Stopped);
        assert::conversion(apca::api::v2::order::Status::Rejected, Status::Rejected);
        assert::conversion(apca::api::v2::order::Status::Suspended, Status::Suspended);
        assert::conversion(apca::api::v2::order::Status::Calculated, Status::Calculated);
        assert::conversion(apca::api::v2::order::Status::Held, Status::Held);
        assert::conversion(apca::api::v2::order::Status::Unknown, Status::Unknown);
    }
}
