use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Id {
    Uuid(Uuid)
}

impl Default for Id {
    fn default() -> Self {
        Id::Uuid(Default::default())
    }
}