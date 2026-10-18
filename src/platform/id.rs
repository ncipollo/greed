use serde::{Serialize, Serializer};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Id {
    Uuid(Uuid),
}

impl Default for Id {
    fn default() -> Self {
        Id::Uuid(Default::default())
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Id::Uuid(uuid) => serializer.serialize_str(&uuid.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::platform::id::Id;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn serialize() {
        let uuid = Uuid::parse_str("3b8a63f3-3b5c-4a3d-9d2b-7a2f8e9c0d1a").unwrap();
        let id = Id::Uuid(uuid);

        let json = serde_json::to_string_pretty(&id).expect("failed to serialize");
        let expected =
            serde_json::to_string_pretty(&json!("3b8a63f3-3b5c-4a3d-9d2b-7a2f8e9c0d1a")).unwrap();

        assert_eq!(json, expected)
    }
}
