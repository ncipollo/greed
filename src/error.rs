use std::fmt::{Debug, Display};

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

    fn from_debug<T: Debug>(debug: T) -> Self {
        let message = format!("debug error: {:?}", debug);
        Self::new(&message)
    }

    fn from_display<T: Display>(display: T) -> Self {
        let message = format!("error: {display}");
        Self::new(&message)
    }
}

impl<T: Display> From<T> for GreedError {
    fn from(value: T) -> Self {
        GreedError::from_display(value)
    }
}

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
            "error: not_found",
        );
        assert_eq!(greed_error, expected)
    }
}
