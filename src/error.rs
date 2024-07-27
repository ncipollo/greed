use std::env::VarError;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct GreedError {
    message: String,
}

impl GreedError {
    pub fn new(message: &str) -> Self {
        Self {
            message: format!("😱 {message}"),
        }
    }

    #[allow(unused)]
    fn from_debug<T: Debug>(debug: T) -> Self {
        let message = format!("debug error: {:?}", debug);
        Self::new(&message)
    }

    pub fn from_display<T: Display + Debug>(display: T) -> Self {
        let message = format!("error: {display}\nDebug info: {:?}", display);
        Self::new(&message)
    }
}

impl Display for GreedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GreedError { }

#[macro_export]
macro_rules! greed_error_from {
    ($err_type:ty) => {
        impl From<$err_type> for crate::error::GreedError {
            fn from(value: $err_type) -> Self {
                crate::error::GreedError::from_display(value)
            }
        }
    };
}

greed_error_from!(csv::Error);
greed_error_from!(std::io::Error);
greed_error_from!(toml::de::Error);
greed_error_from!(VarError);
greed_error_from!(apca::Error);

#[cfg(test)]
mod test {
    use std::io::ErrorKind;
    use crate::error::GreedError;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct DebugTest {
        message: String,
    }

    #[test]
    fn from_debug() {
        let debug = DebugTest {
            message: "message".to_string(),
        };
        let greed_error = GreedError::from_debug(debug);
        let expected = GreedError::new(
            "debug error: DebugTest { message: \"message\" }",
        );
        assert_eq!(greed_error, expected)
    }

    #[test]
    fn from_io_error() {
        let io_error = std::io::Error::new(ErrorKind::NotFound, "not_found");
        let greed_error = GreedError::from(io_error);
        let expected = GreedError::new(
            "error: not_found\nDebug info: Custom { kind: NotFound, error: \"not_found\" }",
        );
        assert_eq!(greed_error, expected)
    }
}
