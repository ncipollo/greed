#[macro_export]
macro_rules! lowercase_enum_display {
    ($enum_type:ty) => {
        impl std::fmt::Display for $enum_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let lower = format!("{:?}", self).to_lowercase();
                write!(f, "{}", lower)
            }
        }
    };
}
